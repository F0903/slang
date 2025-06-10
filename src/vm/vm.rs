use super::{callframe::CallFrame, opcode::OpCode};
#[cfg(debug_assertions)]
use crate::debug::disassemble_instruction;
use crate::{
    collections::{DynArray, HashTable, Stack},
    compiler::{Compiler, FunctionType},
    dbg_println,
    debug::disassemble_chunk,
    error::{Error, Result},
    lexing::scanner::Scanner,
    memory::{DeallocOnDrop, GC, Gc, GcRoots, HeapPtr},
    value::{
        ObjectType,
        Value,
        ValueType,
        object::{self, Closure, InternedString, NativeFunction, ObjectRef},
    },
};

pub const STACK_MAX: usize = 1024;
pub const MAX_CALLFRAMES: usize = 256;

macro_rules! binary_op_result {
    ($stack: expr, $op: tt) => {{
        let b = $stack.pop();
        let a = $stack.pop();
        crate::dbg_println!("BINARY_OP: {} {} {}", a, stringify!($op), b);
        a $op b
    }};
}

macro_rules! binary_op_try {
    ($stack: expr, $op: tt) => {{
        let val = binary_op_result!($stack, $op)?;
        $stack.push(val);
    }};
}

macro_rules! binary_op_from_bool {
    ($stack: expr, $op: tt) => {{
        let val = binary_op_result!($stack, $op);
        $stack.push(Value::bool(val));
    }};
}

#[derive(Debug)]
pub struct Vm {
    stack: Stack<Value, STACK_MAX>,
    globals: HashTable<ObjectRef<InternedString>, Value>,
    callframes: Stack<CallFrame, MAX_CALLFRAMES>,
    open_upvalues: Option<ObjectRef<object::Upvalue>>,
}

impl Vm {
    pub fn new() -> HeapPtr<Self> {
        let me = HeapPtr::alloc(Self {
            stack: Stack::new(),
            globals: HashTable::new(),
            callframes: Stack::new(),
            open_upvalues: None,
        });
        GC.register_roots(&*me);
        me
    }

    pub fn register_native_function(&mut self, function: NativeFunction) {
        let func_name = GC.create_string(&function.name);
        let func_value = Value::object(GC.create_native_function(function).upcast());
        self.globals.set(func_name, func_value);
    }

    pub fn register_native_functions(&mut self, functions: &[NativeFunction]) {
        for function in functions {
            self.register_native_function(*function);
        }
    }

    pub fn get_stack_trace(&self) -> String {
        let mut str_buf = String::new();
        for frame in self.callframes.top_iter() {
            // Since this is a debug function, it doesn't matter that we are cloning here
            let mut frame = frame.clone();
            let function = frame.get_closure().get_function();
            let instruction = unsafe {
                let ip_offset = frame
                    .get_ip()
                    .offset_from(function.get_chunk().get_code_ptr());
                frame.sub_ip((ip_offset as usize) - 1);
                frame.get_ip().read()
            }; // -1 to get the previous instruction that failed

            let function_name = function.get_name();
            str_buf.push_str(&format!(
                "[line {}] in {}",
                function.get_chunk().get_line_number(instruction as usize),
                function_name
                    .as_ref()
                    .map(|name| name.as_str())
                    .unwrap_or("<script>")
            ));
        }
        str_buf
    }

    fn read_byte(&mut self) -> u8 {
        unsafe {
            let frame = self.callframes.top_mut();
            let val = frame.get_ip().read();
            frame.add_ip(1);
            val
        }
    }

    fn read_double(&mut self) -> u16 {
        unsafe {
            let frame = self.callframes.top_mut();
            let val = frame.get_ip().cast::<u16>().read();
            frame.add_ip(2);
            val
        }
    }

    fn read_quad(&mut self) -> u32 {
        unsafe {
            let frame = self.callframes.top_mut();
            let val = frame.get_ip().cast::<u32>().read();
            frame.add_ip(4);
            val
        }
    }

    /// Reads a constant from the chunk with a u8 index.
    fn read_constant(&mut self) -> Value {
        let index = self.read_byte();
        let frame = self.callframes.top_mut();
        frame
            .get_closure()
            .get_function()
            .get_chunk()
            .get_constant(index as u32)
            .clone()
    }

    /// Reads a constant from the chunk with a u16 index.
    fn read_constant_double(&mut self) -> Value {
        let index = self.read_double();
        let frame = self.callframes.top_mut();
        frame
            .get_closure()
            .get_function()
            .get_chunk()
            .get_constant(index as u32)
            .clone()
    }

    /// Reads a constant from the chunk with a u32 index.
    fn read_constant_quad(&mut self) -> Value {
        let index = self.read_quad();
        let frame = self.callframes.top_mut();
        frame
            .get_closure()
            .get_function()
            .get_chunk()
            .get_constant(index)
            .clone()
    }

    fn call(&mut self, closure: ObjectRef<Closure>, arg_count: u8) -> Result<()> {
        let function = closure.function.clone();
        if arg_count != function.get_arity() {
            return Err(Error::Runtime(format!(
                "Expected {} arguments, got {} for function '{}'",
                function.get_arity(),
                arg_count,
                function
                    .get_name()
                    .as_ref()
                    .map(|x| x.as_str())
                    .unwrap_or("<script>")
            )));
        }

        if self.callframes.count() >= MAX_CALLFRAMES {
            return Err(Error::Runtime("Call stack overflow".to_owned()));
        }

        dbg_println!(
            "CALLING FUNCTION: {} with {} args",
            function
                .get_name()
                .as_ref()
                .map(|x| x.as_str())
                .unwrap_or("<script>"),
            arg_count
        );
        disassemble_chunk(
            function.get_chunk(),
            function
                .get_name()
                .as_ref()
                .map(|x| x.as_str())
                .unwrap_or("<script>"),
        );

        let stack_base = self.stack.get_mut_at(0) as *mut Value;
        let callframe_slots = unsafe { stack_base.add(self.stack.count() - arg_count as usize) };
        dbg_println!("STACK BASE -> {:?}", stack_base);
        dbg_println!("CALLFRAME SLOTS -> {:?}", callframe_slots);

        self.callframes.push(CallFrame::new(
            closure,
            function.get_chunk().get_code_ptr(),
            callframe_slots,
        ));
        Ok(())
    }

    fn native_call(&mut self, native_func: ObjectRef<NativeFunction>, arg_count: u8) -> Result<()> {
        if arg_count != native_func.arity {
            return Err(Error::Runtime(format!(
                "Expected {} arguments, got {} for function '{}'",
                native_func.arity, arg_count, native_func.name
            )));
        }

        let args = self.stack.pop_n(arg_count as usize);
        let result = (native_func.function)(args)?;
        self.stack.push(result);
        Ok(())
    }

    fn call_value(&mut self, callee: Value, arg_count: u8) -> Result<()> {
        match callee.get_type() {
            ValueType::Object => {
                let object = callee.as_object();
                match object.get_type() {
                    ObjectType::Closure => self.call(object.as_closure(), arg_count),
                    ObjectType::NativeFunction => {
                        self.native_call(object.as_native_function(), arg_count)
                    }
                    _ => Err(Error::Runtime(format!("'{}' not callable!", callee))),
                }
            }
            _ => Err(Error::Runtime(format!("'{}' not callable!", callee))),
        }
    }

    fn capture_upvalue(&mut self, index: u16) -> ObjectRef<object::Upvalue> {
        let frame = self.callframes.top_mut();
        let local = frame.get_slot_mut(index as usize);

        dbg_println!("CAPTURING UPVALUE FOR '{}'", local);

        let mut previous_upvalue = None;
        let mut upvalue = self.open_upvalues;
        while let Some(upvalue_ref) = upvalue
            && upvalue_ref.addr_gt_addr(local)
        {
            previous_upvalue = Some(upvalue_ref);
            upvalue = upvalue_ref.get_next();
        }

        if let Some(upvalue_ref) = upvalue
            && upvalue_ref.get_location_raw() == local as *const Value
        {
            return upvalue_ref;
        }

        let new_upvalue = if let Some(upvalue_ref) = upvalue {
            GC.create_upvalue(object::Upvalue::new_with_next(local, upvalue_ref))
        } else {
            GC.create_upvalue(object::Upvalue::new(local))
        };

        if previous_upvalue.is_none() {
            self.open_upvalues = Some(new_upvalue);
        } else {
            previous_upvalue.unwrap().set_next(Some(new_upvalue));
        }

        new_upvalue
    }

    fn close_upvalues(&mut self, stack_slot: *const Value) {
        while let Some(open_upvalues) = self.open_upvalues
            && open_upvalues.get_location_raw() >= stack_slot
        {
            let mut upvalue = open_upvalues;
            upvalue.close();
            self.open_upvalues = upvalue.get_next();
        }
    }

    pub fn interpret<'src>(&mut self, source: &'src [u8]) -> Result<()> {
        let mut compiler =
            Compiler::new(HeapPtr::alloc(Scanner::new()), FunctionType::Script).dealloc_on_drop();

        let function_obj = compiler
            .compile(source)
            .map_err(|e| Error::CompileTime(e.to_string()))?;
        let closure = GC.create_closure(Closure::new(function_obj.as_function(), DynArray::new()));
        self.stack.push(Value::object(closure.upcast()));

        self.callframes.push(CallFrame::new(
            closure,
            closure.get_function().get_chunk().get_code_ptr(),
            self.stack.get_mut_at(0),
        ));
        self.call(closure, 0)?;

        loop {
            #[cfg(debug_assertions)]
            unsafe {
                let frame = self.callframes.top_mut();
                let offset = frame.get_ip().offset_from(
                    frame
                        .get_closure()
                        .get_function()
                        .get_chunk()
                        .get_code_ptr(),
                );
                disassemble_instruction(
                    &frame.get_closure().get_function().get_chunk(),
                    offset as usize,
                );
            }
            let instruction = self.read_byte();
            match OpCode::from_code(instruction) {
                OpCode::CloseUpvalue => {
                    // We need a reference to the stack slot, so we can not use pop here and just pass the owned value.
                    let top = self.stack.top_ref();
                    self.close_upvalues(top);
                    self.stack.pop();
                }
                OpCode::SetUpvalue => {
                    let slot = self.read_double();
                    let frame = self.callframes.top_mut();
                    dbg_println!("SET UPVALUE FRAME: {:?}", frame);
                    let new_value = self.stack.peek(0);
                    dbg_println!("SETTING UPVALUE {} TO: {:?}", slot, new_value);
                    frame
                        .get_closure()
                        .get_upvalue(slot as usize)
                        .set(new_value.clone());
                }
                OpCode::GetUpvalue => {
                    let slot = self.read_double();
                    let frame = self.callframes.top_ref();
                    dbg_println!("GET UPVALUE FRAME: {:?}", frame);
                    let upvalue = frame.get_closure().get_upvalue(slot as usize);
                    let value = upvalue.get_value();
                    dbg_println!("GET UPVALUE = {:?}", value);
                    self.stack.push(value.clone());
                }
                OpCode::Closure => {
                    let function = self.read_constant_double().as_object().as_function();
                    dbg_println!("CLOSURE FUNCTION: {:?}", function);

                    // Create and fill upvalue array for Closure object based on
                    // the amount of upvalue references the Function has
                    let mut closure_upvalues =
                        DynArray::new_with_cap(function.get_upvalue_count() as usize);
                    for _ in 0..function.get_upvalue_count() {
                        let is_local = self.read_byte() != 0;
                        let index = self.read_double();
                        if is_local {
                            let upvalue = self.capture_upvalue(index);
                            closure_upvalues.push(upvalue);
                        } else {
                            let frame = self.callframes.top_ref();
                            let upvalue = frame.get_closure().get_upvalue(index as usize);
                            closure_upvalues.push(upvalue);
                        }
                    }

                    let closure =
                        GC.create_closure(Closure::new(function.clone(), closure_upvalues));
                    self.stack.push(Value::object(closure.upcast()));
                }
                OpCode::Return => {
                    let result = self.stack.pop();
                    let frame: CallFrame = self.callframes.pop();
                    self.close_upvalues(frame.get_slots_raw());
                    if self.callframes.count() == 1 {
                        self.stack.pop(); // Pop the "main" function itself (exiting the program)
                        return Ok(());
                    }
                    self.stack
                        .pop_n(frame.get_closure().get_function().get_arity() as usize + 1); // Pop arguments and the function itself
                    self.stack.push(result);
                }
                OpCode::Call => {
                    let arg_count = self.read_byte();
                    let callee = self.stack.peek(arg_count as usize);
                    if let Err(e) = self.call_value(callee.clone(), arg_count) {
                        return Err(Error::Runtime(format!("Function call failed: {}", e)));
                    }
                }
                OpCode::Backjump => {
                    let offset = self.read_double();
                    let frame = self.callframes.top_mut();
                    frame.sub_ip(offset as usize);
                }
                OpCode::Jump => {
                    let offset = self.read_double();
                    let frame = self.callframes.top_mut();
                    frame.add_ip(offset as usize);
                }
                OpCode::JumpIfTrue => {
                    let offset = self.read_double();
                    let frame = self.callframes.top_mut();
                    if !self.stack.peek(0).is_falsey() {
                        frame.add_ip(offset as usize);
                    }
                }
                OpCode::JumpIfFalse => {
                    let offset = self.read_double();
                    let frame = self.callframes.top_mut();
                    if self.stack.peek(0).is_falsey() {
                        frame.add_ip(offset as usize);
                    }
                }
                OpCode::SetLocal => {
                    let slot = self.read_double();
                    let frame = self.callframes.top_mut();
                    let value = self.stack.peek(0).clone();
                    dbg_println!(
                        "SETTING LOCAL {:?} + {:?} = {:?}",
                        frame.get_slots_raw(),
                        slot,
                        value
                    );
                    frame.set_slot(slot as usize, value);
                }
                OpCode::GetLocal => {
                    let slot = self.read_double();
                    let frame = self.callframes.top_ref();
                    let local = frame.get_slot_ref(slot as usize);
                    dbg_println!(
                        "GETTING LOCAL {:?} + {:?} -> {:?}",
                        frame.get_slots_raw(),
                        slot,
                        local
                    );
                    self.stack.push(local.clone());
                }
                OpCode::PopN => {
                    let n = self.read_double();
                    self.stack.pop_n(n as usize);
                }
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::SetGlobal => {
                    let name_string = self.read_constant_quad().as_object().as_string();
                    let value = self.stack.peek(0).clone();
                    dbg_println!("SETTING GLOBAL {} = {}", name_string, value);
                    if self.globals.set(name_string, value) {
                        // If the variable did not already exist at this point, return error
                        self.globals.delete(name_string);
                        return Err(Error::Runtime(format!(
                            "Undefined variable '{}'",
                            name_string
                        )));
                    }
                }
                OpCode::GetGlobal => {
                    let name_string = self.read_constant_quad().as_object().as_string();
                    let global = self.globals.get(&name_string);
                    match global {
                        Some(global) => {
                            let global_value = global.value.clone();
                            dbg_println!("GETTING GLOBAL: {} = ({})", name_string, global_value);
                            self.stack.push(global_value);
                        }
                        None => {
                            return Err(Error::Runtime(format!(
                                "Undefined variable '{}'",
                                name_string
                            )));
                        }
                    }
                }
                OpCode::DefineGlobal => {
                    let name_string = self.read_constant_quad().as_object().as_string();
                    let global_value = self.stack.peek(0).clone();
                    dbg_println!("DEFINING GLOBAL: {} = ({})", name_string, global_value);
                    self.globals.set(name_string, global_value);
                    self.stack.pop();
                }
                OpCode::Constant => {
                    let constant = self.read_constant_quad().clone();
                    dbg_println!("PUSHING CONSTANT: {}", constant);
                    self.stack.push(constant);
                }
                OpCode::None => self.stack.push(Value::none()),
                OpCode::True => self.stack.push(Value::bool(true)),
                OpCode::False => self.stack.push(Value::bool(false)),
                OpCode::Is => binary_op_from_bool!(self.stack, ==),
                OpCode::IsNot => binary_op_from_bool!(self.stack, !=),
                OpCode::Greater => binary_op_from_bool!(self.stack, >),
                OpCode::GreaterEqual => binary_op_from_bool!(self.stack, >=),
                OpCode::Less => binary_op_from_bool!(self.stack, <),
                OpCode::LessEqual => binary_op_from_bool!(self.stack, <=),
                OpCode::Add => {
                    let second = self.stack.pop();
                    let first = self.stack.pop();

                    if first.get_type() == ValueType::Object
                        && second.get_type() == ValueType::Object
                    {
                        return Err(Error::Runtime("Cannot add object types!!".to_owned()));
                    }

                    let result = first + second;
                    self.stack.push(result?);
                }
                OpCode::Subtract => binary_op_try!(self.stack, -),
                OpCode::Multiply => binary_op_try!(self.stack, *),
                OpCode::Divide => binary_op_try!(self.stack, /),
                OpCode::Not => {
                    let val = self.stack.pop();
                    self.stack.push(Value::bool(val.is_falsey()));
                }
                OpCode::Negate => {
                    let val = self.stack.top_mut_offset(0);
                    *val = (-(*val).clone())?;
                }
            }
        }
    }
}

impl Drop for Vm {
    fn drop(&mut self) {
        dbg_println!("DEBUG DROP VM");
        GC.unregister_roots(self);
    }
}

impl GcRoots for Vm {
    fn mark_roots(&mut self, gc: &Gc) {
        // MARK STACK
        for value in self.stack.bottom_iter().cloned() {
            gc.mark_value(value);
        }

        // MARK GLOBALS
        for global in self.globals.entries().map(|x| x.value) {
            gc.mark_value(global);
        }

        // MARK CALLFRAME CLOSURES
        for callframe in self.callframes.bottom_iter() {
            let object = callframe.get_closure().upcast();
            gc.mark_object(object);
        }

        // MARK OPEN UPVALUES
        if let Some(upvalue) = self.open_upvalues {
            let mut upvalue = Some(upvalue);
            while let Some(up) = upvalue {
                gc.mark_object(up.upcast());
                upvalue = up.get_next();
            }
        }
    }
}

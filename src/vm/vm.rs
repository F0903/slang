use super::{VmHeap, callframe::CallFrame, opcode::OpCode};
#[cfg(debug_assertions)]
use crate::debug::disassemble_instruction;
use crate::{
    collections::{HashTable, Stack},
    compiler::{Compiler, FunctionType},
    dbg_println,
    debug::disassemble_chunk,
    error::{Error, Result},
    lexing::scanner::Scanner,
    memory::{Dealloc, DeallocOnDrop, HeapPtr},
    unwrap_enum,
    value::{
        Value,
        object::{Function, InternedString, NativeFunction, Object, ObjectNode},
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
        $stack.push(Value::Bool(val));
    }};
}

pub struct Vm {
    stack: Stack<Value, STACK_MAX>,
    heap: HeapPtr<VmHeap>,
    callframes: Stack<CallFrame, MAX_CALLFRAMES>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
            heap: HeapPtr::alloc(VmHeap {
                objects_head: HeapPtr::null(),
                interned_strings: HashTable::new(),
                globals: HashTable::new(),
            }),
            callframes: Stack::new(),
        }
    }

    fn read_byte(&mut self) -> u8 {
        unsafe {
            let frame = self.callframes.top_mut();
            let val = frame.ip.read();
            frame.ip = frame.ip.add(1);
            val
        }
    }

    fn read_double(&mut self) -> u16 {
        unsafe {
            let frame = self.callframes.top_mut();
            let val = frame.ip.cast::<u16>().read();
            frame.ip = frame.ip.add(2);
            val
        }
    }

    fn read_quad(&mut self) -> u32 {
        unsafe {
            let frame = self.callframes.top_mut();
            let val = frame.ip.cast::<u32>().read();
            frame.ip = frame.ip.add(4);
            val
        }
    }

    /// Reads a constant from the chunk with a u32 index.
    fn read_constant_quad(&mut self) -> &Value {
        let index = self.read_quad();
        let frame = self.callframes.top_mut();
        frame.function.chunk.get_constant(index)
    }

    /// Reads a constant from the chunk with a u8 index.
    fn read_constant(&mut self) -> &Value {
        let index = self.read_byte();
        let frame = self.callframes.top_mut();
        frame.function.chunk.get_constant(index as u32)
    }

    pub fn get_stack_trace(&self) -> String {
        let mut str_buf = String::new();
        for frame in self.callframes.top_iter() {
            let instruction = unsafe {
                let ip_offset = frame.ip.offset_from(frame.function.chunk.get_code_ptr());
                frame.ip.sub((ip_offset as usize) - 1).read()
            }; // -1 to get the previous instruction that failed

            let function_name = &frame.function.name;
            str_buf.push_str(&format!(
                "[line {}] in {}",
                frame.function.chunk.get_line_number(instruction as usize),
                function_name
                    .as_ref()
                    .map(|name| name.as_str())
                    .unwrap_or("<script>")
            ));
        }
        str_buf
    }

    fn call(&mut self, function: Function, arg_count: u8) -> Result<()> {
        if arg_count != function.arity {
            return Err(Error::Runtime(format!(
                "Expected {} arguments, got {} for function '{}'",
                function.arity,
                arg_count,
                function
                    .name
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
                .name
                .as_ref()
                .map(|x| x.as_str())
                .unwrap_or("<script>"),
            arg_count
        );
        disassemble_chunk(
            &function.chunk,
            function
                .name
                .as_ref()
                .map(|x| x.as_str())
                .unwrap_or("<script>"),
        );

        self.callframes.push(CallFrame {
            ip: function.chunk.get_code_ptr(),
            function: function.clone(),
            stack_base_offset: self.stack.count() - arg_count as usize,
        });
        Ok(())
    }

    fn native_call(&mut self, native_func: NativeFunction, arg_count: u8) -> Result<()> {
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
        match callee {
            Value::Object(obj) => match obj.get_object() {
                Object::Function(func) => self.call(func.clone(), arg_count),
                Object::NativeFunction(native_func) => {
                    self.native_call(native_func.clone(), arg_count)
                }
                _ => Err(Error::Runtime(
                    "Cannot call non-function object!".to_owned(),
                )),
            },
            _ => Err(Error::Runtime(format!(
                "Cannot call non-object value!: {}",
                callee
            ))),
        }
    }

    pub fn register_native_function(&mut self, function: NativeFunction) {
        let func_name = InternedString::new(&function.name, &mut self.heap);
        let func_value = Value::Object(ObjectNode::alloc(
            Object::NativeFunction(function),
            &mut self.heap,
        ));

        self.heap.globals.set(func_name, Some(func_value));
    }

    pub fn register_native_functions(&mut self, functions: &[NativeFunction]) {
        for function in functions {
            self.register_native_function(*function);
        }
    }

    fn read_constant_string(&mut self) -> Result<InternedString> {
        let value = self.read_constant_quad();
        let obj = unwrap_enum!(
            value,
            Value::Object,
            "Expected Object value for constant string"
        );
        let string = match obj.get_object() {
            Object::String(s) => s.clone(),
            _ => {
                return Err(Error::Runtime(format!(
                    "Expected string object, found: {:?}",
                    obj.get_object()
                )));
            }
        };
        Ok(string)
    }

    pub fn interpret<'src>(&mut self, source: &'src [u8]) -> Result<()> {
        let mut compiler = Compiler::new(
            HeapPtr::alloc(Scanner::new()),
            self.heap.clone(),
            FunctionType::Script,
        )
        .dealloc_on_drop();
        let function = compiler
            .compile(source)
            .map_err(|e| Error::CompileTime(e.to_string()))?;

        self.stack.push(Value::Object(ObjectNode::alloc(
            Object::Function(function.clone()),
            &mut self.heap,
        )));
        self.callframes.push(CallFrame {
            ip: function.chunk.get_code_ptr(),
            function: function.clone(),
            stack_base_offset: 0,
        });
        self.call(function, 0)?;

        loop {
            #[cfg(debug_assertions)]
            unsafe {
                println!("\n{:?} ", &self.stack);
                let frame = self.callframes.top_mut();
                let offset = frame.ip.offset_from(frame.function.chunk.get_code_ptr());
                disassemble_instruction(&mut frame.function.chunk, offset as usize);
            }
            let instruction = self.read_byte();
            match OpCode::from_code(instruction) {
                OpCode::Return => {
                    let result = self.stack.pop();
                    let frame = self.callframes.pop();
                    if self.callframes.count() == 1 {
                        self.stack.pop(); // Pop the "main" function itself (exiting the program)
                        return Ok(());
                    }
                    self.stack.pop_n(frame.function.arity as usize + 1); // Pop arguments and the function itself
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
                    frame.ip = unsafe { frame.ip.sub(offset as usize) };
                }
                OpCode::Jump => {
                    let offset = self.read_double();
                    let frame = self.callframes.top_mut();
                    frame.ip = unsafe { frame.ip.add(offset as usize) };
                }
                OpCode::JumpIfTrue => {
                    let offset = self.read_double();
                    let frame = self.callframes.top_mut();
                    if !self.stack.peek(0).is_falsey() {
                        frame.ip = unsafe { frame.ip.add(offset as usize) };
                    }
                }
                OpCode::JumpIfFalse => {
                    let offset = self.read_double();
                    let frame = self.callframes.top_mut();
                    if self.stack.peek(0).is_falsey() {
                        frame.ip = unsafe { frame.ip.add(offset as usize) };
                    }
                }
                OpCode::SetLocal => {
                    let slot = self.read_double();
                    let frame = self.callframes.top_ref();
                    let value = self.stack.peek(0).clone();
                    dbg_println!("SETTING LOCAL {} = {}", slot, value);
                    self.stack
                        .offset(frame.stack_base_offset)
                        .set_at(slot as usize, value);
                }
                OpCode::GetLocal => {
                    let slot = self.read_double();
                    let frame = self.callframes.top_ref();
                    let local = self
                        .stack
                        .offset(frame.stack_base_offset)
                        .get_at(slot as usize);
                    dbg_println!("GETTING LOCAL {} = {}", slot, local);
                    self.stack.push(local);
                }
                OpCode::PopN => {
                    let n = self.read_double();
                    self.stack.pop_n(n as usize);
                }
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::SetGlobal => {
                    let name_string = self.read_constant_string()?;
                    let value = self.stack.peek(0).clone();
                    dbg_println!("SETTING GLOBAL {} = {}", name_string, value);
                    if self.heap.globals.set(name_string.clone(), Some(value)) {
                        // If the variable did not already exist at this point, return error
                        self.heap.globals.delete(&name_string);
                        return Err(Error::Runtime(format!(
                            "Undefined variable '{}'",
                            name_string
                        )));
                    }
                }
                OpCode::GetGlobal => {
                    let name_string = self.read_constant_string()?;
                    let global = self.heap.globals.get(&name_string);
                    match global {
                        Some(global) => {
                            let global_value = global.value.clone().ok_or_else(|| {
                                Error::Runtime(format!("Variable '{}' had no value", name_string))
                            })?;
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
                    let name_string = self.read_constant_string()?;
                    let global_value = self.stack.peek(0).clone();
                    dbg_println!("DEFINING GLOBAL: {} = ({})", name_string, global_value);
                    self.heap.globals.set(name_string, Some(global_value));
                    self.stack.pop();
                }
                OpCode::Constant => {
                    let constant = self.read_constant_quad().clone();
                    dbg_println!("PUSHING CONSTANT: {}", constant);
                    self.stack.push(constant);
                }
                OpCode::None => self.stack.push(Value::None),
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::False => self.stack.push(Value::Bool(false)),
                OpCode::Is => binary_op_from_bool!(self.stack, ==),
                OpCode::IsNot => binary_op_from_bool!(self.stack, !=),
                OpCode::Greater => binary_op_from_bool!(self.stack, >),
                OpCode::GreaterEqual => binary_op_from_bool!(self.stack, >=),
                OpCode::Less => binary_op_from_bool!(self.stack, <),
                OpCode::LessEqual => binary_op_from_bool!(self.stack, <=),
                OpCode::Add => {
                    let second = self.stack.pop();
                    let first = self.stack.pop();
                    if let Value::Object(first) = first
                        && let Value::Object(second) = second
                    {
                        match first.get_object() {
                            Object::String(a_str) => match &*second.get_object() {
                                Object::String(b_str) => {
                                    let concat = b_str.concat(&a_str, &mut self.heap);
                                    let new_string = Value::Object(ObjectNode::alloc(
                                        Object::String(concat),
                                        &mut self.heap,
                                    ));
                                    self.stack.push(new_string) // Can "take" pointer value because the pointer will be appended to VM list, so no leak.
                                }
                                _ => {
                                    return Err(Error::Runtime(format!(
                                        "Cannot add string and non-string object: {:?} + {:?}",
                                        a_str, second
                                    )));
                                }
                            },
                            Object::Function(_) => {
                                return Err(Error::Runtime(format!("Cannot add two functions",)));
                            }
                            Object::NativeFunction(_) => {
                                return Err(Error::Runtime(format!(
                                    "Cannot add two native functions"
                                )));
                            }
                        }
                    } else {
                        let result = first + second;
                        self.stack.push(result?);
                    }
                }
                OpCode::Subtract => binary_op_try!(self.stack, -),
                OpCode::Multiply => binary_op_try!(self.stack, *),
                OpCode::Divide => binary_op_try!(self.stack, /),
                OpCode::Not => {
                    let val = self.stack.pop();
                    self.stack.push(Value::Bool(val.is_falsey()));
                }
                OpCode::Negate => {
                    let val = self.stack.top_mut_offset(0);
                    *val = (-(*val).clone())?;
                }
            }
        }
    }

    fn free_objects(&self) {
        let mut obj_container_ptr = self.heap.get_objects_head();
        while !obj_container_ptr.is_null() {
            let next_obj_container_ptr = obj_container_ptr.get_next_object_ptr();

            obj_container_ptr.dealloc();

            obj_container_ptr = next_obj_container_ptr;
        }
    }
}

impl Drop for Vm {
    fn drop(&mut self) {
        self.free_objects();
        self.heap.dealloc();
    }
}

use super::{VmHeap, callframe::CallFrame, opcode::OpCode};
#[cfg(debug_assertions)]
use crate::debug::disassemble_instruction;
use crate::{
    collections::{HashTable, Stack},
    compiler::{Compiler, FunctionType},
    dbg_println,
    error::{Error, Result},
    lexing::scanner::Scanner,
    memory::{Dealloc, HeapPtr},
    value::{
        Value,
        object::{Function, InternedString, NativeFunction, Object, ObjectManager, ObjectNode},
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
        $stack.push(Value::boolean(val));
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
                objects: ObjectManager::new(),
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
        for frame in self.callframes.iter() {
            let instruction = unsafe {
                let ip_offset = frame.ip.offset_from(frame.function.chunk.get_code_ptr());
                frame.ip.sub((ip_offset as usize) - 1).read()
            }; // -1 to get the previous instruction that failed

            let function_name = frame.function.get_name();
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
                    .get_name()
                    .as_ref()
                    .map(|x| x.as_str())
                    .unwrap_or("<script>")
            )));
        }

        if self.callframes.count() >= MAX_CALLFRAMES {
            return Err(Error::Runtime("Call stack overflow".to_owned()));
        }

        let frame = self.callframes.top_mut();
        frame.ip = function.chunk.get_code_ptr();
        frame.function = function;
        frame.stack_offset = self.stack.count() - arg_count as usize - 1; // -1 for the function itself
        Ok(())
    }

    fn call_value(&mut self, callee: Value, arg_count: u8) -> Result<()> {
        if callee.is_object() {
            let obj_ptr = callee.as_object_ptr();
            match unsafe { obj_ptr.assume_init_ref().get_object() } {
                Object::Function(func) => return self.call(func.clone(), arg_count),
                Object::NativeFunction(native_func) => {
                    let args = self.stack.pop_n(arg_count as usize);
                    let result = (native_func.function)(arg_count, args)?;
                    self.stack.push(result);
                }
                _ => return Err(Error::Runtime("Cannot call non-function object".to_owned())),
            }
        }
        Ok(())
    }

    pub fn register_native_function(&mut self, name: &str, function: NativeFunction) {
        let func_name = InternedString::new(name, &mut self.heap);
        let func_value = Value::object(
            ObjectNode::alloc(Object::NativeFunction(function), &mut self.heap.objects).read(),
        );

        self.stack.push(Value::object(
            ObjectNode::alloc(Object::String(func_name.clone()), &mut self.heap.objects).read(),
        ));
        self.stack.push(func_value.clone());
        self.heap.globals.set(func_name, Some(func_value));
        self.stack.pop();
        self.stack.pop();

        // The stack pushes and pops are due to future garbage collection proofing.
    }

    pub fn interpret<'src>(&mut self, source: &'src [u8]) -> Result<()> {
        let mut compiler = Compiler::new(
            HeapPtr::alloc(Scanner::new()),
            self.heap.clone(),
            FunctionType::Script,
        );
        let function = compiler
            .compile_source(source)
            .map_err(|e| Error::CompileTime(e.to_string()))?;

        self.callframes.push(CallFrame {
            ip: function.chunk.get_code_ptr(),
            function: function.clone(),
            stack_offset: 0,
        });
        self.call(function, 0)?;

        loop {
            #[cfg(debug_assertions)]
            {
                print!("\n");
                print!("{:?}", &self.stack);
                unsafe {
                    let frame = self.callframes.top_mut();
                    let offset = frame.ip.offset_from(frame.function.chunk.get_code_ptr());
                    disassemble_instruction(&mut frame.function.chunk, offset as usize);
                }
                //println!("DEBUG HEAP: {:?}", &self.heap);
                print!("\t");
            }

            let instruction = self.read_byte();
            match OpCode::from_code(instruction) {
                OpCode::Return => {
                    let result = self.stack.pop();
                    self.callframes.pop();
                    if self.callframes.count() < 1 {
                        self.stack.pop(); // Pop the "main" function itself (exiting the program)
                        return Ok(());
                    }
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
                        .offset(frame.stack_offset)
                        .set_at(slot as usize, value);
                }
                OpCode::GetLocal => {
                    let slot = self.read_double();
                    let frame = self.callframes.top_ref();
                    let local = self.stack.offset(frame.stack_offset).get_at(slot as usize);
                    dbg_println!("GETTING LOCAL {} = {}", slot, local);
                    self.stack.push(local);
                }
                OpCode::PopN => {
                    let n = self.read_double();
                    for _ in 0..n {
                        self.stack.pop();
                    }
                }
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::SetGlobal => {
                    let name_value = self.read_constant_quad();
                    let name_object = unsafe { name_value.as_object_ptr().assume_init() };
                    let name_object_string = match name_object.get_object() {
                        Object::String(s) => s.clone(),
                        _ => {
                            return Err(Error::Runtime(format!(
                                "Expected string object for global name, got: {:?}",
                                name_object.get_object()
                            )));
                        }
                    };

                    let value = self.stack.peek(0).clone();
                    dbg_println!("SETTING GLOBAL {} = {}", name_object_string, value);
                    if self
                        .heap
                        .globals
                        .set(name_object_string.clone(), Some(value))
                    {
                        // If the variable did not already exist at this point, return error
                        self.heap.globals.delete(&name_object_string);
                        return Err(Error::Runtime(format!(
                            "Undefined variable '{}'",
                            name_object_string
                        )));
                    }
                }
                OpCode::GetGlobal => {
                    let name_value = self.read_constant_quad();
                    let name_object = unsafe { name_value.as_object_ptr().assume_init() };
                    let name_object_string = match name_object.get_object() {
                        Object::String(s) => s.clone(),
                        _ => {
                            return Err(Error::Runtime(format!(
                                "Expected string object for global name, got: {:?}",
                                name_object.get_object()
                            )));
                        }
                    };

                    let global = self.heap.globals.get(&name_object_string);
                    match global {
                        Some(global) => {
                            let global_value = global.value.clone().ok_or_else(|| {
                                Error::Runtime(format!(
                                    "Variable '{}' had no value",
                                    name_object_string
                                ))
                            })?;
                            dbg_println!(
                                "GETTING GLOBAL: {} = ({})",
                                name_object_string,
                                global_value
                            );
                            self.stack.push(global_value);
                        }
                        None => {
                            return Err(Error::Runtime(format!(
                                "Undefined variable '{}'",
                                name_object_string
                            )));
                        }
                    }
                }
                OpCode::DefineGlobal => {
                    let name_value = self.read_constant_quad();
                    let name_object = unsafe { name_value.as_object_ptr().assume_init() };
                    let name_object_string = match name_object.get_object() {
                        Object::String(s) => s.clone(),
                        _ => {
                            return Err(Error::Runtime(format!(
                                "Expected string object for global name, got: {:?}",
                                name_object.get_object()
                            )));
                        }
                    };

                    let global_value = self.stack.peek(0).clone();
                    dbg_println!(
                        "DEFINING GLOBAL: {} = ({})",
                        name_object_string,
                        global_value
                    );
                    self.heap
                        .globals
                        .set(name_object_string, Some(global_value));
                    self.stack.pop();
                }
                OpCode::Constant => {
                    let constant = self.read_constant_quad().clone();
                    dbg_println!("PUSHING CONSTANT: {}", constant);
                    self.stack.push(constant);
                }
                OpCode::None => self.stack.push(Value::none()),
                OpCode::True => self.stack.push(Value::boolean(true)),
                OpCode::False => self.stack.push(Value::boolean(false)),
                OpCode::Is => binary_op_from_bool!(self.stack, ==),
                OpCode::IsNot => binary_op_from_bool!(self.stack, !=),
                OpCode::Greater => binary_op_from_bool!(self.stack, >),
                OpCode::GreaterEqual => binary_op_from_bool!(self.stack, >=),
                OpCode::Less => binary_op_from_bool!(self.stack, <),
                OpCode::LessEqual => binary_op_from_bool!(self.stack, <=),
                OpCode::Add => {
                    let second = self.stack.pop();
                    let first = self.stack.pop();
                    if first.is_object() && second.is_object() {
                        let first = first.as_object_ptr();
                        let first = unsafe { first.assume_init_ref() };
                        let second = second.as_object_ptr();
                        let second = unsafe { second.assume_init_ref() };
                        match &*first.get_object() {
                            Object::String(a_str) => match &*second.get_object() {
                                Object::String(b_str) => {
                                    let concat = b_str.concat(&a_str, &mut self.heap);
                                    let new_string = Value::object(
                                        ObjectNode::alloc(
                                            Object::String(concat),
                                            &mut self.heap.objects,
                                        )
                                        .take(),
                                    );
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
                    self.stack.push(Value::boolean(val.is_falsey()));
                }
                OpCode::Negate => {
                    let val = self.stack.top_mut_offset(0);
                    *val = (-(*val).clone())?;
                }
            }
        }
    }

    fn free_objects(&self) {
        let mut obj_container_ptr = self.heap.objects.get_objects_head();
        while !obj_container_ptr.is_null() {
            let next_obj_container_ptr = obj_container_ptr.get().get_next_object_ptr();

            let mut obj_container = obj_container_ptr.take();
            obj_container.dealloc();

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

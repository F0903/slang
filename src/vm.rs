use {
    crate::{
        chunk::Chunk,
        compiler::Compiler,
        memory::{Dealloc, ManualPtr},
        opcode::OpCode,
        stack::Stack,
        value::{Object, ObjectContainer, Value},
    },
    std::{error::Error, ffi::CString, fmt::Display, ptr::null_mut},
};

#[cfg(debug_assertions)]
use crate::debug::disassemble_instruction;

#[derive(Debug)]
pub enum InterpretError {
    CompileTime(String),
    Runtime(String),
}

impl Display for InterpretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompileTime(msg) => {
                f.write_fmt(format_args!("Encountered compile-time error.\n{}", msg))
            }
            Self::Runtime(msg) => f.write_fmt(format_args!("Encountered runtime error.\n{}", msg)),
        }
    }
}

impl Error for InterpretError {}

type InterpretResult = Result<(), InterpretError>;

macro_rules! binary_op_result {
    ($stack: expr, $op: tt) => {{
        let b = $stack.pop();
        let a = $stack.pop();
        println!("BINARY_OP: {} {} {}", a, stringify!($op), b);
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

// Figure ways around this global variable at some point.
pub static mut GLOBAL_VM: VM = VM::new();

pub struct VM {
    ip: *mut u8,
    stack: Stack<Value>,
    objects_head: ManualPtr<ObjectContainer>,
}

impl VM {
    pub(self) const fn new() -> Self {
        Self {
            ip: null_mut(),
            stack: Stack::new(),
            objects_head: ManualPtr::null(),
        }
    }

    pub const fn get_objects_head(&self) -> ManualPtr<ObjectContainer> {
        self.objects_head
    }

    //TODO: make const when stable
    pub fn set_objects_head(&mut self, object: ManualPtr<ObjectContainer>) {
        self.objects_head = object;
    }

    fn read_byte(&mut self) -> u8 {
        unsafe {
            let val = *self.ip;
            self.ip = self.ip.add(1);
            val
        }
    }

    fn read_long(&mut self) -> u32 {
        unsafe {
            let val = *self.ip.cast::<u32>();
            self.ip = self.ip.add(4);
            val
        }
    }

    fn read_constant_long<'a>(&mut self, chunk: &'a mut Chunk) -> &'a Value {
        let index = self.read_long();
        chunk.get_constant(index)
    }

    fn read_constant<'a>(&mut self, chunk: &'a mut Chunk) -> &'a Value {
        let index = self.read_byte();
        chunk.get_constant(index as u32)
    }

    pub fn interpret(&mut self, source: impl Into<Vec<u8>>) -> InterpretResult {
        let source = CString::new(source).map_err(|_| {
            InterpretError::CompileTime("Could not create a CString from source!".to_owned())
        })?;

        let mut compiler = Compiler::new();
        let chunk = compiler
            .compile(source.as_bytes())
            .map_err(|e| InterpretError::CompileTime(e.to_string()))?;

        self.ip = chunk.borrow().get_code_ptr();

        loop {
            #[cfg(debug_assertions)]
            {
                print!("{:?}", &self.stack);
                unsafe {
                    let offset = self.ip.offset_from(chunk.borrow().get_code_ptr());
                    disassemble_instruction(&mut chunk.borrow_mut(), offset as usize);
                }
            }

            let instruction = self.read_byte();
            match instruction.into() {
                OpCode::ConstantLong => {
                    let chunk = &mut chunk.borrow_mut();
                    let constant = self.read_constant_long(chunk);
                    self.stack.push(constant.clone());
                }
                OpCode::Constant => {
                    let chunk = &mut chunk.borrow_mut();
                    let constant = self.read_constant(chunk);
                    self.stack.push(constant.clone());
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
                    let first = self.stack.pop();
                    let second = self.stack.pop();
                    if first.is_object() && second.is_object() {
                        let first = first.as_object_ptr();
                        let second = second.as_object_ptr();
                        match &*first.get_object() {
                            Object::String(a_str) => match &*second.get_object() {
                                Object::String(b_str) => {
                                    let concat = b_str.concat(&a_str);
                                    self.stack.push(Value::object(
                                        ObjectContainer::alloc(Object::String(concat)).take(),
                                    )) // Can "take" pointer value because the pointer will be appended to VM list, so no leak.
                                }
                                _ => (),
                            },
                            _ => (),
                        }
                    } else {
                        binary_op_try!(self.stack, +)
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
                    let val = self.stack.get_top_mut_ref();
                    *val = (-(*val).clone())?;
                }
                OpCode::Return => {
                    println!("{}", self.stack.pop());
                    return Ok(());
                }
            }
        }
    }

    pub fn free_objects(&self) {
        let mut obj_container_ptr = self.objects_head;
        while !obj_container_ptr.is_null() {
            let next_obj_container_ptr = obj_container_ptr.get().get_next_object_ptr();

            let mut obj_container = obj_container_ptr.take();
            obj_container.dealloc();
            obj_container_ptr.dealloc();

            obj_container_ptr = next_obj_container_ptr;
        }
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        self.free_objects();
    }
}

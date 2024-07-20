use {
    crate::{
        chunk::Chunk,
        compiler::Compiler,
        memory::reallocate,
        opcode::OpCode,
        stack::Stack,
        value::{Object, ObjectType, RawString, Value},
    },
    std::{cell::LazyCell, error::Error, ffi::CString, fmt::Display, ptr::null_mut},
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

pub static mut GLOBAL_VM: VM = VM::new(); // forgive me for i have sinned

pub struct VM {
    ip: *mut u8,
    stack: Stack<Value>,
    objects_head: *mut Object,
}

impl VM {
    pub(self) const fn new() -> Self {
        Self {
            ip: null_mut(),
            stack: Stack::new(),
            objects_head: null_mut(),
        }
    }

    pub fn get_objects_head(&self) -> *mut Object {
        self.objects_head
    }

    pub fn set_objects_head(&mut self, objects: *mut Object) {
        self.objects_head = objects;
    }

    fn read_byte(&mut self) -> u8 {
        unsafe {
            let val = self.ip.read();
            self.ip = self.ip.add(1);
            val
        }
    }

    fn read_long(&mut self) -> u32 {
        unsafe {
            let val = self.ip.cast::<u32>().read();
            self.ip = self.ip.add(4);
            val
        }
    }

    fn read_constant_long(&mut self, chunk: &mut Chunk) -> Value {
        let index = self.read_long();
        chunk.get_constant(index)
    }

    fn read_constant(&mut self, chunk: &mut Chunk) -> Value {
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
                    let constant = self.read_constant_long(&mut chunk.borrow_mut());
                    self.stack.push(constant);
                }
                OpCode::Constant => {
                    let constant = self.read_constant(&mut chunk.borrow_mut());
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
                    let first = self.stack.pop();
                    let second = self.stack.pop();
                    if first.is_object() && second.is_object() {
                        let first = first.as_object();
                        let second = second.as_object();
                        if unsafe { first.read().get_type() } == ObjectType::String
                            && unsafe { first.read().get_type() } == ObjectType::String
                        {
                            let first = first.cast::<RawString>();
                            let second = second.cast::<RawString>();
                            let concat = unsafe { first.read().concat(&second.read()) };
                            self.stack.push(Value::object(concat.cast()))
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

    fn free_objects(&self) {
        unsafe {
            let mut obj_ptr = self.objects_head;
            while obj_ptr != null_mut() {
                let obj = obj_ptr.read();
                let next = obj.get_next_object();
                match obj.get_type() {
                    ObjectType::String => {
                        let string_ptr = obj_ptr.cast::<RawString>();
                        let string = string_ptr.read();
                        reallocate::<u8>(string.get_char_ptr(), string.get_len(), 0);
                        reallocate::<RawString>(string_ptr.cast(), 1, 0);
                    }
                }
                obj_ptr = next;
            }
        }
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        self.free_objects();
    }
}

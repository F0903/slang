mod interpret_error;
mod vm_heap;

use std::{cell::RefCell, rc::Rc};

pub use interpret_error::InterpretError;
pub use vm_heap::VmHeap;

use {
    crate::{
        chunk::Chunk,
        collections::{HashTable, Stack},
        compiler::Compiler,
        memory::Dealloc,
        memory::HeapPtr,
        opcode::OpCode,
        value::{
            Value,
            object::{Object, ObjectContainer, ObjectManager},
        },
    },
    interpret_error::InterpretResult,
    std::{ffi::CString, ptr::null_mut},
};

#[cfg(debug_assertions)]
use crate::debug::disassemble_instruction;
use crate::{dbg_println, lexing::scanner::Scanner};

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
    ip: *mut u8,
    stack: Stack<Value>,
    heap: HeapPtr<VmHeap>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            ip: null_mut(),
            stack: Stack::new(),
            heap: HeapPtr::alloc(VmHeap {
                objects: ObjectManager::new(),
                interned_strings: HashTable::new(),
                globals: HashTable::new(),
            }),
        }
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

        let chunk = Chunk::new();
        let mut compiler = Compiler::new(
            Scanner::new(),
            self.heap.clone(),
            Rc::new(RefCell::new(chunk)),
        );
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
                println!("DEBUG HEAP: {:?}", &self.heap);
            }

            let chunk = &mut chunk.borrow_mut();
            let instruction = self.read_byte();
            match instruction.into() {
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::SetGlobal => {
                    let name_value = self.read_constant_long(chunk);
                    let name_object: ObjectContainer =
                        unsafe { name_value.as_object_ptr().assume_init() };
                    let name_object_string = match name_object.get_object() {
                        Object::String(s) => s.clone(),
                    };
                    if self
                        .heap
                        .globals
                        .set(name_object_string.clone(), Some(self.stack.peek(0).clone()))
                    {
                        // If the variable did not already exist at this point, return error
                        self.heap.globals.delete(&name_object_string);
                        return Err(InterpretError::Runtime(format!(
                            "Undefined variable '{}'",
                            name_object_string
                        )));
                    }
                }
                OpCode::GetGlobal => {
                    let name_value = self.read_constant_long(chunk);
                    let name_object = unsafe { name_value.as_object_ptr().assume_init() };
                    let name_object_string = match name_object.get_object() {
                        Object::String(s) => s.clone(),
                    };
                    let global = self.heap.globals.get(&name_object_string);
                    match global {
                        Some(global) => {
                            self.stack.push(global.value.clone().ok_or_else(|| {
                                InterpretError::Runtime(format!(
                                    "Variable '{}' had no value",
                                    name_object_string
                                ))
                            })?);
                        }
                        None => {
                            return Err(InterpretError::Runtime(format!(
                                "Undefined variable '{}'",
                                name_object_string
                            )));
                        }
                    }
                }
                OpCode::DefineGlobal => {
                    let name_value = self.read_constant_long(chunk);
                    let name_object = unsafe { name_value.as_object_ptr().assume_init() };
                    let name_object_string = match name_object.get_object() {
                        Object::String(s) => s.clone(),
                    };
                    self.heap
                        .globals
                        .set(name_object_string, Some(self.stack.peek(0).clone()));
                }
                OpCode::Constant => {
                    let constant = self.read_constant_long(chunk);
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
                        let first = unsafe { first.assume_init_ref() };
                        let second = second.as_object_ptr();
                        let second = unsafe { second.assume_init_ref() };
                        match &*first.get_object() {
                            Object::String(a_str) => match &*second.get_object() {
                                Object::String(b_str) => {
                                    let concat = b_str.concat(&a_str, &mut self.heap);
                                    let new_string = Value::object(
                                        ObjectContainer::alloc(
                                            Object::String(concat),
                                            &mut self.heap.objects,
                                        )
                                        .take(),
                                    );
                                    self.stack.push(new_string) // Can "take" pointer value because the pointer will be appended to VM list, so no leak.
                                }
                            },
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
                    dbg_println!("{}", self.stack.pop());
                    return Ok(());
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
    }
}

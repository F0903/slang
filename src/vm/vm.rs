use super::VmHeap;

use crate::{
    chunk::Chunk,
    collections::{HashTable, Stack},
    compiler::Compiler,
    error::{Error, Result},
    memory::{Dealloc, HeapPtr},
    opcode::OpCode,
    value::{
        Value,
        object::{Object, ObjectManager, ObjectNode},
    },
};

use std::ptr::null_mut;

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
            let val = self.ip.read();
            self.ip = self.ip.add(1);
            val
        }
    }

    fn read_double(&mut self) -> u16 {
        unsafe {
            let val = self.ip.cast::<u16>().read();
            self.ip = self.ip.add(2);
            val
        }
    }

    fn read_quad(&mut self) -> u32 {
        unsafe {
            let val = self.ip.cast::<u32>().read();
            self.ip = self.ip.add(4);
            val
        }
    }

    /// Reads a constant from the chunk with a u32 index.
    fn read_constant_quad<'a>(&mut self, chunk: &'a mut Chunk) -> &'a Value {
        let index = self.read_quad();
        chunk.get_constant(index)
    }

    /// Reads a constant from the chunk with a u8 index.
    fn read_constant<'a>(&mut self, chunk: &'a mut Chunk) -> &'a Value {
        let index = self.read_byte();
        chunk.get_constant(index as u32)
    }

    pub fn interpret<'src>(&mut self, source: &'src [u8]) -> Result<()> {
        let mut compiler = Compiler::new(
            Scanner::new(),
            self.heap.clone(),
            HeapPtr::alloc(Chunk::new()),
        );
        let mut chunk = compiler
            .compile(source)
            .map_err(|e| Error::CompileTime(e.to_string()))?
            .dealloc_on_drop();

        self.ip = chunk.get_code_ptr();

        loop {
            #[cfg(debug_assertions)]
            {
                print!("\n");
                print!("{:?}", &self.stack);
                unsafe {
                    let offset = self.ip.offset_from(chunk.get_code_ptr());
                    disassemble_instruction(&mut chunk, offset as usize);
                }
                //println!("DEBUG HEAP: {:?}", &self.heap);
                print!("\t");
            }

            let chunk = &mut chunk;
            let instruction = self.read_byte();
            match OpCode::from_code(instruction) {
                OpCode::Backjump => {
                    let offset = self.read_double();
                    self.ip = unsafe { self.ip.sub(offset as usize) };
                }
                OpCode::Jump => {
                    let offset = self.read_double();
                    self.ip = unsafe { self.ip.add(offset as usize) };
                }
                OpCode::JumpIfTrue => {
                    let offset = self.read_double();
                    if !self.stack.peek(0).is_falsey() {
                        self.ip = unsafe { self.ip.add(offset as usize) };
                    }
                }
                OpCode::JumpIfFalse => {
                    let offset = self.read_double();
                    if self.stack.peek(0).is_falsey() {
                        self.ip = unsafe { self.ip.add(offset as usize) };
                    }
                }
                OpCode::SetLocal => {
                    let slot = self.read_double();
                    let value = self.stack.peek(0).clone();
                    dbg_println!("SETTING LOCAL {} = {}", slot, value);
                    self.stack.set_at(slot as usize, value);
                }
                OpCode::GetLocal => {
                    let slot = self.read_double();
                    let local = self.stack.get_at(slot as usize);
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
                    let name_value = self.read_constant_quad(chunk);
                    let name_object = unsafe { name_value.as_object_ptr().assume_init() };
                    let name_object_string = match name_object.get_object() {
                        Object::String(s) => s.clone(),
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
                    let name_value = self.read_constant_quad(chunk);
                    let name_object = unsafe { name_value.as_object_ptr().assume_init() };
                    let name_object_string = match name_object.get_object() {
                        Object::String(s) => s.clone(),
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
                    let name_value = self.read_constant_quad(chunk);
                    let name_object = unsafe { name_value.as_object_ptr().assume_init() };
                    let name_object_string = match name_object.get_object() {
                        Object::String(s) => s.clone(),
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
                    let constant = self.read_constant_quad(chunk);
                    dbg_println!("PUSHING CONSTANT: {}", constant);
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
                            },
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
                    let val = self.stack.peek_mut(0);
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
        self.heap.dealloc();
    }
}

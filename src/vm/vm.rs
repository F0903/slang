use std::ptr::null_mut;

use super::{VmHeap, callframe::CallFrame, opcode::OpCode};
#[cfg(debug_assertions)]
use crate::debug::disassemble_instruction;
use crate::{
    collections::{HashTable, Stack},
    compiler::{Compiler, FunctionType, chunk::Chunk},
    dbg_println,
    error::{Error, Result},
    lexing::scanner::Scanner,
    memory::{Dealloc, HeapPtr},
    value::{
        Value,
        object::{Object, ObjectManager, ObjectNode},
    },
};

pub const STACK_SIZE: usize = 1024;
pub const CALLFRAMES_SIZE: usize = 256;

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
    stack: Stack<Value, STACK_SIZE>,
    heap: HeapPtr<VmHeap>,
    callframes: Stack<CallFrame, CALLFRAMES_SIZE>,
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

    pub fn interpret<'src>(&mut self, source: &'src [u8]) -> Result<()> {
        let mut compiler = Compiler::new(Scanner::new(), self.heap.clone(), FunctionType::Script);
        let function = compiler
            .compile(source)
            .map_err(|e| Error::CompileTime(e.to_string()))?;

        self.callframes.push(CallFrame {
            ip: function.chunk.get_code_ptr(),
            function,
            stack_offset: 0,
        });

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

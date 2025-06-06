use std::fmt::Debug;

use crate::value::{
    Value,
    object::{Closure, ObjectRef},
};

#[derive(Clone)]
pub struct CallFrame {
    closure: ObjectRef<Closure>,
    ip: *mut u8,
    /// Pointer to the point in the stack where this CallFrame begins.
    /// Since CallFrames only live within the VM, this is always valid.
    slots: *mut Value,
}

impl CallFrame {
    pub const fn new(closure: ObjectRef<Closure>, ip: *mut u8, slots: *mut Value) -> Self {
        Self { closure, ip, slots }
    }

    pub const fn get_ip(&mut self) -> *mut u8 {
        self.ip
    }

    pub const fn add_ip(&mut self, add: usize) {
        unsafe {
            self.ip = self.ip.add(add);
        }
    }

    pub const fn sub_ip(&mut self, sub: usize) {
        unsafe {
            self.ip = self.ip.sub(sub);
        }
    }

    pub const fn get_slots_raw(&self) -> *const Value {
        self.slots
    }

    pub const fn get_slots_raw_mut(&mut self) -> *mut Value {
        self.slots
    }

    pub const fn set_slot(&mut self, index: usize, value: Value) {
        unsafe {
            self.slots.add(index).write(value);
        }
    }

    pub const fn get_slot_ref(&self, index: usize) -> &Value {
        unsafe { self.slots.add(index).as_ref_unchecked() }
    }

    pub const fn get_slot_mut(&mut self, index: usize) -> &mut Value {
        unsafe { self.slots.add(index).as_mut_unchecked() }
    }

    pub fn get_closure(&self) -> ObjectRef<Closure> {
        self.closure.clone()
    }
}

impl Debug for CallFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Call Frame | ip: {:?} | slots: {:?} | {}",
            self.ip,
            self.slots,
            self.closure.as_ref()
        ))
    }
}

use std::{fmt::Debug, ptr::NonNull};

use crate::value::{
    Value,
    object::{Closure, ObjectRef},
};

#[derive(Clone)]
pub struct CallFrame {
    closure: ObjectRef<Closure>,
    ip: NonNull<u8>,
    /// Pointer to the point in the stack where this CallFrame begins.
    /// Since CallFrames only live within the VM, this is always valid.
    slots: NonNull<Value>,
}

impl CallFrame {
    #[inline]
    pub const fn new(closure: ObjectRef<Closure>, ip: NonNull<u8>, slots: NonNull<Value>) -> Self {
        Self { closure, ip, slots }
    }

    #[inline]
    pub const fn get_ip(&mut self) -> NonNull<u8> {
        self.ip
    }

    #[inline]
    pub const fn add_ip(&mut self, add: usize) {
        // SAFETY: We are adding a valid offset to the instruction pointer, which is guaranteed to be valid as long as the CallFrame is alive.
        unsafe {
            self.ip = self.ip.add(add);
        }
    }

    #[inline]
    pub const fn sub_ip(&mut self, sub: usize) {
        // SAFETY: We are subtracting a valid offset from the instruction pointer, which is guaranteed to be valid as long as the CallFrame is alive.
        unsafe {
            self.ip = self.ip.sub(sub);
        }
    }

    #[inline]
    pub const fn get_slots_raw(&self) -> NonNull<Value> {
        self.slots
    }

    #[inline]
    pub const fn set_slot(&mut self, index: usize, value: Value) {
        // SAFETY: We are writing to a valid slot in the CallFrame's stack, which is guaranteed to be valid as long as the CallFrame is alive.
        unsafe {
            self.slots.add(index).write(value);
        }
    }

    #[inline]
    pub const fn get_slot_ref(&self, index: usize) -> &Value {
        // SAFETY: We are reading from a valid slot in the CallFrame's stack, which is guaranteed to be valid as long as the CallFrame is alive.
        unsafe { self.slots.add(index).as_ref() }
    }

    #[inline]
    pub const fn get_slot_mut(&mut self, index: usize) -> &mut Value {
        // SAFETY: We are writing to a valid slot in the CallFrame's stack, which is guaranteed to be valid as long as the CallFrame is alive.
        unsafe { self.slots.add(index).as_mut() }
    }

    #[inline]
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

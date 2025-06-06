use std::{
    fmt::{Debug, Display},
    mem::ManuallyDrop,
};

use crate::{
    dbg_println,
    memory::{Dealloc, HeapPtr},
    value::object::{self, Closure, Function, NativeFunction, ObjectRef},
    vm::VmHeap,
};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ObjectType {
    Function,
    NativeFunction,
    Closure,
    Upvalue,
}

pub union ObjectUnion {
    function: ManuallyDrop<Function>,
    native_function: ManuallyDrop<NativeFunction>,
    closure: ManuallyDrop<Closure>,
    upvalue: ManuallyDrop<object::Upvalue>,
}

pub struct Object {
    obj_type: ObjectType,
    casts: ObjectUnion,
    next: HeapPtr<Object>,
}

macro_rules! object_ctor {
    ($name:ident, $variant:ident, $ty:ty, $tag:expr) => {
        #[inline]
        pub fn $name(val: $ty, heap: &mut VmHeap) -> HeapPtr<Self> {
            let inner = ObjectUnion {
                $variant: ManuallyDrop::new(val),
            };
            Self::alloc($tag, inner, heap)
        }
    };
}

macro_rules! object_as_fn {
    ($fn_name:ident, $variant:ident, $ty:ty, $tag:expr) => {
        #[inline]
        pub fn $fn_name(&self) -> ObjectRef<$ty> {
            debug_assert!(
                self.obj_type == $tag,
                concat!(
                    "Tried to access a ",
                    stringify!($tag),
                    " object as a ",
                    stringify!($ty),
                    "!"
                )
            );
            let ptr = unsafe { &self.casts.$variant };
            ObjectRef::new(ptr as *const ManuallyDrop<$ty> as *const $ty)
        }
    };
}

impl Object {
    fn alloc(obj_type: ObjectType, inner: ObjectUnion, heap: &mut VmHeap) -> HeapPtr<Self> {
        dbg_println!("DEBUG OBJECT ALLOC: {:?}", obj_type);

        let me = HeapPtr::alloc(Self {
            obj_type,
            casts: inner,
            next: heap.get_objects_head(),
        });
        heap.set_objects_head(me);
        me
    }

    pub const fn get_next_object_ptr(&self) -> HeapPtr<Object> {
        self.next
    }

    pub const fn get_type(&self) -> ObjectType {
        self.obj_type
    }

    object_ctor!(new_function, function, Function, ObjectType::Function);
    object_ctor!(
        new_native_function,
        native_function,
        NativeFunction,
        ObjectType::NativeFunction
    );
    object_ctor!(new_closure, closure, Closure, ObjectType::Closure);
    object_ctor!(new_upvalue, upvalue, object::Upvalue, ObjectType::Upvalue);

    object_as_fn!(as_function, function, Function, ObjectType::Function);
    object_as_fn!(
        as_native_function,
        native_function,
        NativeFunction,
        ObjectType::NativeFunction
    );
    object_as_fn!(as_closure, closure, Closure, ObjectType::Closure);
    object_as_fn!(as_upvalue, upvalue, object::Upvalue, ObjectType::Upvalue);
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.obj_type {
            ObjectType::Function => Display::fmt(self.as_function().as_ref(), f),
            ObjectType::NativeFunction => Display::fmt(self.as_native_function().as_ref(), f),
            ObjectType::Closure => Display::fmt(self.as_closure().as_ref(), f),
            ObjectType::Upvalue => Display::fmt(self.as_upvalue().as_ref(), f),
        }
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.obj_type {
            ObjectType::Function => {
                let func = self.as_function();
                f.write_fmt(format_args!("<Function> = {}", func.as_ref()))
            }
            ObjectType::NativeFunction => {
                let func = self.as_native_function();
                f.write_fmt(format_args!("<NativeFunction> = {}", func.as_ref()))
            }
            ObjectType::Closure => {
                let clo = self.as_closure();
                f.write_fmt(format_args!("<Closure> = {}", clo.as_ref()))
            }
            ObjectType::Upvalue => {
                let up = self.as_upvalue();
                f.write_fmt(format_args!("<Upvalue> = {}", up.as_ref()))
            }
        }
    }
}

impl Dealloc for Object {
    fn dealloc(&mut self) {
        dbg_println!("DEBUG OBJECT DEALLOC: {:?}", self);
        match self.obj_type {
            ObjectType::Function => (),
            ObjectType::NativeFunction => (),
            ObjectType::Closure => (),
            ObjectType::Upvalue => (),
        }

        // We don't deallocate the next node here, as we want the rest of the objects to remain.
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        (self as *const Self) == (other as *const Self)
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (self as *const Self).partial_cmp(&(other as *const Self))
    }
}

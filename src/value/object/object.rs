use std::{
    fmt::{Debug, Display},
    mem::ManuallyDrop,
    ptr::NonNull,
};

use crate::{
    dbg_println,
    memory::GcPtr,
    value::object::{self, Closure, Function, InternedString, NativeFunction, ObjectRef},
};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ObjectType {
    String,
    Function,
    NativeFunction,
    Closure,
    Upvalue,
}

pub(crate) union ObjectUnion {
    pub(crate) string: ManuallyDrop<InternedString>,
    pub(crate) function: ManuallyDrop<Function>,
    pub(crate) native_function: ManuallyDrop<NativeFunction>,
    pub(crate) closure: ManuallyDrop<Closure>,
    pub(crate) upvalue: ManuallyDrop<object::Upvalue>,
}

pub struct Object {
    obj_type: ObjectType,
    casts: ObjectUnion,
    next: Option<GcPtr<Object>>,
    marked: bool,
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
            // SAFETY: We are guaranteed that the object is of the correct type, so we can safely access the union.
            let variant = unsafe { &self.casts.$variant };

            // SAFETY: We are guaranteed that variant is valid and initialized here.
            let nn = unsafe {
                // We cast the variant out of the ManuallyDrop, as this is already guaranteed by the ObjectRef.
                // We then cast to a *mut despie being being a reference, as ObjectRef<T> is a pointer type.
                // Borrow rules are checked later by the ObjectRef<T> type.
                NonNull::new_unchecked(variant as &$ty as *const _ as *mut _)
            };
            ObjectRef::new(nn, unsafe {
                NonNull::new_unchecked(self as *const _ as *mut _)
            })
        }
    };
}

impl Object {
    #[inline]
    pub(crate) fn alloc(
        obj_type: ObjectType,
        inner: ObjectUnion,
        next: Option<GcPtr<Object>>,
    ) -> GcPtr<Self> {
        dbg_println!("DEBUG OBJECT ALLOC: {:?}", obj_type);

        let me = GcPtr::alloc(Self {
            obj_type,
            casts: inner,
            next,
            marked: false,
        });
        me
    }

    #[inline]
    pub(crate) const fn is_marked(&self) -> bool {
        self.marked
    }

    #[inline]
    pub(crate) const fn mark(&mut self) {
        self.marked = true;
    }

    #[inline]
    pub(crate) const fn unmark(&mut self) {
        self.marked = false;
    }

    #[inline]
    pub(crate) const fn get_next_object(&self) -> Option<GcPtr<Object>> {
        self.next
    }

    #[inline]
    pub(crate) const fn set_next_object(&mut self, next: Option<GcPtr<Object>>) {
        self.next = next;
    }

    #[inline]
    pub const fn get_type(&self) -> ObjectType {
        self.obj_type
    }

    object_as_fn!(as_string, string, InternedString, ObjectType::String);
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
            ObjectType::String => Display::fmt(self.as_string().as_ref(), f),
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
            ObjectType::String => {
                f.write_fmt(format_args!("<String> = {:?}", self.as_string().as_ref()))
            }
            ObjectType::Function => f.write_fmt(format_args!(
                "<Function> = {:?}",
                self.as_function().as_ref()
            )),
            ObjectType::NativeFunction => f.write_fmt(format_args!(
                "<NativeFunction> = {:?}",
                self.as_native_function().as_ref()
            )),
            ObjectType::Closure => {
                f.write_fmt(format_args!("<Closure> = {:?}", self.as_closure().as_ref()))
            }
            ObjectType::Upvalue => {
                f.write_fmt(format_args!("<Upvalue> = {:?}", self.as_upvalue().as_ref()))
            }
        }
    }
}

impl PartialEq for Object {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (self as *const Self) == (other as *const Self)
    }
}

impl PartialOrd for Object {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (self as *const Self).partial_cmp(&(other as *const Self))
    }
}

use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::UnsafeCell,
    mem::ManuallyDrop,
};

use crate::{
    dbg_println,
    memory::HeapPtr,
    value::{
        Object,
        ObjectType,
        object::{
            self,
            Closure,
            Function,
            InternedString,
            NativeFunction,
            ObjectUnion,
            StringInterner,
        },
    },
};

const DEBUG_STRESS: bool = true;

#[global_allocator]
pub static GC: Gc = Gc::new();

macro_rules! object_ctor {
    ($name:ident, $variant:ident, $ty:ty, $tag:expr) => {
        #[inline]
        pub fn $name(&self, val: $ty) -> HeapPtr<Object> {
            let inner = ObjectUnion {
                $variant: ManuallyDrop::new(val),
            };
            self.create_object($tag, inner)
        }
    };
}

struct GcState {
    bytes_allocated: usize,
    next_collect: usize,
    objects_head: HeapPtr<Object>,
    strings: StringInterner,
}

pub struct Gc {
    state: UnsafeCell<GcState>,
}

impl Gc {
    pub const fn new() -> Self {
        Self {
            state: UnsafeCell::new(GcState {
                bytes_allocated: 0,
                next_collect: 0,
                objects_head: HeapPtr::null(),
                strings: StringInterner::new(),
            }),
        }
    }

    fn create_object(&self, obj_type: ObjectType, obj_data: ObjectUnion) -> HeapPtr<Object> {
        let state = unsafe { &mut *self.state.get() };
        let new_head = Object::alloc(obj_type, obj_data, state.objects_head);
        state.objects_head = new_head;
        new_head
    }

    pub fn create_string(&self, str: &str) -> InternedString {
        let state = unsafe { &mut *self.state.get() };
        let string = state.strings.make_string(str);
        string
    }

    pub fn concat_strings(&self, lhs: InternedString, rhs: InternedString) -> InternedString {
        let state = unsafe { &mut *self.state.get() };
        state.strings.concat_strings(lhs, rhs)
    }

    object_ctor!(create_function, function, Function, ObjectType::Function);
    object_ctor!(
        create_native_function,
        native_function,
        NativeFunction,
        ObjectType::NativeFunction
    );
    object_ctor!(create_closure, closure, Closure, ObjectType::Closure);
    object_ctor!(
        create_upvalue,
        upvalue,
        object::Upvalue,
        ObjectType::Upvalue
    );

    fn mark_roots(&self) {}

    pub fn collect(&self) {
        dbg_println!("\n===== GC BEGIN =====");
        dbg_println!("\n===== GC END   =====");
    }
}

unsafe impl GlobalAlloc for Gc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let state = unsafe { &mut *self.state.get() };
        state.bytes_allocated += layout.size();
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let state = unsafe { &mut *self.state.get() };
        state.bytes_allocated -= layout.size();
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let state = unsafe { &mut *self.state.get() };
        let old_size = layout.size();
        if new_size > old_size {
            state.bytes_allocated += new_size - old_size;
        } else {
            state.bytes_allocated -= old_size - new_size;
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

/// SAFETY: We are not going to be using Gc concurrently.
unsafe impl Sync for Gc {}

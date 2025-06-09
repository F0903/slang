use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::UnsafeCell,
    mem::ManuallyDrop,
};

use crate::{
    collections::{DynArray, Stack},
    dbg_println,
    memory::{Dealloc, HeapPtr},
    value::{
        Object,
        ObjectType,
        Value,
        ValueType,
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

pub trait GcRoots {
    fn mark_roots(&mut self, gc: &Gc);
}

struct GcState {
    running: bool,
    bytes_allocated: usize,
    next_collect: usize,
    objects_head: Option<HeapPtr<Object>>,
    strings: StringInterner,
    roots: DynArray<*const dyn GcRoots>,
    gray_stack: DynArray<HeapPtr<Object>>,
}

pub struct Gc {
    /// SAFETY: As of now, everything is still single-threaded, so we should be good going with an UnsafeCell for minimum overhead
    state: UnsafeCell<GcState>,
}

impl Gc {
    pub const fn new() -> Self {
        Self {
            state: UnsafeCell::new(GcState {
                running: false,
                bytes_allocated: 0,
                next_collect: 0,
                objects_head: None,
                strings: StringInterner::new(),
                roots: DynArray::new(),
                gray_stack: DynArray::new(),
            }),
        }
    }

    /// SAFETY: Remember to unregister pointer manually!
    pub fn register_roots(&self, roots: *const dyn GcRoots) {
        let state = unsafe { &mut *self.state.get() };
        state.roots.push(roots);
    }

    pub fn unregister_roots(&self, roots: *const dyn GcRoots) {
        let state = unsafe { &mut *self.state.get() };
        state.roots.remove_value(roots).ok();
    }

    pub fn mark_object(&self, mut object: HeapPtr<Object>) {
        let state = unsafe { &mut *self.state.get() };

        // We don't add NativeFunctions, these obviously don't need GC.
        // We also don't want to mark object that are already marked.
        if object.get_type() == ObjectType::NativeFunction || object.is_marked() {
            return;
        }

        object.mark();
        state.gray_stack.push(object);

        dbg_println!("\t MARKED '{:?}'", object);
    }

    pub fn mark_value(&self, value: Value) {
        if value.get_type() != ValueType::Object {
            return;
        }

        let object = value.as_object();
        self.mark_object(object);
    }

    fn blacken_object(&self, object: HeapPtr<Object>) {
        dbg_println!("\t BLACKEN '{:?}'", object);
        match object.get_type() {
            ObjectType::Upvalue => {
                let up = object.as_upvalue();
                self.mark_value(up.get_value());
            }
            ObjectType::Function => {
                let func = object.as_function();
                if let Some(func_name) = func.get_name().map(|x| Value::string(x)) {
                    self.mark_value(func_name);
                }
                for constant in func.get_chunk().get_constants() {
                    self.mark_value(*constant);
                }
            }
            ObjectType::Closure => {
                let clo = object.as_closure();
                self.mark_object(clo.function.upcast());
                for upvalue in clo.get_upvalues() {
                    self.mark_object(upvalue.upcast());
                }
            }
            // Since we ignore NativeFunctions in self.mark_object() this path should not be reachable
            ObjectType::NativeFunction => unreachable!(),
        }
    }

    fn trace_objects(&self) {
        let state = unsafe { &mut *self.state.get() };
        while state.gray_stack.get_count() > 0 {
            let object = state.gray_stack.pop();
            self.blacken_object(object);
        }
    }

    fn sweep(&self) {
        let state = unsafe { &mut *self.state.get() };

        let mut previous = None;
        let mut object = state.objects_head;
        while let Some(mut obj) = object {
            if obj.is_marked() {
                // If the object is marked (is still reachable) we unmark it and go to the next.
                obj.unmark();
                previous = object;
                object = obj.get_next_object();
                continue;
            }

            // If the object is not marked, we know that it is not reachable anymore, and we dealloc.
            object = obj.get_next_object();
            if let Some(mut prev) = previous {
                prev.set_next_object(object);
            } else {
                state.objects_head = Some(obj);
            }

            obj.dealloc();
        }
    }

    pub fn collect(&self) {
        let state = unsafe { &mut *self.state.get() };
        if state.running {
            // If we are already running we just return.
            // We can land in this path if the GC itself allocates enough memory,
            // and we don't want the GC to GC itself
            return;
        }

        state.running = true;
        dbg_println!("\n===== GC BEGIN =====");

        for roots in (&state.roots).iter() {
            unsafe {
                // Don't ask...
                (*(*roots as *mut dyn GcRoots)).mark_roots(self);
            }
        }
        self.trace_objects();

        dbg_println!("\n===== GC END   =====");
        state.running = false;
    }

    fn create_object(&self, obj_type: ObjectType, obj_data: ObjectUnion) -> HeapPtr<Object> {
        let state = unsafe { &mut *self.state.get() };
        let new_head = Object::alloc(obj_type, obj_data, state.objects_head);
        state.objects_head = Some(new_head);
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
}

unsafe impl GlobalAlloc for Gc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if DEBUG_STRESS {
            GC.collect();
        }

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

            if DEBUG_STRESS {
                GC.collect();
            }
        } else {
            state.bytes_allocated -= old_size - new_size;
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

/// SAFETY: We are not going to be using Gc concurrently.
unsafe impl Sync for Gc {}

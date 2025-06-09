use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::UnsafeCell,
    mem::ManuallyDrop,
};

use crate::{
    collections::DynArray,
    dbg_println,
    memory::{Dealloc, GcRoots, HeapPtr, Markable},
    value::{
        Object,
        ObjectType,
        Value,
        ValueType,
        object::{
            self,
            Closure,
            Function,
            NativeFunction,
            ObjectRef,
            ObjectUnion,
            String,
            StringInterner,
        },
    },
};

#[global_allocator]
pub static GC: Gc = Gc::new();
const GC_HEAP_GROW_FACTOR: usize = 2;

macro_rules! object_ctor {
    ($name:ident, $variant:ident, $ty:ty, $tag:expr, $cast_method:ident) => {
        #[inline]
        pub fn $name(&self, val: $ty) -> ObjectRef<$ty> {
            let inner = ObjectUnion {
                $variant: ManuallyDrop::new(val),
            };
            self.create_object($tag, inner).$cast_method()
        }
    };
}

struct GcState {
    running: bool,
    bytes_allocated: usize,
    next_collect: usize,
    objects_head: Option<HeapPtr<Object>>,
    strings: StringInterner,
    roots: DynArray<*const dyn GcRoots>,
    temp_roots: DynArray<*const dyn Markable>,
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
                next_collect: 1024 * 1024,
                objects_head: None,
                strings: StringInterner::new(),
                roots: DynArray::new(),
                temp_roots: DynArray::new(),
                gray_stack: DynArray::new(),
            }),
        }
    }

    #[inline]
    pub fn should_collect(&self) -> bool {
        let state = unsafe { self.state.get().as_ref_unchecked() };
        state.bytes_allocated >= state.next_collect
    }

    /// SAFETY: Remember to unregister pointer manually!
    pub fn register_temp_root(&self, root: *const dyn Markable) {
        let state = unsafe { self.state.get().as_mut_unchecked() };
        state.temp_roots.push(root);
    }

    pub fn unregister_temp_root(&self, root: *const dyn Markable) {
        let state = unsafe { self.state.get().as_mut_unchecked() };
        state.temp_roots.remove_value(root).ok();
    }

    /// SAFETY: Remember to unregister pointer manually!
    pub fn register_roots(&self, roots: *const dyn GcRoots) {
        let state = unsafe { self.state.get().as_mut_unchecked() };
        state.roots.push(roots);
    }

    pub fn unregister_roots(&self, roots: *const dyn GcRoots) {
        let state = unsafe { self.state.get().as_mut_unchecked() };
        state.roots.remove_value(roots).ok();
    }

    pub fn mark_object(&self, mut object: HeapPtr<Object>) {
        let state = unsafe { self.state.get().as_mut_unchecked() };

        let object_type = object.get_type();
        if object_type == ObjectType::NativeFunction || object.is_marked() {
            // We don't add NativeFunctions, these obviously don't need GC.
            // We also don't want to mark object that are already marked.
            return;
        } else if object_type == ObjectType::String {
            // Strings are a special case that does not need tracing. So we just mark and return.
            object.mark();
            dbg_println!("\t MARKED STRING '{:?}'", object);
            return;
        }

        object.mark();
        state.gray_stack.push(object);

        dbg_println!("\t MARKED '{:?}'", object);
    }

    pub fn mark_value(&self, value: Value) {
        let value_type = value.get_type();
        if value_type != ValueType::Object {
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
                if let Some(func_name) = func.get_name().map(|x| Value::object(x.upcast())) {
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
            // Since we ignore NativeFunctions and Strings in self.mark_object() this path should not be reachable
            ObjectType::NativeFunction | ObjectType::String => unreachable!(),
        }
    }

    fn trace_objects(&self) {
        let state = unsafe { self.state.get().as_mut_unchecked() };
        while state.gray_stack.get_count() > 0 {
            let object = state.gray_stack.pop();
            self.blacken_object(object);
        }
    }

    fn sweep(&self) {
        let state = unsafe { self.state.get().as_mut_unchecked() };

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

    fn sweep_unreachable_strings(&self) {
        let state = unsafe { self.state.get().as_mut_unchecked() };

        let mut strings_to_remove =
            DynArray::new_with_cap(state.strings.get_interned_strings_count() / 2);
        for string in state.strings.get_interned_strings() {
            let mut string_object = string.upcast();
            if string_object.is_marked() {
                // If the string is marked (reachable), we unmark an loop on.
                string_object.unmark();
                continue;
            }

            strings_to_remove.push(string);
        }

        for string in strings_to_remove {
            if let Err(err) = state.strings.remove(string) {
                println!("\t GC ERROR: {}", err);
            }
        }
    }

    pub fn collect(&self) {
        let state = unsafe { self.state.get().as_mut_unchecked() };
        if state.running {
            // If we are already running we just return.
            // We can land in this path if the GC itself allocates enough memory,
            // and we don't want the GC to GC itself
            return;
        }

        state.running = true;
        let start_alloc = state.bytes_allocated;
        dbg_println!("\n===== GC BEGIN =====");
        dbg_println!("\t GC CURRENT ALLOCATION: {}", start_alloc);

        for roots in (&state.roots).iter() {
            unsafe {
                // Don't ask...
                (*(*roots as *mut dyn GcRoots)).mark_roots(self);
            }
        }
        self.trace_objects();
        self.sweep_unreachable_strings();
        self.sweep();
        let end_alloc = state.bytes_allocated;
        state.next_collect = state.bytes_allocated * GC_HEAP_GROW_FACTOR;

        dbg_println!("\t GC CURRENT ALLOCATION: {}", end_alloc);
        dbg_println!("\t GC RECLAIMED {} BYTES", start_alloc - end_alloc);
        dbg_println!("\t GC NEXT RUN: {}", state.next_collect);
        dbg_println!("\n===== GC END   =====");
        state.running = false;
    }

    pub(crate) fn create_object(
        &self,
        obj_type: ObjectType,
        obj_data: ObjectUnion,
    ) -> HeapPtr<Object> {
        let state = unsafe { &mut *self.state.get() };
        let new_head = Object::alloc(obj_type, obj_data, state.objects_head);
        state.objects_head = Some(new_head);
        new_head
    }

    pub fn create_string(&self, str: &str) -> ObjectRef<String> {
        let state = unsafe { &mut *self.state.get() };
        let string = state.strings.make_string(str);
        string
    }

    pub fn concat_strings(&self, lhs: String, rhs: String) -> ObjectRef<String> {
        let state = unsafe { &mut *self.state.get() };
        state.strings.concat_strings(lhs, rhs)
    }

    object_ctor!(
        create_function,
        function,
        Function,
        ObjectType::Function,
        as_function
    );
    object_ctor!(
        create_native_function,
        native_function,
        NativeFunction,
        ObjectType::NativeFunction,
        as_native_function
    );
    object_ctor!(
        create_closure,
        closure,
        Closure,
        ObjectType::Closure,
        as_closure
    );
    object_ctor!(
        create_upvalue,
        upvalue,
        object::Upvalue,
        ObjectType::Upvalue,
        as_upvalue
    );
}

unsafe impl GlobalAlloc for Gc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        #[cfg(feature = "debug_stress_gc")]
        if DEBUG_STRESS {
            self.collect();
        }
        #[cfg(not(feature = "debug_stress_gc"))]
        if self.should_collect() {
            self.collect();
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

            #[cfg(feature = "debug_stress_gc")]
            if DEBUG_STRESS {
                self.collect();
            }
            #[cfg(not(feature = "debug_stress_gc"))]
            if self.should_collect() {
                self.collect();
            }
        } else {
            state.bytes_allocated -= old_size - new_size;
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

/// SAFETY: We are not going to be using Gc concurrently.
unsafe impl Sync for Gc {}

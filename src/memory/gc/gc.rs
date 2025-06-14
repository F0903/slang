use std::{
    alloc::{Allocator, Layout, System},
    cell::UnsafeCell,
    mem::ManuallyDrop,
    ops::DerefMut,
    ptr::NonNull,
};

use crate::{
    collections::DynArray,
    dbg_println,
    memory::{GcPtr, RootMarker, gc::GcScopedRoot},
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
            ObjectRef,
            ObjectUnion,
            StringInterner,
        },
    },
};

pub static GC: Gc = Gc::new();
const GC_HEAP_GROW_FACTOR: usize = 2;

macro_rules! object_ctor {
    ($vis:vis, $name:ident, $variant:ident, $ty:ty, $tag:expr, $cast_method:ident) => {
        #[inline]
        $vis fn $name(&self, val: $ty) -> GcScopedRoot<ObjectRef<$ty>> {
            // SAFETY: We are guaranteed to be in a single-threaded context, so we can safely access the state.
            let state = unsafe { self.state.get().as_mut_unchecked() };
            let new_head = Object::alloc(
                $tag,
                ObjectUnion {
                    $variant: ManuallyDrop::new(val),
                },
                state.objects_head,
            );
            let casted_obj = new_head.$cast_method();
            let rooted_obj = GcScopedRoot::register(casted_obj);
            state.objects_head = Some(new_head);
            rooted_obj
        }
    };
}

struct GcState {
    running: bool,
    bytes_allocated: usize,
    next_collect: usize,
    objects_head: Option<GcPtr<Object>>,
    strings: StringInterner,
    root_markers: DynArray<RootMarker>,
    temp_roots: DynArray<GcPtr<Object>>,
    gray_stack: DynArray<GcPtr<Object>>,
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
                root_markers: DynArray::new(),
                temp_roots: DynArray::new(),
                gray_stack: DynArray::new(),
            }),
        }
    }

    #[inline]
    fn get_state(&self) -> &GcState {
        // SAFETY: We are guaranteed to be in a single-threaded context, so we can safely access the state.
        unsafe { self.state.get().as_ref_unchecked() }
    }

    #[inline]
    fn get_state_mut(&self) -> &mut GcState {
        // SAFETY: We are guaranteed to be in a single-threaded context, so we can safely access the state.
        unsafe { self.state.get().as_mut_unchecked() }
    }

    #[inline]
    #[cfg(not(feature = "debug_stress_gc"))]
    pub fn should_collect(&self) -> bool {
        let state = self.get_state();

        // If we are already running we just return.
        // We can land in this path if the GC itself allocates enough memory,
        // and we don't want the GC to GC itself
        !state.running && (state.bytes_allocated >= state.next_collect)
    }

    /// SAFETY: Remember to unregister pointer manually!
    pub fn register_temp_root(&self, root: GcPtr<Object>) {
        let state = self.get_state_mut();
        state.temp_roots.push(root);
    }

    pub fn unregister_temp_root(&self, root: GcPtr<Object>) {
        let state = self.get_state_mut();
        state
            .temp_roots
            .remove_value(root)
            .expect("Could not remove, temp root object did not exist!");
    }

    /// SAFETY: Remember to unregister pointer manually!
    pub fn add_root_marker(&self, marker: RootMarker) {
        let state = self.get_state_mut();
        state.root_markers.push(marker);
    }

    pub fn remove_root_marker_by_address(&self, address: usize) {
        let state = self.get_state_mut();
        state
            .root_markers
            .remove_predicate(|x| x.get_marker_address() == address)
            .expect("Could not remove, RootRegistrator did not exist!");
    }

    pub fn mark_object(&self, mut object: GcPtr<Object>) {
        let state = self.get_state_mut();

        // We don't want to mark object that are already marked.
        if object.is_marked() {
            return;
        }
        object.mark();

        let object_type = object.get_type();
        if object_type == ObjectType::NativeFunction || object_type == ObjectType::String {
            // We don't add NativeFunctions to the gray stack, these obviously don't need GC.
            // We also don't add strings, since these are interned and need special treatment.
            dbg_println!("| MARKED '{}'", object);
            return;
        }

        state.gray_stack.push(object);

        dbg_println!("| MARKED '{}'", object);
    }

    pub fn mark_value(&self, value: Value) {
        let value_type = value.get_type();
        if value_type != ValueType::Object {
            return;
        }

        let object = value.as_object();
        self.mark_object(object);
    }

    fn blacken_object(&self, object: GcPtr<Object>) {
        dbg_println!("| BLACKEN '{}'", object);
        match object.get_type() {
            ObjectType::Upvalue => {
                let up = object.as_upvalue();
                self.mark_value(up.get_value());
            }
            ObjectType::Function => {
                let func = object.as_function();
                if let Some(func_name) = func.get_name().map(|x| x.upcast().to_value()) {
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

    fn trace_gray_objects(&self) {
        let state = self.get_state_mut();
        while state.gray_stack.get_count() > 0 {
            let object = state.gray_stack.pop();
            self.blacken_object(object);
        }
    }

    fn sweep(&self) {
        let state = self.get_state_mut();

        let mut previous = None;
        let mut object = state.objects_head;
        while let Some(mut obj) = object {
            dbg_println!("| CHECKING OBJECT: () {}", obj);
            if obj.is_marked() {
                // If the object is marked (is still reachable) we unmark it and go to the next.
                obj.unmark();
                previous = object;
                object = obj.get_next_object();
                dbg_println!("|- OBJECT WAS MARKED");
                continue;
            }

            // If the object is not marked, we know that it is not reachable anymore, and we dealloc.
            object = obj.get_next_object();
            if let Some(mut prev) = previous {
                prev.set_next_object(object);
            } else {
                state.objects_head = Some(obj);
            }

            dbg_println!("|- DEALLOCATING OBJECT");
            obj.dealloc();
        }
    }

    fn sweep_unreachable_strings(&self) {
        let state = self.get_state_mut();

        let mut strings_to_remove =
            DynArray::new_with_cap(state.strings.get_interned_strings_count() / 2);
        for string in state.strings.get_interned_strings() {
            let string_object = string.upcast();
            if string_object.is_marked() {
                continue;
            }

            dbg_println!("| REMOVING STRING: {}", string);
            strings_to_remove.push(string);
        }

        for string in strings_to_remove {
            dbg_println!("|- {}", string);
            if let Err(err) = state.strings.remove(string) {
                println!("| GC ERROR: {}", err);
            }
        }
    }

    pub fn collect(&self) {
        let state = self.get_state_mut();

        #[cfg(feature = "debug_stress_gc")]
        if state.running {
            // If we are debug stressing the gc, then it will bypass self.should_collect() and we need to return here.
            return;
        }

        state.running = true;

        #[cfg(debug_assertions)]
        let start_alloc = state.bytes_allocated;
        dbg_println!("\n===== GC BEGIN =====");
        dbg_println!("| GC CURRENT ALLOCATION: {}", start_alloc);

        for marker in state.root_markers.iter_mut() {
            marker.mark_roots(self)
        }
        for root in state.temp_roots.iter_mut() {
            root.deref_mut().mark();
        }

        self.trace_gray_objects();
        self.sweep_unreachable_strings();
        self.sweep();
        state.next_collect = state.bytes_allocated * GC_HEAP_GROW_FACTOR;

        #[cfg(debug_assertions)]
        let end_alloc = state.bytes_allocated;
        dbg_println!("| GC CURRENT ALLOCATION: {}", end_alloc);
        dbg_println!(
            "| GC RECLAIMED {} BYTES",
            (start_alloc as isize - end_alloc as isize)
        );
        dbg_println!("| GC NEXT RUN: {}", state.next_collect);
        dbg_println!("\n===== GC END   =====");
        state.running = false;
    }

    pub fn make_string(&self, str: &str) -> GcScopedRoot<ObjectRef<InternedString>> {
        let state = self.get_state_mut();
        state.strings.make_string(str)
    }

    pub fn concat_strings(
        &self,
        lhs: ObjectRef<InternedString>,
        rhs: ObjectRef<InternedString>,
    ) -> GcScopedRoot<ObjectRef<InternedString>> {
        let state = self.get_state_mut();
        state.strings.concat_strings(lhs, rhs)
    }

    object_ctor!(
        pub(crate),
        create_string,
        string,
        InternedString,
        ObjectType::String,
        as_string
    );
    object_ctor!(
        pub,
        create_function,
        function,
        Function,
        ObjectType::Function,
        as_function
    );
    object_ctor!(
        pub,
        create_native_function,
        native_function,
        NativeFunction,
        ObjectType::NativeFunction,
        as_native_function
    );
    object_ctor!(
        pub,
        create_closure,
        closure,
        Closure,
        ObjectType::Closure,
        as_closure
    );
    object_ctor!(
        pub,
        create_upvalue,
        upvalue,
        object::Upvalue,
        ObjectType::Upvalue,
        as_upvalue
    );

    pub(crate) fn reallocate<T>(&self, ptr: *mut u8, old_cap: usize, new_cap: usize) -> *mut u8 {
        let old_layout = Layout::array::<T>(old_cap).unwrap();
        let new_layout = Layout::array::<T>(new_cap).unwrap();

        if new_cap == 0 {
            if !ptr.is_null() {
                // SAFETY: we just checked that ptr is not null, so we can safely deallocate it.
                unsafe {
                    self.deallocate(NonNull::new_unchecked(ptr), old_layout);
                }
            }
            return std::ptr::null_mut();
        }

        if ptr.is_null() {
            let nn = self.allocate(new_layout).unwrap();
            return nn.as_ptr() as *mut u8;
        }

        if new_cap > old_cap {
            // SAFETY: we just checked that ptr is not null, so we can safely grow it.
            let nn = unsafe {
                self.grow(NonNull::new_unchecked(ptr), old_layout, new_layout)
                    .unwrap()
            };
            nn.as_ptr() as *mut u8
        } else {
            // SAFETY: we just checked that ptr is not null, so we can safely shrink it.
            let nn = unsafe {
                self.shrink(NonNull::new_unchecked(ptr), old_layout, new_layout)
                    .unwrap()
            };
            nn.as_ptr() as *mut u8
        }
    }
}

unsafe impl Allocator for Gc {
    fn allocate(
        &self,
        layout: Layout,
    ) -> std::result::Result<NonNull<[u8]>, std::alloc::AllocError> {
        #[cfg(feature = "debug_stress_gc")]
        self.collect();
        #[cfg(not(feature = "debug_stress_gc"))]
        if self.should_collect() {
            self.collect();
        }

        let state = self.get_state_mut();
        state.bytes_allocated += layout.size();

        // Use System allocator for actual memory
        let ptr = System.allocate(layout)?;
        Ok(ptr)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let state = self.get_state_mut();
        state.bytes_allocated -= layout.size();

        // SAFETY: it is up to the caller to ensure that ptr and layout is valid and was allocated by this allocator.
        unsafe {
            System.deallocate(ptr, layout);
        }
    }

    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, std::alloc::AllocError> {
        #[cfg(feature = "debug_stress_gc")]
        self.collect();
        #[cfg(not(feature = "debug_stress_gc"))]
        if self.should_collect() {
            self.collect();
        }

        let state = self.get_state_mut();
        state.bytes_allocated += layout.size();

        let ptr = System.allocate_zeroed(layout)?;
        Ok(ptr)
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, std::alloc::AllocError> {
        #[cfg(feature = "debug_stress_gc")]
        self.collect();
        #[cfg(not(feature = "debug_stress_gc"))]
        if self.should_collect() {
            self.collect();
        }

        let state = self.get_state_mut();
        if new_layout.size() > old_layout.size() {
            state.bytes_allocated += new_layout.size() - old_layout.size();
        } else {
            state.bytes_allocated -= old_layout.size() - new_layout.size();
        }

        // SAFETY: it is up to the caller to ensure that ptr and layout is valid and was allocated by this allocator.
        unsafe { System.grow(ptr, old_layout, new_layout) }
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, std::alloc::AllocError> {
        #[cfg(feature = "debug_stress_gc")]
        self.collect();
        #[cfg(not(feature = "debug_stress_gc"))]
        if self.should_collect() {
            self.collect();
        }

        let state = self.get_state_mut();
        if new_layout.size() > old_layout.size() {
            state.bytes_allocated += new_layout.size() - old_layout.size();
        } else {
            state.bytes_allocated -= old_layout.size() - new_layout.size();
        }

        // SAFETY: it is up to the caller to ensure that ptr and layout is valid and was allocated by this allocator.
        unsafe { System.grow_zeroed(ptr, old_layout, new_layout) }
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, std::alloc::AllocError> {
        let state = self.get_state_mut();
        state.bytes_allocated -= old_layout.size() - new_layout.size();

        // SAFETY: it is up to the caller to ensure that ptr and layout is valid and was allocated by this allocator.
        unsafe { System.shrink(ptr, old_layout, new_layout) }
    }
}

/// SAFETY: We are not going to be using Gc concurrently.
unsafe impl Sync for Gc {}

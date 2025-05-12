use {
    super::{Object, ObjectManager, StringObject},
    crate::{
        memory::{Dealloc, ManualPtr},
        vm::VmHeap,
    },
    std::fmt::Display,
};
/// A container for objects that "links" them together as a linked list.
/// This is used to keep track of all objects in the VM.
#[derive(Clone, Copy, Debug)]
pub struct ObjectContainer {
    obj: ManualPtr<Object>,
    next: ManualPtr<ObjectContainer>,
}

impl ObjectContainer {
    pub fn alloc(obj: Object, objects: &mut ObjectManager) -> ManualPtr<Self> {
        println!("DEBUG OBJECT ALLOC: {}", obj);

        let head = objects.get_objects_head();

        // Having an additional alloc here for this container type (which is essentially just two pointers) is not optimal, but rust does not allow recursive types without indirection (a pointer) so it cannot be avoided.
        let me = ManualPtr::alloc(Self {
            obj: ManualPtr::alloc(obj),
            next: head,
        });
        objects.set_objects_head(me.clone());
        me
    }

    pub fn alloc_interned_string(str: &str, heap: &mut VmHeap) -> ManualPtr<Self> {
        let str = StringObject::new(str);
        heap.interned_strings.insert(str.clone(), None); // We just care about the key.
        Self::alloc(Object::String(str), &mut heap.objects)
    }

    pub const fn get_object_ptr(&self) -> ManualPtr<Object> {
        self.obj
    }

    pub fn get_object(&self) -> &Object {
        self.obj.get()
    }

    pub const fn get_next_object_ptr(&self) -> ManualPtr<ObjectContainer> {
        self.next
    }
}

impl Display for ObjectContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            match self.obj.get() {
                Object::String(x) => x,
            }
        ))
    }
}

impl Dealloc for ObjectContainer {
    fn dealloc(&mut self) {
        // Don't dealloc the next node
        self.obj.dealloc();
    }
}

impl PartialEq for ObjectContainer {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}

impl PartialOrd for ObjectContainer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            _ => {
                if self == other {
                    Some(std::cmp::Ordering::Equal)
                } else {
                    None
                }
            }
        }
    }
}

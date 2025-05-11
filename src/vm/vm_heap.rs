use crate::{
    collections::HashTable,
    value::object::{ObjectManager, RawString},
};

pub struct VmHeap {
    pub objects: ObjectManager,
    pub interned_strings: HashTable<RawString>,
}

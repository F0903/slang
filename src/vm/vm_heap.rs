use crate::{
    collections::HashTable,
    value::object::{ObjectManager, StringObject},
};

pub struct VmHeap {
    pub objects: ObjectManager,
    pub interned_strings: HashTable<StringObject>,
}

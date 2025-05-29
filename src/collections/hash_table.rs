use super::DynArray;
use crate::{
    hashing::{HashMethod, Hashable},
    value::object::InternedString,
};

const TABLE_MAX_LOAD: f32 = 0.75;

#[derive(Debug)]
pub struct Entry<K, V> {
    pub key: K,
    pub value: Option<V>,
}

#[derive(Debug)]
pub(crate) struct Bucket<K, V> {
    tombstone: bool,
    pub(crate) entry: Option<Entry<K, V>>,
}

pub struct HashTable<K: Hashable + PartialEq + std::fmt::Debug, V: std::fmt::Debug> {
    data: DynArray<Bucket<K, V>>,
}

impl<K: Hashable + PartialEq + std::fmt::Debug, V: std::fmt::Debug> HashTable<K, V> {
    pub fn new() -> Self {
        Self {
            data: DynArray::new(Some(Bucket {
                tombstone: false,
                entry: None,
            })),
        }
    }

    pub fn get_raw_data(&mut self) -> &mut DynArray<Bucket<K, V>> {
        &mut self.data
    }

    fn find_bucket(&mut self, hash: u32) -> &mut Bucket<K, V> {
        let capacity = self.data.get_capacity();

        // Find first empty bucket, or if not, return the first tombstone
        let mut tombstone_index = None;
        let mut index = hash as usize % capacity;
        loop {
            let bucket = self.data.get_unchecked(index);
            if !bucket.tombstone {
                let entry = bucket.entry.as_ref();
                match entry {
                    Some(entry) => {
                        if entry.key.get_hash() == hash {
                            return self.data.get_mut_unchecked(index);
                        }
                    }
                    None => {
                        let return_index = if let Some(tombstone_index) = tombstone_index {
                            tombstone_index
                        } else {
                            index
                        };
                        return self.data.get_mut_unchecked(return_index);
                    }
                }
            } else {
                tombstone_index = Some(index);
            }

            index = (index + 1) % capacity;
        }
    }

    fn grow(&mut self) {
        let new_capacity = self.data.next_growth_capacity();
        let old_buckets = self.data.clone();

        // Reset data completely, but with increased capacity
        self.data = DynArray::new_with_cap(
            new_capacity,
            Some(Bucket {
                tombstone: false,
                entry: None,
            }),
        );

        // We need to count every entry from the beginning, since we are not copying over tombstones
        let mut count = 0;
        for bucket in old_buckets {
            if let Some(entry) = bucket.entry {
                let new_entry_destination = self.find_bucket(entry.key.get_hash());
                new_entry_destination.entry = Some(entry);
                count += 1;
            }
        }

        self.data.set_count(count);
    }

    // Returns true if the key was inserted, false if it was already present (thus overwritten)
    pub fn set(&mut self, key: K, value: Option<V>) -> bool {
        if self.data.get_count() as f32 + 1_f32 > self.data.get_capacity() as f32 * TABLE_MAX_LOAD {
            self.grow();
        }

        let hash = key.get_hash();
        let bucket = self.find_bucket(hash);
        let was_none = bucket.entry.is_none();
        let was_tombstone = bucket.tombstone;

        let entry = Entry { key, value };
        bucket.entry = Some(entry);
        bucket.tombstone = false;

        // Only increase the count if we are inserting a new key (not replacing an existing one or tombstone)
        let new_key = was_none && !was_tombstone;
        if new_key {
            self.data.set_count(self.data.get_count() + 1);
        }

        new_key
    }

    pub fn get(&mut self, key: &K) -> Option<&Entry<K, V>> {
        if self.data.get_count() == 0 {
            return None;
        }

        let bucket = self.find_bucket(key.get_hash());
        if let Some(entry) = &bucket.entry {
            if entry.key == *key {
                return Some(entry);
            }
        }
        None
    }

    pub fn delete_by_hash(&mut self, hash: u32) -> Option<Entry<K, V>> {
        if self.data.get_count() == 0 {
            return None;
        }

        let bucket = self.find_bucket(hash);
        let entry = bucket.entry.take();
        bucket.tombstone = true;
        bucket.entry = None;

        // We don't decrease the count since we just mark the entry as a tombstone

        entry
    }

    pub fn delete(&mut self, key: &InternedString) -> Option<Entry<K, V>> {
        self.delete_by_hash(key.get_hash())
    }
}

impl<V: std::fmt::Debug> HashTable<InternedString, V> {
    pub fn get_by_str<H: HashMethod>(
        &mut self,
        key_name: &str,
    ) -> Option<&Entry<InternedString, V>> {
        if self.data.get_count() == 0 {
            return None;
        }

        let bucket = self.find_bucket(H::hash(key_name.as_bytes()));
        if let Some(entry) = &bucket.entry {
            if entry.key.get_str() == key_name {
                return Some(entry);
            }
        }
        None
    }
}

impl<K: Hashable + PartialEq + std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug
    for HashTable<K, V>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashTable")
            .field("count", &self.data.get_count())
            .field("capacity", &self.data.get_capacity())
            .field(
                "load_factor",
                &(self.data.get_count() as f32 / self.data.get_capacity() as f32),
            )
            .field("buckets", &self.data.get_count())
            .field(
                "data",
                &self
                    .data
                    .memory_iter()
                    .map(|x| unsafe { x.assume_init_ref() })
                    .filter(|x| x.entry.is_some())
                    .collect::<Vec<&Bucket<K, V>>>(),
            )
            .finish()
    }
}

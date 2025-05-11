use crate::value::object::RawString;

use super::DynArray;

const TABLE_MAX_LOAD: f32 = 0.75;

#[derive(Debug)]
struct Entry<T> {
    key: RawString,
    value: T,
}

#[derive(Debug)]
struct Bucket<T> {
    tombstone: bool,
    entry: Option<Entry<T>>,
}

#[derive(Debug)]
pub struct HashTable<T: std::fmt::Debug> {
    data: DynArray<Bucket<T>>,
}

impl<T: std::fmt::Debug> HashTable<T> {
    pub fn new() -> Self {
        Self {
            data: DynArray::new(),
        }
    }

    fn find_bucket(&mut self, hash: u32) -> &mut Bucket<T> {
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
        self.data = DynArray::new_with_cap(new_capacity);

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
    pub fn insert(&mut self, key: RawString, value: T) -> bool {
        if self.data.get_count() as f32 + 1_f32 > self.data.get_capacity() as f32 * TABLE_MAX_LOAD {
            self.grow();
        }

        let hash = key.get_hash();
        let bucket = self.find_bucket(hash);
        let was_tombstone = bucket.tombstone;

        let entry = Entry { key, value };
        bucket.entry = Some(entry);
        bucket.tombstone = false;

        // Only increase the count if we are inserting a new key (not replacing an existing one or tombstone)
        let new_key = bucket.entry.is_none();
        if new_key && !was_tombstone {
            self.data.set_count(self.data.get_count() + 1);
        }

        new_key
    }

    pub fn get(&mut self, key: &RawString) -> Option<&T> {
        if self.data.get_count() == 0 {
            return None;
        }

        let bucket = self.find_bucket(key.get_hash());
        if let Some(entry) = &bucket.entry {
            if entry.key == *key {
                return Some(&entry.value);
            }
        }
        None
    }

    pub fn delete(&mut self, key: &RawString) -> Option<T> {
        if self.data.get_count() == 0 {
            return None;
        }

        let bucket = self.find_bucket(key.get_hash());
        let entry = bucket.entry.take();
        bucket.tombstone = true;
        bucket.entry = None;

        // We don't decrease the count since we just mark the entry as a tombstone

        entry.map(|x| x.value)
    }
}

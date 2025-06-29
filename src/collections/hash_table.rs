use super::DynArray;
use crate::{
    dbg_println,
    hashing::{HashMethod, Hashable},
    value::object::{InternedString, ObjectRef},
};

const TABLE_MAX_LOAD: f32 = 0.75;

#[derive(Debug, Clone)]
pub struct Entry<K, V> {
    pub key: K,
    pub value: V,
}

#[derive(Debug, Clone)]
pub struct Bucket<K, V> {
    tombstone: bool,
    pub entry: Option<Entry<K, V>>,
}

pub struct HashTable<K, V>
where
    K: Hashable + PartialEq + Clone + std::fmt::Debug,
    V: std::fmt::Debug + Clone,
{
    data: DynArray<Bucket<K, V>>,
}

impl<K, V> HashTable<K, V>
where
    K: Hashable + PartialEq + Clone + std::fmt::Debug,
    V: Clone + std::fmt::Debug,
{
    pub const fn new() -> Self {
        Self {
            data: DynArray::new_with_init(Bucket {
                tombstone: false,
                entry: None,
            }),
        }
    }

    pub fn count(&self) -> usize {
        self.data.get_count()
    }

    /// Return iterator over the entries in the hash table.
    pub fn entries(&self) -> impl Iterator<Item = &Entry<K, V>> {
        // Return entries that are Some and not tombstones
        // SAFETY: memory_iter() is guaranteed to be a valid iterator over initialized memory.
        self.data
            .memory_iter()
            .map(|x| unsafe { x.assume_init_ref() })
            .filter(|x| !x.tombstone)
            .filter_map(|x| x.entry.as_ref())
    }

    /// Return a mutable iterator over the entries in the hash table.
    /// Be careful (please)
    pub fn entries_mut(&mut self) -> impl Iterator<Item = &mut Entry<K, V>> {
        // Return entries that are Some and not tombstones
        // SAFETY: memory_iter() is guaranteed to be a valid iterator over initialized memory.
        self.data
            .memory_iter_mut()
            .map(|x| unsafe { x.assume_init_mut() })
            .filter(|x| !x.tombstone)
            .filter_map(|x| x.entry.as_mut())
    }

    fn get_bucket_ref_at(&self, index: usize) -> &Bucket<K, V> {
        // SAFETY: as long as the index is valid (asserted in method) then the data is guaranteed to be initialized.
        unsafe { self.data.get_memory_unchecked(index) }
    }

    fn get_bucket_mut_at(&mut self, index: usize) -> &mut Bucket<K, V> {
        // SAFETY: as long as the index is valid (asserted in method) then the data is guaranteed to be initialized.
        unsafe { self.data.get_memory_mut_unchecked(index) }
    }

    fn find_bucket(&mut self, hash: u32) -> &mut Bucket<K, V> {
        let capacity = self.data.get_capacity();

        // Find first empty bucket, or if not, return the first tombstone
        let mut tombstone_index = None;
        let mut index = hash as usize % capacity;
        loop {
            let bucket = self.get_bucket_ref_at(index);
            if !bucket.tombstone {
                let entry = bucket.entry.as_ref();
                match entry {
                    Some(entry) => {
                        if entry.key.get_hash() == hash {
                            return self.get_bucket_mut_at(index);
                        }
                    }
                    None => {
                        let return_index = if let Some(tombstone_index) = tombstone_index {
                            tombstone_index
                        } else {
                            index
                        };
                        return self.get_bucket_mut_at(return_index);
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
        self.data = DynArray::new_with_cap_and_init(
            new_capacity,
            Bucket {
                tombstone: false,
                entry: None,
            },
        );

        // We need to count every entry from the beginning, since we are not copying over tombstones.
        let mut count = 0;
        for bucket in old_buckets {
            if let Some(entry) = bucket.entry {
                let new_entry_destination = self.find_bucket(entry.key.get_hash());
                new_entry_destination.entry = Some(entry);
                count += 1;
            }
        }

        // SAFETY: the count is guaranteed to be valid, since we just counted all the valid entries.
        unsafe { self.data.set_count(count) };
    }

    // Returns true if the key was inserted, false if it was already present (thus overwritten)
    pub fn set(&mut self, key: K, value: V) -> bool {
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
            // SAFETY: since we are adding a new element, we are increasing the count.
            unsafe { self.data.set_count(self.data.get_count() + 1) };
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

    pub fn delete(&mut self, key: impl Hashable) -> Option<Entry<K, V>> {
        self.delete_by_hash(key.get_hash())
    }
}

impl<V> HashTable<ObjectRef<InternedString>, V>
where
    V: std::fmt::Debug + Clone,
{
    pub fn get_by_str<H: HashMethod>(
        &mut self,
        key_name: &str,
    ) -> Option<&Entry<ObjectRef<InternedString>, V>> {
        if self.data.get_count() == 0 {
            return None;
        }

        let bucket = self.find_bucket(H::hash(key_name.as_bytes()));
        if let Some(entry) = &bucket.entry {
            if entry.key.as_str() == key_name {
                return Some(entry);
            }
        }
        None
    }
}

impl<K, V> std::fmt::Debug for HashTable<K, V>
where
    K: Hashable + PartialEq + Clone + std::fmt::Debug,
    V: Clone + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //SAFETY: memory_iter() is guaranteed to be an iterator over initialized values.
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

impl<K, V> std::fmt::Display for HashTable<K, V>
where
    K: Hashable + PartialEq + Clone + std::fmt::Debug + std::fmt::Display,
    V: Clone + std::fmt::Debug + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("=== HashTable ===\n")?;
        for entry in self.entries() {
            f.write_fmt(format_args!("{} -> {}\n", entry.key, entry.value))?;
        }
        Ok(())
    }
}

impl<K, V> Drop for HashTable<K, V>
where
    K: Hashable + PartialEq + Clone + std::fmt::Debug,
    V: std::fmt::Debug + Clone,
{
    fn drop(&mut self) {
        dbg_println!("DEBUG HASHTABLE DROP");
        // Since the buckets are not guaranteed to be in a contiguous order, we can not rely on the normal DynArray drop for the elements.
        for entry in self.entries_mut() {
            // SAFETY: It's safe to call drop here, since entries_mut() is guaranteed to be an iterator over valid entries.
            unsafe {
                if std::mem::needs_drop::<K>() {
                    std::ptr::drop_in_place(&mut entry.key);
                }
                if std::mem::needs_drop::<V>() {
                    std::ptr::drop_in_place(&mut entry.value);
                }
            }
        }

        // SAFETY: Since we just dropped all elements, we need to set the count to 0.
        unsafe {
            self.data.set_count(0);
        }
    }
}

use std::sync::RwLock;
use traits::{Extent, Insertable, InsertableDyn};
pub struct DatabaseTable<Store: Extent> {
    bitmap: Bitmap,
    data: Store,
    element_size: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Key {
    pub index: usize,
}

unsafe impl Insertable for Key {
    const SIZE: usize = 8;
    fn from_binary(b: Vec<u8>) -> Self {
        Self {
            index: usize::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]),
        }
    }
}
pub enum TableError {
    InvalidKey,
    InvalidLock,
    KeyNotUsed,
}
impl<Store: Extent> DatabaseTable<Store> {
    const BLOCK_SIZE: u32 = 0x1000;
    pub fn new(data: Store, element_size: usize) -> Self {
        let bitmap = Bitmap::new(0);
        Self {
            bitmap,
            data,
            element_size,
        }
    }
    pub fn get<Data: InsertableDyn>(
        &self,
        key: Key,
        ctor: fn(Vec<u8>) -> Data,
    ) -> Result<Data, TableError> {
        if key.index > self.data.len() * self.element_size {
            return Err(TableError::InvalidKey);
        }
        if self.bitmap.get(key.index) == false {
            return Err(TableError::InvalidKey);
        }
        let data = (0..self.element_size)
            .map(|i| self.data[key.index * self.element_size + i])
            .collect();
        Ok(ctor(data))
    }
    pub fn insert<Data: InsertableDyn>(&mut self, data: Data) -> Key {
        if let Some(index) = self.bitmap.get_first_free() {
            self.bitmap.set(index, true);
            let bytes = data.to_binary();
            for i in 0..self.element_size {
                self.data[index * self.element_size + i] = bytes[i];
            }
            return Key { index };
        }
        let bytes = data.to_binary();

        self.data.resize(self.data.len() + self.element_size);
        self.bitmap.resize(self.bitmap.len() + 1);

        let len = self.bitmap.len();
        for i in 0..self.element_size {
            self.data[(len - 1) * self.element_size + i] = bytes[i];
        }
        let index = len - 1;
        self.bitmap.set(index, true);
        return Key { index };
    }
}
struct Bitmap {
    data: Vec<u64>,
    len: usize,
}
impl Bitmap {
    const INT_SIZE: usize = 64;
    pub fn new(len: usize) -> Self {
        let m = len % Self::INT_SIZE;
        let mut alloc_size = len / Self::INT_SIZE;
        if m != 0 {
            alloc_size += 1;
        }
        Bitmap {
            data: vec![0; alloc_size as usize],
            len,
        }
    }
    pub fn get(&self, index: usize) -> bool {
        if index >= self.len {
            panic!("out of bounds")
        }
        let byte = self.data[index / Self::INT_SIZE as usize];
        let bit = (byte >> (index % Self::INT_SIZE)) & 0x1;
        if bit == 0 {
            return false;
        } else {
            return true;
        }
    }
    pub fn get_first_free(&self) -> Option<usize> {
        let mut index = 0;
        for i in self.data.iter() {
            if i != &u64::MAX {
                for j in 0..Self::INT_SIZE {
                    if !i & (1 << j as u64) == (1 << j as u64) {
                        let return_index = index * Self::INT_SIZE + j;
                        if return_index < self.len() {
                            return Some(return_index);
                        } else {
                            return None;
                        }
                    }
                }
            }
            index += 1;
        }
        return None;
    }
    pub fn set(&mut self, index: usize, state: bool) {
        if state == true {
            let set = 1 << index % Self::INT_SIZE;
            self.data[index as usize / Self::INT_SIZE as usize] =
                self.data[index as usize / Self::INT_SIZE as usize] | set;
        } else {
            let set = (1 << index % Self::INT_SIZE) ^ u64::MAX;
            self.data[index as usize / Self::INT_SIZE as usize] =
                self.data[index as usize / Self::INT_SIZE as usize] & set;
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn resize(&mut self, new_size: usize) {
        let new_len = {
            let modulo = new_size % Self::INT_SIZE;
            if modulo != 0 {
                new_size / Self::INT_SIZE + 1
            } else {
                new_size / Self::INT_SIZE
            }
        };
        self.data.resize_with(new_len, || 0);
        self.len = new_size;
    }
}
unsafe impl InsertableDyn for Key {
    fn size(&self) -> u32 {
        8
    }
    fn to_binary(&self) -> Vec<u8> {
        self.index.to_le_bytes().to_vec()
    }
}
pub fn from_binary(data: Vec<u8>) -> u32 {
    u32::from_le_bytes([data[0], data[1], data[2], data[3]])
}
#[cfg(test)]
mod tests {
    use super::*;
    use traits::InMemoryExtent;
    #[test]
    fn test_bitmap() {
        let mut b = Bitmap::new(64);
        assert_eq!(b.get_first_free().unwrap(), 0);
        b.set(0, true);
        assert_eq!(b.get_first_free().unwrap(), 1);
        assert_eq!(b.get(5), false);
    }
    #[test]
    fn get_first_free() {
        let mut b = Bitmap::new(10000);
        for i in 0..10000 {
            assert_eq!(b.get_first_free().unwrap(), i);
            b.set(i, true);
        }
    }

    #[test]
    fn make_db() {
        let _ = DatabaseTable::new(InMemoryExtent::new(), std::mem::size_of::<u32>());
    }
    #[test]
    fn insert_and_get_single() {
        let mut db = DatabaseTable::new(InMemoryExtent::new(), std::mem::size_of::<u32>());
        let k1 = db.insert::<u32>(1);
        assert_eq!(db.get::<u32>(k1, from_binary).ok().unwrap(), 1);
    }
    #[test]
    fn insert_and_get() {
        let mut db = DatabaseTable::new(InMemoryExtent::new(), std::mem::size_of::<u32>());
        let k1 = db.insert::<u32>(1);
        let k2 = db.insert::<u32>(2);
        assert_eq!(db.get::<u32>(k1, from_binary).ok().unwrap(), 1);
        assert_eq!(db.get::<u32>(k2, from_binary).ok().unwrap(), 2);
    }
    #[test]
    fn mass_insert() {
        let mut db = DatabaseTable::new(InMemoryExtent::new(), std::mem::size_of::<u32>());
        let mut keys = vec![];
        for i in 0..100 {
            keys.push((db.insert::<u32>(i), i));
        }
        for (key, value) in keys.iter() {
            assert_eq!(
                db.get::<u32>(key.clone(), from_binary).ok().unwrap(),
                value.clone()
            );
        }
    }
}

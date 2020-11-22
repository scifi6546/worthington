use dyn_clonable::*;
use std::sync::RwLock;
pub unsafe trait Insertable {
    const SIZE: usize;
    fn from_binary(data: Vec<u8>) -> Self;
}
#[clonable]
pub unsafe trait InsertableDyn: Clone {
    /// It is expected that size is constant
    fn size(&self) -> u32;
    fn to_binary(&self) -> Vec<u8>;
}
pub struct DatabaseTable {
    data: Vec<RwLock<Block>>,
}
#[derive(Debug, Clone)]
pub struct Key {
    index: usize,
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
}
impl DatabaseTable {
    const BLOCK_SIZE: u32 = 0x1000;
    pub fn new() -> Self {
        Self { data: vec![] }
    }
    pub fn get<Data: InsertableDyn>(
        &self,
        key: Key,
        ctor: fn(Vec<u8>) -> Data,
    ) -> Result<Data, TableError> {
        if key.index > self.data.len() * Self::BLOCK_SIZE as usize {
            return Err(TableError::InvalidKey);
        }
        if let Some(block) = self.data[key.index / Self::BLOCK_SIZE as usize].read().ok() {
            Ok(block.get_data(key.index as u32 % Self::BLOCK_SIZE, ctor))
        } else {
            return Err(TableError::InvalidLock);
        }
    }
    pub fn insert<Data: InsertableDyn>(&mut self, data: Data) -> Key {
        let mut block_num = 0;
        for block_lock in self.data.iter() {
            let mut min_index = None;
            if let Some(block) = block_lock.read().ok() {
                min_index = block.get_first_free();
            }
            if let Some(index) = min_index {
                if let Some(mut block) = block_lock.write().ok() {
                    block.write_index(index, data);
                    return Key {
                        index: index as usize + block_num * Self::BLOCK_SIZE as usize,
                    };
                }
            }
            block_num += 1;
        }
        //all blocks are used make a new one
        self.data.push(RwLock::new(Block::new::<Data>(
            Self::BLOCK_SIZE,
            data.size(),
        )));
        if let Some(mut block) = self.data[self.data.len() - 1].write().ok() {
            block.write_index(0, data);
        }
        return Key {
            index: (self.data.len() - 1) * Self::BLOCK_SIZE as usize,
        };
    }
}
struct Block {
    data: Vec<u8>,
    bitmap: Bitmap,
    data_size: u32,
}
impl Block {
    fn new<Data: InsertableDyn>(block_size: u32, data_size: u32) -> Self {
        Block {
            data: vec![0; (block_size as usize) * data_size as usize],
            bitmap: Bitmap::new(block_size),
            data_size,
        }
    }
    fn get_data<Data: InsertableDyn>(&self, index: u32, ctor: fn(Vec<u8>) -> Data) -> Data {
        let buffer: Vec<u8> = self.data
            [(index * self.data_size) as usize..((index + 1) * self.data_size) as usize]
            .to_vec();
        ctor(buffer)
    }
    fn write_index<Data: InsertableDyn>(&mut self, index: u32, data: Data) {
        let buffer = data.to_binary();
        for i in (index * self.data_size)..((index + 1) * self.data_size) {
            self.data[i as usize] = buffer[(i - index * self.data_size) as usize];
        }
        self.bitmap.set(index, true);
    }
    fn get_first_free(&self) -> Option<u32> {
        self.bitmap.get_first_free()
    }
}
struct Bitmap {
    data: Vec<u64>,
}
impl Bitmap {
    const INT_SIZE: u32 = 64;
    pub fn new(size: u32) -> Self {
        let m = size % Self::INT_SIZE;
        let mut alloc_size = size / Self::INT_SIZE;
        if m != 0 {
            alloc_size += 1;
        }
        Bitmap {
            data: vec![0; alloc_size as usize],
        }
    }
    #[allow(dead_code)]
    pub fn get(&self, index: u32) -> bool {
        let byte = self.data[index as usize / Self::INT_SIZE as usize];
        let bit = (byte >> (index % Self::INT_SIZE)) & 0x1;
        if bit == 0 {
            return false;
        } else {
            return true;
        }
    }
    pub fn get_first_free(&self) -> Option<u32> {
        let mut index = 0;
        for i in self.data.iter() {
            if i != &u64::MAX {
                for j in 0..Self::INT_SIZE {
                    if !i & (1 << j as u64) == (1 << j as u64) {
                        return Some(index as u32 * Self::INT_SIZE + j as u32);
                    }
                }
            }
            index += 1;
        }
        return None;
    }
    pub fn set(&mut self, index: u32, state: bool) {
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
}
unsafe impl InsertableDyn for u32 {
    fn size(&self) -> u32 {
        4
    }
    fn to_binary(&self) -> Vec<u8> {
        let bytes = self.to_le_bytes();
        vec![bytes[0], bytes[1], bytes[2], bytes[3]]
    }
}
unsafe impl InsertableDyn for u8 {
    fn size(&self) -> u32 {
        1
    }
    fn to_binary(&self) -> Vec<u8> {
        vec![self.clone()]
    }
}
unsafe impl<T: InsertableDyn + Clone> InsertableDyn for Vec<T> {
    fn size(&self) -> u32 {
        todo!()
    }
    fn to_binary(&self) -> Vec<u8> {
        todo!()
    }
}
unsafe impl InsertableDyn for Box<dyn InsertableDyn> {
    fn size(&self) -> u32 {
        todo!()
    }
    fn to_binary(&self) -> Vec<u8> {
        todo!()
    }
}
unsafe impl InsertableDyn for &Box<dyn InsertableDyn> {
    fn size(&self) -> u32 {
        todo!()
    }
    fn to_binary(&self) -> Vec<u8> {
        todo!()
    }
}
unsafe impl Insertable for usize {
    const SIZE: usize = 8;
    fn from_binary(d: Vec<u8>) -> Self {
        usize::from_le_bytes([d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]])
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
        let _ = DatabaseTable::new();
    }
    #[test]
    fn insert_and_get() {
        let mut db: DatabaseTable = DatabaseTable::new();
        let k1 = db.insert::<u32>(1);
        let k2 = db.insert::<u32>(2);
        assert_eq!(db.get::<u32>(k1, from_binary).ok().unwrap(), 1);
        assert_eq!(db.get::<u32>(k2, from_binary).ok().unwrap(), 2);
    }
    #[test]
    fn mass_insert() {
        let mut db: DatabaseTable = DatabaseTable::new();
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

#[macro_use]
extern crate anyhow;
use anyhow::Result;
use thiserror::Error;
use traits::{Extent, InsertableDyn};
#[derive(Error, Debug)]
enum TableError {
    #[error("Key is out of range")]
    KeyOutOfRange,
    #[error("Key Unused: (#key)")]
    KeyUnused { key: Key },
    #[error("Invalid Extent size(#size)")]
    InvalidExtentSize { size: usize },
}
#[derive(Clone, Debug)]
pub struct Key {
    index: usize,
}
///SizedTable Layout
///
///|Size (bytes) | Datatype| Description|
///|-------------|---------|------------|
///|8|Header|Contains size of containing data|
///|BlockSize/8|Bitmap| Bitmap containing whether or not item is used (1 if occupied 0 if unused)|
///|BlockSize*DataSize|Block Data|Contains data for blocks|
///
///Bitmap and block data repeat
pub struct SizedTable<E: Extent> {
    extent: E,
    data_size: usize,
}
impl<E: Extent> SizedTable<E> {
    const HEADER_SIZE: usize = 8;
    const BITMAP_SIZE: usize = 0xff;
    const BLOCK_SIZE: usize = Self::BITMAP_SIZE * 8;
    /// Tries to load table from extent. Fails if extent is in invalid state
    pub fn try_load(extent: E) -> Result<Self> {
        if extent.len() < 8 {
            return Err(anyhow!(
                "{}",
                TableError::InvalidExtentSize { size: extent.len() }
            ));
        }
        let l: Vec<u8> = (0..8).map(|i| extent[i]).collect();
        let data_size = usize::from_le_bytes([l[0], l[1], l[2], l[3], l[4], l[5], l[6], l[7]]);
        if (extent.len() - Self::HEADER_SIZE) % (Self::BITMAP_SIZE + Self::BLOCK_SIZE * data_size)
            != 0
        {
            return Err(anyhow!(
                "{}",
                TableError::InvalidExtentSize { size: extent.len() }
            ));
        }
        Ok(Self { extent, data_size })
    }
    /// Creates a fresh table.
    pub fn new(mut extent: E, data_size: usize) -> Result<Self> {
        extent.resize(Self::HEADER_SIZE)?;
        let data_buffer = data_size.to_le_bytes();
        for i in 0..8 {
            extent[i] = data_buffer[i];
        }
        Ok(Self { extent, data_size })
    }
    /// Inserts data into table.
    /// Linear insertion time
    pub fn insert(&mut self, data: Box<dyn InsertableDyn>) -> Result<Key> {
        //first iterate through bitmaps then if one is unused fill it else append new block
        for i in 0..self.get_number_blocks() {
            if let Some(index) = get_first_0(self.load_bitmap(i)) {
                let start = index * self.data_size
                    + i * (Self::BLOCK_SIZE * self.data_size + Self::BITMAP_SIZE)
                    + Self::BITMAP_SIZE
                    + Self::HEADER_SIZE;
                let buffer = data.to_binary();
                for i in 0..self.data_size {
                    self.extent[i + start] = buffer[i];
                }
                let bitmap_location = i * (Self::BLOCK_SIZE * self.data_size + Self::BITMAP_SIZE)
                    + Self::HEADER_SIZE
                    + index / 8;

                self.extent[bitmap_location] = self.extent[bitmap_location] | 1 << (index % 8);
                return Ok(Key { index });
            }
        }
        let old_len = self.extent.len();
        let new_index = self.get_number_blocks() * Self::BLOCK_SIZE;
        self.extent
            .resize(old_len + Self::BITMAP_SIZE + Self::BLOCK_SIZE * self.data_size)?;
        for i in old_len..old_len + Self::BITMAP_SIZE {
            self.extent[i] = 0;
        }
        let buffer = data.to_binary();
        for i in 0..self.data_size {
            self.extent[i + old_len + Self::BITMAP_SIZE] = buffer[i];
        }
        self.extent[old_len] = 0x1;
        Ok(Key { index: new_index })
    }
    /// Gets data from key
    /// constant retrival time
    pub fn get<Data: InsertableDyn>(&self, key: Key, ctor: fn(Vec<u8>) -> Data) -> Result<Data> {
        if ((key.index / Self::BLOCK_SIZE) as i64) < self.get_number_blocks() as i64 - 1 {
            Err(anyhow!("{}", TableError::KeyOutOfRange))
        } else {
            let block_number = key.index / Self::BLOCK_SIZE;
            let bitmap = self.load_bitmap(block_number)[(key.index % Self::BLOCK_SIZE) / 8];
            let index_in_bitmap = (key.index % Self::BLOCK_SIZE) % 8;
            let bit = bitmap & (0x1 << index_in_bitmap) >> index_in_bitmap;
            if bit == 0 {
                return Err(anyhow!("{}", TableError::KeyUnused { key }));
            }
            let block = self.load_block(block_number);
            let start_index = (key.index % (Self::BLOCK_SIZE)) * self.data_size;
            let data = (start_index..start_index + self.data_size)
                .map(|i| block[i])
                .collect();
            Ok(ctor(data))
        }
    }
    fn get_number_blocks(&self) -> usize {
        (self.extent.len() - Self::HEADER_SIZE)
            / (Self::BITMAP_SIZE + Self::BLOCK_SIZE * self.data_size)
    }
    fn load_bitmap(&self, block_number: usize) -> Vec<u8> {
        let start = block_number * (Self::BLOCK_SIZE * self.data_size + Self::BITMAP_SIZE)
            + Self::HEADER_SIZE;
        (start..start + Self::BITMAP_SIZE)
            .map(|i| self.extent[i].clone())
            .collect()
    }
    fn load_block(&self, block_number: usize) -> Vec<u8> {
        let start = block_number * (Self::BLOCK_SIZE * self.data_size + Self::BITMAP_SIZE)
            + Self::HEADER_SIZE
            + Self::BITMAP_SIZE;
        (start..start + Self::BLOCK_SIZE * self.data_size)
            .map(|i| self.extent[i].clone())
            .collect()
    }
}
//gets first 0 in bitmap if it exists
fn get_first_0(bitmap: Vec<u8>) -> Option<usize> {
    let mut index = 0;
    for byte in bitmap.iter() {
        if byte.clone() != u8::MAX {
            for i in 0..8 {
                if byte & (1 << i) != (1 << i) {
                    return Some(index * 8 + i);
                }
            }
        }
        index += 1;
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    use traits::{DrianableExtent, InMemoryExtent, Insertable};

    #[test]
    fn it_works() {
        let mut e = InMemoryExtent::new();
        let drain = DrianableExtent::new(&mut e);
        let mut t = SizedTable::new(drain, 0usize.size() as usize).ok().unwrap();
        let k = t.insert(Box::new(0usize)).ok().unwrap();
        assert_eq!(t.get(k, usize::from_binary).ok().unwrap(), 0);
    }
    #[test]
    fn two_inserts() {
        let mut e = InMemoryExtent::new();
        let drain = DrianableExtent::new(&mut e);
        let mut t = SizedTable::new(drain, 0usize.size() as usize).ok().unwrap();
        let k = t.insert(Box::new(0usize)).ok().unwrap();
        assert_eq!(k.index, 0);
        assert_eq!(t.get(k.clone(), usize::from_binary).ok().unwrap(), 0);
        let k2 = t.insert(Box::new(1usize)).ok().unwrap();
        assert_eq!(k2.index, 1);

        assert_eq!(t.get(k, usize::from_binary).ok().unwrap(), 0);
        assert_eq!(t.get(k2.clone(), usize::from_binary).ok().unwrap(), 1);
    }
    #[test]
    fn test_bitmap_one() {
        let arr = vec![0];
        assert_eq!(get_first_0(arr), Some(0));
    }
    #[test]
    fn test_bitmap_two() {
        let arr = vec![0b01];
        assert_eq!(get_first_0(arr), Some(1));
    }
    #[test]
    fn couple_inserts() {
        let mut e = InMemoryExtent::new();
        let drain = DrianableExtent::new(&mut e);
        let mut t = SizedTable::new(drain, 0usize.size() as usize).ok().unwrap();
        let k_v: Vec<(Key, usize)> = (0..10_000)
            .map(|i| (t.insert(Box::new(i)).ok().unwrap(), i.clone()))
            .collect();
        for (key, value) in k_v.iter() {
            assert_eq!(
                t.get(key.clone(), usize::from_binary).ok().unwrap(),
                value.clone()
            );
        }
    }
    #[test]
    fn recover() {
        let mut e = InMemoryExtent::new();
        let mut drain = DrianableExtent::new(&mut e);
        let mut drain2 = drain.take();
        let mut t = SizedTable::new(drain, 0usize.size() as usize).ok().unwrap();
        let k = t.insert(Box::new(0usize)).ok().unwrap();
        assert_eq!(t.get(k.clone(), usize::from_binary).ok().unwrap(), 0);
        let mut t2 = SizedTable::try_load(drain2).ok().unwrap();
        assert_eq!(t2.get(k, usize::from_binary).ok().unwrap(), 0);
    }
}

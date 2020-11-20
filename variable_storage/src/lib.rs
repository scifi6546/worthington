use std::cmp::{max, min};
use std::ops::{Index, IndexMut};
use table::Insertable;
unsafe impl Insertable for Key {
    fn size(&self) -> u32 {
        8
    }
    fn to_binary(&self) -> Vec<u8> {
        self.index.to_le_bytes().to_vec()
    }
}
pub trait Extent: Index<usize, Output = u8> + IndexMut<usize, Output = u8> {
    /// Resizes extent. If extent is grown no garuentees are made about the contents of the new
    /// data
    fn resize(&mut self, new_size: usize);
    /// Gets the number of availible bytes
    fn len(&self) -> usize;
}
#[derive(Clone)]
pub struct Key {
    index: usize,
}
pub struct VariableExtent<ExtentT: Extent> {
    data_store: ExtentT,
}
impl<ExtentT: Extent> VariableExtent<ExtentT> {
    const FAT_BLOCK_SIZE: usize = 0x100;
    const HEADER_SIZE: usize = 0x4 + 0x4 + 0x8;
    const BLOCK_USABLE_SIZE: usize = Self::FAT_BLOCK_SIZE - Self::HEADER_SIZE;
    /// Creates a new Extent
    pub fn new(mut data_store: ExtentT) -> Self {
        data_store.resize(Self::FAT_BLOCK_SIZE);

        let one_buff = (1 as u32).to_le_bytes();
        for i in 0..4 {
            data_store[i] = one_buff[i];
        }
        for i in 4..Self::FAT_BLOCK_SIZE {
            data_store[i as usize] = 0;
        }
        Self { data_store }
    }
    /// Gets the data associated with a key
    pub fn get_entry(&self, key: Key) -> Vec<u8> {
        let fat = self.find_key(key);
        self.load_block(fat)
    }
    /// Adds a new Entery with the specified data
    pub fn add_entry(&mut self, buffer: Vec<u8>) -> Key {
        let key_buffer = self.load_block(0);
        let free_key = self.find_free_entery();
        self.initilize_block(free_key);
        self.append_block(free_key, buffer);
        self.append_block(0, free_key.to_le_bytes().to_vec());
        return Key {
            index: key_buffer.len() / std::mem::size_of::<u64>(),
        };
    }
    pub fn contains_key(&self, key: Key) -> bool {
        let listing = self.load_block(0);
        if key.index * 8 >= listing.len() {
            false
        } else {
            true
        }
    }
    /// Finds a free fat entery. Does not initilize entry
    fn find_free_entery(&mut self) -> usize {
        for i in 1..self.get_number_blocks() {
            let is_used = {
                let buff: Vec<u8> = (i * Self::FAT_BLOCK_SIZE..(i + 1) * Self::FAT_BLOCK_SIZE)
                    .map(|j| self.data_store[j])
                    .collect();
                u32::from_le_bytes([buff[0], buff[1], buff[2], buff[3]])
            };
            if is_used == 0 {
                return i;
            }
        }
        let new_key = self.get_number_blocks();
        self.data_store.resize((new_key + 1) * Self::FAT_BLOCK_SIZE);
        return new_key;
    }
    fn get_number_blocks(&self) -> usize {
        return self.data_store.len() / Self::FAT_BLOCK_SIZE;
    }
    /// Writes new data to a entery specified at a index. Resizes if needed
    /// Index must be inside of buffer. todo: add better error handeling
    pub fn write_entry(&mut self, key: Key, index: usize, mut buffer: Vec<u8>) {
        let mut block_number = self.find_key(key);
        let mut traversed_size = 0;
        loop {
            let size = self.get_block_size(block_number);
            // Hit index
            if !((index + buffer.len() < traversed_size) || index > traversed_size) {
                //if buffer is completly contaned inside of block
                if buffer.len() + index - traversed_size <= Self::BLOCK_USABLE_SIZE {
                    for i in 0..buffer.len() {
                        self.data_store[block_number * Self::FAT_BLOCK_SIZE
                            + Self::HEADER_SIZE
                            + index
                            - traversed_size
                            + i] = buffer[i];
                    }
                    let new_size = max(size, index - traversed_size + buffer.len());
                    self.set_block_size(block_number, new_size);
                    return;
                //else write to end of buffer
                } else {
                    let start_index = block_number * Self::FAT_BLOCK_SIZE
                        + Self::HEADER_SIZE
                        + if traversed_size > index {
                            0
                        } else {
                            index - traversed_size
                        };
                    let copy_size = min(Self::BLOCK_USABLE_SIZE, buffer.len());
                    //let copy_size = Self::BLOCK_USABLE_SIZE - (index - traversed_size);
                    for i in 0..copy_size {
                        self.data_store[i + start_index] = buffer[i];
                    }
                    buffer = buffer.split_off(copy_size);
                    self.set_block_size(block_number, Self::BLOCK_USABLE_SIZE);
                    let new_block = self.find_free_entery();
                    self.initilize_block(new_block);
                    self.set_next_block(block_number, new_block);
                    traversed_size += copy_size;
                    block_number = new_block;
                    let new_size = min(Self::BLOCK_USABLE_SIZE, buffer.len());
                    self.set_block_size(block_number, new_size);
                }
            } else {
                let next_block = self.get_next_block(block_number);
                if next_block == 0 {
                    todo!("handle error if index is outside of last block");
                } else {
                    traversed_size += self.get_block_size(block_number);
                    block_number = next_block;
                }
            }
        }
    }
    /// Initilizes a block to zero size
    fn initilize_block(&mut self, block_num: usize) {
        assert!(block_num * Self::FAT_BLOCK_SIZE <= self.data_store.len());
        let block_start = block_num * Self::FAT_BLOCK_SIZE;

        let is_used_bytes = 1u32.to_le_bytes();
        for i in 0..4 {
            self.data_store[i + block_start] = is_used_bytes[i];
        }
        let size_bytes = 0u32.to_le_bytes();
        for i in 0..4 {
            self.data_store[i + 4 + block_start] = size_bytes[i];
        }
        let next_addr_bytes = 0u64.to_le_bytes();
        for i in 0..8 {
            self.data_store[i + 8 + block_start] = next_addr_bytes[i];
        }
    }
    /// Appends data to the end of a block. allocates new blocks as needed
    fn append_block(&mut self, mut block_num: usize, data: Vec<u8>) {
        loop {
            let size = self.get_block_size(block_num);
            //if there is space in current block
            if size < Self::FAT_BLOCK_SIZE - Self::HEADER_SIZE {
                //if the data can be wholy contained in a sngle block
                if size + data.len() <= Self::FAT_BLOCK_SIZE - Self::HEADER_SIZE {
                    let start_index = block_num * Self::FAT_BLOCK_SIZE + Self::HEADER_SIZE + size;
                    for i in 0..data.len() {
                        self.data_store[start_index + i] = data[i];
                    }
                    self.set_block_size(block_num, size + data.len());
                    return;
                // The data can not be contained in a single block
                } else {
                    let start_index = block_num * Self::FAT_BLOCK_SIZE + Self::HEADER_SIZE + size;
                    for i in 0..(Self::FAT_BLOCK_SIZE - Self::HEADER_SIZE) - size {
                        self.data_store[i + start_index] = data[i];
                    }
                    self.set_block_size(block_num, Self::FAT_BLOCK_SIZE - Self::HEADER_SIZE);
                    assert_eq!(self.get_next_block(block_num), 0);
                    let next_block = self.find_free_entery();
                    self.initilize_block(next_block);
                    self.set_next_block(block_num, next_block);
                    block_num = next_block;
                }
            //If current block is full
            } else {
                let mut next_block = self.get_next_block(block_num);
                //allocate new block
                if next_block == 0 {
                    next_block = self.find_free_entery();
                    self.initilize_block(next_block);
                    self.set_next_block(block_num, next_block);
                }
                block_num = next_block;
            }
        }
    }
    /// gets the next block number
    fn get_next_block(&self, block: usize) -> usize {
        assert!(block * Self::FAT_BLOCK_SIZE <= self.data_store.len());
        let mut next_buff = [0; 8];
        for i in 0..8 {
            next_buff[i] = self.data_store[block * Self::FAT_BLOCK_SIZE + 8 + i];
        }
        u64::from_le_bytes(next_buff) as usize
    }
    /// Sets the next block header in block
    fn set_next_block(&mut self, block: usize, next_block: usize) {
        assert!(block * Self::FAT_BLOCK_SIZE <= self.data_store.len());
        let next_buff = next_block.to_le_bytes();
        for i in 0..8 {
            self.data_store[block * Self::FAT_BLOCK_SIZE + 8 + i] = next_buff[i];
        }
    }
    //gets the allocated size of a given block
    fn get_block_size(&self, block: usize) -> usize {
        assert!(block * Self::FAT_BLOCK_SIZE <= self.data_store.len());
        let mut size_buff = [0; 4];
        for i in 0..4 {
            size_buff[i] = self.data_store[block * Self::FAT_BLOCK_SIZE + 4 + i];
        }
        u32::from_le_bytes(size_buff) as usize
    }
    fn set_block_size(&mut self, block: usize, new_size: usize) {
        assert!(block * Self::FAT_BLOCK_SIZE <= self.data_store.len());
        let size_buff = new_size.to_le_bytes();
        for i in 0..4 {
            self.data_store[block * Self::FAT_BLOCK_SIZE + 4 + i] = size_buff[i];
        }
    }
    fn load_block(&self, block_num: usize) -> Vec<u8> {
        let mut block_start = block_num * Self::FAT_BLOCK_SIZE;
        let mut buff = vec![];
        loop {
            let is_used = {
                let buff: Vec<u8> = (block_start..block_start + 4)
                    .map(|i| self.data_store[i])
                    .collect();
                u32::from_le_bytes([buff[0], buff[1], buff[2], buff[3]])
            };
            if is_used != 1 {
                panic!()
            }
            let size = {
                let buff: Vec<u8> = (block_start + 4..block_start + 8)
                    .map(|i| self.data_store[i])
                    .collect();
                u32::from_le_bytes([buff[0], buff[1], buff[2], buff[3]])
            };
            let next_addr = {
                let buff: Vec<u8> = (block_start + 8..block_start + 16)
                    .map(|i| self.data_store[i])
                    .collect();
                u64::from_le_bytes([
                    buff[0], buff[1], buff[2], buff[3], buff[4], buff[5], buff[6], buff[7],
                ])
            };
            for i in 0..size {
                buff.push(self.data_store[i as usize + block_start + 16]);
            }
            if next_addr == 0 {
                return buff;
            } else {
                block_start = next_addr as usize * Self::FAT_BLOCK_SIZE;
            }
        }
    }
    /// Finds the fat block associated with the key in the key listing table
    fn find_key(&self, key: Key) -> usize {
        let listing = self.load_block(0);
        let mut block_number = [0; 8];
        for i in 0..8 {
            block_number[i] = listing[key.index * 8 + i]
        }
        u64::from_le_bytes(block_number) as usize
    }
}
pub struct InMemoryExtent {
    data: Vec<u8>,
}
impl InMemoryExtent {
    pub fn new() -> Self {
        InMemoryExtent { data: vec![] }
    }
}
impl Index<usize> for InMemoryExtent {
    type Output = u8;
    fn index(&self, idx: usize) -> &Self::Output {
        if idx >= self.data.len() {
            panic!("index out of bounds")
        }
        &self.data[idx]
    }
}
impl IndexMut<usize> for InMemoryExtent {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        if idx >= self.data.len() {
            panic!("index mut out of bounds")
        }
        &mut self.data[idx]
    }
}
impl Extent for InMemoryExtent {
    fn resize(&mut self, new_size: usize) {
        self.data.resize(new_size, 0)
    }
    fn len(&self) -> usize {
        self.data.len()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_extent() {
        let _ = VariableExtent::new(InMemoryExtent::new());
    }
    #[test]
    fn add_zero_length() {
        let mut e = VariableExtent::new(InMemoryExtent::new());
        let key = e.add_entry(vec![]);
        assert_eq!(e.get_entry(key), vec![]);
    }
    #[test]
    fn write_data() {
        let mut e = VariableExtent::new(InMemoryExtent::new());
        let key = e.add_entry(vec![]);
        e.write_entry(key.clone(), 0, vec![1]);
        assert_eq!(e.get_entry(key), vec![1]);
    }
    #[test]
    fn write_empty() {
        let mut e = VariableExtent::new(InMemoryExtent::new());
        let key = e.add_entry(vec![]);
        let v: Vec<u8> = (1..10000).map(|_| 0).collect();
        e.write_entry(key.clone(), 0, v.clone());
        assert_eq!(e.get_entry(key), v);
    }
    #[test]
    fn write_several() {
        let mut e = VariableExtent::new(InMemoryExtent::new());
        let v: Vec<(Key, u8)> = (0..100)
            .map(|i| (e.add_entry(vec![i.clone()]), i.clone()))
            .collect();
        for (key, data) in v.iter() {
            assert_eq!(e.get_entry(key.clone()), vec![data.clone()]);
        }
    }
    #[test]
    fn contains_key() {
        let mut e = VariableExtent::new(InMemoryExtent::new());
        let fake = Key { index: 100 };
        assert_eq!(e.contains_key(fake), false);
        let real = e.add_entry(vec![]);
        assert_eq!(e.contains_key(real), true);
    }
}

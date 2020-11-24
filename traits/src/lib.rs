#[macro_use]
extern crate macro_lib;

mod node_base;
use dyn_clonable::*;
pub use node_base::{Node, NodeElementHash, NodeHash};
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
#[clonable]
pub trait VariableSizeInsert: Clone {
    fn get_data(&self) -> Vec<u8>;
}
impl VariableSizeInsert for String {
    fn get_data(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
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
unsafe impl Insertable for f32 {
    const SIZE: usize = 4;
    fn from_binary(d: Vec<u8>) -> Self {
        f32::from_le_bytes([d[0], d[1], d[2], d[3]])
    }
}
unsafe impl InsertableDyn for f32 {
    fn size(&self) -> u32 {
        4
    }
    fn to_binary(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

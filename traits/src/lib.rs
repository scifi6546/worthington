#[macro_use]
extern crate macro_lib;
use std::ops::{Index, IndexMut};

mod insertable_impl;
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
#[clonable]
pub trait VariableSizeInsert: Clone {
    fn get_data_variable(&self) -> Vec<u8>;
}
impl VariableSizeInsert for String {
    fn get_data_variable(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}
pub trait Extent: Index<usize, Output = u8> + IndexMut<usize, Output = u8> {
    /// Resizes extent. If extent is grown no garuentees are made about the contents of the new
    /// data
    fn resize(&mut self, new_size: usize);
    /// Gets the number of availible bytes
    fn len(&self) -> usize;
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

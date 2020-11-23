#[macro_use]
extern crate macro_lib;
use graph::{Node, NodeElementHash, NodeHash};
use traits::{Insertable, InsertableDyn, VariableSizeInsert};
#[derive(NodeT)]
struct Bar {}
fn main() {
    println!("Hello, world!");
}

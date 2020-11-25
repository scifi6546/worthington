#[macro_use]
extern crate macro_lib;
use traits::{InsertableDyn, Node, NodeElementHash, NodeHash, VariableSizeInsert};
#[derive(GraphInsertable)]
struct Bar {}
#[derive(GraphInsertable)]
struct Foo {}
#[derive(GraphInsertable)]
struct Person {
    age: f32,
}
fn main() {
    println!("Foo: {}", Foo::SELF_HASH.hash);
    println!("Bar: {}", Bar::SELF_HASH.hash);
}

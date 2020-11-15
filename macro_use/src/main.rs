#[macro_use]
extern crate macro_lib;
fn main() {
    database_schema!(foo);
    println!("Hello, world!");
}

use graph::prelude::*;
#[macro_use]
extern crate macro_lib;
#[derive(GraphInsertable, Debug, PartialEq)]
struct Empty {}
/*
#[derive(GraphInsertable, Debug, PartialEq)]
struct Person {
    age: u64,
    name: String,
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use graph::Database;
    #[test]
    fn it_works() {
        let mut db = Database::new();
        let e = db.insert(Empty {});
        assert_eq!(db.get::<Empty>(e).unwrap(), Empty {});
    }
}

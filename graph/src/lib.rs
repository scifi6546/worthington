use std::collections::HashMap;
use table::DatabaseTable;
use variable_storage::{InMemoryExtent, VariableExtent};
#[derive(Clone)]
pub struct Key {}
struct NodeElementHash {
    hash: usize,
}
pub trait Node {
    //hash of the database name
    const Hash: usize;
}
pub struct Database {
    //For elements with a variable size
    variable: HashMap<NodeElementHash, VariableExtent<InMemoryExtent>>,
    //For elements with a fixed size
}
impl Database {
    pub fn new() -> Self {
        unimplemented!()
    }
    pub fn insert<Data: Node>(&mut self, data: Data) -> Key {
        unimplemented!()
    }
    pub fn connect(&mut self, key1: Key, key2: Key) {
        unimplemented!()
    }
    pub fn get_connected(&self, key: Key) -> Vec<Key> {
        unimplemented!()
    }
    pub fn get<Data: Node>(&self, key: Key) -> Option<Data> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(PartialEq, Debug)]
    struct Person {
        name: String,
    }
    #[derive(PartialEq, Debug)]
    struct Pet {
        species: String,
    }
    impl Node for Person {
        const Hash: usize = 0;
    }
    impl Node for Pet {
        const Hash: usize = 1;
    }
    #[test]
    fn basic_api() {
        let mut db = Database::new();
        let bill = db.insert(Person {
            name: "Bill".to_string(),
        });
        let dog = db.insert(Pet {
            species: "dog".to_string(),
        });
        db.connect(bill, dog.clone());
        assert_eq!(
            db.get::<Person>((db.get_connected(dog)[0]).clone())
                .unwrap(),
            Person {
                name: "Bill".to_string()
            }
        );

        assert_eq!(2 + 2, 4);
    }
}

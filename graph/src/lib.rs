use dyn_clonable::*;
use std::collections::HashMap;
use table::{DatabaseTable, Insertable, Key as TableKey};
use variable_storage::{InMemoryExtent, Key as VariableKey, VariableExtent};
trait VariableSizeInsert {
    fn get_data(&self) -> Vec<u8>;
}
impl VariableSizeInsert for String {
    fn get_data(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}
#[derive(Clone)]
pub struct Key {
    key: VariableKey,
}
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct NodeElementHash {
    hash: usize,
}
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct NodeHash {
    hash: usize,
}
#[derive(Clone)]
struct NodeKeyStorage {
    //link to self members in node_contents keys
    self_members: TableKey,
    //Links
    linked_nodes: Vec<VariableKey>,
}
impl VariableSizeInsert for NodeKeyStorage {
    fn get_data(&self) -> Vec<u8> {
        let mut buffer = self.self_members.to_binary();
        for key in self.linked_nodes.iter() {
            buffer.append(&mut key.to_binary());
        }
        buffer
    }
}
unsafe impl Insertable for NodeElementHash {
    fn size(&self) -> u32 {
        8
    }
    fn to_binary(&self) -> Vec<u8> {
        self.hash.to_le_bytes().to_vec()
    }
}
#[derive(Clone)]
struct NodeStorage {
    node_static_sized_keys: Vec<(NodeElementHash, TableKey)>,
    node_dynamic_sized_keys: Vec<(NodeElementHash, VariableKey)>,
}
unsafe impl Insertable for NodeStorage {
    fn size(&self) -> u32 {
        let static_size = if self.node_static_sized_keys.len() > 0 {
            self.node_static_sized_keys.len() as u32
                * (self.node_static_sized_keys[0].0.size()
                    + self.node_static_sized_keys[0].1.size())
        } else {
            0
        };
        let variable_size = if self.node_dynamic_sized_keys.len() > 0 {
            self.node_dynamic_sized_keys.len() as u32
                * (self.node_dynamic_sized_keys[0].0.size()
                    + self.node_dynamic_sized_keys[0].1.size())
        } else {
            0
        };
        static_size + variable_size
    }
    fn to_binary(&self) -> Vec<u8> {
        let mut buffer = vec![];
        for (hash, key) in self.node_static_sized_keys.iter() {
            buffer.append(&mut hash.clone().to_binary());
            buffer.append(&mut key.clone().to_binary());
        }
        for (hash, key) in self.node_dynamic_sized_keys.iter() {
            buffer.append(&mut hash.clone().to_binary());
            buffer.append(&mut key.clone().to_binary());
        }
        buffer
    }
}
/// Schema of a Node
struct NodeSchema {}
pub trait Node {
    //hash of the database name
    const HASH: NodeHash;
    fn get_data(
        &self,
    ) -> (
        Vec<(NodeElementHash, Box<dyn Insertable>)>,
        Vec<(NodeElementHash, Box<dyn VariableSizeInsert>)>,
    );
}
pub struct Database {
    //Listing of all node elements keys
    node_storage: VariableExtent<InMemoryExtent>,
    //Listing of location of data members of node
    node_contents: HashMap<NodeHash, DatabaseTable>,
    //For elements with a variable size
    variable: HashMap<NodeElementHash, VariableExtent<InMemoryExtent>>,
    sized: HashMap<NodeElementHash, DatabaseTable>, //For elements with a fixed size
}
impl Database {
    pub fn new() -> Self {
        Self {
            node_storage: VariableExtent::new(InMemoryExtent::new()),
            node_contents: HashMap::new(),
            variable: HashMap::new(),
            sized: HashMap::new(),
        }
    }
    pub fn insert<Data: Node>(&mut self, data: Data) -> Key {
        let (sized_data_vec, unsized_data_vec) = data.get_data();
        let node_static_sized_keys: Vec<(NodeElementHash, TableKey)> = sized_data_vec
            .iter()
            .map(|(hash, data)| {
                if self.sized.contains_key(hash) {
                    (
                        hash.clone(),
                        (self.sized.get_mut(hash).unwrap())
                            .insert::<Box<dyn Insertable>>(data.clone()),
                    )
                } else {
                    //add hash
                    self.sized.insert(hash.clone(), DatabaseTable::new());
                    (hash.clone(), self.sized.get_mut(hash).unwrap().insert(data))
                }
            })
            .collect();
        let node_dynamic_sized_keys: Vec<(NodeElementHash, VariableKey)> = unsized_data_vec
            .iter()
            .map(|(hash, data)| {
                if self.variable.contains_key(hash) {
                    (
                        hash.clone(),
                        self.variable
                            .get_mut(hash)
                            .unwrap()
                            .add_entry(data.get_data()),
                    )
                } else {
                    self.variable
                        .insert(hash.clone(), VariableExtent::new(InMemoryExtent::new()));
                    (
                        hash.clone(),
                        self.variable
                            .get_mut(hash)
                            .unwrap()
                            .add_entry(data.get_data()),
                    )
                }
            })
            .collect();
        let node = NodeStorage {
            node_static_sized_keys,
            node_dynamic_sized_keys,
        };
        if !self.node_contents.contains_key(&Data::HASH) {
            self.node_contents.insert(Data::HASH, DatabaseTable::new());
        }
        let key = self
            .node_contents
            .get_mut(&Data::HASH)
            .unwrap()
            .insert(node);
        let node_keys = NodeKeyStorage {
            self_members: key,
            //Links
            linked_nodes: vec![],
        };
        Key {
            key: self.node_storage.add_entry(node_keys.get_data()),
        }
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
        const HASH: NodeHash = NodeHash { hash: 0 };
        fn get_data(
            &self,
        ) -> (
            Vec<(NodeElementHash, Box<dyn Insertable>)>,
            Vec<(NodeElementHash, Box<dyn VariableSizeInsert>)>,
        ) {
            (
                vec![],
                vec![(NodeElementHash { hash: 0 }, Box::new(self.name.clone()))],
            )
        }
    }
    impl Node for Pet {
        const HASH: NodeHash = NodeHash { hash: 1 };
        fn get_data(
            &self,
        ) -> (
            Vec<(NodeElementHash, Box<dyn Insertable>)>,
            Vec<(NodeElementHash, Box<dyn VariableSizeInsert>)>,
        ) {
            (
                vec![],
                vec![(NodeElementHash { hash: 1 }, Box::new(self.species.clone()))],
            )
        }
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

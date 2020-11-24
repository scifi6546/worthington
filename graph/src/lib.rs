use std::collections::HashMap;
use table::{DatabaseTable, Key as TableKey};
use traits::{Insertable, InsertableDyn, Node, NodeElementHash, NodeHash, VariableSizeInsert};
pub mod prelude {
    pub use traits::{
        Insertable, InsertableDyn, Node, NodeElementHash, NodeHash, VariableSizeInsert,
    };
}
use variable_storage::{InMemoryExtent, Key as VariableKey, VariableExtent};
#[derive(Clone)]
pub struct Key {
    key: VariableKey,
}
#[derive(Clone)]
struct NodeKeyStorage {
    //link to self members in node_contents keys
    self_members: TableKey,
    //hash of self
    self_hash: NodeHash,
    //Links
    linked_nodes: Vec<VariableKey>,
}
impl VariableSizeInsert for NodeKeyStorage {
    fn get_data(&self) -> Vec<u8> {
        let mut buffer = self.self_members.to_binary();
        buffer.append(&mut self.self_hash.to_binary());
        for key in self.linked_nodes.iter() {
            buffer.append(&mut key.to_binary());
        }
        buffer
    }
}
impl NodeKeyStorage {
    fn from_binary(data: Vec<u8>) -> Self {
        let num_keys = (data.len() - TableKey::SIZE - NodeHash::SIZE) / VariableKey::SIZE;
        let self_members = TableKey::from_binary(data.clone());
        let self_hash = NodeHash::from_binary(
            data.clone()[TableKey::SIZE..TableKey::SIZE + NodeHash::SIZE].to_vec(),
        );
        let linked_nodes = (0..num_keys)
            .map(|i| {
                VariableKey::from_binary(
                    data.clone()[i * VariableKey::SIZE + TableKey::SIZE + NodeHash::SIZE
                        ..i * VariableKey::SIZE
                            + TableKey::SIZE
                            + VariableKey::SIZE
                            + NodeHash::SIZE]
                        .to_vec(),
                )
            })
            .collect();
        Self {
            self_members,
            self_hash,
            linked_nodes,
        }
    }
}
#[derive(Clone)]
struct NodeStorage {
    node_static_sized_keys: Vec<(NodeElementHash, TableKey)>,
    node_dynamic_sized_keys: Vec<(NodeElementHash, VariableKey)>,
}
unsafe impl InsertableDyn for NodeStorage {
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
        static_size + variable_size + 8 + 8
    }
    fn to_binary(&self) -> Vec<u8> {
        let mut buffer = vec![];
        buffer.append(&mut self.node_static_sized_keys.len().to_le_bytes().to_vec());
        for (hash, key) in self.node_static_sized_keys.iter() {
            buffer.append(&mut hash.clone().to_binary());
            buffer.append(&mut key.clone().to_binary());
        }
        buffer.append(&mut self.node_dynamic_sized_keys.len().to_le_bytes().to_vec());
        for (hash, key) in self.node_dynamic_sized_keys.iter() {
            buffer.append(&mut hash.clone().to_binary());
            buffer.append(&mut key.clone().to_binary());
        }
        buffer
    }
}
impl NodeStorage {
    fn from_binary(d: Vec<u8>) -> Self {
        let sized_len = usize::from_le_bytes([d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]]);
        let node_static_size = NodeElementHash::SIZE + TableKey::SIZE;
        let node_static_sized_keys = (0..sized_len)
            .map(|i| {
                (
                    NodeElementHash::from_binary(
                        d[sized_len + node_static_size * i
                            ..sized_len + node_static_size * i + NodeElementHash::SIZE]
                            .to_vec(),
                    ),
                    TableKey::from_binary(
                        d[sized_len + node_static_size * i + NodeElementHash::SIZE
                            ..sized_len
                                + node_static_size * i
                                + NodeElementHash::SIZE
                                + TableKey::SIZE]
                            .to_vec(),
                    ),
                )
            })
            .collect();

        let sized_size = sized_len * (NodeElementHash::SIZE + TableKey::SIZE) + usize::SIZE;
        let unsized_len = usize::from_le_bytes([
            d[sized_size + 0],
            d[sized_size + 1],
            d[sized_size + 2],
            d[sized_size + 3],
            d[sized_size + 4],
            d[sized_size + 5],
            d[sized_size + 6],
            d[sized_size + 7],
        ]);

        let variable_size = NodeElementHash::SIZE + VariableKey::SIZE;
        let node_dynamic_sized_keys = (0..unsized_len)
            .map(|i| {
                (
                    NodeElementHash::from_binary(
                        d[sized_size + 8 + variable_size * i
                            ..sized_size + 8 + variable_size * i + NodeElementHash::SIZE]
                            .to_vec(),
                    ),
                    VariableKey::from_binary(
                        d[sized_size + 8 + variable_size * i + NodeElementHash::SIZE
                            ..sized_size
                                + 8
                                + variable_size * i
                                + NodeElementHash::SIZE
                                + TableKey::SIZE]
                            .to_vec(),
                    ),
                )
            })
            .collect();
        Self {
            node_static_sized_keys,
            node_dynamic_sized_keys,
        }
    }
}
pub enum DatabseError {
    InvalidKey(Key),
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
                            .insert::<Box<dyn InsertableDyn>>(data.clone()),
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
            self_hash: Data::HASH,
            //Links
            linked_nodes: vec![],
        };
        Key {
            key: self.node_storage.add_entry(node_keys.get_data()),
        }
    }
    pub fn connect(&mut self, key1: Key, key2: Key) -> Result<(), DatabseError> {
        if !self.node_storage.contains_key(key1.clone().key) {
            return Err(DatabseError::InvalidKey(key1));
        }
        if !self.node_storage.contains_key(key2.clone().key) {
            return Err(DatabseError::InvalidKey(key2));
        }
        let mut k1_data = self.node_storage.get_entry(key1.clone().key);
        k1_data.append(&mut key2.key.to_binary());
        self.node_storage.write_entry(key1.clone().key, 0, k1_data);
        let mut k2_data = self.node_storage.get_entry(key2.clone().key);
        k2_data.append(&mut key1.key.to_binary());
        self.node_storage.write_entry(key2.key, 0, k2_data);
        Ok(())
    }
    pub fn get_connected(&self, key: Key) -> Vec<Key> {
        let data = NodeKeyStorage::from_binary(self.node_storage.get_entry(key.key));
        return data
            .linked_nodes
            .iter()
            .map(|key| Key { key: key.clone() })
            .collect();
    }
    pub fn get<Data: Node>(&self, key: Key) -> Option<Data> {
        let data = NodeKeyStorage::from_binary(self.node_storage.get_entry(key.key));
        let data_locations = self.node_contents[&data.self_hash]
            .get(data.self_members, NodeStorage::from_binary)
            .ok()
            .unwrap();

        let variable = data_locations
            .node_dynamic_sized_keys
            .iter()
            .map(|(hash, key)| (hash.clone(), self.variable[hash].get_entry(key.clone())))
            .collect();
        let sized = data_locations
            .node_static_sized_keys
            .iter()
            .map(|(hash, key)| {
                (
                    hash.clone(),
                    self.sized[hash].get(key.clone(), |d| d).ok().unwrap(),
                )
            })
            .collect();
        Some(Data::from_data(sized, variable))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(PartialEq, Debug, Clone)]
    struct Person {
        name: String,
    }
    #[derive(PartialEq, Debug, Clone)]
    struct Pet {
        species: String,
    }
    impl Node for Person {
        const HASH: NodeHash = NodeHash { hash: 0 };
        fn get_data(
            &self,
        ) -> (
            Vec<(NodeElementHash, Box<dyn InsertableDyn>)>,
            Vec<(NodeElementHash, Box<dyn VariableSizeInsert>)>,
        ) {
            (
                vec![],
                vec![(NodeElementHash { hash: 0 }, Box::new(self.name.clone()))],
            )
        }
        fn from_data(
            sized: Vec<(NodeElementHash, Vec<u8>)>,
            variable: Vec<(NodeElementHash, Vec<u8>)>,
        ) -> Self {
            assert_eq!(sized.len(), 0);
            assert_eq!(variable.len(), 1);
            assert_eq!(variable[0].0, NodeElementHash { hash: 0 });
            Self {
                name: String::from_utf8(variable[0].1.clone()).ok().unwrap(),
            }
        }
    }
    impl Node for Pet {
        const HASH: NodeHash = NodeHash { hash: 1 };
        fn get_data(
            &self,
        ) -> (
            Vec<(NodeElementHash, Box<dyn InsertableDyn>)>,
            Vec<(NodeElementHash, Box<dyn VariableSizeInsert>)>,
        ) {
            (
                vec![],
                vec![(NodeElementHash { hash: 1 }, Box::new(self.species.clone()))],
            )
        }
        fn from_data(
            sized: Vec<(NodeElementHash, Vec<u8>)>,
            variable: Vec<(NodeElementHash, Vec<u8>)>,
        ) -> Self {
            todo!()
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
    }
    #[test]
    fn insert_and_get() {
        let mut db = Database::new();
        let bill_data = Person {
            name: "Bill".to_string(),
        };

        let bill = db.insert(bill_data.clone());
        assert_eq!(db.get::<Person>(bill).unwrap(), bill_data);
    }
}

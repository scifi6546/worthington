use crate::{Insertable, InsertableDyn, VariableSizeInsert};
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct NodeElementHash {
    pub hash: usize,
}
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct NodeHash {
    pub hash: usize,
}
unsafe impl Insertable for NodeElementHash {
    const SIZE: usize = 8;
    fn from_binary(d: Vec<u8>) -> Self {
        Self {
            hash: usize::from_le_bytes([d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]]),
        }
    }
}
unsafe impl InsertableDyn for NodeElementHash {
    fn size(&self) -> u32 {
        8
    }
    fn to_binary(&self) -> Vec<u8> {
        self.hash.to_le_bytes().to_vec()
    }
}
unsafe impl InsertableDyn for NodeHash {
    fn size(&self) -> u32 {
        8
    }
    fn to_binary(&self) -> Vec<u8> {
        self.hash.to_le_bytes().to_vec()
    }
}
unsafe impl Insertable for NodeHash {
    const SIZE: usize = 8;
    fn from_binary(d: Vec<u8>) -> Self {
        Self {
            hash: usize::from_le_bytes([d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]]),
        }
    }
}
pub trait Node {
    //hash of the database name
    const SELF_HASH: NodeHash;
    fn get_sized_hashes() -> Vec<NodeElementHash>;
    fn get_variable_hashes() -> Vec<NodeElementHash>;
    fn get_data(
        &self,
    ) -> (
        Vec<(NodeElementHash, Box<dyn InsertableDyn>)>,
        Vec<(NodeElementHash, Box<dyn VariableSizeInsert>)>,
    );
    fn from_data(
        sized: Vec<(NodeElementHash, Vec<u8>)>,
        variable: Vec<(NodeElementHash, Vec<u8>)>,
    ) -> Self;
}
impl Node for f32 {
    const SELF_HASH: NodeHash = NodeHash { hash: hash!(f32) };
    fn get_sized_hashes() -> Vec<NodeElementHash> {
        vec![NodeElementHash { hash: hash!(f32) }]
    }
    fn get_variable_hashes() -> Vec<NodeElementHash> {
        vec![]
    }

    fn get_data(
        &self,
    ) -> (
        Vec<(NodeElementHash, Box<dyn InsertableDyn>)>,
        Vec<(NodeElementHash, Box<dyn VariableSizeInsert>)>,
    ) {
        return (
            vec![(
                NodeElementHash {
                    hash: Self::SELF_HASH.hash,
                },
                Box::new(self.clone()),
            )],
            vec![],
        );
    }
    fn from_data(
        sized: Vec<(NodeElementHash, Vec<u8>)>,
        _variable: Vec<(NodeElementHash, Vec<u8>)>,
    ) -> Self {
        assert_eq!(sized[0].clone().0.hash, Self::SELF_HASH.hash);
        Self::from_binary(sized[0].1.clone())
    }
}

impl Node for u64 {
    const SELF_HASH: NodeHash = NodeHash { hash: hash!(u64) };
    fn get_sized_hashes() -> Vec<NodeElementHash> {
        vec![NodeElementHash { hash: hash!(u64) }]
    }
    fn get_variable_hashes() -> Vec<NodeElementHash> {
        vec![]
    }

    fn get_data(
        &self,
    ) -> (
        Vec<(NodeElementHash, Box<dyn InsertableDyn>)>,
        Vec<(NodeElementHash, Box<dyn VariableSizeInsert>)>,
    ) {
        return (
            vec![(
                NodeElementHash {
                    hash: Self::SELF_HASH.hash,
                },
                Box::new(self.clone()),
            )],
            vec![],
        );
    }
    fn from_data(
        sized: Vec<(NodeElementHash, Vec<u8>)>,
        _variable: Vec<(NodeElementHash, Vec<u8>)>,
    ) -> Self {
        assert_eq!(sized[0].clone().0.hash, Self::SELF_HASH.hash);
        Self::from_binary(sized[0].1.clone())
    }
}

impl Node for String {
    const SELF_HASH: NodeHash = NodeHash {
        hash: hash!(String),
    };
    fn get_sized_hashes() -> Vec<NodeElementHash> {
        vec![]
    }
    fn get_variable_hashes() -> Vec<NodeElementHash> {
        vec![NodeElementHash {
            hash: hash!(String),
        }]
    }

    fn get_data(
        &self,
    ) -> (
        Vec<(NodeElementHash, Box<dyn InsertableDyn>)>,
        Vec<(NodeElementHash, Box<dyn VariableSizeInsert>)>,
    ) {
        return (
            vec![],
            vec![(
                NodeElementHash {
                    hash: Self::SELF_HASH.hash,
                },
                Box::new(self.clone()),
            )],
        );
    }
    fn from_data(
        _sized: Vec<(NodeElementHash, Vec<u8>)>,
        variable: Vec<(NodeElementHash, Vec<u8>)>,
    ) -> Self {
        assert_eq!(variable[0].clone().0.hash, Self::SELF_HASH.hash);
        Self::from_utf8(variable[0].1.clone()).ok().unwrap()
    }
}

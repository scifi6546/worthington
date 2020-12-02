use std::collections::HashMap;
use table::DatabaseTable;
use traits::{Extent, InMemoryExtent, NodeElementHash, NodeHash};
use variable_storage::VariableExtent;
pub struct TableStartup<E: Extent> {
    pub node_storage: VariableExtent<E>,
    pub node_contents: HashMap<NodeHash, DatabaseTable<E>>,
    pub variable: HashMap<NodeElementHash, VariableExtent<E>>,
    pub sized: HashMap<NodeElementHash, DatabaseTable<E>>,
}
pub trait TableManager {
    type ExtentType: Extent;
    fn get(&mut self) -> TableStartup<Self::ExtentType>;
    fn get_node_contents(
        &mut self,
        hash: NodeHash,
        data_size: usize,
    ) -> DatabaseTable<Self::ExtentType>;
    fn get_sized(
        &mut self,
        hash: NodeElementHash,
        data_size: usize,
    ) -> DatabaseTable<Self::ExtentType>;
    fn get_variable(&mut self, hash: NodeElementHash) -> VariableExtent<Self::ExtentType>;
}
//manages extent in memory
pub struct InMemoryManager {}
impl InMemoryManager {
    pub fn new() -> Self {
        Self {}
    }
}
impl TableManager for InMemoryManager {
    type ExtentType = InMemoryExtent;
    fn get(&mut self) -> TableStartup<Self::ExtentType> {
        TableStartup {
            node_storage: VariableExtent::new(InMemoryExtent::new()),
            node_contents: HashMap::new(),
            variable: HashMap::new(),
            sized: HashMap::new(),
        }
    }
    fn get_node_contents(
        &mut self,
        hash: NodeHash,
        data_size: usize,
    ) -> DatabaseTable<Self::ExtentType> {
        DatabaseTable::new(InMemoryExtent::new(), data_size)
    }
    fn get_sized(
        &mut self,
        hash: NodeElementHash,
        data_size: usize,
    ) -> DatabaseTable<Self::ExtentType> {
        DatabaseTable::new(InMemoryExtent::new(), data_size)
    }
    fn get_variable(&mut self, hash: NodeElementHash) -> VariableExtent<Self::ExtentType> {
        VariableExtent::new(InMemoryExtent::new())
    }
}

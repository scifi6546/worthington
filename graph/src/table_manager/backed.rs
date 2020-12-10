use super::{TableManager, TableStartup};
use anyhow::Result;
use file_extent::FileExtent;
use std::collections::HashMap;
use std::fs::create_dir;
use std::path::Path;
use table::DatabaseTable;
use thiserror::Error;
use traits::{Extent, InMemoryExtent, NodeElementHash, NodeHash};
use variable_storage::VariableExtent;
#[derive(Error, Debug)]
enum BackedManagerError {
    #[error("root file not a directory")]
    RootNotDirectory,
}
pub struct BackedManager {
    root_path: String,
}
impl BackedManager {
    pub fn new(root_path: String) -> Result<Self> {
        let path = Path::new(&root_path);
        if !path.exists() {
            create_dir(path)?;
        } else if !path.is_dir() {
            return Err(anyhow!("{}", BackedManagerError::RootNotDirectory));
        }
        todo!()
    }
}
///File Structure:
///{DB NAME}
///|-node_storage.var
///|-node_contents.static
///|-contents
///||-{hash0}.static
///||-{hash1}.static
///||-{hash2}.static
///     .
///     .
///     .
///||-{hashn}.static
///|-variable
///||-{hash0}.var
///||-{hash1}.var
///||-{hash2}.var
///     .
///     .
///     .
///||-{hashn}.var
///|-static
///||-{hash0}.static
///||-{hash1}.static
///||-{hash2}.static
///     .
///     .
///     .
///||-{hashn}.static
impl TableManager for BackedManager {
    type ExtentType = FileExtent;

    fn get(&mut self) -> Result<TableStartup<Self::ExtentType>> {
        Ok(TableStartup {
            node_storage: VariableExtent::new(FileExtent::new(
                (self.root_path + "/node_storage.var").to_string(),
            )?),
            node_contents: HashMap::new(),
            variable: HashMap::new(),
            sized: HashMap::new(),
        })
    }
    fn get_node_contents(
        &mut self,
        hash: NodeHash,
        data_size: usize,
    ) -> DatabaseTable<Self::ExtentType> {
        todo!()
    }
    fn get_sized(
        &mut self,
        hash: NodeElementHash,
        data_size: usize,
    ) -> DatabaseTable<Self::ExtentType> {
        todo!()
    }
    fn get_variable(&mut self, hash: NodeElementHash) -> VariableExtent<Self::ExtentType> {
        todo!()
    }
}

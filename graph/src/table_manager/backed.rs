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
    #[error("{path} is not a directory")]
    NotDirectory { path: String },
    #[error("contents file not a directory")]
    ContentsNotDirectory,
}
pub struct BackedManager {
    root_path: String,
}
impl BackedManager {
    pub fn new(root_path_string: String) -> Result<Self> {
        let root_path = Path::new(&root_path_string);
        if !root_path.exists() {
            create_dir(root_path)?;
        } else if !root_path.is_dir() {
            return Err(anyhow!("{}", BackedManagerError::RootNotDirectory));
        }
        if !root_path.join("contents").exists() {
            create_dir(root_path.join("contents"))?
        } else if !root_path.join("contents").is_dir() {
            return Err(anyhow!("{}", BackedManagerError::ContentsNotDirectory));
        }
        if !root_path.join("variable").exists() {
            create_dir(root_path.join("variable"))?
        } else if !root_path.join("variable").is_dir() {
            return Err(anyhow!(
                "{}",
                BackedManagerError::NotDirectory {
                    path: root_path.join("variable").to_str().unwrap().to_string()
                }
            ));
        }
        if !root_path.join("static").exists() {
            create_dir(root_path.join("static"))?
        } else if !root_path.join("static").is_dir() {
            return Err(anyhow!(
                "{}",
                BackedManagerError::NotDirectory {
                    path: root_path.join("static").to_str().unwrap().to_string()
                }
            ));
        }
        Ok(Self {
            root_path: root_path_string,
        })
    }
}
///File Structure:
///{DB NAME}
///|-node_storage.var
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
        todo!()
        /*
        let mut node_contents = HashMap::new();
        for entry in Path::new(&self.root_path).join("contents").read_dir()? {
            let entry_hash = entry?.path().file_stem().unwrap().to_str().unwrap();
            let storage = DatabaseTable::new(
                FileExtent::new(entry?.path().to_str().unwrap().to_string())?,
                node_contents_size,
            );
            node_contents.insert(
                NodeHash {
                    hash: entry_hash.parse()?,
                },
                storage,
            );
        }

        Ok(TableStartup {
            node_storage: VariableExtent::new(FileExtent::new(
                (self.root_path.clone() + "/node_storage.var").to_string(),
            )?),

            node_contents: node_contents,
            variable: HashMap::new(),
            sized: HashMap::new(),
        })
        */
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

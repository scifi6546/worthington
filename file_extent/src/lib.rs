#[macro_use]
extern crate anyhow;
use anyhow::Result;
use libc::{
    __errno_location, c_void, close, mmap, mremap, munmap, open, strerror, MREMAP_MAYMOVE, O_RDWR,
    PROT_READ, PROT_WRITE,
};
use std::fs::File;
use std::ops::{Index, IndexMut};
use std::path::Path;
use thiserror::Error;
use traits::Extent;
#[derive(Error, Debug)]
enum FileExtentError {
    #[error("mmap failed: {errno}")]
    MmapFailed { errno: i32 },
    #[error("open call failed")]
    OpenFailed,
    #[error("remap failed: {errno}")]
    RemapFailed { errno: i32 },
}
//Extent that writes back to mmaped file
pub struct FileExtent {
    file_map: *mut c_void,
    file_size: usize,
}
impl FileExtent {
    pub fn new(path: &Path) -> Result<Self> {
        let file_size = {
            if !path.exists() {
                File::create(path)?
            } else {
                File::open(path)?
            }
        }
        .metadata()?
        .len() as usize;

        let fd = unsafe { open(path.to_str().unwrap().as_ptr() as *const i8, O_RDWR) };
        if fd <= 0 {
            return Err(anyhow!("open call failed: {}", FileExtentError::OpenFailed));
        }
        let file_map: *mut c_void = unsafe {
            mmap(
                0 as *mut c_void,
                file_size,
                PROT_READ | PROT_WRITE,
                O_RDWR,
                fd,
                0,
            )
        };
        if file_map == (-1 as i64) as *mut c_void {
            return Err(anyhow!(
                "mmap failed: {}",
                FileExtentError::MmapFailed {
                    errno: unsafe { *__errno_location() }
                }
            ));
        }
        unsafe { close(fd) };
        Ok(Self {
            file_map,
            file_size,
        })
    }
}
impl Drop for FileExtent {
    fn drop(&mut self) {
        unsafe {
            munmap(self.file_map, self.file_size);
        }
    }
}
impl Extent for FileExtent {
    fn resize(&mut self, new_size: usize) -> Result<()> {
        let new_map = unsafe { mremap(self.file_map, self.file_size, new_size, MREMAP_MAYMOVE) };
        if new_map == (-1 as i64) as *mut c_void {
            let errno = unsafe {
                //horrifically unsafe if two threads errorout
                //at the same time errno would be accessed at the same time
                *__errno_location()
            };
            return Err(anyhow!(
                "resize failed: {}",
                FileExtentError::RemapFailed { errno }
            ));
        }
        self.file_size = new_size;
        self.file_map = new_map;
        Ok(())
    }
    fn len(&self) -> usize {
        self.file_size
    }
}
impl Index<usize> for FileExtent {
    type Output = u8;
    fn index(&self, idx: usize) -> &Self::Output {
        todo!()
    }
}
impl IndexMut<usize> for FileExtent {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        todo!()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;
    use std::fs::{read_dir, remove_dir, remove_file, DirBuilder, File};
    fn create_test(test_name: String) {
        if !Path::new("test_folder").exists() {
            DirBuilder::new().create("test_folder");
        }
    }
    fn remove_test(test_name: String) {
        remove_file("test_folder/".to_string() + &test_name);
        if let Some(iter) = read_dir("test_folder").ok() {
            if iter.count() == 0 {
                remove_dir("test_folder");
            }
        }
    }
    fn test(test_name: String, test: fn(&Path) -> Result<()>) {
        create_test(test_name.clone());

        let p = "test_folder/".to_string() + &test_name;
        if let Some(e) = test(&Path::new(&p)).err() {
            panic!("{}", e);
        }
        remove_test(test_name);
    }
    #[test]
    fn it_works() {
        test("basic".to_string(), |p| {
            FileExtent::new(p)?;
            Ok(())
        });
    }
    #[test]
    fn resize() {
        test("resize".to_string(), |p| {
            let mut f = FileExtent::new(p)?;
            f.resize(100)?;
            assert_eq!(f.len(), 100);
            Ok(())
        });
    }
    #[test]
    fn write() {
        test("write".to_string(), |p| {
            let mut f = FileExtent::new(p)?;
            f.resize(1000)?;
            assert_eq!(f.len(), 100);
            let v: Vec<u8> = (0..1000).map(|i: i32| i.to_le_bytes()[0]).collect();
            for i in 0..1000 {
                f[i] = v[i];
            }
            for i in 0..1000 {
                assert_eq!(f[i], v[i]);
            }
            Ok(())
        });
    }
}

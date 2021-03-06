#[macro_use]
extern crate anyhow;
use anyhow::Result;
use libc::{
    __errno_location, c_void, close, mmap, mremap, munmap, open, MAP_FAILED, MAP_SHARED,
    MREMAP_MAYMOVE, O_RDWR, PROT_READ, PROT_WRITE,
};
use std::cmp::max;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::ops::{Index, IndexMut};
use std::path::Path;
use thiserror::Error;
use traits::Extent;
#[derive(Error, Debug)]
enum FileExtentError {
    #[error("mmap failed: {errno}")]
    MmapFailed { errno: i32 },
    #[error("open call failed errno: {errno}")]
    OpenFailed { errno: i32 },
    #[error("unmap failed: {errno}")]
    UnMapFailed { errno: i32 },
    #[error("remap failed: {errno}")]
    RemapFailed { errno: i32 },
    #[error("close failed for fd: {fd}, errno: {errno}")]
    CloseFailed { errno: i32, fd: i32 },
}
//Extent that writes back to mmaped file
pub struct FileExtent {
    file_map: *mut c_void,
    file_size: usize,
    path_string: String,
}
impl FileExtent {
    /// if path exists loads existing file. If it does not then file is created.
    /// Preconditions: path_string points to a regular file, and is readable and writable.
    pub fn new(path_string: String) -> Result<Self> {
        let path = Path::new(&path_string);
        let file_size = {
            if !path.exists() {
                File::create(path.clone())?
            } else {
                File::open(path.clone())?
            }
        }
        .metadata()?
        .len() as usize;

        let fd = unsafe { open((&path).to_str().unwrap().as_ptr() as *const i8, O_RDWR) };
        if fd == -1 {
            let errno = unsafe { *__errno_location() };
            return Err(anyhow!(
                "open call failed: {}",
                FileExtentError::OpenFailed { errno }
            ));
        }

        let file_map: *mut c_void = unsafe {
            mmap(
                0 as *mut c_void,
                max(file_size, 1),
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                fd,
                0,
            )
        };
        if file_map == MAP_FAILED {
            return Err(anyhow!(
                "mmap failed: {}",
                FileExtentError::MmapFailed {
                    errno: unsafe { *__errno_location() }
                }
            ));
        }
        unsafe {
            if close(fd) == -1 {
                let errno = *__errno_location();
                return Err(anyhow!(
                    "close in ctor failed {}",
                    FileExtentError::CloseFailed { errno, fd },
                ));
            }
        };
        Ok(Self {
            file_map,
            file_size,
            path_string,
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
        if new_size > self.file_size {
            unsafe {
                if munmap(self.file_map, max(self.file_size, 1)) == -1 {
                    let errno = *__errno_location();
                    return Err(anyhow!(
                        "unmap failed: {}",
                        FileExtentError::UnMapFailed { errno }
                    ));
                }
                OpenOptions::new()
                    .append(true)
                    .open(self.path_string.clone())?
                    .write_all(&mut vec![0; new_size - self.file_size])?;
                let fd = open(self.path_string.as_str().as_ptr() as *const i8, O_RDWR);
                if fd == -1 {
                    let errno = *__errno_location();
                    return Err(anyhow!(
                        "open call failed: {}",
                        FileExtentError::OpenFailed { errno }
                    ));
                }

                let file_map: *mut c_void = mmap(
                    0 as *mut c_void,
                    max(self.file_size, 1),
                    PROT_READ | PROT_WRITE,
                    MAP_SHARED,
                    fd,
                    0,
                );
                if file_map == MAP_FAILED {
                    return Err(anyhow!(
                        "mmap failed: {}",
                        FileExtentError::MmapFailed {
                            errno: *__errno_location()
                        }
                    ));
                }
                close(fd);
                self.file_map = file_map;
            }
        }
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
        if idx < self.file_size {
            unsafe {
                (self.file_map.offset(idx as isize) as *const u8)
                    .as_ref()
                    .unwrap() as &u8
            }
        } else {
            panic!("index out of bounds")
        }
    }
}
impl IndexMut<usize> for FileExtent {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        if idx < self.file_size {
            unsafe {
                (self.file_map.offset(idx as isize) as *mut u8)
                    .as_mut()
                    .unwrap() as &mut u8
            }
        } else {
            panic!("index out of bounds")
        }
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
    fn test(test_name: String, test: fn(String) -> Result<()>) {
        create_test(test_name.clone());

        let p = "test_folder/".to_string() + &test_name;
        if let Some(e) = test(p).err() {
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
            assert_eq!(f.len(), 1000);
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
    #[test]
    fn reload_from_disk() {
        test("reload_from_disk".to_string(), |p| {
            let v: Vec<u8> = (0..1000).map(|i: i32| i.to_le_bytes()[0]).collect();
            {
                let mut f = FileExtent::new(p.clone())?;
                f.resize(1000)?;
                assert_eq!(f.len(), 1000);
                for i in 0..1000 {
                    f[i] = v[i];
                }
                for i in 0..1000 {
                    assert_eq!(f[i], v[i]);
                }
            }
            {
                let mut f = FileExtent::new(p)?;
                assert_eq!(f.len(), 1000);
                let v: Vec<u8> = (0..1000).map(|i: i32| i.to_le_bytes()[0]).collect();
                for i in 0..1000 {
                    assert_eq!(f[i], v[i]);
                }
            }

            Ok(())
        });
    }
}

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use crate::file::{File, FileStream, FileLock};


#[derive(Clone)]
pub(crate) struct StorageObject {
    files: Arc<RwLock<HashMap<String, FileLock>>>,
}

impl StorageObject {
    pub fn new() -> Self {
        StorageObject {
            files: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new file with `filename`. If such file already exists, it will be
    /// overwritten.
    ///
    /// Returns `FileLock`, a.k.a.: `Arc<RwLock<File>>`
    pub fn new_file(&self, filename: String) -> FileLock {
        let mut files = self.files.write().unwrap();
        files.insert(filename.clone(), File::new_lock());

        files.get(&filename).unwrap().clone()
    }

    /// Return `Some(Filestream)` if such file exists else return None
    pub fn get_filestream(&self, filename: &String) -> Option<FileStream> {
        let file = self.files.read().unwrap();
        match file.get(filename) {
            Some(fl) => Some(FileStream::new(fl.clone())),
            None => None,
        }
    }
}

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use axum::body::Bytes;

#[derive(Clone)]
pub(crate) struct File {
    chunks: Vec<Bytes>,
}

type FileLock = Arc<RwLock<File>>;

impl File {
    pub fn new() -> Self {
        File { chunks: Vec::new() }
    }

    pub fn new_lock() -> FileLock {
        Arc::new(RwLock::new(File::new()))
    }

    pub fn push(&mut self, chunk: Bytes) -> () {
        self.chunks.push(chunk);
    }
}

#[derive(Clone)]
pub(crate) struct StorageObject {
    path: String,
    files: Arc<RwLock<HashMap<String, FileLock>>>,
}

impl StorageObject {
    pub fn new(path: String) -> Self {
        StorageObject {
            path,
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

    pub fn get_file(&self, filename: &String) -> Option<FileLock> {
        let files = self.files.read().unwrap();
        match files.get(filename) {
            Some(fl) => Some(fl.clone()),
            None => None,
        }
    }
}

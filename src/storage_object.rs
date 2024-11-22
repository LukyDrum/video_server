use std::sync::{Arc, RwLock};

use axum::body::Bytes;


#[derive(Clone)]
pub(crate) struct Segment {
    chunks: Vec<Bytes>,
}

type SegmentLock = Arc<RwLock<Segment>>;

impl Segment {
    pub fn new() -> Self {
        Segment { chunks: Vec::new() }
    }

    pub fn new_lock() -> SegmentLock {
        Arc::new(RwLock::new(Segment::new()))
    }

    pub fn push(&mut self, chunk: Bytes) -> () {
        self.chunks.push(chunk);
        println!("Pushed chunk!");
    }
}

#[derive(Clone)]
pub(crate) struct StorageObject {
    path: String,
    metadata: Arc<RwLock<Option<Bytes>>>,
    segments: Arc<RwLock<Vec<SegmentLock>>>,
}

impl StorageObject {
    pub fn new(path: String) -> Self {
        StorageObject {
            path,
            metadata: Arc::new(RwLock::new(None)),
            segments: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn update_meta(&self, bytes: Bytes) -> () {
        let mut meta = self.metadata.write().unwrap();
        meta.replace(bytes);
    }

    pub fn new_segment(&self) -> SegmentLock {
        let mut segments = self.segments.write().unwrap();
        segments.push(Segment::new_lock());

        segments.last().unwrap().clone()
    }

    pub fn get_segment(&self, index: usize) -> Option<SegmentLock> {
        let segments = self.segments.read().unwrap();
       
        match segments.get(index) {
            Some(sl) => Some(sl.clone()),
            None => None
        }
    }
}

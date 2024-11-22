use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

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
    }
}

#[derive(Clone)]
pub(crate) struct StorageObject {
    path: String,
    segments: Arc<RwLock<HashMap<String, SegmentLock>>>,
}

impl StorageObject {
    pub fn new(path: String) -> Self {
        StorageObject {
            path,
            segments: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new segment with `filename`. If such segment already exists, it will be
    /// overwritten.
    ///
    /// Returns `SegmentLock`, a.k.a.: `Arc<RwLock<Segment>>`
    pub fn new_segment(&self, filename: String) -> SegmentLock {
        let mut segments = self.segments.write().unwrap();
        segments.insert(filename.clone(), Segment::new_lock());

        segments.get(&filename).unwrap().clone()
    }

    pub fn get_segment(&self, filename: &String) -> Option<SegmentLock> {
        let segments = self.segments.read().unwrap();
        match segments.get(filename) {
            Some(sl) => Some(sl.clone()),
            None => None,
        }
    }
}

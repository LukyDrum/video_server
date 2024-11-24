use std::{
    cell::RefCell,
    sync::{Arc, RwLock},
    task::Poll,
    time::Instant,
};

use axum::body::{self, Bytes};
use futures::Stream;

/// Represents a stored file consisting of chunks (bytes)
#[derive(Clone)]
pub(crate) struct File {
    chunks: Vec<Bytes>,
    is_complete: bool,
    last_update: Instant,
}

pub type FileLock = Arc<RwLock<File>>;

impl File {
    pub fn new() -> Self {
        File {
            chunks: Vec::new(),
            is_complete: false,
            last_update: Instant::now(),
        }
    }

    pub fn new_lock() -> FileLock {
        Arc::new(RwLock::new(File::new()))
    }

    pub fn push(&mut self, chunk: Bytes) {
        self.chunks.push(chunk);
        self.last_update = Instant::now();
    }

    pub fn get_chunks(&self) -> &Vec<Bytes> {
        &self.chunks
    }
    
    pub fn set_as_complete(&mut self) {
        self.is_complete = true;
    }
}

/// How long before considering the file as complete
const FILE_TIMEOUT: u128 = 1000; // 1000ms = 1s

/// Used for sending responses including this file
pub struct FileStream {
    original_file: FileLock,
    index: RefCell<usize>,
}

impl FileStream {
    pub fn new(filelock: FileLock) -> Self {
        FileStream {
            original_file: filelock,
            index: RefCell::new(0)
        }
    }
}

impl Stream for FileStream {
    type Item = Result<body::Bytes, std::io::Error>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        // Get read lock
        let file = self.original_file.read().unwrap();

        let mut index = self.index.borrow_mut();
        let chunks_size = file.get_chunks().len();
        // Check if we can poll new value
        if *index < chunks_size {
            // Wrap chunk in Poll::Ready and increase index for next chunk
            let res = Poll::Ready(Some(Ok(file.get_chunks()[*index].clone())));
            *index += 1;
            res
        } else {
            // Check how long since the file was updated
            let since = Instant::now().duration_since(file.last_update);
            // Stop stream if the file is complete or the file timeout is met
            if file.is_complete || since.as_millis() > FILE_TIMEOUT {
                return Poll::Ready(None);
            }
            // Schedule waking
            _cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}


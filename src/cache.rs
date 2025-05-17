use crate::utils::{extract_header, extract_request_uri};
use std::collections::VecDeque;

const MAX_CACHE_SIZE: usize = 10;
const MAX_RESPONSE_SIZE: usize = 102400; // 100 KiB
const MAX_REQUEST_SIZE: usize = 2000; // 2000 bytes
const HOST_HEADER: &str = "Host";

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub request: Vec<u8>,
    pub response: Vec<u8>,
}

pub struct Cache {
    entries: VecDeque<CacheEntry>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            entries: VecDeque::new(),
        }
    }

    pub fn get(&mut self, req: &[u8]) -> Option<CacheEntry> {
        if req.len() > MAX_REQUEST_SIZE {
            eprintln!("Request too large to cache");
            return None;
        }

        // Find the entry in the cache
        let pos = self.entries.iter().position(|entry| entry.request == req)?;

        // Extract and remove the entry
        let entry = self.entries.remove(pos).expect("Failed to remove entry");

        // Move the entry to the front (most recently used)
        self.entries.push_front(entry.clone());

        Some(entry)
    }

    pub fn put(&mut self, request: Vec<u8>, response: Vec<u8>) {
        if response.len() > MAX_RESPONSE_SIZE {
            eprintln!("Response too large to cache");
            return;
        }
        if request.len() > MAX_REQUEST_SIZE {
            eprintln!("Request too large to cache");
            return;
        }

        // Check if the cache is full
        if self.entries.len() >= MAX_CACHE_SIZE {
            if let Some(evicted) = self.entries.pop_back() {
                if let Ok(request_str) = String::from_utf8(evicted.request.clone()) {
                    let host = extract_header(&request_str, HOST_HEADER).unwrap_or_default();
                    let uri = extract_request_uri(&request_str).unwrap_or_default();
                    println!("Evicting {host} {uri} from cache");
                }
            }
        }

        let entry = CacheEntry {
            request,
            response
        };

        self.entries.push_front(entry);
        eprintln!("Cache size: {}", self.entries.len());
    }
}

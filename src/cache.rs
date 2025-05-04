use crate::utils::{extract_header, extract_max_age, extract_request_uri};
use std::collections::VecDeque;
use std::time::SystemTime;

const MAX_CACHE_SIZE: usize = 10;
const MAX_RESPONSE_SIZE: usize = 102400; // 100 KiB
const MAX_REQUEST_SIZE: usize = 2000; // 2000 bytes

const NO_CACHE_DIRECTIVES: [&str; 6] = [
    "private",
    "no-store",
    "no-cache",
    "max-age=0",
    "must-revalidate",
    "proxy-revalidate",
];

const CACHE_CONTROL_HEADER: &str = "Cache-Control";
const HOST_HEADER: &str = "Host";

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub request: Vec<u8>,
    pub response: Vec<u8>,
    pub added_time: SystemTime,
    pub max_age: Option<u32>,
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

        let request_str = String::from_utf8_lossy(req);
        // Find the entry in the cache
        let pos = self.entries.iter().position(|entry| entry.request == req)?;

        // Extract and remove the entry
        let entry = self.entries.remove(pos).expect("Failed to remove entry");

        // Move the entry to the front (most recently used)
        self.entries.push_front(entry.clone());

        if self.is_entry_stale(&entry) {
            println!(
                "Stale entry for host: {} {}",
                extract_header(&request_str, HOST_HEADER).expect("Failed to extract Host"),
                extract_request_uri(&request_str).expect("Failed to extract request URI")
            );
            return None;
        }

        Some(entry)
    }

    fn is_entry_stale(&self, entry: &CacheEntry) -> bool {
        if let Some(max_age) = entry.max_age {
            if let Ok(elapsed) = SystemTime::now().duration_since(entry.added_time) {
                return elapsed.as_secs() > max_age as u64;
            }
        }
        false
    }

    fn should_cache_response(&self, response: &[u8]) -> bool {
        let response_str = String::from_utf8_lossy(&response);

        if let Some(header) = extract_header(&response_str, CACHE_CONTROL_HEADER) {
            if NO_CACHE_DIRECTIVES
                .iter()
                .any(|&directive| header.contains(directive))
            {
                return false;
            }
        }
        true
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

        let request_str = String::from_utf8_lossy(&request);
        let host = extract_header(&request_str, HOST_HEADER).unwrap_or_default();
        let uri = extract_request_uri(&request_str).unwrap_or_default();
        let max_age = extract_max_age(&request_str);

        if !self.should_cache_response(&response) {
            eprintln!("Response marked as non-cacheable for host: {host} and URI: {uri}");
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
            response,
            added_time: SystemTime::now(),
            max_age,
        };

        self.entries.push_front(entry);
        eprintln!("Cache size: {}", self.entries.len());
    }
}

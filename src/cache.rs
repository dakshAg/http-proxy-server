use crate::utils::{extract_header, extract_max_age, extract_request_uri};
use std::time::SystemTime;

const MAX_CACHE_SIZE: usize = 10;
const MAX_RESPONSE_SIZE: usize = 102400; // 100 KiB
const MAX_REQUEST_SIZE: usize = 2000; // 2000 bytes
const HOST_HEADER: &str = "Host";

const NO_CACHE_DIRECTIVES: [&str; 6] = [
    "private",
    "no-store",
    "no-cache",
    "max-age=0",
    "must-revalidate",
    "proxy-revalidate",
];

const CACHE_CONTROL_HEADER: &str = "Cache-Control";

/// Represents a single cache entry, storing the request, response, timestamps, and cache metadata.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The HTTP request as a byte vector.
    pub request: Vec<u8>,
    /// The HTTP response as a byte vector.
    pub response: Vec<u8>,
    /// The time when the entry was added to the cache.
    pub added_time: SystemTime,
    /// The last time this entry was accessed.
    pub last_used: SystemTime,
    /// The max-age value for cache expiration, if present.
    pub max_age: Option<u32>,
}

/// Represents the cache, which holds a fixed-size array of cache entries.
pub struct Cache {
    /// The array of cache entries (may be None if slot is empty).
    pub entries: [Option<CacheEntry>; MAX_CACHE_SIZE],
}

impl Cache {
    /// Creates a new, empty cache.
    pub fn new() -> Self {
        Cache {
            entries: Default::default(),
        }
    }

    /// Retrieves a cache entry matching the given request, if present and valid.
    ///
    /// # Arguments
    ///
    /// * `req` - The HTTP request as a byte vector.
    ///
    /// # Returns
    ///
    /// An `Option<CacheEntry>` containing the entry if found and valid, or `None` otherwise.
    pub fn get(&mut self, req: &Vec<u8>) -> Option<CacheEntry> {
        if req.len() > MAX_REQUEST_SIZE {
            eprintln!("Request too large to cache");
            return None;
        }
        for entry_opt in self.entries.iter_mut() {
            if let Some(entry) = entry_opt {
                if entry.request == *req {
                    entry.last_used = SystemTime::now();
                    return Some(entry.clone());
                }
            }
        }
        None
    }

    /// Adds a new cache entry for the given request and response, if valid.
    ///
    /// # Arguments
    ///
    /// * `request` - The HTTP request as a byte vector.
    /// * `response` - The HTTP response as a byte vector.
    ///
    /// # Returns
    ///
    /// `true` if the entry was successfully added, `false` otherwise.
    pub fn put(&mut self, request: Vec<u8>, response: Vec<u8>) -> bool {
        if response.len() > MAX_RESPONSE_SIZE {
            eprintln!("Response too large to cache");
            return false;
        }
        if request.len() > MAX_REQUEST_SIZE {
            eprintln!("Request too large to cache");
            return false;
        }

        let request_str = String::from_utf8_lossy(&request);
        let host = extract_header(&request_str, HOST_HEADER).unwrap_or_default();
        let uri = extract_request_uri(&request_str).unwrap_or_default();
        let max_age = extract_max_age(String::from_utf8_lossy(&response).trim());

        if !self.should_cache_response(&response) {
            println!("Not caching {host} {uri}");
            return false;
        }

        let now = SystemTime::now();
        let entry = CacheEntry {
            request,
            response,
            added_time: now,
            last_used: now,
            max_age,
        };

        // Find an empty slot in the cache and add the entry
        if let Some(slot) = self.entries.iter_mut().find(|e| e.is_none()) {
            *slot = Some(entry);
            return true;
        }
        true
    }

    /// Evicts the least recently used cache entry if the cache is full.
    pub fn evict_if_full(&mut self) {
        // Check if the cache is full
        if self.entries.iter().all(|entry| entry.is_some()) {
            // Find the least recently used entry
            let mut lru_index = 0;
            let mut lru_entry = &self.entries[0];
            let mut oldest_time = SystemTime::now();
            for (i, entry_opt) in self.entries.iter().enumerate() {
                if let Some(entry) = entry_opt {
                    if entry.last_used < oldest_time {
                        oldest_time = entry.last_used;
                        lru_index = i;
                        lru_entry = entry_opt;
                    }
                }
            }
            let entry = lru_entry.as_ref().unwrap();
            let request_str = String::from_utf8_lossy(&entry.request);
            let host = extract_header(&request_str, HOST_HEADER).unwrap_or_default();
            let uri = extract_request_uri(&request_str).unwrap_or_default();
            println!("Evicting {host} {uri} from cache");
            // Evict the least recently used entry
            self.entries[lru_index] = None;
        }
    }

    /// Evicts a specific cache entry matching the given request.
    ///
    /// # Arguments
    ///
    /// * `request` - The HTTP request as a byte vector.
    pub fn evict(&mut self, request: &Vec<u8>) {
        for entry_opt in self.entries.iter_mut() {
            if let Some(entry) = entry_opt {
                if entry.request.eq(request) {
                    *entry_opt = None;
                    break;
                }
            }
        }
    }

    /// Determines whether the given response should be cached based on its headers.
    ///
    /// # Arguments
    ///
    /// * `response` - The HTTP response as a byte vector.
    ///
    /// # Returns
    ///
    /// `true` if the response should be cached, `false` otherwise.
    fn should_cache_response(&self, response: &Vec<u8>) -> bool {
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

    /// Checks whether a cache entry is stale based on its max-age value.
    ///
    /// # Arguments
    ///
    /// * `entry` - The cache entry to check.
    ///
    /// # Returns
    ///
    /// `true` if the entry is stale, `false` otherwise.
    pub fn is_entry_stale(entry: &CacheEntry) -> bool {
        if let Some(max_age) = entry.max_age {
            if let Ok(elapsed) = SystemTime::now().duration_since(entry.added_time) {
                return elapsed.as_secs() > max_age as u64;
            }
        }
        false
    }

    /// Updates a cache entry with a new response, evicting the old entry if necessary.
    ///
    /// # Arguments
    ///
    /// * `request` - The HTTP request as a byte vector.
    /// * `response` - The new HTTP response as a byte vector.
    pub fn update_entry(&mut self, request: &Vec<u8>, response: Vec<u8>) {
        self.evict(request);

        // Try to add the new response to the cache
        // If the entry was not cached, show eviction message
        if !self.put(request.clone(), response) {
            let request_str = String::from_utf8_lossy(request);
            let host = extract_header(&request_str, HOST_HEADER).unwrap_or_default();
            let uri = extract_request_uri(&request_str).unwrap_or_default();
            println!("Evicting {host} {uri} from cache");
        }
    }
}

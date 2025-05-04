use crate::utils::extract_header;
use std::collections::VecDeque;
use std::time::SystemTime;

const MAX_CACHE_SIZE: usize = 10;
const MAX_RESPONSE_SIZE: usize = 102400; // 100 KiB

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
        let pos = self.entries.iter().position(|entry| entry.request == req)?;
        let entry = self.entries.remove(pos).unwrap(); // remove from current position
        self.entries.push_front(entry.clone()); // move to front (most recently used)
        
        if let Some(max_age) = entry.max_age {
            if let Ok(elapsed) = SystemTime::now().duration_since(entry.added_time) {
                if elapsed.as_secs() > max_age as u64 {
                   println!(
                        "Stale entry for host: {} {}",
                        extract_header(&String::from_utf8_lossy(&entry.request), "Host").unwrap_or_default(),
                        String::from_utf8_lossy(&entry.request).lines().next().unwrap_or_default()
                    );
                    return None;
                }
            }
        }
        
        Some(entry)
    }

    pub fn put(&mut self, request: Vec<u8>, response: Vec<u8>) {
        if response.len() > MAX_RESPONSE_SIZE {
            eprintln!("Response too large to cache");
            return;
        }
        let request_str = String::from_utf8_lossy(&request);
        let response_str = String::from_utf8_lossy(&response);
        
        if let Some(header) = extract_header(&String::from_utf8_lossy(&response), "Cache-Control") {
            let directives = [
                "private",
                "no-store",
                "no-cache",
                "max-age=0",
                "must-revalidate",
                "proxy-revalidate",
            ];
            if directives
                .iter()
                .any(|&directive| header.contains(directive))
            {
                eprintln!("Response marked as non-cacheable for host: {} and URI: {}", extract_header(&String::from_utf8_lossy(&request), "Host").unwrap_or_default(), request_str.lines().next().unwrap_or_default());
                return;
            }
        }
        
        let max_age = if let Some(header) = extract_header(&String::from_utf8_lossy(&response), "Cache-Control") {
                    if let Some(pos) = header.find("max-age=") {
                        header[pos + 8..]
                            .split(',')
                            .next()
                            .and_then(|v| v.trim().parse::<u32>().ok())
                    } else {
                        None
                    }
                } else {
                    None
                };

        if self.entries.len() >= MAX_CACHE_SIZE {
            if let Some(evicted) = self.entries.pop_back() {
                if let Ok(request_str) = String::from_utf8(evicted.request.clone()) {
                    let host = extract_header(&request_str, "Host").unwrap_or_default();
                    if let Some((method, uri)) = request_str.lines().next().unwrap_or_default().split_once(' ') {
                        println!("Evicting {} {} from cache", host, uri);
                    }
                }
            }
            self.entries.pop_back(); // evict least recently used
        }

        let entry = CacheEntry {
            request,
            response,
            added_time: SystemTime::now(),
            max_age,
        };

        self.entries.push_front(entry);
        eprint!("Cache size: {}", self.entries.len());
    }
}

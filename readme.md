# compsys-p2: HTTP Proxy Server in Rust

## Overview

This project implements a simple HTTP proxy server in Rust with optional caching functionality. The proxy listens for incoming HTTP requests, forwards them to the origin server, and can cache responses to improve performance for repeated requests.

## Features
- Forwards HTTP requests from clients to origin servers
- Caches HTTP responses (when enabled) with support for cache expiration based on HTTP headers
- Simple in-memory cache with eviction policy
- Command-line interface for enabling/disabling cache

## Project Structure
- `src/main.rs`: Main entry point, server loop, and client handler logic
- `src/cache.rs`: In-memory cache implementation and cache entry management
- `src/utils.rs`: Utility functions for parsing HTTP requests and responses

## Usage

### Build

```
make
```

### Run

```
./htproxy <proxy_name> <port> [-c]
```
- `<proxy_name>`: Name for the proxy instance (not used in logic, but required)
- `<port>`: Port number to listen on
- `-c`: (Optional) Enable caching

#### Example
```
./htproxy myproxy 8080 -c
```
This starts the proxy on port 8080 with caching enabled.

## How Caching Works
- The cache stores up to 10 responses, each up to 100 KiB in size
- Entries expire based on HTTP `Cache-Control` headers (e.g., `max-age`)
- Requests with certain cache-control directives (e.g., `no-cache`, `private`) are not cached
- Least recently used (LRU) entries are evicted when the cache is full

## Limitations
- Only supports HTTP (not HTTPS)
- No authentication or access control
- Cache is in-memory and not persistent

## License
MIT

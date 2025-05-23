//! Main entry point for the HTTP proxy server.
//!
//! This file contains the main function and the client handler logic for the proxy server.

mod cache;
mod utils;

use crate::cache::Cache;
use crate::utils::{extract_header, extract_request_uri, print_request_tail};
use std::env;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

/// Reads the full HTTP request from the client stream until the end of headers ("\r\n\r\n").
///
/// # Arguments
///
/// * `stream` - The TCP stream representing the client connection.
///
/// # Returns
///
/// A byte vector containing the full HTTP request.
fn read_http_request(stream: &mut TcpStream) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut temp = [0; 1024];
    loop {
        let n = stream.read(&mut temp).expect("Could not read from stream");
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..n]);
        if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }
    buffer
}

/// Handles an individual client connection, processing the HTTP request and interacting with the cache.
///
/// # Arguments
///
/// * `stream` - The TCP stream representing the client connection.
/// * `cache` - The mutable reference to the cache instance.
/// * `is_cache` - Boolean flag indicating whether caching is enabled.
fn handle_client(mut stream: TcpStream, cache: &mut Cache, is_cache: bool) {
    println!("Accepted");

    let request = read_http_request(&mut stream);
    let request_str = String::from_utf8_lossy(&request);

    let origin_server = extract_header(&request_str, "host").expect("Could not extract header");

    let uri = extract_request_uri(&request_str).expect("Could not extract URI");

    let mut stale_entry = false;

    print_request_tail(&request_str);
    if is_cache {
        // Check if the request is already in the cache
        if let Some(entry) = cache.get(&request) {
            // Check if the entry is stale
            if Cache::is_entry_stale(&entry) {
                stale_entry = true;
                println!("Stale entry for {origin_server} {uri}");
            } else {
                // If the entry is not stale, serve it from the cache
                println!("Serving {origin_server} {uri} from cache");
                stream
                    .write_all(&entry.response)
                    .expect("Could not write cached response to stream");
                stream.shutdown(Shutdown::Both).ok();
                return;
            }
        } else {
            // If the entry was not found in cache, ensure the cache is not full
            // for the next entry
            cache.evict_if_full();
        }
    }

    println!("GETting {} {}", origin_server, uri);

    // Open a connection to the origin server and send the request
    let mut server_stream =
        TcpStream::connect(format!("{origin_server}:80")).expect("Could not connect to server");
    server_stream.write_all(&request).unwrap();

    // Read the response headers from the server
    let mut response_headers = Vec::new();
    let mut header_end = None;
    while header_end.is_none() {
        let mut byte = [0; 1];
        if server_stream.read(&mut byte).unwrap_or(0) == 0 {
            break;
        }
        response_headers.push(byte[0]);
        if response_headers.len() >= 4
            && &response_headers[response_headers.len() - 4..] == b"\r\n\r\n"
        {
            header_end = Some(response_headers.len());
        }
    }

    // Write the response headers to the client stream
    stream.write_all(&response_headers).unwrap();
    let response_str = String::from_utf8_lossy(&response_headers);
    let content_length = extract_header(&response_str, "content-length")
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0);

    // Read the response body from the server and write it to the client stream
    let mut total_read = 0;
    let mut temp_buffer = [0; 1024];
    let mut server_buffer = Vec::new();

    while total_read < content_length {
        let to_read = std::cmp::min(1024, content_length - total_read);
        let bytes_read = server_stream.read(&mut temp_buffer[..to_read]).unwrap_or(0);
        if bytes_read == 0 {
            break;
        }
        server_buffer.extend_from_slice(&temp_buffer[..bytes_read]);
        stream.write_all(&temp_buffer[..bytes_read]).unwrap();
        total_read += bytes_read;
    }
    println!("Response body length {}", content_length);


    if is_cache {
        // Store the full response (headers + body) in the cache
        let mut full_response = response_headers.clone();
        full_response.extend_from_slice(&server_buffer);

        // If a stale entry was found, update it, otherwise add a new entry
        if stale_entry {
            cache.update_entry(&request, full_response);
        } else {
            cache.put(request, full_response);
        }
    }

    // Close the connection to the client and the server
    stream.shutdown(Shutdown::Both).ok();
    server_stream.shutdown(Shutdown::Both).ok();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let port = &args[2];

    let is_cache = args.contains(&String::from("-c"));

    let mut cache = Cache::new();

    // Start the server and listen for incoming connections
    let listener =
        TcpListener::bind(format!("[::]:{port}")).expect("Could not listen for connections");

    // Accept incoming connections and handle them
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Process each connection
                handle_client(stream, &mut cache, is_cache);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}

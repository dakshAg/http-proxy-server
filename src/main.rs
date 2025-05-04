mod cache;
mod utils;

use crate::cache::Cache;
use crate::utils::{extract_header, extract_request_uri};
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const HOST_HEADER: &str = "Host";
const CONTENT_LENGTH_HEADER: &str = "Content-Length";

fn handle_client(mut stream: TcpStream, cache: &mut Cache, is_cache: bool) {
    let mut buffer = [0; 1024];

    stream
        .read(&mut buffer)
        .expect("Could not read from stream");

    let request = buffer.to_vec();

    let request_str = String::from_utf8_lossy(&request);
    let origin_server = extract_header(&request_str, HOST_HEADER).expect("Could not extract header");
    let uri = extract_request_uri(&request_str).expect("Could not extract URI");

    let last_line = request_str.lines().last().expect("No last line found");
    println!("Request tail {}", last_line);

    println!("GETting {} {}", origin_server, uri);

    // Check if the response is in the cache
    if is_cache {
        if let Some(entry) = cache.get(&request) {
            println!("Serving {origin_server} {uri} from cache");
            stream
                .write_all(&entry.response)
                .expect("Could not write cached response to stream");
            return;
        }
    }

    // If not in cache, connect to the origin server and write the request
    let mut server_stream = TcpStream::connect(format!("{origin_server}:80")).unwrap();
    server_stream.write_all(&request).unwrap();

    let mut server_buffer = Vec::new();
    let mut temp_buffer = [0; 1024];

    // Read the response from the server and write it to the client stream
    while let Ok(bytes_read) = server_stream.read(&mut temp_buffer) {
        if bytes_read == 0 {
            break; // Connection closed
        }
        server_buffer.extend_from_slice(&temp_buffer[..bytes_read]);
        stream
            .write_all(&temp_buffer[..bytes_read])
            .expect("Could not write to stream");
    }

    let content_length =
        extract_header(&request_str, CONTENT_LENGTH_HEADER).expect("No Content-Length found");
    println!("Response body length {content_length}",);
    
    // Cache the response if the cache is enabled
    if is_cache {
        cache.put(request, server_buffer.clone());
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let port = &args[2];

    let is_cache = args.contains(&String::from("-c"));

    let mut cache = Cache::new();

    // Start the server and listen for incoming connections
    let listener =
        TcpListener::bind(format!("127.0.0.1:{port}")).expect("Could not listen for connections");
    println!("Listening on {}", listener.local_addr().unwrap());
    
    // Accept incoming connections and handle them
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Accepted");
                handle_client(stream, &mut cache, is_cache);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}

mod cache;
mod utils;

use crate::cache::Cache;
use crate::utils::extract_header;
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream, cache: &mut Cache, is_cache: bool) {
    let mut buffer = [0; 1024];

    stream
        .read(&mut buffer)
        .expect("Could not read from stream");

    let request = buffer.to_vec();

    let request_str = String::from_utf8_lossy(&request);
    let origin_server = extract_header(&request_str, "Host").unwrap_or_default();

    if let Some(last_line) = request_str.lines().last() {
        println!("Request tail {}", last_line);
    }
    
    let mut urix = String::new();
    let request_line = request_str.lines().next().unwrap_or_default();
    if let Some((method, uri)) = request_line.split_once(' ') {
        if method == "GET" {
            println!("GETting {} {}", origin_server, uri);
            urix = uri.to_string(); // Save `uri` for later use
        }
    }

    // Check if the response is in the cache
    if is_cache {
        if let Some(entry) = cache.get(&request) {
            println!(
                "Serving {} {} from cache",
                origin_server, urix
            );
            stream
                .write_all(&entry.response)
                .expect("Could not write cached response to stream");
            return;
        }
    }

    let mut server_stream = TcpStream::connect(format!("{origin_server}:80")).unwrap();
    server_stream.write_all(&request).unwrap();

    let mut server_buffer = Vec::new();
    let mut temp_buffer = [0; 1024];

    while let Ok(bytes_read) = server_stream.read(&mut temp_buffer) {
        if bytes_read == 0 {
            break; // Connection closed
        }
        server_buffer.extend_from_slice(&temp_buffer[..bytes_read]);
        stream
            .write_all(&temp_buffer[..bytes_read])
            .expect("Could not write to stream");
    }

    let content_length = extract_header(&request_str, "Content-Length");
    println!(
        "Response body length {}",
        content_length.unwrap_or_default()
    );
    if is_cache {
        cache.put(request, server_buffer.clone());
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let port = &args[2];

    let is_cache = args.contains(&String::from("-c"));

    let mut cache = Cache::new();

    let listener =
        TcpListener::bind(format!("127.0.0.1:{port}")).expect("Could not listen for connections");
    println!("Listening on {}", listener.local_addr().unwrap());
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

mod cache;
mod utils;

use crate::cache::Cache;
use crate::utils::{extract_header, extract_request_uri};
use std::env;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream, cache: &mut Cache, is_cache: bool) {
    println!("Accepted");

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

    let request = buffer.clone();
    let request_str = String::from_utf8_lossy(&request);

    let origin_server = extract_header(&request_str, "host").expect("Could not extract header");

    let uri = extract_request_uri(&request_str).expect("Could not extract URI");

    // Only consider the header part (before \r\n\r\n)
    let header_end_pos = request_str.find("\r\n\r\n").unwrap_or(request_str.len());
    let header_section = &request_str[..header_end_pos];
    let last_header_line = header_section.lines().last().unwrap_or("");
    println!("Request tail {}", last_header_line);

    if is_cache {
        if let Some(entry) = cache.get(&request) {
            println!("Serving {origin_server} {uri} from cache");
            stream
                .write_all(&entry.response)
                .expect("Could not write cached response to stream");
            stream.shutdown(Shutdown::Both).ok();
            return;
        }else{
            cache.evict_if_full();
        }
    }

    println!("GETting {} {}", origin_server, uri);

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
        cache.put(request, full_response);
    }
    stream.shutdown(Shutdown::Both).ok();
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
                // Process each connection sequentially (no threads)
                handle_client(stream, &mut cache, is_cache);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}

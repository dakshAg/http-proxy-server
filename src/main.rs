//use crate::cache::Cache;
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let args: Vec<String> = env::args().collect();
    let port = &args[2];

    // Start the server and listen for incoming connections
    let listener =
        TcpListener::bind(format!("127.0.0.1:{port}")).expect("Could not listen for connections");
    println!("Listening on {}", listener.local_addr().unwrap());

    // Accept incoming connections and handle them
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Accepted");

                let mut buffer = [0; 1024];

                stream
                    .read(&mut buffer)
                    .expect("Could not read from stream");

                let request = buffer.to_vec();

                let request_str = String::from_utf8_lossy(&request);

                let origin_server =
                    extract_header(&request_str, "host").expect("Could not extract header");

                let uri = extract_request_uri(&request_str).expect("Could not extract URI");

                let last_line = request_str.lines().last().expect("No last line found");
                println!("Request tail {}", last_line);

                println!("GETting {} {}", origin_server, uri);

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

                let response_str = String::from_utf8_lossy(&server_buffer);

                let content_length = extract_header(&response_str, "content-length")
                    .expect("No Content-Length found");

                println!("Response body length {content_length}");
            }

            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}

pub fn extract_header(request: &str, header: &str) -> Option<String> {
    let header = format!("{}: ", header.to_lowercase());
    let request = request.to_lowercase();
    let start = request.find(&header)?;
    let end = request[start..].find("\r\n")?;
    Some(request[start + header.len()..start + end].to_string())
}

pub fn extract_request_uri(request: &str) -> Option<String> {
    let request_line = request.lines().next()?;
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    Some(parts[1].to_string())
}

/// Extracts the value of a specific header from an HTTP request string.
///
/// # Arguments
///
/// * `request` - The raw HTTP request as a string.
/// * `header` - The name of the header to extract.
///
/// # Returns
///
/// An `Option<String>` containing the header value if found, or `None` otherwise.
pub fn extract_header(request: &str, header: &str) -> Option<String> {
    let header = format!("{}: ", header.to_lowercase());
    let request = request.to_lowercase();
    let start = request.find(&header)?;
    let end = request[start..].find("\r\n")?;
    Some(request[start + header.len()..start + end].to_string())
}

/// Extracts the URI from the request line of an HTTP request string.
///
/// # Arguments
///
/// * `request` - The raw HTTP request as a string.
///
/// # Returns
///
/// An `Option<String>` containing the request URI if found, or `None` otherwise.
pub fn extract_request_uri(request: &str) -> Option<String> {
    let request_line = request.lines().next()?;
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    Some(parts[1].to_string())
}

/// Extracts the `max-age` value from the `Cache-Control` header in an HTTP response string.
///
/// # Arguments
///
/// * `response` - The raw HTTP response as a string.
///
/// # Returns
///
/// An `Option<u32>` containing the max-age value if found, or `None` otherwise.
pub fn extract_max_age(response: &str) -> Option<u32> {
    let header = extract_header(response, "Cache-Control")?;
    eprintln!("Cache-Control header found: {}", header);
    if let Some(pos) = header.find("max-age=") {
        eprintln!("Max-age header found: {}", &header[pos..]);
        header[pos + 8..]
            .split(',')
            .next()
            .and_then(|v| v.trim().parse::<u32>().ok())
    } else {
        None
    }
}

/// Prints the tail (body) of an HTTP request string for debugging purposes.
///
/// # Arguments
///
/// * `request` - The raw HTTP request as a string.
pub fn print_request_tail(request: &str) {
    let header_end_pos = request.find("\r\n\r\n").unwrap_or(request.len());
    let header_section = &request[..header_end_pos];
    let last_header_line = header_section.lines().last().unwrap_or("");
    println!("Request tail {}", last_header_line);
}


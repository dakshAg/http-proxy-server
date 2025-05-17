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
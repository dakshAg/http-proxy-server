pub fn extract_header(request: &str, header: &str) -> Option<String> {
    let header = format!("{header}: ");
    let start = request.find(&header)?;
    let end = request[start..].find("\r\n")?;
    Some(request[start + header.len()..start + end].to_string())
}

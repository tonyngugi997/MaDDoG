use std::{
    fs,
    net::{TcpListener, TcpStream},
    io::{BufReader, prelude::*},
};
use std::collections::HashMap;

struct HttpRequest {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: String,
}

fn get_mime_type(path: &str) -> &'static str {
    let extension = path
        .rsplit_once('.')
        .map(|(_, ext)| ext)
        .unwrap_or("");
    
    match extension {
        "html" | "htm" => "text/html",
        "css"          => "text/css",
        "js"           => "application/javascript",
        "json"         => "application/json",
        "png"          => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif"          => "image/gif",
        "svg"          => "image/svg+xml",
        "txt"          => "text/plain",
        _              => "application/octet-stream",
    }
}

#[derive(Debug)]
enum HttpStatus {
    Ok,
    NotFound,
    InternalServerError,
}

impl HttpStatus {
    fn code(&self) -> u16 {
        match self {
            HttpStatus::Ok => 200,
            HttpStatus::NotFound => 404,
            HttpStatus::InternalServerError => 500,
        }
    }
    
    fn reason_phrase(&self) -> &'static str {
        match self {
            HttpStatus::Ok => "OK",
            HttpStatus::NotFound => "NOT FOUND",
            HttpStatus::InternalServerError => "INTERNAL SERVER ERROR",
        }
    }
}

struct HttpResponse {
    status: HttpStatus,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpResponse {
    fn text(status: HttpStatus, body: String) -> Self {
        let mut headers = HashMap::new();
        let body_bytes = body.into_bytes();
        
        headers.insert("Content-Length".to_string(), body_bytes.len().to_string());
        headers.insert("Content-Type".to_string(), "text/plain".to_string());
        
        HttpResponse { status, headers, body: body_bytes }
    }
    
    fn json(body: String) -> Self {
        let mut headers = HashMap::new();
        let body_bytes = body.into_bytes();
        
        headers.insert("Content-Length".to_string(), body_bytes.len().to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        HttpResponse { status: HttpStatus::Ok, headers, body: body_bytes }
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        let status_line = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status.code(),
            self.status.reason_phrase()
        );
        
        let headers_string: String = self.headers
            .iter()
            .map(|(k, v)| format!("{}: {}\r\n", k, v))
            .collect();
        
        let mut response = Vec::new();
        response.extend_from_slice(status_line.as_bytes());
        response.extend_from_slice(headers_string.as_bytes());
        response.extend_from_slice(b"\r\n");
        response.extend_from_slice(&self.body);
        
        response
    }
}

fn read_static_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let safe_path = path.trim_start_matches('/');
    let full_path = format!("static/{}", safe_path);
    fs::read(&full_path)
}

fn file_error_to_response(error: std::io::Error) -> HttpResponse {
    use std::io::ErrorKind;
    
    match error.kind() {
        ErrorKind::NotFound => {
            let body = String::from("404 - File not found");
            HttpResponse::text(HttpStatus::NotFound, body)
        }
        _other_error => {
            eprintln!("⚠️ Server error reading file: {}", _other_error);
            let body = String::from("500 - Internal server error");
            HttpResponse::text(HttpStatus::InternalServerError, body)
        }
    }
}

fn serve_static_file(path: &str) -> HttpResponse {
    match read_static_file(path) {
        Ok(body) => {
            let mime = get_mime_type(path);
            
            let mut headers = HashMap::new();
            headers.insert("Content-Length".to_string(), body.len().to_string());
            headers.insert("Content-Type".to_string(), mime.to_string());
            
            HttpResponse {
                status: HttpStatus::Ok,
                headers,
                body,
            }
        }
        Err(error) => file_error_to_response(error),
    }
}

fn read_static_file_secure(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let sanitized = path.trim_start_matches('/');
    
    // Block directory traversal attacks
    if sanitized.contains("..") || sanitized.contains('~') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Path traversal attempt detected"
        ));
    }
    
    // Reject absolute paths
    if sanitized.starts_with('/') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Absolute path not allowed"
        ));
    }
    
    let full_path = format!("static/{}", sanitized);
    fs::read(&full_path)
}

fn serve_static_file_secure(path: &str) -> HttpResponse {
    match read_static_file_secure(path) {
        Ok(body) => {
            let mime = get_mime_type(path);
            
            let mut headers = HashMap::new();
            headers.insert("Content-Length".to_string(), body.len().to_string());
            headers.insert("Content-Type".to_string(), mime.to_string());
            
            HttpResponse {
                status: HttpStatus::Ok,
                headers,
                body,
            }
        }
        Err(error) => {
            // Check if it's a security violation
            if error.kind() == std::io::ErrorKind::InvalidInput {
                eprintln!("🚨 Security: {} - {}", path, error);
                let body = String::from("403 - Forbidden: Invalid path");
                return HttpResponse::text(HttpStatus::NotFound, body);
            }
            file_error_to_response(error)
        }
    }
}

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:5000") {
        Ok(listener) => {
            println!("✅ Server bound to port 5000");
            listener
        }
        Err(error) => {
            println!("❌ Failed to start server: {}", error);
            return;
        }
    };

    println!("👂 Server listening on port 5000");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream),
            Err(error) => println!("⚠️ Connection failed: {}", error),
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&stream);

    let mut lines = buf_reader.by_ref().lines();
    let request_line = match lines.next() {
        Some(Ok(line)) => line,
        _ => {
            println!("❌ Failed to read request line");
            return;
        }
    };

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();
    let version = parts.next().unwrap_or("").to_string();

    let mut headers: HashMap<String, String> = HashMap::new();
    for line in lines.by_ref() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(":") {
            headers.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    let mut body = String::new();
    if let Some(length) = headers.get("Content-Length") {
        if let Ok(len) = length.parse::<usize>() {
            let mut buf = vec![0; len];
            let _ = buf_reader.read_exact(&mut buf);
            body = String::from_utf8_lossy(&buf).to_string();
        }
    }

    let request = HttpRequest {
        method,
        path,
        version,
        headers,
        body,
    };

    println!("📨 {} {}", request.method, request.path);

    let response = match request.path.as_str() {
        "/" => {
            let content = fs::read_to_string("response.html")
                .unwrap_or_else(|_| String::from("404 FILE NOT FOUND"));
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                content.len(),
                content
            )
        }
        "/api/chat" => {
            let content = String::from("{\"message\": \"Hello from the API!\"}");
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                content.len(),
                content
            )
        }
        "/api/about" => {
            let content = String::from("{\"info\": \"What about Rust?\"}");
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                content.len(),
                content
            )
        }
        _ => {
            let content = String::from("404 NOT FOUND");
            format!(
                "HTTP/1.1 404 NOT FOUND\r\nContent-Length: {}\r\n\r\n{}",
                content.len(),
                content
            )
        }
    };

    let _ = stream.write_all(response.as_bytes());
}
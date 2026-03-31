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
    let buf_reader = BufReader::new(&stream);
    let mut lines = buf_reader.lines();

    // Request line
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

    // Headers
    let mut headers = HashMap::new();

    for line in lines {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.is_empty() {
            break;
        }

        if let Some((key, value)) = line.split_once(":") {
            headers.insert(
                key.trim().to_string(),
                value.trim().to_string(),
            );
        }
    }

    let request = HttpRequest {
        method,
        path,
        version,
        headers,
    };

    println!("📨 {} {} {}", request.method, request.path, request.version);

    for (key, value) in &request.headers {
        println!("🔑 {}: {}", key, value);
    }

    // Response
 /*   let (status_line, contents) = match fs::read_to_string("response.html") {
        Ok(content) => ("HTTP/1.1 200 OK", content),
        Err(_) => ("HTTP/1.1 404 NOT FOUND", String::from("404 - File not found")),
    };

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    let _ = stream.write_all(response.as_bytes());
}*/

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
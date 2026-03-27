use std::{
    fs,
    net::{TcpListener, TcpStream},
    io::{BufReader, prelude::*},
};

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:5000") {
        Ok(listener) => {
            println!("✅ Server bound to port 5000");
            listener // Return the listener
        }
        Err(error) => {
            println!("❌ Failed to start server: {}", error);
            return; 
        }
    };
    
    println!("👂 Server listening on port 5000");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(error) => {
                println!("⚠️  Connection failed: {}", error);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    
    let mut http_request = Vec::new();
    
    for line_result in buf_reader.lines() {
        match line_result {
            Ok(line) => {
                if line.is_empty() {
                    break;
                }
                http_request.push(line);
            }
            Err(error) => {
                println!("❌ Error reading line: {}", error);
                return;
            }
        }
    }
    
    println!("📨 Received {} headers", http_request.len());
    if !http_request.is_empty() {
        println!("📝 First line: {}", http_request[0]);
    }
    
    let (status_line, contents) = match fs::read_to_string("response.html") {
        Ok(content) => {
            println!("📄 Found response.html ({} bytes)", content.len());
            ("HTTP/1.1 200 OK", content)
        }
        Err(error) => {
            println!("❌ Could not read response.html: {}", error);
            ("HTTP/1.1 404 NOT FOUND", String::from("404 - File not found"))
        }
    };
    
    let length = contents.len();
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line, length, contents
    );
    
    // Send the response
    match stream.write_all(response.as_bytes()) {
        Ok(_) => {
            println!("✅ Response sent successfully!");
        }
        Err(error) => {
            println!("❌ Failed to send response: {}", error);
        }
    }
    
    println!("--- Connection closed ---\n");
}
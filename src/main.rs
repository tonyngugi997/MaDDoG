use std::{
    fs,
    net::{TcpListener, TcpStream},
    io::{BufReader, prelude::*},   
};

fn main () {
    let listener = TcpListener::bind("127.0.0.1:5000").unwrap();
    println!("server listening on port 5000");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }

}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string("response.html").unwrap();
    let length = contents.len();
    let reponse = 
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(reponse.as_bytes()).unwrap();

}
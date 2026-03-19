use std::{
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
    println!("Request: {:#?}", http_request);
    let response = "HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nHello";
    stream.write_all(response.as_bytes()).unwrap();
}
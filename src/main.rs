use std::{
    net::{TcpListener, TcpStream},
    io::{BufReader, prelude::*},   
};

fn main () {
    let listener = TcpListener::bind("0.0.0.0:5000").unwrap();
    println!("server listening on port 5000");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);

    }

    fn handle_connection(mut stream: TcpStream) {
        let buf_reader = BufReader::new(&stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();
        println!("htt_request: {:#?}", http_request);  

    }
}
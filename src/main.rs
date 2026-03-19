use std::net::TcpListener;

fn main () {
    let listener = TcpListener::bind("0.0.0.0:5000").unwrap();
    println!("server listening on port 5000");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("connection established");
    }







}
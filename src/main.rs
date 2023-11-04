// Uncomment this block to pass the first stage
use std::{
    io::{BufRead, Read, Write},
    net::TcpListener,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 1024];
                if let Err(e) = stream.read_exact(&mut buffer) {
                    eprintln!("failed to read: {}", e);
                }

                let lines: Vec<_> = buffer.lines().flatten().collect();
                let first_line = lines.first().unwrap();
                let mut parts = first_line.split_whitespace();
                let path = parts.nth(1);
                match path {
                    Some("/") => stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap(),
                    _ => stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap(),
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

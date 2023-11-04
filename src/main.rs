// Uncomment this block to pass the first stage
use std::{
    io::{BufRead, Read, Write},
    net::TcpListener,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = Vec::new();
                println!("{buffer:?}");

                if let Err(e) = stream.read_to_end(&mut buffer) {
                    eprintln!("failed to read: {}", e);
                }

                let lines: Vec<_> = buffer.lines().flatten().collect();
                println!("{:?}", lines);
                let first_line = lines.first().unwrap();
                let mut parts = first_line.split_whitespace();
                match parts.nth(1) {
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

// Uncomment this block to pass the first stage
use anyhow::Context;
use std::{
    io::{BufRead, Read, Write},
    net::TcpListener,
};

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        let mut stream = stream?;

        let mut buf = [0; 1024];
        stream.read(&mut buf)?;

        let lines: Vec<_> = buf.lines().flatten().collect();
        let first_line = lines.first().unwrap();
        let mut parts = first_line.split_whitespace();
        match parts.nth(1) {
            Some("/") => {
                stream
                    .write_all(b"HTTP/1.1 200 OK\r\n\r\n")
                    .context("failed to write 200")?;
            }
            _ => {
                stream
                    .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                    .context("failed to write 404")?;
            }
        }
    }

    Ok(())
}

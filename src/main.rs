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
            Some(path) => {
                let (s, rest) = path.split_at(1);
                if s == "/" && rest.is_empty() {
                    stream
                        .write_all(b"HTTP/1.1 200 OK\r\n\r\n")
                        .context("failed to write 200")?;
                    continue;
                }
                let (maybe_echo, rest) = rest.split_once("/").context("invalid path")?;
                match maybe_echo {
                    "echo" => {
                        let header = vec![
                            "HTTP/1.1 200 OK",
                            "Content-Type: text/plain",
                            "Content-Length: ",
                        ]
                        .join("\r\n");
                        let resp = format!("{}{}\r\n\r\n{}", header, rest.len(), rest);
                        stream
                            .write_all(resp.as_bytes())
                            .context("failed to write content")?;
                    }
                    _ => {
                        stream
                            .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                            .context("failed to write 404")?;
                    }
                }
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

// Uncomment this block to pass the first stage
use anyhow::Context;
use clap::Parser;
use nom::InputTake;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    thread,
};

#[derive(Debug)]
struct Path<'a> {
    path: &'a str,
}

impl<'a> From<&'a str> for Path<'a> {
    fn from(s: &'a str) -> Self {
        let mut parts = s.split_whitespace();
        let _ = parts.next().unwrap();
        let path = parts.next().unwrap();
        Self { path }
    }
}

#[derive(Debug)]
struct Request<'a> {
    path: Path<'a>,
    headers: HashMap<&'a str, &'a str>,
}

impl<'a> Request<'a> {
    pub fn parse(lines: Vec<&'a str>) -> anyhow::Result<Self> {
        let mut headers = HashMap::new();
        let path: Path = lines
            .first()
            .map(|f| Path::from(*f))
            .context("failed to parse path")?;

        for header_line in lines.iter().skip(1) {
            if let Some((key, value)) = header_line.split_once(":") {
                let value = value.trim_start();
                headers.insert(key, value);
            }
        }

        Ok(Self { path, headers })
    }

    pub fn path(&self) -> &str {
        self.path.path
    }
}

struct Response;

impl Response {
    fn new() -> Self {
        Self
    }

    pub fn success<S>(&self, mut stream: S) -> anyhow::Result<()>
    where
        S: Write,
    {
        stream
            .write_all(b"HTTP/1.1 200 OK\r\n\r\n")
            .context("failed to write success")?;
        Ok(())
    }

    pub fn text<S>(&self, mut stream: S, text: &str) -> anyhow::Result<()>
    where
        S: Write,
    {
        let header = vec![
            "HTTP/1.1 200 OK",
            "Content-Type: text/plain",
            "Content-Length: ",
        ]
        .join("\r\n");
        let resp = format!("{}{}\r\n\r\n{}", header, text.len(), text);
        stream
            .write_all(resp.as_bytes())
            .context("failed to write text")?;
        Ok(())
    }

    pub fn file<S>(&self, mut stream: S, file: &Vec<u8>) -> anyhow::Result<()>
    where
        S: Write,
    {
        stream.write(
            b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: ",
        )?;
        stream.write(format!("{}\r\n\r\n", file.len()).as_bytes())?;
        stream.write(file.as_slice())?;
        stream.flush()?;
        Ok(())
    }

    pub fn not_found<S>(&self, mut stream: S) -> anyhow::Result<()>
    where
        S: Write,
    {
        stream
            .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
            .context("failed to write content")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The directory to serve files from, as an absolute path.
    #[clap(long)]
    directory: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        let mut stream = stream?;
        thread::spawn(move || -> anyhow::Result<()> {
            let mut buf = [0; 1024];
            stream.read(&mut buf)?;

            let args = Args::parse();
            let dir = args.directory;

            let s = String::from_utf8_lossy(&buf);
            let lines = s.split("\r\n").collect::<Vec<_>>();
            let request = Request::parse(lines)?;
            let resp = Response::new();

            match request.path() {
                "/" => {
                    resp.success(stream)?;
                }
                "/user-agent" => {
                    resp.text(stream, request.headers["User-Agent"])?;
                }
                p if p.starts_with("/files/") && dir.is_some() => {
                    let (filename, _) = p.take_split(7);
                    let file_path =
                        PathBuf::from(dir.context("missing directory")?).join(&filename);
                    if file_path.exists() {
                        let mut file = std::fs::File::open(file_path)?;
                        let mut contents = Vec::new();
                        file.read(&mut contents)?;
                        resp.file(stream, &contents)?;
                    } else {
                        resp.not_found(stream)?;
                    }
                }
                p if p.starts_with("/echo/") => {
                    let (str, _) = p.take_split(6);
                    resp.text(stream, str)?;
                }
                _ => {
                    resp.not_found(stream)?;
                }
            }

            Ok(())
        });
    }

    Ok(())
}

use anyhow::Context;
use clap::Parser;
use nom::InputTake;
use request::Request;
use response::Response;
use std::{
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    thread,
};

mod request;
mod response;

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

        thread::scope(|s| {
            s.spawn(move || -> anyhow::Result<()> {
                let args = Args::parse();
                let dir = args.directory;

                let mut buf = [0; 2048];
                stream.read(&mut buf).unwrap();
                let input = std::str::from_utf8(&buf).unwrap();

                let (_, request) = Request::parse(input).unwrap();
                let resp = Response::new();
                match request.path() {
                    "/" => {
                        resp.success(stream)?;
                    }
                    "/user-agent" => {
                        if let Some(user_agent) = request.headers.0.get("User-Agent") {
                            resp.text(stream, user_agent)?;
                        }
                    }
                    p if p.starts_with("/files/") && dir.is_some() => match request.method() {
                        "GET" => {
                            let (filename, _) = p.take_split(7);
                            let file_path =
                                PathBuf::from(dir.context("missing directory")?).join(&filename);
                            if file_path.exists() {
                                let mut file = std::fs::File::open(file_path)?;
                                let mut contents = Vec::new();
                                file.read_to_end(&mut contents)?;
                                resp.file(stream, &contents)?;
                            } else {
                                resp.not_found(stream)?;
                            }
                        }
                        "POST" => {
                            let (filename, _) = p.take_split(7);
                            let file_path =
                                PathBuf::from(dir.context("missing directory")?).join(&filename);
                            let mut file = std::fs::File::create(file_path)?;
                            if let Some(body) = request.body {
                                file.write_all(&body.0)?;
                            }
                            resp.created(stream)?;
                        }
                        m => {
                            unimplemented!("unsupported method: {m}");
                        }
                    },
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
        });
    }

    Ok(())
}

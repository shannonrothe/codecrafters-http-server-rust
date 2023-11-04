use std::io::Write;

use anyhow::Context;

pub struct Response;

impl Response {
    pub fn new() -> Self {
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
        stream.write(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: ")?;
        stream.write(format!("{}\r\n\r\n", text.len()).as_bytes())?;
        stream.write(text.as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    pub fn created<S>(&self, mut stream: S) -> anyhow::Result<()>
    where
        S: Write,
    {
        stream
            .write_all(b"HTTP/1.1 201 Created\r\n\r\n")
            .context("failed to write success")?;
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

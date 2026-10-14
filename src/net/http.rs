use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use super::tls::TlsStream;
use super::url::Url;

const USER_AGENT: &str = "Nami/0.1";
const TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

pub fn get(url: &Url) -> io::Result<Response> {
    let tcp = connect(&url.host, url.port)?;
    tcp.set_read_timeout(Some(TIMEOUT))?;
    tcp.set_write_timeout(Some(TIMEOUT))?;

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: {USER_AGENT}\r\nAccept: text/html,*/*\r\nAccept-Encoding: identity\r\n\r\n",
        url.path, url.host
    );

    if url.scheme == "https" {
        let mut tls = TlsStream::connect(tcp, &url.host)?;
        tls.write_all(request.as_bytes())?;
        read_response(tls)
    } else {
        let mut tcp = tcp;
        tcp.write_all(request.as_bytes())?;
        read_response(tcp)
    }
}

fn connect(host: &str, port: u16) -> io::Result<TcpStream> {
    let mut last_err = io::Error::other(format!("não resolveu {host}"));
    for addr in (host, port).to_socket_addrs()? {
        match TcpStream::connect_timeout(&addr, TIMEOUT) {
            Ok(tcp) => return Ok(tcp),
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

pub fn read_response<R: Read>(stream: R) -> io::Result<Response> {
    let mut reader = BufReader::new(stream);

    let status_line = read_line(&mut reader)?;
    let mut parts = status_line.splitn(3, ' ');
    let _version = parts.next();
    let status = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| io::Error::other(format!("status inválido: {status_line}")))?;

    let mut headers = Vec::new();
    loop {
        let line = read_line(&mut reader)?;
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
        }
    }

    let mut response = Response { status, headers, body: Vec::new() };

    let chunked = response
        .header("transfer-encoding")
        .is_some_and(|v| v.to_ascii_lowercase().contains("chunked"));
    let length = response.header("content-length").and_then(|v| v.parse::<usize>().ok());

    response.body = if chunked {
        read_chunked(&mut reader)?
    } else if let Some(len) = length {
        let mut body = vec![0; len];
        reader.read_exact(&mut body)?;
        body
    } else {
        let mut body = Vec::new();
        reader.read_to_end(&mut body)?;
        body
    };

    Ok(response)
}

fn read_line<R: BufRead>(reader: &mut R) -> io::Result<String> {
    let mut buf = Vec::new();
    reader.read_until(b'\n', &mut buf)?;
    while matches!(buf.last(), Some(b'\n' | b'\r')) {
        buf.pop();
    }
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

fn read_chunked<R: BufRead>(reader: &mut R) -> io::Result<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        let line = read_line(reader)?;
        let size_hex = line.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_hex, 16)
            .map_err(|_| io::Error::other(format!("chunk inválido: {line}")))?;
        if size == 0 {
            while !read_line(reader)?.is_empty() {}
            return Ok(body);
        }
        let start = body.len();
        body.resize(start + size, 0);
        reader.read_exact(&mut body[start..])?;
        read_line(reader)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_content_length() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 5\r\n\r\nhelloextra";
        let r = read_response(&raw[..]).unwrap();
        assert_eq!(r.status, 200);
        assert_eq!(r.header("content-type"), Some("text/html"));
        assert_eq!(r.body, b"hello");
    }

    #[test]
    fn parses_chunked() {
        let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nWiki\r\n6;ext=1\r\npedia \r\n0\r\nTrailer: x\r\n\r\n";
        let r = read_response(&raw[..]).unwrap();
        assert_eq!(r.body, b"Wikipedia ");
    }

    #[test]
    fn reads_until_close() {
        let raw = b"HTTP/1.0 404 Not Found\r\n\r\nnada";
        let r = read_response(&raw[..]).unwrap();
        assert_eq!(r.status, 404);
        assert_eq!(r.body, b"nada");
    }
}

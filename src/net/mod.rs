pub mod http;
pub mod tls;
pub mod url;

use std::fmt;
use std::io;

pub use url::Url;

const MAX_REDIRECTS: usize = 10;

#[derive(Debug)]
pub struct Resource {
    pub url: Url,
    pub body: String,
    pub view_source: bool,
}

#[derive(Debug)]
pub enum LoadError {
    Url(url::UrlError),
    Io(io::Error),
    TooManyRedirects,
    BadRedirect,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Url(e) => write!(f, "{e}"),
            LoadError::Io(e) => write!(f, "{e}"),
            LoadError::TooManyRedirects => write!(f, "redirecionamentos demais"),
            LoadError::BadRedirect => write!(f, "redirecionamento sem Location"),
        }
    }
}

impl std::error::Error for LoadError {}

impl From<url::UrlError> for LoadError {
    fn from(e: url::UrlError) -> Self {
        LoadError::Url(e)
    }
}

impl From<io::Error> for LoadError {
    fn from(e: io::Error) -> Self {
        LoadError::Io(e)
    }
}

pub fn load(input: &str) -> Result<Resource, LoadError> {
    let (input, view_source) = match input.trim().strip_prefix("view-source:") {
        Some(rest) => (rest, true),
        None => (input.trim(), false),
    };
    let url = Url::parse(input)?;
    let (url, bytes) = fetch(url)?;
    Ok(Resource {
        url,
        body: String::from_utf8_lossy(&bytes).into_owned(),
        view_source,
    })
}

fn fetch(mut url: Url) -> Result<(Url, Vec<u8>), LoadError> {
    for _ in 0..=MAX_REDIRECTS {
        match url.scheme.as_str() {
            "about" => return Ok((url, Vec::new())),
            "data" => {
                let body = decode_data(&url.path);
                return Ok((url, body));
            }
            "file" => {
                let path = String::from_utf8_lossy(&url::percent_decode(&url.path)).into_owned();
                let body = std::fs::read(path)?;
                return Ok((url, body));
            }
            _ => {}
        }

        let response = http::get(&url)?;
        if (300..400).contains(&response.status) {
            let location = response.header("location").ok_or(LoadError::BadRedirect)?;
            url = url.resolve(location)?;
            continue;
        }
        return Ok((url, response.body));
    }
    Err(LoadError::TooManyRedirects)
}

fn decode_data(spec: &str) -> Vec<u8> {
    let (meta, data) = spec.split_once(',').unwrap_or(("", spec));
    if meta.ends_with(";base64") {
        return base64_decode(data);
    }
    url::percent_decode(data)
}

fn base64_decode(input: &str) -> Vec<u8> {
    let value = |c: u8| match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' | b'-' => Some(62),
        b'/' | b'_' => Some(63),
        _ => None,
    };
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut acc: u32 = 0;
    let mut bits = 0;
    for v in input.bytes().filter_map(value) {
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_data_url() {
        let r = load("data:text/html,<h1>Oi%20Nami</h1>").unwrap();
        assert_eq!(r.body, "<h1>Oi Nami</h1>");
    }

    #[test]
    fn loads_base64_data_url() {
        let r = load("data:text/plain;base64,TmFtaQ==").unwrap();
        assert_eq!(r.body, "Nami");
    }

    #[test]
    fn detects_view_source() {
        let r = load("view-source:data:text/html,<p>x</p>").unwrap();
        assert!(r.view_source);
        assert_eq!(r.body, "<p>x</p>");
    }

    #[test]
    fn loads_file() {
        let dir = std::env::temp_dir().join("nami-test.html");
        std::fs::write(&dir, "<p>arquivo</p>").unwrap();
        let r = load(&format!("file://{}", dir.display())).unwrap();
        assert_eq!(r.body, "<p>arquivo</p>");
    }
}

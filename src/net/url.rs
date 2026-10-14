use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url {
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub fragment: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UrlError {
    MissingScheme,
    UnsupportedScheme(String),
    MissingHost,
    InvalidPort(String),
}

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlError::MissingScheme => write!(f, "URL sem esquema"),
            UrlError::UnsupportedScheme(s) => write!(f, "esquema não suportado: {s}"),
            UrlError::MissingHost => write!(f, "URL sem host"),
            UrlError::InvalidPort(p) => write!(f, "porta inválida: {p}"),
        }
    }
}

impl std::error::Error for UrlError {}

pub fn default_port(scheme: &str) -> u16 {
    match scheme {
        "https" => 443,
        _ => 80,
    }
}

impl Url {
    pub fn parse(input: &str) -> Result<Url, UrlError> {
        let input = input.trim();
        let (scheme, rest) = input.split_once(':').ok_or(UrlError::MissingScheme)?;
        let scheme = scheme.to_ascii_lowercase();

        match scheme.as_str() {
            "data" | "about" => {
                return Ok(Url {
                    scheme,
                    host: String::new(),
                    port: 0,
                    path: rest.to_string(),
                    fragment: None,
                })
            }
            "http" | "https" | "file" => {}
            _ => return Err(UrlError::UnsupportedScheme(scheme)),
        }

        let rest = rest.strip_prefix("//").ok_or(UrlError::MissingHost)?;
        let (rest, fragment) = match rest.split_once('#') {
            Some((r, f)) => (r, Some(f.to_string())),
            None => (rest, None),
        };
        let split_at = rest.find(['/', '?']).unwrap_or(rest.len());
        let (authority, path) = rest.split_at(split_at);
        let path = match path {
            "" => "/".to_string(),
            p if p.starts_with('?') => format!("/{p}"),
            p => p.to_string(),
        };

        if scheme == "file" {
            return Ok(Url { scheme, host: authority.to_string(), port: 0, path, fragment });
        }

        if authority.is_empty() {
            return Err(UrlError::MissingHost);
        }
        let (host, port) = match authority.rsplit_once(':') {
            Some((h, p)) => (h, p.parse().map_err(|_| UrlError::InvalidPort(p.to_string()))?),
            None => (authority, default_port(&scheme)),
        };

        Ok(Url {
            host: host.to_ascii_lowercase(),
            port,
            scheme,
            path,
            fragment,
        })
    }

    pub fn origin(&self) -> String {
        if self.port == default_port(&self.scheme) {
            format!("{}://{}", self.scheme, self.host)
        } else {
            format!("{}://{}:{}", self.scheme, self.host, self.port)
        }
    }

    pub fn resolve(&self, href: &str) -> Result<Url, UrlError> {
        let href = href.trim();
        if href.contains("://") || href.starts_with("data:") || href.starts_with("about:") {
            return Url::parse(href);
        }
        if let Some(rest) = href.strip_prefix("//") {
            return Url::parse(&format!("{}://{rest}", self.scheme));
        }
        if let Some(fragment) = href.strip_prefix('#') {
            let mut url = self.clone();
            url.fragment = Some(fragment.to_string());
            return Ok(url);
        }

        let base_path = self.path.split('?').next().unwrap_or("/");
        let joined = if href.starts_with('/') {
            href.to_string()
        } else if href.starts_with('?') {
            format!("{base_path}{href}")
        } else {
            let dir = &base_path[..base_path.rfind('/').map_or(0, |i| i + 1)];
            format!("{dir}{href}")
        };

        let (joined, fragment) = match joined.split_once('#') {
            Some((p, f)) => (p.to_string(), Some(f.to_string())),
            None => (joined, None),
        };
        let (path_part, query) = match joined.split_once('?') {
            Some((p, q)) => (p.to_string(), Some(q.to_string())),
            None => (joined, None),
        };

        let mut url = self.clone();
        url.path = normalize_path(&path_part);
        if let Some(q) = query {
            url.path.push('?');
            url.path.push_str(&q);
        }
        url.fragment = fragment;
        Ok(url)
    }
}

fn normalize_path(path: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();
    let trailing_slash = path.ends_with('/') || path.ends_with("/.") || path.ends_with("/..");
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            s => segments.push(s),
        }
    }
    let mut out = format!("/{}", segments.join("/"));
    if trailing_slash && out != "/" {
        out.push('/');
    }
    out
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.scheme.as_str() {
            "data" | "about" => write!(f, "{}:{}", self.scheme, self.path)?,
            "file" => write!(f, "file://{}{}", self.host, self.path)?,
            _ => write!(f, "{}{}", self.origin(), self.path)?,
        }
        if let Some(fragment) = &self.fragment {
            write!(f, "#{fragment}")?;
        }
        Ok(())
    }
}

pub fn percent_decode(input: &str) -> Vec<u8> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
            if let Some(b) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_http() {
        let u = Url::parse("http://Example.org/index.html").unwrap();
        assert_eq!(u.scheme, "http");
        assert_eq!(u.host, "example.org");
        assert_eq!(u.port, 80);
        assert_eq!(u.path, "/index.html");
    }

    #[test]
    fn parses_https_with_port_query_fragment() {
        let u = Url::parse("https://localhost:8443?q=1#top").unwrap();
        assert_eq!(u.port, 8443);
        assert_eq!(u.path, "/?q=1");
        assert_eq!(u.fragment.as_deref(), Some("top"));
        assert_eq!(u.to_string(), "https://localhost:8443/?q=1#top");
    }

    #[test]
    fn parses_file_and_data() {
        let f = Url::parse("file:///tmp/a.html").unwrap();
        assert_eq!(f.path, "/tmp/a.html");
        let d = Url::parse("data:text/html,<b>oi</b>").unwrap();
        assert_eq!(d.path, "text/html,<b>oi</b>");
    }

    #[test]
    fn rejects_unknown_scheme() {
        assert!(matches!(Url::parse("gopher://x"), Err(UrlError::UnsupportedScheme(_))));
    }

    #[test]
    fn resolves_relative() {
        let base = Url::parse("http://a.com/dir/page.html?x=1").unwrap();
        assert_eq!(base.resolve("other.html").unwrap().to_string(), "http://a.com/dir/other.html");
        assert_eq!(base.resolve("../up.html").unwrap().to_string(), "http://a.com/up.html");
        assert_eq!(base.resolve("/root").unwrap().to_string(), "http://a.com/root");
        assert_eq!(base.resolve("//b.com/x").unwrap().to_string(), "http://b.com/x");
        assert_eq!(base.resolve("?y=2").unwrap().to_string(), "http://a.com/dir/page.html?y=2");
        assert_eq!(base.resolve("#s").unwrap().to_string(), "http://a.com/dir/page.html?x=1#s");
        assert_eq!(base.resolve("https://c.com").unwrap().to_string(), "https://c.com/");
    }

    #[test]
    fn decodes_percent() {
        assert_eq!(percent_decode("a%20b%3C"), b"a b<");
        assert_eq!(percent_decode("100%"), b"100%");
    }
}

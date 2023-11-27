use url::Url;

pub fn parse_url(input: &str) -> Result<(String, String, u16, String), url::ParseError> {
    let url = Url::parse(input)?;
    let scheme = url.scheme();
    let host = url.host_str().unwrap();
    let port = url.port_or_known_default().unwrap();
    let path = url.path().to_string();
    Ok((scheme.to_string(), host.to_string(), port, path))
}

fn main() {
    let input = "http://info.cern.ch/hypertext/WWW/TheProject.html";
    let (scheme, host, port, path) = parse_url(input).unwrap();
    println!("Esquema: {}", scheme);
    println!("Host: {}", host);
    println!("Porta: {}", port);
    println!("Caminho: {}", path);
}

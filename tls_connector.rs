extern crate native_tls;
extern crate url;

use native_tls::TlsConnector;
use std::io::{self, Read};
use url::Url;

fn connect() {
    let connector = TlsConnector::new().unwrap();

    let stream = connector
        .connect(host, port)
        .expect("Failed to connect to host");

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    println!("Response: {}", response);
}
extern crate native_tls;

use native_tls::TlsConnector;

use std::io::{self, Read};

pub fn connect(host: &str, port: &i16) {
    let connector = TlsConnector::new().unwrap();
    
    let stream = connector
        .connect(host, port)
        .expect("Failed to connect to host");

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    println!("Response: {}", response);
}

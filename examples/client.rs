use http::Uri;
use may_http::client::*;
use std::io::{self, Read};

fn client_get(uri: Uri) -> io::Result<()> {
    let mut client = {
        let host = uri.host().unwrap_or("127.0.0.1");
        let port = uri.port_u16().unwrap_or(80);
        HttpClient::connect((host, port))?
    };

    let mut s = String::new();
    for _ in 0..100 {
        let uri = uri.clone();
        let mut rsp = client.get(uri)?;
        rsp.read_to_string(&mut s)?;
        println!("get rsp={}", s);
        s.clear();
    }
    Ok(())
}

fn main() {
    env_logger::init();
    let uri: Uri = "http://127.0.0.1:8080/".parse().unwrap();
    client_get(uri).unwrap();
}

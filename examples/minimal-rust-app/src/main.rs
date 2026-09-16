use std::io::{Read, Write};
use std::net::TcpListener;

// Matches Oxide's runtime/src/container.rs, which hardcodes 3000/tcp as the
// container's exposed port.
fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").expect("failed to bind :3000");
    println!("oxide-minimal-app listening on :3000");

    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let mut buf = [0u8; 512];
        let _ = stream.read(&mut buf);

        let body = b"oxide minimal app: OK";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.write_all(body);
    }
}

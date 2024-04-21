use std::net::TcpStream;
use std::io::{Read, Write};

fn main() {
    let mut stream = match TcpStream::connect("127.0.0.1:8080") {
        Ok(s) => s,
        Err(e) => {
            println!("Couldn't connect to server: {}", e);
            return;
        }
    };

    let mut response_buffer = [0; 1024];
    
    // write message to server
    stream.write("Hello server!".as_bytes()).expect("Failed to write response to server :(");
    
    // read server response
    stream.read(&mut response_buffer).expect("Failed to read response from server :(");
    let stringified_response = String::from_utf8_lossy(&mut response_buffer);
    println!("From server: {}", stringified_response);
}

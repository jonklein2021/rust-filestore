use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    let mut request_buffer = [0; 1024];

    // Read data from stream into the buffer array
    stream.read(&mut request_buffer).expect("Failed to read from client :(");

    // Convert data in the buffer to a UTF-8 string
    let request = String::from_utf8_lossy(&request_buffer);
    println!("From client: {}", request);

    // Send message to client
    stream.write("Hello client!".as_bytes()).expect("Failed to write response to client :(");
    return;
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");
    println!("Server listening on 127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| handle_client(stream)); // closure
                // handle_client(stream); // client hangs with this
            }
            Err(e) => {
                eprintln!("Failed to establish connection: {}", e); // write error to stderr stream
            }
        }
    }
}

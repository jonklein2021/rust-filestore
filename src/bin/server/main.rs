// use std::fs::File;
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

    /* Below is for browser clients
    
    // Open index.html and create response from it
    let mut f = match File::open("index.html") {
        Ok(file) => file,
        Err(_) => {
            // If the file doesn't exist or cannot be opened, return a 404 Not Found response
            let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
            stream.write(response.as_bytes()).expect("Failed to write failure-response message :(");
            return;
        }
    };

    let mut response = Vec::new();
    if let Err(e) = f.read_to_end(&mut response) {
        println!("Failed to read index.html: {}", e);
        return;
    }

    // Write the HTTP response header
    let http_response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", response.len());
    stream.write(http_response.as_bytes()).expect("Failed to write response header :(");

    // Write the file content to the TCP stream
    stream.write(&response).expect("Failed to write file content :(");
    
    */
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

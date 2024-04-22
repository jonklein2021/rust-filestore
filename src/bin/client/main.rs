use std::io;
use std::error::Error;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:8080";
    let stream = TcpStream::connect(addr).await?;

    // wait for the socket to be writable
    stream.writable().await?;
    
    // loop until write to server is successful
    loop {
        match stream.try_write(b"Hello Server!") {
            Ok(_) => break,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e.into()),
        }
    }    

    // wait until server is readable
    stream.readable().await?;
    
    let mut response_buffer = vec![0; 1024];

    // loop until stream is read into buffer successfully
    loop {
        match stream.try_read(&mut response_buffer) {
            Ok(n) => {
                response_buffer.truncate(n);
                break;
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue, // blocking error; try again
            Err(e) => return Err(e.into()), // panic if any other other error
        }
    }
    
    // read server response
    let stringified_response = String::from_utf8_lossy(&mut response_buffer);
    println!("From server: {}", stringified_response);

    Ok(())
}

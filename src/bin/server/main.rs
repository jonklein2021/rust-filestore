use std::io;
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};

async fn handle_client(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut request_buffer = vec![0; 1024];

    // wait until client is readable
    stream.readable().await?;

    // loop until read from stream reads successfully
    loop {
        match stream.try_read(&mut request_buffer) {
            Ok(n) => {
                request_buffer.truncate(n);
                break;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue; // blocking error; try again
            }
            Err(e) => {
                return Err(e.into()); // panic if any other other error
            } 
        }
    }

    // print client message
    let request = String::from_utf8_lossy(&request_buffer);
    println!("From client: {}", request);

    // wait for the socket to be writable
    stream.writable().await?;

    // loop until write is successful
    loop {
        match stream.try_write(b"Hello Client!") {
            Ok(_n) => {
                break;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue; // readiness event is a false positive
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((socket, _)) => { // socket is tokio::net::TcpStream, _ is address
                if let Err(e) = handle_client(socket).await {
                    return Err(e.into());
                }
            }
            Err(e) => return Err(e.into())
        }
    }
    
}

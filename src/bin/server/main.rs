use std::io;
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

async fn handle_client(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    // wait until client is readable
    stream.readable().await?;
    
    // max buffer size = 1.048576 MB
    let mut request_buffer = vec![0; 1<<20];

    // loop until read from stream reads successfully
    loop {
        match stream.try_read(&mut request_buffer) {
            Ok(n) => {
                request_buffer.truncate(n); // free excess space
                break;
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue, // blocking error; try again
            Err(e) => return Err(e.into()) // panic on any other error
        }
    }

    // store client's file
    let mut f = File::create("file.txt").await?;
    f.write_all(&request_buffer).await?;

    // wait for the socket to be writable
    stream.writable().await?;

    // loop until response message to client is successfully sent
    loop {
        match stream.try_write(b"File stored successfully") {
            Ok(_) => break,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue, // readiness event is a false positive
            Err(e) => return Err(e.into())
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on {}", addr);

    // handle client connections in infinite loop
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

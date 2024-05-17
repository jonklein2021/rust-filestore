//! Jon Klein
//! server/main.rs
//! 2024
//!
//! Server source code for https://github.com/jonklein2021/rust-filestore/
//! This accepts a TCP connection with a client, and stores, retrieves, or
//! deletes a file of their choice
//!

// lib.rs
use rust_filestore::Operation;
use rust_filestore::{deserialize_request};

use std::io;
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWriteExt;
use tokio::fs::{self, File};

async fn handle_client(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    // wait until client is readable
    stream.readable().await?;
    
    // max buffer size = 1.048576 MB
    let mut request_buffer = vec![0; 1<<20];

    // loop until read from stream reads successfully
    loop {
        match stream.try_read(&mut request_buffer) {
            Ok(n) => {
                request_buffer.truncate(n); // excess space
                break;
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue, // blocking error; try again
            Err(e) => return Err(e.into()) // panic on any other error
        }
    }

    // handle client's request
    let req = deserialize_request(&request_buffer).await?;

    // response to client to be replaced in following match statement
    let response_message = match req.op {
        Operation::READ => {
            // return file to user if it exists
            String::from("File successfully returned.")
        },
        Operation::WRITE => {
            // store file on disk
            let path = format!("files/{}", &req.filename);
    
            // ensure the directory exists
            if let Some(parent) = std::path::Path::new(&path).parent() {
                fs::create_dir_all(parent).await?;
            }

            // create file
            let mut file = File::create(&path).await?;
            file.write_all(&req.filebytes).await?;
            file.flush().await?;

            String::from("File successfully stored.")
        },
        Operation::DELETE => {
            // delete file from disk
            String::from("File successfully deleted.")
        },
        Operation::LIST => {
            // list all names of file currently stored
            String::from("Files successfully listed.")
        }
    };

    // wait for the socket to be writable
    stream.writable().await?;

    // loop until response to client is successfully sent
    loop {
        match stream.try_write(response_message.as_bytes()) {
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

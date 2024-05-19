//! Jon Klein
//! server/main.rs
//! 2024
//!
//! Server source code for https://github.com/jonklein2021/rust-filestore/
//! This accepts a TCP connection with a client, and stores, retrieves, or
//! deletes a file of their choice
//!

// lib.rs
use rust_filestore::{Operation, Response};
use rust_filestore::{deserialize_request, serialize_response};

use std::io;
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::fs::File;

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
    
    // server-side path to file
    let path = format!("files/{}", &req.filename);

    // response to client to be replaced in following match statement
    let response = match req.op {
        Operation::READ => { // return file to user if it exists
            match File::open(&path).await {
                Ok(mut file) => {
                    let mut contents = vec![];
                    file.read_to_end(&mut contents).await?;

                    Response {
                        ok: true,
                        msg: String::from("File successfully returned."),
                        filename: Some(req.filename.clone()),
                        filebytes: Some(contents)
                    }
                }
                Err(_) => Response {
                    ok: false,
                    msg: String::from("File not found on server."),
                    filename: None,
                    filebytes: None
                }
            }
        },
        Operation::WRITE => { // store file on disk
            // create the directory if it doesn't exist
            if let Some(parent) = std::path::Path::new(&path).parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            // create file
            let mut file = File::create(&path).await?;
            file.write_all(&req.filebytes).await?;
            file.flush().await?;

            Response {
                ok: true,
                msg: String::from("File successfully stored."),
                filename: None,
                filebytes: None
            }
        },
        Operation::DELETE => { // delete file from disk
            match tokio::fs::remove_file(&path).await {
                Ok(_) => Response {
                        ok: true,
                        msg: String::from("File successfully deleted."),
                        filename: None,
                        filebytes: None
                    },
                Err(_) => Response {
                    ok: false,
                    msg: String::from("File could not be deleted."),
                    filename: None,
                    filebytes: None
                }
            }
        },
        Operation::LIST => { // list all names of file currently stored
            // read directory asynchronously, store filenames in a string
            let mut files = String::new();
            let mut entries = tokio::fs::read_dir("files").await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    files += &path.to_string_lossy();
                    files += "\n";
                }
            }

            Response {
                ok: true,
                msg: files,
                filename: None,
                filebytes: None
            }
        }
    };

    let response_buffer = serialize_response(&response).await?;

    // wait for the socket to be writable
    stream.writable().await?;

    // loop until response to client is successfully sent
    loop {
        match stream.try_write(&response_buffer) {
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

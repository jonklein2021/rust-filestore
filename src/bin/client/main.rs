//! Jon Klein
//! client/main.rs
//! 2024
//!
//! Client source code for https://github.com/jonklein2021/rust-filestore/
//! This code parses command-line arguments, establishes a connection with 
//! a TCP server, and performs an individual file operation per execution
//!

extern crate getopts;
use getopts::Options;

// lib.rs
use rust_filestore::{Operation, Request};
use rust_filestore::{serialize_request, deserialize_response};

use std::io;
use std::env;
use std::path::Path;
use std::error::Error;

use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::fs::File;

struct Config {
    addr: String, // default is 127.0.0.0:8080
    filename: Option<String>, // path to file
    operation: Operation // what to do
}

impl Config {
    fn println(&self) {
        if let Some(filename) = &self.filename {
            println!("IP/Port = {}, File = {}, Operation = {}", self.addr, filename, self.operation.to_string());
        } else {
            println!("IP/Port = {}, Operation = {}", self.addr, self.operation.to_string());
        }
    }
}

fn usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [file] -a [addr] [--read | --write | --delete]", program);
    print!("{}", opts.usage(&brief));
}

fn parse_args(args: Vec<String>) -> Result<Config, Box<dyn Error>> {
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("a", "addr", "server address and port", "<ip>"); // 127.0.0.1:8080 by default
    
    // operations: exactly one of {r, w, d} is required
    opts.optflagopt("r", "read", "read from server", "<file>");
    opts.optflagopt("w", "write", "write file to server", "<file>");
    opts.optflagopt("d", "delete", "delete file on server", "<file>");
    opts.optflag("l", "list", "list all files on server");

    // help option
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(e) => return Err(e.into())
    };

    // -h flag
    if matches.opt_present("h") {
        usage(&program, opts);
        return Err("Help menu".into());
    }

    // ensure that exactly one operation is provided
    let options = vec!["r", "w", "d", "l"];
    if options.iter().filter(|&&opt| matches.opt_present(opt)).count() != 1 {
        return Err("Select exactly one of {-r, -w, -d, -l}".into());
    }

    // ip and port
    let addr = if matches.opt_present("a") {
        let arg = matches.opt_str("a").unwrap();
        let parts_vec: Vec<&str> = arg.split(":").collect();
        if parts_vec.len() != 2 {
            return Err("Bad address/port. Example: 127.0.0.1:8080".into());
        }
        let ip = parts_vec[0].trim();
        let port = parts_vec[1].trim();
        if let Ok(_) = ip.parse::<std::net::Ipv4Addr>() {
            if let Ok(_) = port.parse::<u16>() {
                arg
            } else {
                return Err("Bad address/port. Example: 127.0.0.1:8080".into());
            }
        } else {
            return Err("Bad address/port. Example: 127.0.0.1:8080".into());
        }
    } else {
        // no argument given, use default
        String::from("127.0.0.1:8080")
    };

    // filename, operation
    let mut filename: Option<String> = None;
    let mut operation = Operation::LIST;
    for (i, opt) in options.iter().enumerate() {
        if matches.opt_present(opt) && opt != &"l" {
            operation = Operation::from_u8(i as u8).unwrap();
            filename = Some(matches.opt_str(opt).ok_or("Missing file.")?);
            break;
        }
    }

    return Ok(Config {addr, filename, operation});
}

// send file and operation to server
async fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    // package arguments into a request
    let mut filebytes = vec![];
    let filename = config.filename.as_deref().unwrap_or("");
    let basename = Path::new(filename).file_name().unwrap_or_default().to_string_lossy().to_string();

    if config.operation == Operation::WRITE && !filename.is_empty() {
        if let Ok(mut f) = File::open(filename).await {
            f.read_to_end(&mut filebytes).await?;
        } else {
            return Err("File not found".into());
        }
    }
    
    let req = Request {
        op: config.operation,
        filename: basename,
        filebytes
    };

    let request_buffer = serialize_request(&req).await?;
    
    // establish connection with server
    let stream = TcpStream::connect(&config.addr).await?;

    // wait for the socket to be writable
    stream.writable().await?;
    
    // loop until write to server is successful
    loop {
        match stream.try_write(&request_buffer) {
            Ok(_) => break,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e.into())
        }
    }

    // wait until server is readable
    stream.readable().await?;
    
    let mut response_buffer = vec![0; 1<<20]; // about 1MB

    // loop until stream is read into buffer successfully
    loop {
        match stream.try_read(&mut response_buffer) {
            Ok(n) => {
                response_buffer.truncate(n);
                break;
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue, // blocking error; try again
            Err(e) => return Err(e.into()) // panic if any other error
        }
    }
    
    // receive and deserialize server response
    let response = deserialize_response(&response_buffer).await?;
    
    // write file to disk if there is one
    if let (Some(filename), Some(filebytes)) = (&response.filename, &response.filebytes) {
        if config.operation == Operation::READ {
            let path = format!("received/{}", filename);

            // create the directory if it doesn't exist
            if let Some(parent) = Path::new(&path).parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            // write file to disk
            let mut file = File::create(&path).await?;
            file.write_all(filebytes).await?;
            file.flush().await?;
            println!("File '{}' saved successfully.", filename);
        }
    }

    if response.ok {
        println!("{}", &response.msg);
        Ok(())
    } else {
        Err(response.msg.into())
    }

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config = match parse_args(args) {
        Ok(cfg) => cfg,
        // help menu short circuit is encoded as an error and is caught below
        Err(ref e) if e.to_string() == String::from("Help menu") => {
            return Ok(());
        },
        Err(e) => return Err(e.into()) // panic on other errors
    };
    config.println();
    run(&config).await
}

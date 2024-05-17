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
use rust_filestore::{serialize_request};

use std::io;
use std::env;
use std::error::Error;

use tokio::net::TcpStream;
use tokio::fs::File;

struct Config {
    addr: String, // default is 127.0.0.0:8080
    filename: String, // path to file
    operation: Operation // what to do with file
}

impl Config {
    fn println(&self) {
        println!("IP/Port = {}, File = {}, Operation = {}", self.addr, self.filename, self.operation.to_string());
    }
}

fn usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} <file> -a <addr> [--read | --write | --delete]", program);
    print!("{}", opts.usage(&brief));
}

fn parse_args(args: Vec<String>) -> Result<Config, Box<dyn Error>> {
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("a", "addr", "server address and port", "<ip>"); // 127.0.0.1:8080 by default
    
    // operations: exactly one of {r, w, d} is required
    opts.optflag("r", "read", "read from server");
    opts.optflag("w", "write", "write file to server");
    opts.optflag("d", "delete", "delete file on server");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(e) => return Err(e.into())
    };

    // -h flag or no file provided
    if matches.opt_present("h") || matches.free.is_empty() {
        usage(&program, opts);
        return Err("Help menu".into());
    }

    // too many operations provided
    if (matches.opt_present("r") && matches.opt_present("w")) || 
        (matches.opt_present("w") && matches.opt_present("d")) || 
        (matches.opt_present("r") && matches.opt_present("r")) {
        return Err("Select exactly one of {-r, -w, -d}".into());
    }

    // file
    let file = matches.free[0].clone();

    // ip and port
    let address_port = if matches.opt_present("a") {
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

    // operation
    let op = if matches.opt_present("r") {
        Operation::READ
    } else if matches.opt_present("d") {
        Operation::DELETE
    } else {
        Operation::WRITE // -w and default option
    };

    return Ok(Config {
        addr: address_port,
        filename: file,
        operation: op
    });
}

// send file and operation to server
async fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    // establish connection with server
    let stream = TcpStream::connect(&config.addr).await?;

    // open file, serialize request
    let f = File::open(&config.filename).await?;

    let mut req = Request {
        op: config.operation,
        filename: config.filename.clone(),
        file: f
    };

    let request_buffer = serialize_request(&mut req).await?;

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
    
    let mut response_buffer = vec![0; 1024];

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
    
    // read server response
    let stringified_response = String::from_utf8_lossy(&mut response_buffer);
    println!("From server: {}", stringified_response);

    Ok(())
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
    return run(&config).await;
}

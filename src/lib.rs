//! Jon Klein
//! lib.rs
//! 2024
//!
//! Common functions for client and server files
//! This file defines serialilzation and deserialization
//! functions for requests and responses as well as
//! helpful debugger functions
//!

#![allow(dead_code)]

use std::error::Error;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Copy, Clone)]
pub enum Operation {
    READ, // read file from server, as u8 = 0
    WRITE, // write file to server, as u8 = 1
    DELETE, // delete file from server, as u8 = 2
}

impl Operation {
    pub fn to_string(&self) -> &str {
        match self {
            Operation::READ => "READ",
            Operation::WRITE => "WRITE",
            Operation::DELETE => "DELETE"
        }
    }
}

pub struct Request {
    pub op: Operation,
    pub filename: String,
    pub file: File,
}

// {op, filename, file} -> [op, len(filename), filename, len(file), file]
pub async fn serialize_request(req: &mut Request) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut result = Vec::new();

    // push op
    result.push(req.op as u8);

    // push filename
    let filename_bytes = req.filename.as_bytes();
    let filename_len = filename_bytes.len() as u32;
    result.extend_from_slice(&filename_len.to_be_bytes());
    result.extend_from_slice(filename_bytes);

    // get file content
    let mut contents = vec![];
    req.file.read_to_end(&mut contents).await?; // why mutable ref?

    // push file contents
    let file_len = contents.len() as u32;
    result.extend_from_slice(&file_len.to_be_bytes());
    result.extend_from_slice(&contents);
    
    Ok(result)
}

// [op, len(filename), filename, len(file), file] -> {op, filename, file}
// pub fn deserialize_request(data: Vec<u8>) -> Request {    
//     let result = Request{

//     };

//     return result;
// }

pub mod debug {
    fn print_type_of<T>(_: &T) {
        println!("{}", std::any::type_name::<T>())
    }
}

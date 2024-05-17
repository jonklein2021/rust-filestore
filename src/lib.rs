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
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Copy, Clone)]
pub enum Operation {
    READ, // read file from server, as u8 = 0
    WRITE, // write file to server, as u8 = 1
    DELETE, // delete file from server, as u8 = 2
}

impl Operation {
    pub fn from_u8(value: u8) -> Option<Operation> {
        match value {
            0 => Some(Operation::READ),
            1 => Some(Operation::WRITE),
            2 => Some(Operation::DELETE),
            _ => None,
        }
    }
    
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
// Deserialize function
pub async fn deserialize_request(data: Vec<u8>) -> Result<Request, Box<dyn Error>> {
    let mut pos = 0;

    // read op
    let op = Operation::from_u8(data[pos]).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid operation"))?;
    pos += 1;

    // read len(filename)
    let filename_len = u32::from_be_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
    pos += 4;

    // read filename
    let filename = String::from_utf8(data[pos..pos+filename_len].to_vec()).map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8 sequence"))?;
    pos += filename_len;

    // read len(file)
    let file_len = u32::from_be_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
    pos += 4;

    // read file
    let file_contents = data[pos..pos+file_len].to_vec();

    // write contents to a temporary file
    let mut file = File::create(&filename).await?;
    file.write_all(&file_contents).await?;
    file.flush().await?; 

    // Open the file for reading
    let file = File::open(&filename).await?;

    Ok(Request{op, filename, file})
}

pub mod debug {
    fn print_type_of<T>(_: &T) {
        println!("{}", std::any::type_name::<T>())
    }
}

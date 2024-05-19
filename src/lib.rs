//! Jon Klein
//! lib.rs
//! 2024
//!
//! Common structs and functions for client and server files
//! This file defines serialilzation and deserialization
//! functions for requests and responses
//!

use std::error::Error;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Operation {
    READ, // read file from server, as u8 = 0
    WRITE, // write file to server, as u8 = 1
    DELETE, // delete file from server, as u8 = 2
    LIST, // list all files on server, as u8 = 3
}

impl Operation {
    pub fn from_u8(value: u8) -> Option<Operation> {
        match value {
            0 => Some(Operation::READ),
            1 => Some(Operation::WRITE),
            2 => Some(Operation::DELETE),
            3 => Some(Operation::LIST),
            _ => None,
        }
    }
    
    pub fn to_string(&self) -> &str {
        match self {
            Operation::READ => "READ",
            Operation::WRITE => "WRITE",
            Operation::DELETE => "DELETE",
            Operation::LIST => "LIST"
        }
    }
}

pub struct Request {
    pub op: Operation,
    pub filename: String,
    pub filebytes: Vec<u8>
}

// Request{op, filename, file} -> Vec[op, len(filename), filename, len(file), file]
pub async fn serialize_request(req: &Request) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut result = Vec::new();

    // push op
    result.push(req.op as u8);

    // push filename
    let filename_bytes = req.filename.as_bytes();
    let filename_len = filename_bytes.len() as u32;
    result.extend_from_slice(&filename_len.to_be_bytes());
    result.extend_from_slice(filename_bytes);

    // push file contents
    let file_len = req.filebytes.len() as u32;
    result.extend_from_slice(&file_len.to_be_bytes());
    result.extend_from_slice(&req.filebytes);
    
    Ok(result)
}

// Vec[op, len(filename), filename, len(file), file] -> Request{op, filename, file}
pub async fn deserialize_request(data: &Vec<u8>) -> Result<Request, Box<dyn Error>> {
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

    // read number of bytes of file
    let file_len = u32::from_be_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
    pos += 4;

    // read file bytes
    let filebytes = data[pos..pos+file_len].to_vec();

    Ok(Request{op, filename, filebytes})
}

pub struct Response {
    pub ok: bool,
    pub msg: String,
    pub filename: Option<String>,
    pub filebytes: Option<Vec<u8>>
}

// Response{ok, msg, filename, filebytes} -> Vec[ok, len(msg), msg, len(filename), filename, len(filebytes), filebytes]
pub async fn serialize_response(res: &Response) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut result = Vec::new();

    // push ok
    result.push(res.ok as u8);

    // push msg
    let msg_bytes = res.msg.as_bytes();
    let msg_len = msg_bytes.len() as u32;
    result.extend_from_slice(&msg_len.to_be_bytes());
    result.extend_from_slice(msg_bytes);

    // push filename
    if let Some(name) = &res.filename {
        let filename_bytes = name.as_bytes();
        let filename_len = filename_bytes.len() as u32;
        result.extend_from_slice(&filename_len.to_be_bytes());
        result.extend_from_slice(filename_bytes);
    }

    // push filebytes
    if let Some(bytes) = &res.filebytes {
        let filebytes_len = bytes.len() as u32;
        result.extend_from_slice(&filebytes_len.to_be_bytes());
        result.extend_from_slice(bytes);
    }
    
    Ok(result)
}

// Vec[ok, len(msg), msg, len(filename), filename, len(filebytes), filebytes] -> Response{ok, msg, filename, filebytes}
pub async fn deserialize_response(data: &Vec<u8>) -> Result<Response, Box<dyn Error>> {
    let mut pos = 0;

    // read ok
    let ok = data[pos] != 0;
    pos += 1;

    // read msg
    let msg_len = u32::from_be_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
    pos += 4;

    let msg = String::from_utf8(data[pos..pos+msg_len].to_vec()).map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8 sequence"))?;
    pos += msg_len;

    // stop if pos has reached end of vec
    // this occurs in response to read requests
    if pos >= data.len() {
        return Ok(Response{ok, msg, filename: None, filebytes: None});
    }

    // read len(filename)
    let filename_len = u32::from_be_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
    pos += 4;
    
    // read filename
    let filename = String::from_utf8(data[pos..pos+filename_len].to_vec()).map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8 sequence"))?;
    pos += filename_len;

    // read number of bytes of file
    let filebytes_len = u32::from_be_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
    pos += 4;
    
    // read file bytes
    let filebytes = data[pos..pos+filebytes_len].to_vec();

    Ok(Response{ok, msg, filename: Some(filename), filebytes: Some(filebytes)})
}

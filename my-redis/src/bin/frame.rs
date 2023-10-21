// allow dead code
#![allow(dead_code)]
use lib::Connection;
fn main() {
    println!("Hello, world!");
}

// enum called Frame with two variants: Array and SimpleString
use bytes::Bytes;

enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}


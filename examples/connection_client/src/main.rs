#![allow(unused)]

use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use std::error::Error;
use ddbb_libs::connection::Connection;
use ddbb_libs::frame::Frame;
use ddbb_libs::data_structure::{CommandEntry, FrameCast, MessageEntry};
use bytes::{ Bytes };

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to a peer
    let mut tcp_stream = TcpStream::connect("127.0.0.1:6142").await?;
    let mut connection = Connection::new(tcp_stream);
    /// Getting command from user.
    /// ...
    /// Command got.
    let command1 = CommandEntry::SetValue {
        key: "tempKey".to_string(),
        value: Bytes::from("tempValue"),
    };

    connection.write_frame(&command1.to_frame()).await;
    let res1 = connection.read_frame().await.unwrap().unwrap();
    match *MessageEntry::from_frame(&res1).unwrap(){
        MessageEntry::Success {msg} => {
            println!("Receive success msg: {}", msg);
        }

        MessageEntry::Error {err_msg} => {
            println!("Receive err_msg: {}", err_msg);

        }
    }
    Ok(())
}
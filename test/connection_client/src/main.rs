#![allow(unused)]
use tokio::time::{sleep, Duration};
use bytes::Bytes;
use ddbb_libs::connection::Connection;
use ddbb_libs::data_structure::{CommandEntry, FrameCast, MessageEntry};
use ddbb_libs::frame::Frame;
use std::error::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to a peer
    let mut tcp_stream ;
    loop {
        if let Ok(stream)= TcpStream::connect("127.0.0.1:6142").await{
            tcp_stream = stream;
            break;
        }
    }
    let mut connection = Connection::new(tcp_stream);
    /// Getting command from user.
    /// ...
    /// Command got.
    let command1 = CommandEntry::SetValue {
        key: "tempKey".to_string(),
        value: Bytes::from("tempValue"),
    };
    loop {
        sleep(Duration::from_millis(2000)).await;
        if let Ok(_) = connection.write_frame(&command1.to_frame()).await{
            println!("sent");
        } else {
            println!("reconn");
            connection.reconnect("127.0.0.1:6142".to_string()).await;
        }
    }
    let res1 = connection.read_frame().await.unwrap().unwrap();
    match *MessageEntry::from_frame(&res1).unwrap() {
        MessageEntry::Success { msg } => {
            println!("Receive success msg: {}", msg);
        }

        MessageEntry::Error { err_msg } => {
            println!("Receive err_msg: {}", err_msg);
        }
    }
    Ok(())
}

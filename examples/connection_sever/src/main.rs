#![allow(unused)]

use tokio::io;
use tokio::net::TcpListener;
use ddbb_libs::connection::Connection;
use ddbb_libs::data_structure::{CommandEntry, FrameCast, MessageEntry};
use ddbb_libs::Error;
use ddbb_libs::frame::Frame;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await.unwrap();

    loop {
        let (mut stream, addr) = listener.accept().await?;
        let mut connection = Connection::new(stream);
        tokio::spawn(async move {
            let req1 = connection.read_frame().await.unwrap().unwrap();
            match *CommandEntry::from_frame(&req1).unwrap() {
                CommandEntry::SetValue { key, value } => {
                    println!("Receive command: {}, {:?}", key, value);
                    let res = MessageEntry::Success { msg: "Operation success".to_string() };
                    connection.write_frame(&res.to_frame()).await.unwrap_or(());
                }

                _ => {}
            }
        });
    }
}
#![allow(unused)]

use tokio::io;
use tokio::net::TcpListener;
use ddbb_libs::connection::Connection;
use ddbb_libs::data_structure::{CommandEntry, DataEntry, FrameCast, MessageEntry};
use ddbb_libs::Error;
use ddbb_libs::frame::Frame;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await.unwrap();

    loop {
        let (mut stream, _) = listener.accept().await?;
        let mut connection = Connection::new(stream);
        tokio::spawn(async move {
            let req1 = connection.read_frame().await.unwrap().unwrap();

            match *CommandEntry::from_frame(&req1).unwrap() {
                CommandEntry::SetValue { key, value } => {
                    println!("Receive command: {}, {:?}", key, value);
                    let res = MessageEntry::Success { msg: "Operation success".to_string() };
                    connection.write_frame(&res.to_frame()).await.unwrap_or(());
                },
                CommandEntry::GetValue { key } => {
                    println!("Receive command: {}", key);
                    let s = String::from("test_get_value");
                    let res = DataEntry::KeyValue { key: key, value: s.into()};
                    connection.write_frame(&res.to_frame()).await.unwrap_or(());
                },
                _ => {}
            }
        });
    }
}
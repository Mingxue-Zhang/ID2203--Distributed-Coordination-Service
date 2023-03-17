#![allow(unused)]

use ddbb_libs::connection::Connection;
<<<<<<< HEAD
use ddbb_libs::data_structure::{CommandEntry, FrameCast, MessageEntry};
=======
use ddbb_libs::data_structure::{CommandEntry, DataEntry, FrameCast, MessageEntry};
use ddbb_libs::Error;
>>>>>>> master
use ddbb_libs::frame::Frame;
use ddbb_libs::Error;
use tokio::io;
use tokio::net::TcpListener;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await.unwrap();

    loop {
        let (mut stream, addr) = listener.accept().await?;
        let mut connection = Connection::new(stream);
        tokio::spawn(async move {
<<<<<<< HEAD
            loop {
                if let Ok(Some(req1)) = connection.read_frame().await {
                    if Connection::got_reconnect_msg(&req1) {
                        println!("Receive reconnect msg: {:?}", req1);
                    } else {
                        match *CommandEntry::from_frame(&req1).unwrap() {
                            CommandEntry::SetValue { key, value } => {
                                println!("Receive command: {}, {:?}", key, value);
                            }

                            _ => {}
                        }
                    }
                } else  {
                    // connection droped
                    println!("##Connection drop");
                    break;
                }
                println!("##receive thread");
                sleep(Duration::from_millis(1000)).await;
=======
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
>>>>>>> master
            }
        });
    }
}

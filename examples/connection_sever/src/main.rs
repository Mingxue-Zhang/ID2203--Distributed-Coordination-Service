#![allow(unused)]

use ddbb_libs::connection::Connection;
use ddbb_libs::data_structure::{CommandEntry, DataEntry, FrameCast, MessageEntry};
use ddbb_libs::Error;
use ddbb_libs::frame::Frame;
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
            }
        });
    }
}

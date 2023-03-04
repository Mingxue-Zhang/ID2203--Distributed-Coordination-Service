use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use std::error::Error;
use ddbb_libs::connection::Connection;
use ddbb_libs::frame::Frame;
use bytes::{ Bytes };

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to a peer
    let mut tcp_stream = TcpStream::connect("127.0.0.1:6142").await?;
    let mut connection = Connection::new(tcp_stream);
    let frame = Frame::Array(vec![
        Frame::Simple("Hello ".to_string()),
        Frame::Bulk(Bytes::from("world!")),
        Frame::Null,
        Frame::Error("This is an Frame::Error, with a Frame:Integer:".to_string()),
        Frame::Integer(150)
    ]);
    connection.write_frame(&frame).await.unwrap_or(());
    let pong = connection.read_frame().await;
    println!("{:?}", pong);
    Ok(())
}
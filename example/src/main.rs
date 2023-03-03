use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use std::error::Error;
use ddbb_libs::connection::Connection;
use ddbb_libs::frame::Frame;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to a peer
    let mut stream = TcpStream::connect("127.0.0.1:6142").await?;
    let mut connection = Connection::new(stream);
    connection.write_frame(&Frame::Simple("hello world".to_string())).await.unwrap_or(());
    // Write some data.
    connection.write_frame(&Frame::Simple("hello world".to_string())).await.unwrap_or(());
    loop {
        connection.write_frame(&Frame::Simple("hello world".to_string())).await.unwrap_or(());

    }
    Ok(())
}
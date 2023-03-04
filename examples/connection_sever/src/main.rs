use tokio::io;
use tokio::net::TcpListener;
use ddbb_libs::connection::Connection;
use ddbb_libs::frame::Frame;

#[tokio::main]
async fn main() -> io::Result<()> {

    let listener = TcpListener::bind("127.0.0.1:6142").await.unwrap();

    loop {
        let (mut stream, _) = listener.accept().await?;
        let mut connection = Connection::new(stream);
        tokio::spawn(async move {
            let f = connection.read_frame().await;
            println!("{:?}", f);
            connection.write_frame(&Frame::Simple("Pong!".to_string())).await.unwrap_or(());
        });
    }
}
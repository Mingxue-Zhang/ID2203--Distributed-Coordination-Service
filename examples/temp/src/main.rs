#![allow(unused)]
use tokio::time::{sleep, Duration};
use tokio::{runtime::Builder, sync::mpsc, time};

#[tokio::main]
async fn main() {
    let handler = tokio::spawn(async {
        tokio::spawn(async {
            async {
                sleep(Duration::from_millis(2000)).await;
    
                println!("temp");
            }
            .await
        })
        .await;
    });
    sleep(Duration::from_millis(1000)).await;
    handler.abort();
    loop {}
}

#![allow(unused)]
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub use ddbb_libs::{Error, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use ddbb_libs::{data_structure::FrameCast, frame::Frame};
use omni_paxos_server::op_data_structure::LogEntry;

mod config;
mod omni_paxos_server;

use omnipaxos_core::messages::ballot_leader_election::BLEMessage;
use omnipaxos_core::messages::sequence_paxos::{PaxosMessage, PaxosMsg};
use omnipaxos_core::util::NodeId;
use serde_json;
use serde::{Serialize, Deserialize};
use tokio::time::{sleep, Duration};

use omni_paxos_server::op_connection::OmniSIMO;

use crate::omni_paxos_server::op_connection::test_send;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Temp {
    key: String
}


#[tokio::main]
async fn main() {
    let mut peers: HashMap<NodeId, String> = HashMap::new();
    peers.insert(2, "127.0.0.1:5660".to_string());

    let mut omni_simo = OmniSIMO::new("127.0.0.1:5661".to_string(), peers);
    let omni_simo = Arc::new(Mutex::new(omni_simo));

    // send message

    // start sender and listener
    let omni_simo_copy1 = omni_simo.clone();
    let omni_simo_copy2 = omni_simo.clone();
    let omni_simo_copy3 = omni_simo.clone();

    tokio::select! {
        e = OmniSIMO::start_incoming_listener(omni_simo_copy1) => {println!("e: {:?}", e);}
        e = OmniSIMO::start_sender(omni_simo_copy2) => {println!("e: {:?}", e);}
        _ = test_send(omni_simo_copy3) => {}
    }

}



// test err in tokio::spawn
async fn async_error() -> Result<()> {
    let handler = tokio::spawn(async move {
        sleep(Duration::from_millis(1000)).await;
        async_err().await.unwrap()
    });
    println!("handler.await.unwrap(): {:?}", handler.await);;
    return Ok(());
}
async fn async_err() -> Result<()> {
    return Err("teest".into());
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_async_error() {
        async_error().await.unwrap();
    }

    #[tokio::test]
    async fn test_sleep() {
        sleep(Duration::from_millis(1000)).await;
        println!("temp");
    }
}

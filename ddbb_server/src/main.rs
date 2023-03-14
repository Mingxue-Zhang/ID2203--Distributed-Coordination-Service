#![allow(unused)]
use std::collections::HashMap;

pub use ddbb_libs::{Error, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use ddbb_libs::{data_structure::FrameCast, frame::Frame};
use omni_paxos_server::op_data_structure::LogEntry;

mod config;
mod omni_paxos_server;

use omnipaxos_core::messages::ballot_leader_election::BLEMessage;
use omnipaxos_core::messages::sequence_paxos::{PaxosMessage, PaxosMsg};
use serde_json;
use serde::{Serialize, Deserialize};
use tokio::time::{sleep, Duration};

use omni_paxos_server::op_connection::OmniSIMO;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Temp {
    key: String
}


#[tokio::main]
async fn main() {
    // let ble = BLEMessage {
    //     from: 1,
    //     to: 2,
    //     msg: omnipaxos_core::messages::ballot_leader_election::HeartbeatMsg::Request(
    //         omnipaxos_core::messages::ballot_leader_election::HeartbeatRequest { round: 1 },
    //     ),
    // };
    // let omni_message: PaxosMessage<Temp, ()> = PaxosMessage {
    //     from: 1,
    //     to: 2,
    //     msg: PaxosMsg::ProposalForward(vec![Temp { key: "tempKey".to_string()}]),
    // };
    // let b = serde_json::to_string(&omni_message).unwrap();
    // let c: PaxosMessage<Temp, ()> = serde_json::from_str(&b[..]).unwrap();
    // println!("{:?}", c);

    
    // println!("OmniSIMO::start_incoming_listener().await: {:?}", OmniSIMO::start_incoming_listener().await);;
    sleep(Duration::from_millis(10000)).await;

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
}

#![allow(unused)]
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use ddbb_libs::{data_structure::FrameCast, frame::Frame};
pub use ddbb_libs::{Error, Result};
use omni_paxos_server::op_data_structure::LogEntry;

mod config;
mod omni_paxos_server;
use omni_paxos_server::OmniMessage;
use omnipaxos_core::messages::ballot_leader_election::BLEMessage;
use omnipaxos_core::messages::sequence_paxos::{PaxosMessage, PaxosMsg};
use omnipaxos_core::util::NodeId;
use serde::{Deserialize, Serialize};
use serde_json;
use tokio::time::{sleep, Duration};

use omni_paxos_server::op_connection::OmniSIMO;

use crate::omni_paxos_server::op_data_structure::Snapshot;

async fn test_send(msg: OmniMessage, simo: Arc<Mutex<OmniSIMO>>) {
    // wait for server starting up
    sleep(Duration::from_millis(1000)).await;
    tokio::spawn(async move {
        {
            let simo = simo.lock().unwrap();
            simo.send_message(&msg);
            simo.send_message(&msg);
            simo.send_message(&msg);
            simo.send_message(&msg);
        }
    })
    .await;
    tokio::spawn(async { loop {} }).await;
}

async fn test_receive(simo: Arc<Mutex<OmniSIMO>>) {
    loop {
        {
            if let Some(msg) = simo
                .lock()
                .unwrap()
                .incoming_buffer
                .lock()
                .unwrap()
                .pop_front()
            {
                println!("receive msg: {:?}", msg);
            }
        }
        sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::main]
async fn main() {
    // let mut peers: HashMap<NodeId, String> = HashMap::new();
    // peers.insert(2, "127.0.0.1:5660".to_string());

    // let mut omni_simo = OmniSIMO::new("127.0.0.1:5661".to_string(), peers);
    // let omni_simo = Arc::new(Mutex::new(omni_simo));

    // // message
    // let paxos_message: PaxosMessage<LogEntry, Snapshot> = PaxosMessage {
    //     from: 1,
    //     to: 2,
    //     msg: PaxosMsg::ProposalForward(vec![LogEntry::SetValue {
    //         key: "testKey".to_string(),
    //         value: Vec::from("tempValue"),
    //     }]),
    // };
    // let msg = OmniMessage::SequencePaxos(paxos_message);

    // // start sender and listener
    // let omni_simo_copy1 = omni_simo.clone();
    // let omni_simo_copy2 = omni_simo.clone();
    // let omni_simo_copy3 = omni_simo.clone();
    // let omni_simo_copy4 = omni_simo.clone();

    // tokio::select! {
    //     e = OmniSIMO::start_incoming_listener(omni_simo_copy1) => {println!("e: {:?}", e);}
    //     e = OmniSIMO::start_sender(omni_simo_copy2) => {println!("e: {:?}", e);}
    //     _ = test_send(msg, omni_simo_copy3) => {println!("89");}
    //     _ = test_receive(omni_simo_copy4) => {println!("90");}
    // }

    let a = async { loop {} };
    let b = async { println!("ss"); };
    tokio::select! {
        biased;
        _ = b => {}
        _ = a => {}

    }
}

#[cfg(test)]
mod test {
    use super::*;
}

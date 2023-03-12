#![allow(unused)]
mod omni_paxos_server;
mod config;

use omnipaxos_core::messages::ballot_leader_election::BLEMessage;
use serde_json::Result;

fn main() {
    let ble = BLEMessage{
        from: 1,
        to: 2,
        msg: omnipaxos_core::messages::ballot_leader_election::HeartbeatMsg::Request(omnipaxos_core::messages::ballot_leader_election::HeartbeatRequest { round: 1 })
    };
    let serialized_ble = serde_json::to_vec(&ble).unwrap_or([0].to_vec());
    let deserialize_ble: BLEMessage = serde_json::from_slice(&serialized_ble[..]).unwrap();
    println!("Hello, world!");
    println!("{:?}", deserialize_ble);

}

#![allow(unused)]
pub use ddbb_libs::{Error, Result};

use omni_paxos_server::op_data_structure::BLEMessageEntry;
use ddbb_libs::{frame::Frame, data_structure::FrameCast};

mod config;
mod omni_paxos_server;

use omnipaxos_core::messages::ballot_leader_election::BLEMessage;
use serde_json;

fn main() {
    let ble = BLEMessage {
        from: 1,
        to: 2,
        msg: omnipaxos_core::messages::ballot_leader_election::HeartbeatMsg::Request(
            omnipaxos_core::messages::ballot_leader_election::HeartbeatRequest { round: 1 },
        ),
    };
    let a = BLEMessageEntry { ble_msg: ble };
    let b = a.to_frame();
    let c = BLEMessageEntry::from_frame(&b).unwrap();
    println!("{:?}", c);
}

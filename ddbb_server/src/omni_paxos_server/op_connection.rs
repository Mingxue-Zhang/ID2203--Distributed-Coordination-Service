use omnipaxos_core::messages::Message as OmniMessage;
use omnipaxos_core::util::NodeId;

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use super::op_data_structure::{BLEMessageEntry, LogEntry};
use ddbb_libs::connection::Connection;

type OmniMessageBuf = Arc<Mutex<VecDeque<OmniMessage<LogEntry, ()>>>>;

/// single incoming and multiple outgoing connection for OmniPaxos instances' communication
pub struct OmniSIMO {
    outgoing_buffer: OmniMessageBuf,
    incoming_buffer: OmniMessageBuf,
    connections: HashMap<NodeId, Connection>,
}

impl OmniSIMO {
    pub fn build(self_addr: &str, peers: HashMap<NodeId, &str>) -> OmniSIMO {
        OmniSIMO{
            outgoing_buffer: Arc::new(Mutex::new(VecDeque::new())),
            incoming_buffer: Arc::new(Mutex::new(VecDeque::new())),
            connections: todo!(),
        };
    }
}
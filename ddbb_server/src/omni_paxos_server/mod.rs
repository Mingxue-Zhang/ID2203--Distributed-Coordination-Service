use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::{runtime::Builder, sync::mpsc, time};

use omnipaxos_core::{
    messages::Message, omni_paxos::*, util::LogEntry as OmniLogEntry, util::NodeId,
};
use omnipaxos_storage::memory_storage::MemoryStorage;

use self::{op_connection::OmniSIMO, op_data_structure::Snapshot};
use crate::config::{ELECTION_TIMEOUT, OUTGOING_MESSAGE_PERIOD};
use op_data_structure::LogEntry;

pub mod op_connection;
pub mod op_data_structure;

type OmniPaxosInstance = OmniPaxos<LogEntry, Snapshot, MemoryStorage<LogEntry, ()>>;
pub(crate) type OmniMessage = Message<LogEntry, Snapshot>;

pub struct OmniPaxosServer {
    pub omni_paxos_instance: Arc<Mutex<OmniPaxosInstance>>,
    pub omni_simo: Arc<Mutex<OmniSIMO>>,
}

impl OmniPaxosServer {
    async fn send_outgoing_msgs(&mut self) {
        let messages:Vec<OmniMessage> = self.omni_paxos_instance.lock().unwrap().outgoing_messages();
        for msg in messages {
            self.omni_simo.lock().unwrap().send_message(&msg);
        }
        
    }

    pub(crate) async fn run(&mut self) {
        let mut outgoing_interval = time::interval(OUTGOING_MESSAGE_PERIOD);
        let mut election_interval = time::interval(ELECTION_TIMEOUT);
        loop {
            tokio::select! {
                biased;

                _ = election_interval.tick() => { self.omni_paxos_instance.lock().unwrap().election_timeout(); },
                _ = outgoing_interval.tick() => { self.send_outgoing_msgs().await; },
                Ok(in_msg) = OmniSIMO::receive_message(self.omni_simo.clone()) => { self.omni_paxos_instance.lock().unwrap().handle_incoming(in_msg); },
                else => { }
            }
        }
    }
}

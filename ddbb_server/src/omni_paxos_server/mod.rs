use omnipaxos_core::{
    messages::Message, omni_paxos::*, util::LogEntry as OmniLogEntry,
    util::NodeId as OmniNodeId,
};
use omnipaxos_storage::memory_storage::MemoryStorage;
use tokio::{runtime::Builder, sync::mpsc};

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::config::{ELECTION_TIMEOUT, OUTGOING_MESSAGE_PERIOD};

use op_data_structure::LogEntry;

use self::op_data_structure::Snapshot;

pub mod op_connection;
pub mod op_data_structure;

type OmniPaxosInstance = OmniPaxos<LogEntry, Snapshot, MemoryStorage<LogEntry, ()>>;
type OmniMessage = Message<LogEntry, Snapshot>;

// pub struct OmniPaxosServer {
//     pub omni_paxos_instance: Arc<Mutex<OmniPaxosInstance>>,
//     pub incoming: mpsc::Receiver<Message<LogEntry, ()>>,
//     pub outgoing: HashMap<NodeId, mpsc::Sender<Message<KeyValue, KVSnapshot>>>,
// }

// impl OmniPaxosServer {
//     async fn send_outgoing_msgs(&mut self) {
//         let messages = self.omni_paxos_instance.lock().unwrap().outgoing_messages();
//         for msg in messages {
//             let receiver = msg.get_receiver();
//             let channel = self
//                 .outgoing
//                 .get_mut(&receiver)
//                 .expect("No channel for receiver");
//             let _ = channel.send(msg).await;
//         }
//     }

//     pub(crate) async fn run(&mut self) {
//         let mut outgoing_interval = time::interval(OUTGOING_MESSAGE_PERIOD);
//         let mut election_interval = time::interval(ELECTION_TIMEOUT);
//         loop {
//             tokio::select! {
//                 biased;

//                 _ = election_interval.tick() => { self.omni_paxos_instance.lock().unwrap().election_timeout(); },
//                 _ = outgoing_interval.tick() => { self.send_outgoing_msgs().await; },
//                 Some(in_msg) = self.incoming.recv() => { self.omni_paxos_instance.lock().unwrap().handle_incoming(in_msg); },
//                 else => { }
//             }
//         }
//     }
// }

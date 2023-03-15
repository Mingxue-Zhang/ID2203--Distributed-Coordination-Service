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

pub type OmniPaxosInstance = OmniPaxos<LogEntry, Snapshot, MemoryStorage<LogEntry, ()>>;
pub type OmniMessage = Message<LogEntry, Snapshot>;

pub struct OmniPaxosServer {
    pub omni_paxos_instance: Arc<Mutex<OmniPaxosInstance>>,
    pub omni_simo: Arc<Mutex<OmniSIMO>>,
}

impl OmniPaxosServer {
    async fn send_outgoing_msgs(&mut self) {
        let messages: Vec<OmniMessage> =
            self.omni_paxos_instance.lock().unwrap().outgoing_messages();
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
                Ok(in_msg) = OmniSIMO::receive_message(self.omni_simo.clone()) => {
                    println!("receive: {:?}", in_msg);
                    self.omni_paxos_instance.lock().unwrap().handle_incoming(in_msg); },
                else => { }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use omnipaxos_core::omni_paxos::OmniPaxosConfig;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_omni_paxos_server() {
        let mut node_ids: [u64; 3] = [1, 2, 3];
        let mut servers: HashMap<NodeId, String> = HashMap::new();
        servers.insert(1, "127.0.0.1:6550".to_string());
        servers.insert(2, "127.0.0.1:6551".to_string());
        servers.insert(3, "127.0.0.1:6552".to_string());

        let mut op_server_handles = HashMap::new();

        for (nodeid, nodeaddr) in servers.clone() {
            let peer_ids: Vec<&u64> = servers.keys().filter(|&&x| x != nodeid).collect();
            let peer_ids: Vec<u64> = peer_ids.iter().copied().map(|x| *x).collect();
            let mut peers: HashMap<NodeId, String> = HashMap::new();
            for peerid in peer_ids.clone() {
                peers.insert(peerid, servers.get(&peerid).unwrap().clone());
            }

            let op_config = OmniPaxosConfig {
                pid: nodeid,
                configuration_id: 1,
                peers: peer_ids,
                ..Default::default()
            };
            let omni: Arc<Mutex<OmniPaxosInstance>> =
                Arc::new(Mutex::new(op_config.build(MemoryStorage::default())));
            let omni_simo = OmniSIMO::new(servers.get(&nodeid).unwrap().to_string(), peers);
            let omni_simo = Arc::new(Mutex::new(omni_simo));

            let omni_simo_copy1 = omni_simo.clone();
            let omni_simo_copy2 = omni_simo.clone();
            tokio::spawn(async move {
                OmniSIMO::start_incoming_listener(omni_simo_copy1).await;
            });
            tokio::spawn(async move {
                // OmniSIMO::start_sender(omni_simo_copy2).await;
            });

            let mut op_server = OmniPaxosServer {
                omni_paxos_instance: omni.clone(),
                omni_simo,
            };
            let join_handle = tokio::spawn({
                async move {
                    op_server.run().await;
                }
            });
            op_server_handles.insert(nodeid, (omni, join_handle));
        }

        sleep(Duration::from_millis(3000)).await;
        let (first_server, _) = op_server_handles.get(&1).unwrap();
        // check which server is the current leader
        let leader = first_server
            .lock()
            .unwrap()
            .get_current_leader()
            .expect("Failed to get leader");
        println!("Elected leader: {}", leader);

        let follower = node_ids.iter().find(|&&p| p != leader).unwrap();
        let (follower_server, _) = op_server_handles.get(follower).unwrap();

        let kv1 = LogEntry::SetValue {
            key: "k1".to_string(),
            value: Vec::from("v1"),
        };

        println!("Adding value: {:?} via server {}", kv1, follower);
    }
}

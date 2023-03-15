#![allow(unused)]
mod config;
mod omni_paxos_server;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// @temp

use omni_paxos_server::op_data_structure;
use tokio::{runtime::Builder, sync::mpsc, time};

use omnipaxos_core::{
    messages::Message, omni_paxos::*, util::LogEntry as OmniLogEntry, util::NodeId,
};
use omnipaxos_storage::memory_storage::MemoryStorage;

use crate::config::{ELECTION_TIMEOUT, OUTGOING_MESSAGE_PERIOD, WAIT_DECIDED_TIMEOUT};
use crate::omni_paxos_server::{OmniPaxosInstance, OmniPaxosServer};
use omni_paxos_server::{op_connection::OmniSIMO, op_data_structure::Snapshot};
use omnipaxos_core::omni_paxos::OmniPaxosConfig;
use op_data_structure::LogEntry;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
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
            OmniSIMO::start_incoming_listener(omni_simo_copy1)
                .await
                .unwrap();
        });
        tokio::spawn(async move {
            OmniSIMO::start_sender(omni_simo_copy2).await.unwrap();
        });

        let mut op_server = OmniPaxosServer {
            omni_paxos_instance: omni.clone(),
            omni_simo,
        };
        let join_handle = tokio::spawn({
            async move {
                sleep(Duration::from_millis(2000)).await;
                op_server.run().await;
            }
        });
        op_server_handles.insert(nodeid, (omni, join_handle));
    }

    sleep(Duration::from_millis(6000)).await;

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
    follower_server
        .lock()
        .unwrap()
        .append(kv1)
        .expect("append failed");
    sleep(Duration::from_millis(1000)).await;

    std::thread::sleep(WAIT_DECIDED_TIMEOUT);
}

#[cfg(test)]
mod test {
    use super::*;
}

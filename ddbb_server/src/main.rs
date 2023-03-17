#![allow(unused)]
pub mod config;
mod ddbb_server;
pub mod omni_paxos_server;
use ddbb_server::DDBB;
use log::{debug, error, info, log_enabled, Level};
use std::collections::HashMap;
use std::env::set_var;
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
    // setup the logger
    set_var("RUST_LOG", "debug");
    env_logger::init();
    // error!("this is printed by default");
    // info!("info temp");

    let mut node_ids: [u64; 3] = [1, 2, 3];
    let mut servers: HashMap<NodeId, String> = HashMap::new();
    servers.insert(1, "127.0.0.1:6550".to_string());
    servers.insert(2, "127.0.0.1:6551".to_string());
    servers.insert(3, "127.0.0.1:6552".to_string());

    let mut ddbbs: Vec<Arc<Mutex<DDBB>>> = Vec::new();
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
        let omni: OmniPaxosInstance = op_config.build(MemoryStorage::default());
        // !! peer.clone
        let simo = OmniSIMO::new(nodeaddr.to_string(), peers.clone());
        let mut ddbb = DDBB::new(nodeid, nodeaddr.clone(), peers, simo, omni);
        let ddbb = Arc::new(Mutex::new(ddbb));
        
        let ddbb_copy = ddbb.clone();
        let omni_server_handler = tokio::spawn(async move {
            DDBB::start(ddbb_copy).await.unwrap();
        });

        ddbbs.insert(ddbbs.len(), ddbb);
    }

    sleep(Duration::from_millis(1000)).await;

    ddbbs.get(0).unwrap().lock().unwrap().set("key1".to_string(), Vec::from([1])).unwrap();
    ddbbs.get(1).unwrap().lock().unwrap().set("key2".to_string(), Vec::from([2])).unwrap();
    ddbbs.get(2).unwrap().lock().unwrap().set("key3".to_string(), Vec::from([3])).unwrap();
    sleep(Duration::from_millis(1000)).await;
    let ddbb2 = ddbbs.get(2).unwrap().clone();
    ddbb2.lock().unwrap().show_wal_store();
    info!("read key == key4 from ddbb2: {:?}", ddbb2.lock().unwrap().get("key3".to_string()));


}

// #[tokio::main]
// async fn main() {
//     // setup the logger
//     set_var("RUST_LOG", "debug");
//     env_logger::init();
//     // error!("this is printed by default");
//     // info!("info temp");

//     let mut node_ids: [u64; 5] = [1, 2, 3, 4, 5];
//     let mut servers: HashMap<NodeId, String> = HashMap::new();
//     servers.insert(1, "127.0.0.1:6550".to_string());
//     servers.insert(2, "127.0.0.1:6551".to_string());
//     servers.insert(3, "127.0.0.1:6552".to_string());
//     servers.insert(4, "127.0.0.1:6553".to_string());
//     servers.insert(5, "127.0.0.1:6554".to_string());

//     let mut op_server_handles = HashMap::new();

//     for (nodeid, nodeaddr) in servers.clone() {
//         let peer_ids: Vec<&u64> = servers.keys().filter(|&&x| x != nodeid).collect();
//         let peer_ids: Vec<u64> = peer_ids.iter().copied().map(|x| *x).collect();
//         let mut peers: HashMap<NodeId, String> = HashMap::new();
//         for peerid in peer_ids.clone() {
//             peers.insert(peerid, servers.get(&peerid).unwrap().clone());
//         }

//         let op_config = OmniPaxosConfig {
//             pid: nodeid,
//             configuration_id: 1,
//             peers: peer_ids,
//             ..Default::default()
//         };
//         let omni: Arc<Mutex<OmniPaxosInstance>> =
//             Arc::new(Mutex::new(op_config.build(MemoryStorage::default())));
//         let omni_simo = OmniSIMO::new(servers.get(&nodeid).unwrap().to_string(), peers);
//         let omni_simo = Arc::new(Mutex::new(omni_simo));

//         let omni_simo_copy1 = omni_simo.clone();
//         let omni_simo_copy2 = omni_simo.clone();
//         let omni_simo_copy3 = omni_simo.clone();

//         let mut op_server = OmniPaxosServer {
//             omni_paxos_instance: omni.clone(),
//             omni_simo,
//         };
//         let join_handle = tokio::spawn({
//             async move {
//                 OmniSIMO::start_incoming_listener(omni_simo_copy1)
//                     .await
//                     .unwrap();
//                 OmniSIMO::start_sender(omni_simo_copy2).await.unwrap();
//                 op_server.run().await;
//             }
//         });
//         op_server_handles.insert(nodeid, (omni, join_handle, omni_simo_copy3));
//     }

//     sleep(Duration::from_millis(4000)).await;

//     let (first_server, _, _) = op_server_handles.get(&1).unwrap();
//     // check which server is the current leader
//     let leader = first_server
//         .lock()
//         .unwrap()
//         .get_current_leader()
//         .expect("Failed to get leader");
//     info!("Elected leader: {}", leader);

//     let follower = node_ids.iter().find(|&&p| p != leader).unwrap();
//     let (follower_server, _, _) = op_server_handles.get(follower).unwrap();

//     let kv1 = LogEntry::SetValue {
//         key: "k1".to_string(),
//         value: Vec::from("v1"),
//     };
//     info!("Adding value: {:?} via server {}", kv1, follower);
//     follower_server
//         .lock()
//         .unwrap()
//         .append(kv1)
//         .expect("append failed");

//     let kv2 = LogEntry::SetValue {
//         key: "k2".to_string(),
//         value: Vec::from("v2"),
//     };
//     info!("Adding value: {:?} via server {}", kv2, leader);
//     let (leader_server, leader_join_handle, leader_simo) = op_server_handles.get(&leader).unwrap();
//     leader_server
//         .lock()
//         .unwrap()
//         .append(kv2)
//         .expect("append failed");

//     // wait for the entries to be decided...
//     sleep(WAIT_DECIDED_TIMEOUT).await;

//     let committed_ents = leader_server
//         .lock()
//         .unwrap()
//         .read_decided_suffix(0)
//         .expect("Failed to read expected entries");
//     let mut simple_store = Vec::new();
//     for ent in committed_ents {
//         match ent {
//             OmniLogEntry::Decided(kv) => {
//                 simple_store.insert(simple_store.len(), kv);
//             }
//             _ => {} // ignore not committed entries
//         }
//     }
//     info!("KV store: {:?}", simple_store);

//     info!("Killing leader: {}...", leader);
//     let dropped_leader_omni = leader_server;

//     {
//         info!("conected 149: {:?}", leader_simo.lock().unwrap().connected);
//         leader_simo
//             .lock()
//             .unwrap()
//             .connected
//             .lock()
//             .unwrap()
//             .retain(|&x| x == leader);
//         info!("conected 149: {:?}", leader_simo.lock().unwrap().connected);
//     }
//     let droped_leader = leader;

//     // leader_join_handle.abort();

//     // wait for new leader to be elected...
//     sleep(Duration::from_millis(1000)).await;
//     let leader = follower_server
//         .lock()
//         .unwrap()
//         .get_current_leader()
//         .expect("Failed to get leader");
//     info!("Elected new leader: {}", leader);

//     let kv3 = LogEntry::SetValue {
//         key: "k3".to_string(),
//         value: Vec::from("v3"),
//     };
//     info!("Adding value: {:?} via server {}", kv3, leader);
//     let (leader_server, _, _) = op_server_handles.get(&leader).unwrap();
//     leader_server
//         .lock()
//         .unwrap()
//         .append(kv3)
//         .expect("append failed");
//     // wait for the entries to be decided...
//     std::thread::sleep(WAIT_DECIDED_TIMEOUT);

//     let committed_ents = leader_server
//         .lock()
//         .unwrap()
//         .read_decided_suffix(2)
//         .expect("Failed to read expected entries");
//     for ent in committed_ents {
//         match ent {
//             OmniLogEntry::Decided(kv) => {
//                 simple_store.insert(simple_store.len(), kv);
//             }
//             _ => {} // ignore not committed entries
//         }
//     }

//     info!("Restart dropped leader: {}...", droped_leader);
//     let mut peers1: Vec<NodeId> = node_ids
//         .iter()
//         .filter(|&&p| p != droped_leader)
//         .copied()
//         .collect();
//     leader_simo
//         .lock()
//         .unwrap()
//         .connected
//         .lock()
//         .unwrap()
//         .append(&mut peers1.clone());
//     for nodeid in peers1 {
//         let (peer_omni, _, _) = op_server_handles.get(&nodeid).unwrap();
//         dropped_leader_omni.lock().unwrap().reconnected(nodeid);
//         peer_omni.lock().unwrap().reconnected(droped_leader);
//         info!("Send reconnected dropped leader to: {}...", nodeid);
//     }

//     sleep(Duration::from_millis(3000)).await;

//     // append from leader
//     let kv4 = LogEntry::SetValue {
//         key: "k4".to_string(),
//         value: Vec::from("v4"),
//     };
//     info!("Adding value: {:?} via server {}", kv4, leader);
//     leader_server
//         .lock()
//         .unwrap()
//         .append(kv4)
//         .expect("append failed");
//     std::thread::sleep(WAIT_DECIDED_TIMEOUT);

//     // retrieve from restarted node
//     let committed_ents = dropped_leader_omni
//         .lock()
//         .unwrap()
//         .read_decided_suffix(0)
//         .expect("Failed to read expected entries");
//     let mut simple_store = Vec::new();
//     for ent in committed_ents {
//         match ent {
//             OmniLogEntry::Decided(kv) => {
//                 simple_store.insert(simple_store.len(), kv);
//             }
//             _ => {} // ignore not committed entries
//         }
//     }
//     info!("Commited: {:?}", simple_store);
// }

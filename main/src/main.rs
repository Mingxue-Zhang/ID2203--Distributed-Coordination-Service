#![allow(unused)]
use log::{debug, error, info, log_enabled, Level};
use omnipaxos_core::{
    messages::Message, omni_paxos::OmniPaxosConfig, omni_paxos::*, util::LogEntry as OmniLogEntry,
    util::NodeId,
};
use tokio::time::{sleep, Duration};
use tokio::{runtime::Builder, sync::mpsc, time};

use std::collections::HashMap;
use std::env::set_var;
use std::sync::{Arc, Mutex};

use ddbb_server::config::{ELECTION_TIMEOUT, OUTGOING_MESSAGE_PERIOD, WAIT_DECIDED_TIMEOUT};
use ddbb_server::ddbb_server::DDBB;
use ddbb_server::omni_paxos_server::{
    op_connection::OmniSIMO, op_data_structure::LogEntry, op_data_structure::Snapshot,
    OmniPaxosInstance, OmniPaxosServer,
};
use omnipaxos_storage::memory_storage::MemoryStorage;

#[tokio::main]
async fn main() {
    // setup the logger
    set_var("RUST_LOG", "debug");
    env_logger::init();
    // error!("this is printed by default");
    // info!("info temp");

    // initialize
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

    let ddbb1 = ddbbs.get(0).unwrap();
    // user cmd
    let sign = format!(">>");
    use std::io::{Write};

    loop {
        //Get the input command
        print!("{}", sign);
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(err) => println!("Could not parse input: {}", err)
        }
        let input_vector:Vec<&str> = input.trim().split(" ").collect();

        if input_vector[0] == "read" {
            if input_vector.len() == 2 {
                let res = DDBB::lin_read(ddbb1.clone(), input_vector[1].to_string()).await;
                match res {
                    Ok(value)=>{
                        println!("{:?}", value.unwrap())
                    },
                    Err(e) =>{
                        unwrap_failed("called `Result::unwrap()` on an `Err` value", &e)
                    }
                }

            } else {
                println!(" -> ERROR: Incorrect  command");
            }


        }
        else if input_vector[0] == "write" {
            if input_vector.len() == 3 {
                let res = DDBB::lin_write(ddbb1.clone(), input_vector[1].to_string(), input_vector[2].as_bytes().to_vec()).await;
                match res {
                    Ok(value)=>{
                        println!("Succesfully wrote.")
                    },
                    Err(e) =>{
                        unwrap_failed("called `Result::unwrap()` on an `Err` value", &e)
                    }
                }
            } else {
                println!(" -> ERROR: Incorrect command");
            }

        }
        else{
            //If it is not a put or a get
            println!(" -> ERROR: Unknown command");
        }
    }
    //test
    // let ddbb1 = ddbbs.get(0).unwrap();
    // let ddbb2 = ddbbs.get(1).unwrap();
    // let ddbb3 = ddbbs.get(2).unwrap();
    //
    // ddbb1
    //     .lock()
    //     .unwrap()
    //     .set("key1".to_string(), Vec::from([1]))
    //     .unwrap();
    // ddbb1
    //     .lock()
    //     .unwrap()
    //     .set("key4".to_string(), Vec::from([4]))
    //     .unwrap();
    // ddbb2
    //     .lock()
    //     .unwrap()
    //     .set("key2".to_string(), Vec::from([2]))
    //     .unwrap();
    // ddbb1
    //     .lock()
    //     .unwrap()
    //     .set("key2".to_string(), Vec::from([2, 1]))
    //     .unwrap();
    // ddbb3
    //     .lock()
    //     .unwrap()
    //     .set("key1".to_string(), Vec::from([1, 1]))
    //     .unwrap();
    // ddbb1
    //     .lock()
    //     .unwrap()
    //     .set("key3".to_string(), Vec::from([3]))
    //     .unwrap();
    //
    // DDBB::lin_write(ddbb1.clone(), "key3".to_string(), Vec::from([3, 1])).await;
    // DDBB::lin_write(ddbb3.clone(), "key2".to_string(), Vec::from([2, 2])).await;
    // DDBB::lin_write(ddbb1.clone(), "key1".to_string(), Vec::from([1, 2])).await;
    // DDBB::lin_read(ddbb1.clone(), "key3".to_string()).await;
    // ddbb2.lock().unwrap().get("key1".to_string());
    // DDBB::lin_read(ddbb1.clone(), "key3".to_string()).await;
    // DDBB::lin_read(ddbb2.clone(), "key2".to_string()).await;
    // ddbb1.lock().unwrap().compact();
    // DDBB::lin_read(ddbb1.clone(), "key3".to_string()).await;
    // DDBB::lin_read(ddbb2.clone(), "key2".to_string()).await;
    // DDBB::lin_read(ddbb3.clone(), "key1".to_string()).await;
    //
    // sleep(Duration::from_millis(1000)).await;
    // ddbb1.lock().unwrap().show_wal_store();
    // ddbb2.lock().unwrap().show_wal_store();
    //
    // DDBB::lin_read(ddbb1.clone(), "key3".to_string()).await;
    // DDBB::lin_read(ddbb2.clone(), "key2".to_string()).await;
    // ddbb1.lock().unwrap().compact();
    // DDBB::lin_read(ddbb1.clone(), "key3".to_string()).await;
    // DDBB::lin_read(ddbb2.clone(), "key2".to_string()).await;
    // DDBB::lin_read(ddbb3.clone(), "key1".to_string()).await;
    //
    // sleep(Duration::from_millis(1000)).await;
    // ddbb1.lock().unwrap().show_wal_store();
    // ddbb2.lock().unwrap().show_wal_store();
    //
    // ddbb1.lock().unwrap().compact();
    // sleep(Duration::from_millis(1000)).await;
    // ddbb1.lock().unwrap().show_wal_store();
    //
    // ddbb1.lock().unwrap().compact();
    // sleep(Duration::from_millis(1000)).await;
    // ddbb1.lock().unwrap().show_wal_store();
}

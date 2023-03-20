#![allow(unused)]
use log::{debug, error, info, log_enabled, Level};
use omnipaxos_core::{
    messages::Message, omni_paxos::OmniPaxosConfig, omni_paxos::*, util::LogEntry as OmniLogEntry,
    util::NodeId,
};
use tokio::time::{sleep, Duration};
use tokio::{runtime::Builder, sync::mpsc, time};
use std::error::Error;
use std::collections::HashMap;
use std::env::set_var;
use std::string;
use std::sync::{Arc, Mutex};

use ddbb_server::config::{ELECTION_TIMEOUT, OUTGOING_MESSAGE_PERIOD, WAIT_DECIDED_TIMEOUT};
use ddbb_server::ddbb_server::DDBB;
use ddbb_server::omni_paxos_server::{
    op_connection::OmniSIMO, op_data_structure::LogEntry, op_data_structure::Snapshot,
    OmniPaxosInstance, OmniPaxosServer,
};
//StructOpt - used for getting input from the command line
use structopt::StructOpt;
//Serde - used for serializing (turning into bytes) and deserializing messages
use serde::{Serialize, Deserialize};
use omnipaxos_storage::memory_storage::MemoryStorage;
#[derive(Debug, Serialize, Deserialize, StructOpt)]
struct Node {
    #[structopt(long)]
    pid: u64,
    #[structopt(long)]
    ip_addr: String,
    #[structopt(long)]
    peer_ids: Vec<u64>,
    #[structopt(long)]
    peers_addrs: Vec<String>
}
#[tokio::main]
async fn main() {
    // setup the logger
    set_var("RUST_LOG", "debug");
    env_logger::init();
    // error!("this is printed by default");
    // info!("info temp");

    // initialize
    let node = Node::from_args();
    // let mut node_ids: Vec<u64> = vec![1, 2, 3];
    let node_id:u64 = node.pid;
    let node_addr:String = node.ip_addr;
    let peer_ids = node.peer_ids;
    let peers_addrs = node.peers_addrs;

    // let mut servers: HashMap<NodeId, String> = HashMap::new();
    // servers.insert(node_id, node_addr);
    // println!("Initializing node {} with peers {:?}", node_id, node_addr);
    // servers.insert(2, "127.0.0.1:6551".to_string());
    // servers.insert(3, "127.0.0.1:6552".to_string());

    let mut ddbbs: Vec<Arc<Mutex<DDBB>>> = Vec::new();
    // add_to_cluster(ddbbs.clone(), servers.clone(),node_ids.clone());
    // for (nodeid, nodeaddr) in servers {
        // let peer_ids: Vec<&u64> = servers.keys().filter(|&&x| x != nodeid).collect();
        // let peer_ids: Vec<u64> = peer_ids.iter().copied().map(|x| *x).collect();
        let mut peers: HashMap<NodeId, String> = HashMap::new();
        for i in 0..peer_ids.len() {
            peers.insert(peer_ids[i], &peers_addrs[i]);
        }

        let op_config = OmniPaxosConfig {
            pid: node_id,
            configuration_id: 1,
            peers: peer_ids,
            ..Default::default()
        };
        let omni: OmniPaxosInstance = op_config.build(MemoryStorage::default());
        // !! peer.clone
        let simo = OmniSIMO::new(node_addr.to_string(), peers.clone());
        let mut ddbb = DDBB::new(node_id, node_addr.clone(), peers, simo, omni);
        let ddbb = Arc::new(Mutex::new(ddbb));

        let ddbb_copy = ddbb.clone();
        let omni_server_handler = tokio::spawn(async move {
            DDBB::start(ddbb_copy).await.unwrap();
        });

        ddbbs.insert(ddbbs.len(), ddbb);
    // }
    

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
                        println!("Error occurred!")
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
                        println!("Error occurred!")
                    }
                }
            } else {
                println!(" -> ERROR: Incorrect command");
            }

        }
        // else if input_vector[0] == "add-node"{
        //     if input_vector.len() == 3 {
        //         let correct_v = input_vector[2].to_string();
        //         let new_node_id:u64= input_vector[1].to_string().parse().unwrap();
        //         // node_ids.push(new_node_id);
        //         // servers.insert(new_node_id, correct_v);
        //         // let res= add_to_cluster(ddbbs.clone(), servers.clone() ,node_ids.clone()).unwrap();
        //         let res = servers.insert(new_node_id, correct_v).unwrap();

        //         match res {
        //             correct_v=>{
        //                 println!("Succesfully added.")
        //             },
        //             _ =>{
        //                 println!("Error occurred!")
        //             }
        //         }
        //     } else {
        //         println!(" -> ERROR: Incorrect command");
        //     }
        // }
        // else if input_vector[0] == "show"{
        //     if input_vector.len() == 1 {
        //         println!("Configuration:");
        //         println!("id\t|\taddress");
        //         println!("{:?}\t|\t{:?}", node_id, node_addr);
        //         for i in 0..peer_ids.len() {
        //             println!("{:?}\t|\t{:?}", peer_ids[i], peers_addrs[i].clone());
        //             // peers.(peer_ids[i], &peers_addrs[i]);
        //         }
        //     } else {
        //         println!(" -> ERROR: Incorrect command");
        //     }
        // }
        else{
            //If it is not a put or a get
            println!(" -> ERROR: Unknown command");
        }
    }
    
}
// fn add_to_cluster(mut ddbbs:Vec<Arc<Mutex<DDBB>>>, mut servers: HashMap<NodeId, String>, mut node_ids:Vec<u64>) -> Result<(), Box<dyn Error>>{
    
// }
#![allow(unused)]
use log::info;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use std::collections::HashMap;
use std::env::set_var;
use std::process::Command;
use std::time::Instant;

use ddbb_server::ddbb_server::DDBB;
use ddbb_server::omni_paxos_server::op_data_structure::LogEntry;

use crate::configs::{LOG_CUNCURRENT_NUM, NODES_NUM_OF_CLUSTER};
use crate::libs::{generate_cluster, output_trace, run_ddbb, TestCommandEntry};
use libs::{generate_commands, LogEntryWithTime};

pub mod configs;
mod libs;

#[tokio::main]
async fn main() {
    set_var("RUST_LOG", "debug");
    env_logger::init();

    let t = Instant::now();

    let servers = generate_cluster(NODES_NUM_OF_CLUSTER);
    let mut commands = generate_commands(NODES_NUM_OF_CLUSTER * LOG_CUNCURRENT_NUM);

    let mut handlers: Vec<JoinHandle<Vec<LogEntryWithTime>>> = Vec::new();
    for i in 0..NODES_NUM_OF_CLUSTER {
        let mut commands_local: Vec<TestCommandEntry> = Vec::new();
        for command in &commands[(i * LOG_CUNCURRENT_NUM).try_into().unwrap()
            ..(LOG_CUNCURRENT_NUM * (i + 1)).try_into().unwrap()]
        {
            commands_local.insert(commands_local.len(), command.clone());
        }
        let servers = servers.clone();

        let handler = tokio::spawn(async move { run_ddbb(i + 1, servers, commands_local).await });
        handlers.insert(i.try_into().unwrap(), handler);
    }

    let mut global_trace: Vec<LogEntryWithTime> = Vec::new();
    for handler in handlers {
        let mut local_trace = handler.await.unwrap();
        global_trace.append(&mut local_trace);
    }

    let mut local_trace: HashMap<String, Vec<LogEntryWithTime>> = HashMap::new();
    for event in global_trace {
        let mut key_local: String;
        match event.clone().log {
            TestCommandEntry::LINRead { key, value } => key_local = key,
            TestCommandEntry::LINWrite { key, value } => key_local = key,
        }
        if let Some(_) = local_trace.get(&key_local) {
            local_trace.get_mut(&key_local).unwrap().insert(0, event);
        } else {
            local_trace.insert(key_local, Vec::from([event]));
        }
    }

    for (key_local, trace) in local_trace.clone() {
        output_trace(key_local.clone(), t, &trace);
        info!("Linearizability check of key == {:?}", key_local);
        let path = &format!("kv_checker/test/trace_{:}.txt", key_local);
        let mut checker = Command::new("kv_checker/kv_checker")
            .arg(path)
            .spawn()
            .unwrap();
        checker.wait().unwrap();
    }

    // output_trace("global".to_string(), t, &global_trace);

    let mut checker = Command::new("kv_checker/kv_checker").arg("kv_checker/test/trace_2.txt").spawn().unwrap();
    checker.wait().unwrap();

    // let command1 = &mut commands[0..LOG_CUNCURRENT_NUM.try_into().unwrap()];
    // for command in command1 {
    //     info!("commands: {:?}", command);
    // }
}

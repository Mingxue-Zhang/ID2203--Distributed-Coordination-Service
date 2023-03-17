use log::{debug, info};
use omnipaxos_core::{omni_paxos::OmniPaxos, util::LogEntry as OmniLogEntry, util::NodeId};
use tokio::time::{sleep, Duration};

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ddbb_libs::{Error, Result};

use crate::config::LOG_RETRIEVE_INTERVAL;
use crate::omni_paxos_server::{op_connection::OmniSIMO, OmniPaxosInstance, OmniPaxosServer};
use crate::op_data_structure::LogEntry;

pub struct DDBB {
    node_info: NodeInfo,
    wal_store: WALStore,
    peers: Arc<Mutex<HashMap<NodeId, String>>>,
    simo: Arc<Mutex<OmniSIMO>>,
    omni: Arc<Mutex<OmniPaxosInstance>>,
}

#[derive(Debug)]
struct NodeInfo {
    id: NodeId,
    addr: String,
}

#[derive(Debug)]
struct WALStore {
    len: u64,
    store: Vec<LogEntry>,
}

impl WALStore {
    pub fn new() -> Self {
        Self {
            store: Vec::new(),
            len: 0,
        }
    }

    pub fn append(&mut self, log: LogEntry) {
        // append to head
        self.store.insert(0, log);
    }

    pub fn len(&self) -> u64 {
        self.store.len().try_into().unwrap()
    }
}

impl DDBB {
    pub fn new(
        id: NodeId,
        self_addr: String,
        peers: HashMap<NodeId, String>,
        simo: OmniSIMO,
        omni: OmniPaxosInstance,
    ) -> Self {
        let mut peers = Arc::new(Mutex::new(peers));
        let mut simo = Arc::new(Mutex::new(simo));
        let mut omni = Arc::new(Mutex::new(omni));
        DDBB {
            node_info: NodeInfo {
                id,
                addr: self_addr,
            },
            peers,
            simo,
            omni,
            wal_store: WALStore::new(),
        }
    }

    pub async fn start(ddbb: Arc<Mutex<DDBB>>) -> Result<()> {
        let mut simo: Arc<Mutex<OmniSIMO>>;
        let mut op_server: OmniPaxosServer;
        {
            simo = ddbb.lock().unwrap().simo.clone();
            let omni = ddbb.lock().unwrap().omni.clone();
            op_server = OmniPaxosServer {
                omni_paxos_instance: omni.clone(),
                omni_simo: simo.clone(),
            };

            // start log retrieval
            tokio::spawn(async move {
                loop {
                    ddbb.lock().unwrap().retrieve_logs_from_omni();
                    sleep(Duration::from_millis(LOG_RETRIEVE_INTERVAL)).await;
                }
            });
        }

        Self::start_simo(simo).await?;
        op_server.run().await;
        return Ok(());
    }

    async fn start_simo(simo: Arc<Mutex<OmniSIMO>>) -> Result<()> {
        let omni_simo_copy1 = simo.clone();
        let omni_simo_copy2 = simo.clone();
        OmniSIMO::start_incoming_listener(omni_simo_copy1).await?;
        OmniSIMO::start_sender(omni_simo_copy2).await?;
        return Ok(());
    }

    pub fn set(&self, key: String, value: Vec<u8>) -> Result<()> {
        let log = LogEntry::SetValue { key, value };
        self.put_log_into_omni(log)
    }

    pub fn get(&self, key: String) -> Option<Vec<u8>> {
        let key_in = key;
        for log in self.wal_store.store.iter(){
            if let LogEntry::SetValue { key, value } = log{
                if key_in.eq(key){
                    return Some(value.clone());
                }
            }
        }
        return None;
    }

    // temp
    pub fn show_wal_store (&self){
        info!("Wal of {:?}:", self.node_info.id);
        for log in self.wal_store.store.iter(){
            info!("\t{:?}", log);
        }
    }

    fn retrieve_logs_from_omni(&mut self) {
        let committed_ents = self
            .omni
            .lock()
            .unwrap()
            .read_decided_suffix(self.wal_store.len());
        if let Some(entrys) = committed_ents {
            for entry in entrys {
                match entry {
                    OmniLogEntry::Decided(log) => {
                        self.wal_store.append(log);
                    }
                    _ => {}
                }
            }
        }
    }

    fn put_log_into_omni(&self, log: LogEntry) -> Result<()> {
        let result = self.omni.lock().unwrap().append(log);
        if let Ok(()) = result {
            return Ok(());
        } else {
            return Err("append faild".into());
        }
    }
}

mod test {
    use super::*;
}

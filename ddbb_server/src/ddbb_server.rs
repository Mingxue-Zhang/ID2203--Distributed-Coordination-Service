use log::{debug, info};
use omnipaxos_core::{omni_paxos::OmniPaxos, util::LogEntry as OmniLogEntry, util::NodeId};
use serde_json::Map;
use tokio::{
    runtime::Handle,
    time::{sleep, Duration},
};

use std::{
    clone,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::config::{LIN_WRITE_TIMES_OUT, LOG_RETRIEVE_INTERVAL, WAIT_DECIDED_TIMEOUT};
use crate::omni_paxos_server::{op_connection::OmniSIMO, OmniPaxosInstance, OmniPaxosServer};
use crate::op_data_structure::LogEntry;
use ddbb_libs::{Error, Result};

pub struct DDBB {
    node_info: NodeInfo,
    wal_store: Arc<Mutex<WALStore>>,
    kv_store: KVStore,
    peers: Arc<Mutex<HashMap<NodeId, String>>>,
    simo: Arc<Mutex<OmniSIMO>>,
    omni: Arc<Mutex<OmniPaxosInstance>>,
    timestamp: u64,
}

#[derive(Debug)]
struct NodeInfo {
    id: NodeId,
    addr: String,
}

#[derive(Debug)]
struct WALStore {
    idx: u64,
    store: Vec<LogEntry>,
}

impl WALStore {
    pub fn new() -> Self {
        Self {
            store: Vec::new(),
            idx: 0,
        }
    }

    pub fn append(&mut self, log: LogEntry) {
        // append to head
        self.store.insert(0, log);
    }

    pub fn diceded(&self) -> u64 {
        self.idx
    }
}

#[derive(Debug)]
struct KVStore {
    store: HashMap<String, Vec<u8>>,
}

impl KVStore {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: String, value: Vec<u8>) {
        self.store.insert(key, value);
    }

    pub fn get(&self, key: String) -> Option<&Vec<u8>> {
        self.store.get(&key)
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
            wal_store: Arc::new(Mutex::new(WALStore::new())) ,
            kv_store: KVStore::new(),
            timestamp: 0,
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

    pub fn add_ts(&mut self) {
        self.timestamp += 1;
    }

    fn find_log_by_opid(&self, addr: String, ts: u64) -> Option<LogEntry> {
        let mut opid_temp: (String, u64);
        let mut ts_temp: u64;
        for log in self.wal_store.lock().unwrap().store.iter() {
            match log.clone() {
                LogEntry::LINRead { opid, key, value } => opid_temp = opid,
                LogEntry::LINWrite { opid, key, value } => opid_temp = opid,
                _ => break,
            };
            if opid_temp.0.eq(&addr) && opid_temp.1 == ts {
                return Some(log.clone());
            }
        }
        return None;
    }

    pub fn set(&mut self, key: String, value: Vec<u8>) -> Result<()> {
        self.kv_store.store.insert(key.clone(), value.clone());
        let log = LogEntry::SetValue { key, value };
        self.put_log_into_omni(log)
    }

    pub fn get(&self, key: String) -> Option<Vec<u8>> {
        if let Some(value) = self.kv_store.get(key) {
            return Some(value.clone());
        } else {
            return None;
        }
    }

    pub async fn lin_write(ddbb: Arc<Mutex<DDBB>>, key: String, value: Vec<u8>) -> Result<()> {
        let ts: u64;
        let self_addr: String;
        {
            let mut ddbb = ddbb.lock().unwrap();
            ddbb.add_ts();
            ts = ddbb.timestamp;
            self_addr = ddbb.node_info.addr.clone()
        }

        let log = LogEntry::LINWrite {
            opid: (self_addr.clone(), ts),
            key,
            value,
        };
        ddbb.lock().unwrap().put_log_into_omni(log.clone());
        sleep(WAIT_DECIDED_TIMEOUT).await;
        let mut times: u64 = 0;
        loop {
            if let Some(_) = ddbb.lock().unwrap().find_log_by_opid(self_addr.clone(), ts) {
                // debug!("tried times: {:?}", times);
                return Ok(());
            };
            times += 1;
            if times >= LIN_WRITE_TIMES_OUT {
                return Err("Lin write failed".into());
            }

            sleep(Duration::from_millis(LOG_RETRIEVE_INTERVAL)).await;
        }
    }

    pub async fn lin_read(ddbb: Arc<Mutex<DDBB>>, key: String) -> Result<Option<Vec<u8>>> {
        let ts: u64;
        let self_addr: String;
        {
            let mut ddbb = ddbb.lock().unwrap();
            ddbb.add_ts();
            ts = ddbb.timestamp;
            self_addr = ddbb.node_info.addr.clone()
        }

        let log = LogEntry::LINRead {
            opid: (self_addr.clone(), ts),
            key,
            value: None,
        };
        ddbb.lock().unwrap().put_log_into_omni(log.clone());
        sleep(WAIT_DECIDED_TIMEOUT).await;
        let mut times: u64 = 0;
        loop {
            {
                let ddbb = ddbb.lock().unwrap();
                if let Some(log) = ddbb.find_log_by_opid(self_addr.clone(), ts) {
                    // debug!("tried times: {:?}", times);
                    if let LogEntry::LINRead { opid, key, value } = log {
                        return Ok(value);
                    }
                };
            }
            times += 1;
            if times >= LIN_WRITE_TIMES_OUT {
                return Err("Lin read failed".into());
            }

            sleep(Duration::from_millis(LOG_RETRIEVE_INTERVAL)).await;
        }
    }

    // temp: for debug
    pub fn show_wal_store(&self) {
        info!("Wal of {:?}:", self.node_info.id);
        for log in self.wal_store.lock().unwrap().store.iter() {
            info!("\t{:?}", log);
        }
        info!("\tkv store: {:?}", self.kv_store);
    }

    fn retrieve_logs_from_omni(&mut self) {
        let committed_ents = self
            .omni
            .lock()
            .unwrap()
            .read_decided_suffix(self.wal_store.lock().unwrap().diceded());
        if let Some(entrys) = committed_ents {
            self.wal_store.lock().unwrap().idx += 1;
            for entry in entrys {
                match entry {
                    OmniLogEntry::Decided(log) => match log.clone() {
                        LogEntry::SetValue { key, value } => {
                            self.wal_store.lock().unwrap().append(log.clone());
                            self.kv_store.store.insert(key.clone(), value.clone());
                        }
                        LogEntry::LINRead { key, opid, value } => {
                            let value = self.get(key.clone());
                            self.wal_store.lock().unwrap()
                                .append(LogEntry::LINRead { opid, key, value });
                        }
                        LogEntry::LINWrite { opid, key, value } => {
                            self.kv_store.store.insert(key, value);
                            self.wal_store.lock().unwrap().append(log.clone());
                        }
                        LogEntry::Compact => {
                            self.wal_store.lock().unwrap().append(log.clone());
                            self.snapshot();
                        }
                    },
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

    fn snapshot(&mut self) {
        let mut befor_first_compact = true;
        let mut befor_second_compact = true;
        let mut can_discard_write: HashMap<String, bool> = HashMap::new();
        let mut new_log_vec: Vec<LogEntry> = Vec::new();
        let mut wal_store = self.wal_store.lock().unwrap();
        // self.show_wal_store();
        for log in wal_store.store.iter() {
            match log.clone() {
                LogEntry::SetValue { key, value } => {
                    if befor_first_compact && befor_second_compact {
                        new_log_vec.insert(new_log_vec.len(), log.clone());
                        can_discard_write.insert(key, true);
                    } else if !befor_first_compact && befor_second_compact {
                        if let Some(true) = can_discard_write.get(&key) {
                            // do nothing
                        } else {
                            new_log_vec.insert(new_log_vec.len(), log.clone());
                            can_discard_write.insert(key, true);
                        }
                    } else if !befor_first_compact && !befor_second_compact {
                        if let Some(true) = can_discard_write.get(&key) {
                            // do nothing
                        } else {
                            new_log_vec.insert(new_log_vec.len(), log.clone());
                            can_discard_write.insert(key, true);
                        }
                    }
                }
                LogEntry::LINRead { opid, key, value } => {
                    if befor_first_compact && befor_second_compact {
                        new_log_vec.insert(new_log_vec.len(), log.clone());
                    } else if !befor_first_compact && befor_second_compact {
                        new_log_vec.insert(new_log_vec.len(), log.clone());
                    } else if !befor_first_compact && !befor_second_compact {

                    }
                }
                LogEntry::LINWrite { opid, key, value } => {
                    if befor_first_compact && befor_second_compact {
                        new_log_vec.insert(new_log_vec.len(), log.clone());
                        can_discard_write.insert(key, true);
                    } else if !befor_first_compact && befor_second_compact {
                        if let Some(true) = can_discard_write.get(&key) {
                            // do nothing
                        } else {
                            new_log_vec.insert(new_log_vec.len(), log.clone());
                            can_discard_write.insert(key, true);
                        }
                    } else if !befor_first_compact && !befor_second_compact {
                        if let Some(true) = can_discard_write.get(&key) {
                            // do nothing
                        } else {
                            new_log_vec.insert(new_log_vec.len(), log.clone());
                            can_discard_write.insert(key, true);
                        }
                    }
                }
                LogEntry::Compact => {
                    if befor_first_compact && befor_second_compact {
                        befor_first_compact = false;
                        new_log_vec.insert(new_log_vec.len(), log.clone());
                    } else if !befor_first_compact && befor_second_compact {
                        befor_second_compact = false;
                    }
                }
            };
        }
        // info!("new logs: {:?}", new_log_vec);
        wal_store.store.clear();
        wal_store.store.append(&mut new_log_vec);
    }

    pub fn compact(&self) {
        self.put_log_into_omni(LogEntry::Compact);
    }
}

mod test {
    use super::*;
}

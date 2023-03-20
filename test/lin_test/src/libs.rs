use log::debug;
use log::{error, info, log_enabled, Level};
use rand::Rng;
use tokio::time::{sleep, Duration};

use std::collections::HashMap;
use std::io::Write;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use ddbb_server::ddbb_server::DDBB;
use ddbb_server::omni_paxos_server::op_connection::OmniSIMO;
use ddbb_server::omni_paxos_server::op_data_structure::LogEntry;
use ddbb_server::omni_paxos_server::OmniPaxosInstance;
use omnipaxos_core::omni_paxos::OmniPaxosConfig;
use omnipaxos_core::util::NodeId;
use omnipaxos_storage::memory_storage::MemoryStorage;

use crate::configs::{ELECTION_TIMEOUT, LOG_CUNCURRENT_NUM, STRAT_PORT};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TestCommandEntry {
    LINRead { key: String, value: Option<Vec<u8>> },
    LINWrite { key: String, value: Vec<u8> },
}

#[derive(Clone, Debug)]
pub struct LogEntryWithTime {
    nodeid: NodeId,
    pub log: TestCommandEntry,
    start: Instant,
    end: Instant,
}

impl PartialEq for LogEntryWithTime {
    fn eq(&self, other: &Self) -> bool {
        self.log == other.log && self.start == other.start && self.end == other.end
    }
}

impl PartialOrd for LogEntryWithTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.end.partial_cmp(&other.start) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.end.partial_cmp(&other.end)
    }
}

pub fn generate_commands(num: u64) -> Vec<TestCommandEntry> {
    let mut res = Vec::new();
    let mut rng = rand::thread_rng();
    for i in 0..num {
        match rng.gen_range(0..2) {
            0 => res.insert(
                0,
                TestCommandEntry::LINWrite {
                    key: rng.gen_range(0..num / LOG_CUNCURRENT_NUM).to_string(),
                    value: generate_ran_u8_lst(1),
                },
            ),
            1 => res.insert(
                0,
                TestCommandEntry::LINRead {
                    key: rng.gen_range(0..num / LOG_CUNCURRENT_NUM).to_string(),
                    value: None,
                },
            ),
            _ => {}
        }
    }
    return res;
}

fn generate_ran_u8_lst(len: u8) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut res = Vec::new();
    for i in 0..len {
        res.insert(0, rng.gen_range(0..255));
    }
    return res;
}

pub fn check_log_consistency(logs_vec: &Vec<Vec<LogEntry>>) -> bool {
    let mut is_cons = true;
    let len = logs_vec.get(0).unwrap().len();
    for i in 0..logs_vec.len() {
        if logs_vec[i].len() != len {
            return false;
        }
    }

    for i in 0..len {
        let log = logs_vec.get(0).unwrap().get(i).unwrap();
        for j in 0..logs_vec.len() {
            if logs_vec.get(j).unwrap().get(i).unwrap() != log {
                is_cons = false;
            }
        }
    }

    return is_cons;
}

pub fn check_log_lin(logs: Vec<LogEntryWithTime>) {}

pub fn generate_cluster(node_num: u64) -> HashMap<NodeId, String> {
    let mut res = HashMap::new();
    for i in 1..node_num + 1 {
        let mut addr = "127.0.0.1:".to_string();
        addr.push_str((STRAT_PORT + i).to_string().as_str());
        res.insert(i, addr);
    }
    return res;
}

pub async fn run_ddbb(
    nodeid: NodeId,
    cluster: HashMap<NodeId, String>,
    commands: Vec<TestCommandEntry>,
) -> Vec<LogEntryWithTime> {
    let mut local_trace: Vec<LogEntryWithTime> = Vec::new();
    let node_addr = cluster.get(&nodeid).unwrap();
    let peer_ids: Vec<&u64> = cluster.keys().filter(|&&x| x != nodeid).collect();
    let peer_ids: Vec<u64> = peer_ids.iter().copied().map(|x| *x).collect();
    let mut peers: HashMap<NodeId, String> = HashMap::new();
    for peerid in peer_ids.clone() {
        peers.insert(peerid, cluster.get(&peerid).unwrap().clone());
    }

    let op_config = OmniPaxosConfig {
        pid: nodeid,
        configuration_id: 1,
        peers: peer_ids,
        ..Default::default()
    };
    let omni: OmniPaxosInstance = op_config.build(MemoryStorage::default());
    // !! peer.clone
    let simo = OmniSIMO::new(node_addr.to_string(), peers.clone());
    let mut ddbb = DDBB::new(nodeid, node_addr.to_string(), peers, simo, omni);
    let mut ddbb: Arc<Mutex<DDBB>> = Arc::new(Mutex::new(ddbb));

    // run ddbb
    let ddbb_copy = ddbb.clone();
    let omni_server_handler = tokio::spawn(async move {
        DDBB::start(ddbb_copy).await.unwrap();
    });

    sleep(Duration::from_millis(ELECTION_TIMEOUT)).await;

    for command in commands {
        match command {
            TestCommandEntry::LINRead { key, value } => {
                let start = Instant::now();
                let value = DDBB::lin_read(ddbb.clone(), key.clone()).await.unwrap();
                let mut command = TestCommandEntry::LINRead { key, value };
                let mut ent = LogEntryWithTime {
                    nodeid,
                    log: command,
                    start,
                    end: Instant::now(),
                };
                local_trace.insert(local_trace.len(), ent);
            }
            TestCommandEntry::LINWrite { key, value } => {
                let start = Instant::now();
                DDBB::lin_write(ddbb.clone(), key.clone(), value.clone()).await;
                let command = TestCommandEntry::LINWrite { key, value };
                let mut ent = LogEntryWithTime {
                    nodeid,
                    log: command,
                    start,
                    end: Instant::now(),
                };
                local_trace.insert(local_trace.len(), ent);
            }
        }
    }

    sleep(Duration::from_millis(500)).await;
    return local_trace;
}

pub fn output_trace(
    key: String,
    begin_t: Instant,
    global_trace_of_one_key: &Vec<LogEntryWithTime>,
) {
    let mut idx = 0;
    let path = &format!("kv_checker/test/trace_{:}.txt", key);
    let mut file = std::fs::File::create(path).expect("create failed");
    for log in global_trace_of_one_key.into_iter() {
        let nodeid = log.nodeid;
        let mut ent_pair = format!("");
        let mut invoke = format!("{{:type :invoke, ");
        let mut ok = format!("{{:type :ok, ");
        match &log.log {
            TestCommandEntry::LINRead { key, value } => {
                let mut is_null = true;
                let mut val: u8 = 0;
                if let Some(v) = value {
                    is_null = false;
                    val = v[0];
                };
                if is_null {
                    invoke += &format!(":f :read, :value {:?} ", "nil");
                    ok += &format!(":f :read, :value {:?} ", "nil");
                } else {
                    invoke += &format!(":f :read, :value {:?}", val);
                    ok += &format!(":f :read, :value {:?}", val);
                }
                ent_pair += &format!("{{:f :LINRead, :key {:?}, :value {:?}", key, value);
            }
            TestCommandEntry::LINWrite { key, value } => {
                invoke += &format!(":f :write, :value {:?}", value[0]);
                ok += &format!(":f :write, :value {:?}", value[0]);
                ent_pair += &format!("{{:f :LINWrite, :key {:?}, :value {:?}", key, value);
            }
        }
        invoke += &format!(
            ", :process {:?}, :time {:?}, :index {:?}}}\n",
            nodeid,
            get_durition(begin_t, log.start),
            idx
        );
        ok += &format!(
            ", :process {:?}, :time {:?}, :index {:?}}}\n",
            nodeid,
            get_durition(begin_t, log.end),
            idx + 1
        );
        ent_pair += &format!(
            ", :start {:?}, :end {:?}, :index {:?}}}\n",
            log.start
                .checked_duration_since(begin_t)
                .unwrap()
                .as_secs_f64(),
            log.end
                .checked_duration_since(begin_t)
                .unwrap()
                .as_secs_f64(),
            idx
        );
        // file.write_all(ent_pair.as_bytes()).expect("write failed");
        file.write_all(invoke.as_bytes()).expect("write failed");
        file.write_all(ok.as_bytes()).expect("write failed");

        idx = idx + 2;
    }
    file.try_clone().unwrap();
}

pub async fn check(traces: HashMap<String, Vec<LogEntryWithTime>>) {
    for (key_local, trace) in traces.clone() {
        // output_trace(key_local.clone(), t, &trace);
        info!("======================================");
        info!("Linearizability check of key == {:?}", key_local);
        let path = &format!("kv_checker/test/trace_{:}.txt", key_local);
        let mut checker = Command::new("kv_checker/kv_checker")
            .arg("kv_checker/test/test_demo.txt")
            .spawn()
            .unwrap();
        checker.wait().unwrap();
        sleep(Duration::from_millis(500)).await;
    }
}

fn get_durition(t1: Instant, t2: Instant) -> u64 {
    let mut duri = t2.checked_duration_since(t1).unwrap().as_secs_f64();
    duri = duri * 1000000000.0;
    return duri as u64;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_log_consistency() {
        let log = LogEntry::LINRead {
            opid: ("fqw1".to_string(), 5),
            key: "asd1".to_string(),
            value: Some(Vec::from([25])),
        };
        let log2 = LogEntry::LINWrite {
            opid: ("asf2".to_string(), 5),
            key: "fwqf1".to_string(),
            value: Vec::from([245]),
        };
        let log3 = LogEntry::LINRead {
            opid: ("1dwa".to_string(), 5),
            key: "1asd".to_string(),
            value: Some(Vec::from([245])),
        };
        let log4 = LogEntry::LINWrite {
            opid: ("2asdsa".to_string(), 5),
            key: "daasds1".to_string(),
            value: Vec::from([22]),
        };

        let logs1 = Vec::from([log.clone(), log4.clone(), log2.clone(), log3.clone()]);
        let logs2 = Vec::from([log.clone(), log4.clone(), log2.clone(), log3.clone()]);
        let logs3 = Vec::from([log.clone(), log4.clone(), log2.clone(), log3.clone()]);

        let logs_vec = Vec::from([logs1, logs2, logs3]);
        println!("chech: {:?}", check_log_consistency(&logs_vec));
        info!("check: {:?}", check_log_consistency(&logs_vec));
    }

    #[test]
    fn test_partial_eq_log_entry_with_time() {
        let command = TestCommandEntry::LINRead {
            key: "temp".to_string(),
            value: None,
        };
        let ent1 = LogEntryWithTime {
            nodeid: 1,
            log: command.clone(),
            start: Instant::now(),
            end: Instant::now(),
        };
        let ent2 = LogEntryWithTime {
            nodeid: 1,
            log: command.clone(),
            start: Instant::now(),
            end: Instant::now(),
        };
        println!("ent1: {:?}", ent1);
        println!("ent2: {:?}", ent2);
        println!("eq: {:?}", ent1 < ent2);
        println!("eq1: {:?}", ent1.end < ent2.start);
    }

    #[test]
    fn test_duri() {
        let t = Instant::now();
        get_durition(t, Instant::now());
        get_durition(t, Instant::now());
        get_durition(t, Instant::now());
        get_durition(t, Instant::now());
    }

    #[test]
    fn test_output_global_trace() {
        let t = Instant::now();
        let mut arr: Vec<LogEntryWithTime> = Vec::new();
        let test1 = LogEntryWithTime {
            nodeid: 1,
            log: TestCommandEntry::LINWrite {
                key: "hello1".to_string(),
                value: vec![1],
            },
            start: Instant::now(),
            end: Instant::now(),
        };
        let test2 = LogEntryWithTime {
            nodeid: 2,
            log: TestCommandEntry::LINWrite {
                key: "hello2".to_string(),
                value: vec![2],
            },
            start: Instant::now(),
            end: Instant::now(),
        };
        //test1.start.elapsed()
        arr.push(test1);
        arr.push(test2);
        output_trace("".to_string(), t, &arr);
    }
}

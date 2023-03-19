use log::debug;
use log::{error, info, log_enabled, Level};
use rand::Rng;

use std::time::Instant;

use ddbb_server::omni_paxos_server::op_data_structure::LogEntry;

use crate::configs::LOG_CUNCURRENT_NUM;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TestCommandEntry {
    LINRead { key: String, value: Option<Vec<u8>> },
    LINWrite { key: String, value: Vec<u8> },
}

#[derive(Clone, Debug)]
pub struct LogEntryWithTime {
    log: TestCommandEntry,
    start: Instant,
    end: Instant,
}

impl PartialEq for LogEntryWithTime{
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
                    value: generate_ran_u8_lst(4),
                },
            ),
            1 => res.insert(
                0,
                TestCommandEntry::LINRead {
                    key: rng.gen_range(0..num / LOG_CUNCURRENT_NUM).to_string(),
                    value: None
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
        if logs_vec[i].len() != len{
            return  false;
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
    fn test_partial_eq_log_entry_with_time(){
        let command = TestCommandEntry::LINRead { key: "temp".to_string(), value:None };
        let ent1 = LogEntryWithTime { log: command.clone(), start: Instant::now(), end: Instant::now() };
        let ent2 = LogEntryWithTime { log: command.clone(), start: Instant::now(), end: Instant::now() };
        println!("ent1: {:?}", ent1);
        println!("ent2: {:?}", ent2);
        println!("eq: {:?}", ent1 < ent2);
        println!("eq1: {:?}", ent1.end < ent2.start);

    }
}

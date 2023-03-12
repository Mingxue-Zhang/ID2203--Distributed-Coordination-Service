use omnipaxos_core::{
    messages::Message,
    omni_paxos::*,
    util::LogEntry as OmniLogEntry,
    util::NodeId as OmniNodeId,
};
use omnipaxos_storage::memory_storage::MemoryStorage;
use tokio::{runtime::Builder, sync::mpsc};

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use data_structure::LogEntry;

mod omni_paxos;
mod data_structure;


type OmniPaxosKV = OmniPaxos<LogEntry, (), MemoryStorage<LogEntry, ()>>;


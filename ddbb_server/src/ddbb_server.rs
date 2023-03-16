use omnipaxos_core::util::NodeId;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ddbb_libs::{Error, Result};

use crate::omni_paxos_server::{op_connection::OmniSIMO, OmniPaxosInstance};

pub struct DDBB {
    node_info: NodeInfo,
    peers: Arc<Mutex<HashMap<NodeId, String>>>,
    simo: Arc<Mutex<OmniSIMO>>,
    omni: Arc<Mutex<OmniPaxosInstance>>,
}

struct NodeInfo {
    id: NodeId,
    addr: String,
}

impl DDBB {
    pub fn new(
        id: NodeId,
        self_addr: String,
        peers: HashMap<NodeId, String>,
        simo: OmniSIMO,
        omni: OmniPaxosInstance,
    ) -> Self {
        DDBB {
            node_info: NodeInfo {
                id,
                addr: self_addr,
            },
            peers: Arc::new(Mutex::new(peers)),
            simo: Arc::new(Mutex::new(simo)),
            omni: Arc::new(Mutex::new(omni)),
        }
    }

    pub async fn start(&self) -> Result<()>{


        return Ok(());
    }

    async fn start_simo() {
        
    }
}

use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use omnipaxos_core::util::NodeId;
use ddbb_libs::data_structure::FrameCast;
use ddbb_libs::connection::{self, Connection};
use ddbb_libs::{Error, Result};

use crate::config::RETRIEVE_INTERVAL;
use super::op_data_structure::{LogEntry, OmniMessageEntry, Snapshot};
use super::OmniMessage;

type OmniMessageBuf = Arc<Mutex<VecDeque<OmniMessage>>>;

/// single incoming and multiple outgoing connection for OmniPaxos instances' communication
#[derive(Clone, Debug)]
pub struct OmniSIMO {
    self_addr: String,
    /// #Example: nodeid: 6, addr: "127.0.0.1:25536"
    peers: Arc<Mutex<HashMap<NodeId, String>>>,
    pub outgoing_buffer: OmniMessageBuf,
    pub incoming_buffer: OmniMessageBuf,
}

impl OmniSIMO {
    pub fn new(self_addr: String, peers: HashMap<NodeId, String>) -> Self {
        OmniSIMO {
            outgoing_buffer: Arc::new(Mutex::new(VecDeque::new())),
            incoming_buffer: Arc::new(Mutex::new(VecDeque::new())),
            self_addr,
            peers: Arc::new(Mutex::new(peers)),
        }
    }

    pub fn send_message(&self, omni_message: &OmniMessage) {
        self.outgoing_buffer
            .lock()
            .unwrap()
            .push_back(omni_message.clone());
    }

    pub async fn receive_message(simo: Arc<Mutex<OmniSIMO>>) -> Result<OmniMessage> {
        let buf = simo.lock().unwrap().incoming_buffer.clone();
        loop {
            {
                if let Some(msg) = buf.lock().unwrap().pop_front() {
                    return Ok(msg);
                }
            }
            sleep(Duration::from_millis(RETRIEVE_INTERVAL)).await;
        }
    }

    /// #Descriptions: start the sender of an omni simo
    pub async fn start_sender(simo: Arc<Mutex<OmniSIMO>>) -> Result<()> {
        let peers = simo.lock().unwrap().peers.clone();
        let outgoing_buffer = simo.lock().unwrap().outgoing_buffer.clone();
        loop {
            {
                if let Some(outgoing_message) = outgoing_buffer.lock().unwrap().pop_front() {
                    let receiver = outgoing_message.get_receiver();
                    if let Some(receive_addr) = peers.lock().unwrap().get(&receiver) {
                        let mut tcp_stream = TcpStream::connect(receive_addr).await?;
                        let mut connection = Connection::new(tcp_stream);
                        let omni_msg_entry = OmniMessageEntry {
                            omni_msg: outgoing_message,
                        };
                        connection.write_frame(&omni_msg_entry.to_frame()).await;
                    }
                }
            }
            sleep(Duration::from_millis(RETRIEVE_INTERVAL)).await;
        }
    }

    /// #Descriptions: start the listener of an omni simo
    pub async fn start_incoming_listener(simo: Arc<Mutex<OmniSIMO>>) -> Result<()> {
        let self_addr = simo.lock().unwrap().self_addr.clone();
        let incoming_buffer = simo.lock().unwrap().incoming_buffer.clone();
        let listener = TcpListener::bind(&self_addr).await?;
        // thread of incoming listener
        loop {
            let (mut stream, addr) = listener.accept().await.unwrap();
            let mut connection = Connection::new(stream);
            let incoming_buffer_copy = incoming_buffer.clone();
            // thread of new connection
            tokio::spawn(async move {
                Self::process_connection(incoming_buffer_copy, connection).await;
            });
        }
        return Ok(());
    }

    async fn process_connection(
        incoming_buffer: OmniMessageBuf,
        mut connection: Connection,
    ) -> Result<()> {
        let msg_frame = connection.read_frame().await?.unwrap();
        let omni_message_entry = *OmniMessageEntry::from_frame(&msg_frame).unwrap();
        incoming_buffer
            .lock()
            .unwrap()
            .push_back(omni_message_entry.omni_msg);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use omnipaxos_core::messages::{
        ballot_leader_election::BLEMessage,
        sequence_paxos::{PaxosMessage, PaxosMsg},
    };
    use tokio::time::{sleep, Duration};

    async fn test_send(msg: OmniMessage, simo: Arc<Mutex<OmniSIMO>>) {
        // wait for server starting up
        sleep(Duration::from_millis(1000)).await;

        loop {
            {
                let simo = simo.lock().unwrap();
                simo.send_message(&msg);
            }
            sleep(Duration::from_millis(1000)).await;
        }
    }

    async fn test_receive(simo: Arc<Mutex<OmniSIMO>>) {
        loop {
            let msg = OmniSIMO::receive_message(simo.clone()).await.unwrap();
            println!("receive: {:?}", msg);
        }
    }

    #[tokio::test]
    async fn test_omni_simo() {
        let mut peers: HashMap<NodeId, String> = HashMap::new();
        peers.insert(2, "127.0.0.1:5660".to_string());

        let mut omni_simo = OmniSIMO::new("127.0.0.1:5661".to_string(), peers);
        let omni_simo = Arc::new(Mutex::new(omni_simo));

        // message
        let paxos_message: PaxosMessage<LogEntry, Snapshot> = PaxosMessage {
            from: 1,
            to: 2,
            msg: PaxosMsg::ProposalForward(vec![LogEntry::SetValue {
                key: "testKey".to_string(),
                value: Vec::from("tempValue"),
            }]),
        };
        let msg = OmniMessage::SequencePaxos(paxos_message);

        // start sender and listener
        let omni_simo_copy1 = omni_simo.clone();
        let omni_simo_copy2 = omni_simo.clone();
        let omni_simo_copy3 = omni_simo.clone();
        let omni_simo_copy4 = omni_simo.clone();

        tokio::spawn(test_send(msg, omni_simo_copy3));
        tokio::select! {
            e = OmniSIMO::start_incoming_listener(omni_simo_copy1) => {println!("e: {:?}", e);}
            e = OmniSIMO::start_sender(omni_simo_copy2) => {println!("e: {:?}", e);}
            _ = test_receive(omni_simo_copy4) => {}
        }
    }

    #[tokio::test]
    async fn test_omni_simo_peer() {
        let mut peers: HashMap<NodeId, String> = HashMap::new();
        peers.insert(1, "127.0.0.1:5661".to_string());
        let mut omni_simo = OmniSIMO::new("127.0.0.1:5660".to_string(), peers);
        let omni_simo = Arc::new(Mutex::new(omni_simo));

        // message
        let paxos_message: PaxosMessage<LogEntry, Snapshot> = PaxosMessage {
            from: 2,
            to: 1,
            msg: PaxosMsg::ProposalForward(vec![LogEntry::SetValue {
                key: "testKey".to_string(),
                value: Vec::from("tempValue"),
            }]),
        };
        let msg = OmniMessage::SequencePaxos(paxos_message);

        let omni_simo_copy1 = omni_simo.clone();
        let omni_simo_copy2 = omni_simo.clone();
        let omni_simo_copy3 = omni_simo.clone();
        let omni_simo_copy4 = omni_simo.clone();

        tokio::spawn(test_send(msg, omni_simo_copy3));
        // start sender and listener
        tokio::select! {
            e = OmniSIMO::start_incoming_listener(omni_simo_copy1) => {println!("e: {:?}", e);}
            e = OmniSIMO::start_sender(omni_simo_copy2) => {println!("e: {:?}", e);}
            _ = test_receive(omni_simo_copy4) => {}
        }
    }
}

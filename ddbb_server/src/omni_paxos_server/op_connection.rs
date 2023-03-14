use ddbb_libs::data_structure::FrameCast;
use omnipaxos_core::util::NodeId;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use super::op_data_structure::{LogEntry, OmniMessageEntry, Snapshot};
use super::OmniMessage;
use ddbb_libs::connection::{self, Connection};
use ddbb_libs::{Error, Result};

type OmniMessageBuf = Arc<Mutex<VecDeque<OmniMessage>>>;

/// single incoming and multiple outgoing connection for OmniPaxos instances' communication
#[derive(Clone, Debug)]
pub struct OmniSIMO {
    self_addr: String,
    /// #Example: nodeid: 6, addr: "127.0.0.1:25536"
    peers: Arc<Mutex<HashMap<NodeId, String>>>,
    outgoing_buffer: OmniMessageBuf,
    incoming_buffer: OmniMessageBuf,
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

    fn send_message(&self, omni_message: &OmniMessage) {
        println!("39");
        println!("40 lock: {:?}", self.outgoing_buffer);
        let a = self.outgoing_buffer.lock();
        println!("40");
        self.outgoing_buffer
            .lock()
            .unwrap()
            .push_back(omni_message.clone());
    }

    /// #Descriptions: start the sender of an omni simo
    pub async fn start_sender(simo: Arc<Mutex<OmniSIMO>>) -> Result<()> {
        println!("51");
        let peers = simo.lock().unwrap().peers.clone();
        let outgoing_buffer = simo.lock().unwrap().outgoing_buffer.clone();
        loop {
            println!("55");
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
            {
                println!("68");
                // @temp: interval of checking send buffer
                sleep(Duration::from_millis(100)).await;
                println!("71");
            }
            println!("lock73: {:?}", outgoing_buffer);
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
        println!("receive: {:?}", omni_message_entry.omni_msg);
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

    async fn test_send(simo: Arc<Mutex<OmniSIMO>>) {
        tokio::spawn(async move {
            let paxos_message: PaxosMessage<LogEntry, Snapshot> = PaxosMessage {
                from: 1,
                to: 2,
                msg: PaxosMsg::ProposalForward(vec![LogEntry::SetValue {
                    key: "testKey".to_string(),
                    value: Vec::from("tempValue"),
                }]),
            };
            let omni_message = OmniMessage::SequencePaxos(paxos_message);

            println!("121");
            simo.lock().unwrap().send_message(&omni_message);
            println!("125");
            simo.lock().unwrap().send_message(&omni_message);
            simo.lock().unwrap().send_message(&omni_message);
            simo.lock().unwrap().send_message(&omni_message);
            println!("127");
        })
        .await;
    }

    #[tokio::test]
    async fn test_omni_simo() {
        let mut peers: HashMap<NodeId, String> = HashMap::new();
        peers.insert(2, "127.0.0.1:5660".to_string());

        let mut omni_simo = OmniSIMO::new("127.0.0.1:5661".to_string(), peers);
        let omni_simo = Arc::new(Mutex::new(omni_simo));

        // send message

        // start sender and listener
        let omni_simo_copy1 = omni_simo.clone();
        let omni_simo_copy2 = omni_simo.clone();
        let omni_simo_copy3 = omni_simo.clone();

        tokio::select! {
            e = OmniSIMO::start_incoming_listener(omni_simo_copy1) => {println!("e: {:?}", e);}
            e = OmniSIMO::start_sender(omni_simo_copy2) => {println!("e: {:?}", e);}
            _ = test_send(omni_simo_copy3) => {}
        }
    }

    #[tokio::test]
    async fn test_omni_simo_peer() {
        let mut peers: HashMap<NodeId, String> = HashMap::new();
        peers.insert(1, "127.0.0.1:5661".to_string());
        let mut omni_simo = OmniSIMO::new("127.0.0.1:5660".to_string(), peers);
        let omni_simo = Arc::new(Mutex::new(omni_simo));

        let omni_simo_copy1 = omni_simo.clone();
        let omni_simo_copy2 = omni_simo.clone();

        // start sender and listener
        tokio::select! {
            e = OmniSIMO::start_incoming_listener(omni_simo_copy1) => {println!("e: {:?}", e);}
            e = OmniSIMO::start_sender(omni_simo_copy2) => {println!("e: {:?}", e);}
        }
    }
}

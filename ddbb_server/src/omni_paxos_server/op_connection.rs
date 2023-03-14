use omnipaxos_core::messages::Message as OmniMessage;
use omnipaxos_core::util::NodeId;
use tokio::io;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use super::op_data_structure::{OmniMessageEntry, LogEntry};
use ddbb_libs::connection::{self, Connection};
use ddbb_libs::{Error, Result};

type OmniMessageBuf = Arc<Mutex<VecDeque<OmniMessage<LogEntry, ()>>>>;

/// single incoming and multiple outgoing connection for OmniPaxos instances' communication
pub struct OmniSIMO {
    self_addr: String,
    // nodeid: 6, addr: "127.0.0.1:25536"
    peers: HashMap<NodeId, String>,
    outgoing_buffer: OmniMessageBuf,
    incoming_buffer: OmniMessageBuf,
}

impl OmniSIMO {
    pub fn build(self_addr: &String, peers: &HashMap<NodeId, String>) -> OmniSIMO {
        OmniSIMO {
            outgoing_buffer: Arc::new(Mutex::new(VecDeque::new())),
            incoming_buffer: Arc::new(Mutex::new(VecDeque::new())),
            self_addr: self_addr.clone(),
            peers: peers.clone(),
        }
    }

    // handle incoming connection reqs
    async fn start_incoming_listener(self_addr: &String) -> JoinHandle<Result<()>> {
        let listener = TcpListener::bind(self_addr).await.unwrap();

        // thread for incoming listener
        return tokio::spawn(async move {
            loop {
                let (mut stream, addr) = listener.accept().await.unwrap();
                let mut connection = Connection::new(stream);
                // thread for new connection
                tokio::spawn(async move {
                    Self::process_connection(connection);
                });
            }
            // thread of req incoming listener
        });
    }

    async fn process_connection(mut connection: Connection) -> Result<()> {
        let msg_frame = connection.read_frame().await?.unwrap();
        todo!("MessageEntry");
        // match *CommandEntry::from_frame(&msg_frame).unwrap() {
        //     CommandEntry::SetValue { key, value } => {
        //         println!("Receive command: {}, {:?}", key, value);
        //         let res = MessageEntry::Success {
        //             msg: "Operation success".to_string(),
        //         };
        //         connection.write_frame(&res.to_frame()).await.unwrap_or(());
        //     }

        //     _ => {}
        // }
        Ok(())
    }
}

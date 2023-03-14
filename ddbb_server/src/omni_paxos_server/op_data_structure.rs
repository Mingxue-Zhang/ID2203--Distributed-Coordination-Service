use bytes::Bytes;
use omnipaxos_core::messages::Message;
use serde_json;

use ddbb_libs::data_structure::FrameCast;
use ddbb_libs::frame::Frame;
pub use ddbb_libs::data_structure::LogEntry; 

use super::OmniMessage;

use crate::{Error, Result};

pub type Snapshot = ();

/// for network transportation of omnipaxos_core::messages::Message
#[derive(Clone, Debug)]
pub struct OmniMessageEntry {
    pub(crate) omni_msg: OmniMessage,
}

impl FrameCast for OmniMessageEntry {
    fn to_frame(&self) -> Frame {
        Frame::Array(vec![
            // begin tag
            Frame::Simple("OmniMessageEntry".to_string()),
            Frame::Bulk(serde_json::to_vec(&self.omni_msg).unwrap().into()),
        ])
    }

    fn from_frame(frame: &Frame) -> Result<Box<Self>> {
        match frame {
            Frame::Array(ref frame_vec) => match frame_vec.as_slice() {
                /// MessageEntry::Success
                [begin_tag, msg] if *begin_tag == "OmniMessageEntry" => {
                    if let Frame::Bulk(serialized_ble) = msg {
                        let omni_msg: OmniMessage = serde_json::from_slice(&serialized_ble).unwrap();
                        Ok(Box::new(OmniMessageEntry { omni_msg }))
                    } else {
                        Err(frame.to_error()).into()
                    }
                }

                _ => Err(frame.to_error()).into(),
            },

            _ => Err(frame.to_error()).into(),
        }
    }
}

#[cfg(test)]
mod tests {

    use omnipaxos_core::messages::{ballot_leader_election::BLEMessage, sequence_paxos::{PaxosMsg, PaxosMessage}};

    use super::*;

    #[test]
    fn test_serialize() {
        let log = LogEntry::SetValue {
            key: "testKey".to_string(),
            value: Vec::from("tempValue"),
        };
        println!("log: {:?}", log);
        let c = serde_json::to_vec(&log).unwrap();
        println!("serialized: {:?}", c);
        let d: LogEntry = serde_json::from_slice(&c).unwrap();
        println!("deserialized: {:?}", d);
    }

    #[test]
    fn test_omnimessage_entry() {
        let paxos_message: PaxosMessage<LogEntry, Snapshot> = PaxosMessage {
            from: 1,
            to: 2,
            msg: PaxosMsg::ProposalForward(vec![LogEntry::SetValue {
                key: "testKey".to_string(),
                value: Vec::from("tempValue"),
            }]),
        };

        let omni_message = OmniMessage::SequencePaxos(paxos_message);
        let omni_entry = OmniMessageEntry {
            omni_msg: omni_message,
        };
        println!("omni message entry: {:?}", omni_entry);
        let omni_frame = omni_entry.to_frame();
        println!("frame: {:?}", omni_frame);
        let omni_deserialized = OmniMessageEntry::from_frame(&omni_frame).unwrap();
        println!("deframe: {:?}", omni_deserialized);
    }
}

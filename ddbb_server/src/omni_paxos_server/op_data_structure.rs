use omnipaxos_core::messages::ballot_leader_election::BLEMessage;
use serde_json;

use ddbb_libs::data_structure::FrameCast;
use ddbb_libs::frame::Frame;
pub use ddbb_libs::data_structure::LogEntry;

use crate::{Error, Result};

/// For network transportation
#[derive(Clone, Debug)]
pub struct BLEMessageEntry {
    pub(crate) ble_msg: BLEMessage,
}

impl FrameCast for BLEMessageEntry {
    fn to_frame(&self) -> Frame {
        Frame::Array(vec![
            // begin tag
            Frame::Simple("BLEMessageEntry".to_string()),
            Frame::Bulk(serde_json::to_vec(&self.ble_msg).unwrap().into()),
        ])
    }

    fn from_frame(frame: &Frame) -> Result<Box<Self>> {
        match frame {
            Frame::Array(ref frame_vec) => match frame_vec.as_slice() {
                /// MessageEntry::Success
                [begin_tag, msg] if *begin_tag == "BLEMessageEntry" => {
                    if let Frame::Bulk(serialized_ble) = msg {
                        let ble_msg: BLEMessage = serde_json::from_slice(&serialized_ble).unwrap();
                        Ok(Box::new(BLEMessageEntry { ble_msg }))
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
    use super::*;

    #[test]
    fn test_blemessage_entry() {
        let ble = BLEMessage {
            from: 1,
            to: 2,
            msg: omnipaxos_core::messages::ballot_leader_election::HeartbeatMsg::Request(
                omnipaxos_core::messages::ballot_leader_election::HeartbeatRequest { round: 1 },
            ),
        };
        let ble_entry = BLEMessageEntry { ble_msg: ble };
        let ble_frame = ble_entry.to_frame();
        let ble_deserialized = BLEMessageEntry::from_frame(&ble_frame).unwrap();
        println!("{:?}", ble_deserialized);
    }
}
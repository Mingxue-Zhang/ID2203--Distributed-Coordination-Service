use omnipaxos_core::messages::ballot_leader_election::BLEMessage;
use serde_json;

pub use ddbb_libs::data_structure::{ LogEntry, FrameCast };
pub use ddbb_libs::frame::Frame;

use crate::{Error, Result};

/// For network transportation
#[derive(Clone, Debug)]
pub struct BLEMessageEntry{
    pub(crate) ble_msg: BLEMessage,
}

impl FrameCast for BLEMessageEntry{
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
            }

            _ => Err(frame.to_error()).into()
        }
    }
}
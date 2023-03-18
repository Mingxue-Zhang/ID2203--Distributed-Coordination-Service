use crate::frame::Frame;
use crate::Error;
/// data structures of ddbb system
use bytes::Bytes;
use serde::{Deserialize, Serialize};

pub trait FrameCast {
    fn to_frame(&self) -> Frame;

    fn from_frame(frame: &Frame) -> Result<Box<Self>, Error>;
}

/// For ddbb user.
#[derive(Clone, Debug)]
pub enum DataEntry {
    KeyValue { key: String, value: Bytes },
}

/// For omni-paxos.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LogEntry {
    SetValue {
        key: String,
        value: Vec<u8>,
    },
    LINRead {
        opid: (String, u64),
        key: String,
        value: Option<Vec<u8>>,
    },
    LINWrite {
        opid: (String, u64),
        key: String,
        value: Vec<u8>,
    },
}

/// For ddbb_client and ddbb_sever.
#[derive(Clone, Debug)]
pub enum CommandEntry {
    SetValue { key: String, value: Bytes },
    GetValue { key: String },
    Empty,
}

/// For ddbb_client and ddbb_server
#[derive(Clone, Debug)]
pub enum MessageEntry {
    Success { msg: String },
    Error { err_msg: String },
}

impl FrameCast for MessageEntry {
    fn to_frame(&self) -> Frame {
        return match self {
            /// MessageEntry::Success
            MessageEntry::Success { msg } => {
                Frame::Array(vec![
                    // begin tag
                    Frame::Simple("MessageEntry::Success".to_string()),
                    Frame::Simple(msg.to_string()),
                ])
            }

            /// MessageEntry::Error
            MessageEntry::Error { err_msg } => {
                Frame::Array(vec![
                    // begin tag
                    Frame::Simple("MessageEntry::Error".to_string()),
                    Frame::Simple(err_msg.to_string()),
                ])
            }
        };
    }

    fn from_frame(frame: &Frame) -> Result<Box<Self>, Error> {
        match frame {
            Frame::Array(ref frame_vec) => match frame_vec.as_slice() {
                /// MessageEntry::Success
                [begin_tag, msg] if *begin_tag == "MessageEntry::Success" => {
                    Ok(Box::new(MessageEntry::Success {
                        msg: msg.to_string(),
                    }))
                }

                /// MessageEntry::Error
                [begin_tag, err_msg] if *begin_tag == "MessageEntry::Error" => {
                    Ok(Box::new(MessageEntry::Error {
                        err_msg: err_msg.to_string(),
                    }))
                }

                _ => Err(frame.to_error()).into(),
            },

            _ => Err(frame.to_error()).into(),
        }
    }
}

impl FrameCast for DataEntry {
    fn to_frame(&self) -> Frame {
        return match self {
            /// DataEntry::KeyValue
            DataEntry::KeyValue { key, value } => {
                Frame::Array(vec![
                    // begin tag
                    Frame::Simple("DataEntry::KeyValue".to_string()),
                    Frame::Simple(key.to_string()),
                    Frame::Bulk(value.clone()),
                ])
            }
        };
    }

    fn from_frame(frame: &Frame) -> Result<Box<Self>, Error> {
        match frame {
            Frame::Array(ref frame_vec) => match frame_vec.as_slice() {
                /// DataEntry::KeyValue
                [begin_tag, key, value] if *begin_tag == "DataEntry::KeyValue" => {
                    Ok(Box::new(DataEntry::KeyValue {
                        key: key.to_string(),
                        value: Bytes::from(value.to_string()),
                    }))
                }
                _ => Err(frame.to_error()).into(),
            },

            _ => Err(frame.to_error()).into(),
        }
    }
}

impl FrameCast for LogEntry {
    fn to_frame(&self) -> Frame {
        Frame::Array(vec![
            // begin tag
            Frame::Simple("LogEntry".to_string()),
            Frame::Bulk(serde_json::to_vec(&self).unwrap().into()),
        ])
    }

    fn from_frame(frame: &Frame) -> Result<Box<LogEntry>, Error> {
        match frame {
            Frame::Array(ref frame_vec) => match frame_vec.as_slice() {
                /// MessageEntry::Success
                [begin_tag, msg] if *begin_tag == "LogEntry" => {
                    if let Frame::Bulk(serialized_msg) = msg {
                        let result: LogEntry = serde_json::from_slice(&serialized_msg).unwrap();
                        Ok(Box::new(result))
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

impl FrameCast for CommandEntry {
    fn to_frame(&self) -> Frame {
        return match self {
            /// CommandEntry::SetValue
            CommandEntry::SetValue { key, value } => {
                Frame::Array(vec![
                    // begin tag
                    Frame::Simple("CommandEntry::SetValue".to_string()),
                    Frame::Simple(key.to_string()),
                    Frame::Bulk(value.clone()),
                ])
            }

            /// CommandEntry::GetValue
            CommandEntry::GetValue { key } => {
                Frame::Array(vec![
                    // begin tag
                    Frame::Simple("CommandEntry::GetValue".to_string()),
                    Frame::Simple("CommandEntry::GetValue".to_string()), //不知道为什么要多加一行，不然会报错
                    Frame::Simple(key.to_string()),
                ])
            }
            CommandEntry::Empty => Frame::Array(vec![]),
        };
    }

    fn from_frame(frame: &Frame) -> Result<Box<Self>, Error> {
        match frame {
            Frame::Array(ref frame_vec) => match frame_vec.as_slice() {
                /// CommandEntry::GetValue
                [begin_tag, key, value] if *begin_tag == "CommandEntry::GetValue" => {
                    Ok(Box::new(CommandEntry::GetValue {
                        key: key.to_string(),
                    }))
                }

                /// CommandEntry::SetValue
                [begin_tag, key, value] if *begin_tag == "CommandEntry::SetValue" => {
                    Ok(Box::new(CommandEntry::SetValue {
                        key: key.to_string(),
                        value: Bytes::from(value.to_string()),
                    }))
                }

                /// CommandEntry::GetValue
                [begin_tag, key, value] if *begin_tag == "CommandEntry::GetValue" => {
                    Ok(Box::new(CommandEntry::GetValue {
                        key: key.to_string(),
                    }))
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
    fn test_log_entry() {
        let log = LogEntry::SetValue {
            key: "testKey".to_string(),
            value: Vec::from("tempValue"),
        };
        println!("log: {:?}", log);
        let frame = log.to_frame();
        let de_frame = LogEntry::from_frame(&frame).unwrap();
        println!("de frame: {:?}", de_frame);
    }
}

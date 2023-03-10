/// data structures of ddbb system
use bytes::Bytes;
use crate::frame::{Frame};
use crate::Error;


pub trait FrameCast {
    fn to_frame(&self) -> Frame;

    fn from_frame(frame: &Frame) -> Result<Box<Self>, Error>;
}

/// For ddbb user.
#[derive(Clone, Debug)]
pub enum DataEntry {
    KeyValue {
        key: String,
        value: Bytes,
    },
}

/// For omni-paxos.
#[derive(Clone, Debug)]
pub enum LogEntry {
    SetValue {
        key: String,
        value: Bytes,
    },
}

/// For ddbb_client and ddbb_sever.
#[derive(Clone, Debug)]
pub enum CommandEntry {
    SetValue {
        key: String,
        value: Bytes,
    },
    GetValue {
        key: String,
    },
    Empty,
}

/// For ddbb_client and ddbb_server
#[derive(Clone, Debug)]
pub enum MessageEntry {
    Success {
        msg: String
    },
    Error {
        err_msg: String,
    },
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
                [begin_tag, msg] if *begin_tag == "MessageEntry::Success" => Ok(Box::new(MessageEntry::Success {
                    msg: msg.to_string(),
                })),

                /// MessageEntry::Error
                [begin_tag, err_msg] if *begin_tag == "MessageEntry::Error" => Ok(Box::new(MessageEntry::Error {
                    err_msg: err_msg.to_string(),
                })),

                _ => Err(frame.to_error()).into(),
            }

            _ => Err(frame.to_error()).into()
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
                [begin_tag, key, value] if *begin_tag == "DataEntry::KeyValue" => Ok(Box::new(DataEntry::KeyValue {
                    key: key.to_string(),
                    value: Bytes::from(value.to_string()),
                })),
                _ => Err(frame.to_error()).into(),
            }

            _ => Err(frame.to_error()).into()
        }
    }
}

impl FrameCast for LogEntry {
    fn to_frame(&self) -> Frame {
        return match self {
            /// LogEntry::SetValue
            LogEntry::SetValue { key, value } => {
                Frame::Array(vec![
                    // begin tag
                    Frame::Simple("LogEntry::SetValue".to_string()),
                    Frame::Simple(key.to_string()),
                    Frame::Bulk(value.clone()),
                ])
            }
        };
    }

    fn from_frame(frame: &Frame) -> Result<Box<LogEntry>, Error> {
        match frame {
            Frame::Array(ref frame_vec) => match frame_vec.as_slice() {
                /// LogEntry::SetValue
                [begin_tag, key, value] if *begin_tag == "LogEntry::SetValue" => Ok(Box::new(LogEntry::SetValue {
                    key: key.to_string(),
                    value: Bytes::from(value.to_string()),
                })),
                _ => Err(frame.to_error()).into(),
            }

            _ => Err(frame.to_error()).into()
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
                    Frame::Simple("CommandEntry::GetValue".to_string()), //???????????????????????????????????????????????????
                    Frame::Simple(key.to_string()),
                ])
            }
            CommandEntry::Empty=>{
                Frame::Array(vec![])
            }
        };
    }

    fn from_frame(frame: &Frame) -> Result<Box<Self>, Error> {
        match frame {
            Frame::Array(ref frame_vec) => match frame_vec.as_slice() {

                /// CommandEntry::GetValue
                [begin_tag, key, value] if *begin_tag == "CommandEntry::GetValue" => Ok(Box::new(CommandEntry::GetValue {
                    key: key.to_string(),
                })),

                /// CommandEntry::SetValue
                [begin_tag, key, value] if *begin_tag == "CommandEntry::SetValue" => Ok(Box::new(CommandEntry::SetValue {
                    key: key.to_string(),
                    value: Bytes::from(value.to_string()),
                })),


                _ => Err(frame.to_error()).into(),
            }
            _ => Err(frame.to_error()).into()
        }
    }
}

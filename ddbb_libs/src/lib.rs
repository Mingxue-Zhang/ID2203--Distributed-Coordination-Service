#![allow(unused)]

pub mod frame;
pub mod connection;
pub mod data_structure;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// Just for convenience.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use bytes::{Buf, BufMut, Bytes, BytesMut};
    use crate::data_structure::{CommandEntry, FrameCast, DataEntry, LogEntry};
    use crate::frame::Frame;
    use crate::frame::Frame::Null;
    use super::*;

    #[test]
    fn test_frame() {
        /// Examples of Fame data types
        // let frame = Frame::Array(vec![
        //     Frame::Simple("Hello ".to_string()),
        //     Frame::Bulk(Bytes::from("world!")),
        //     Frame::Null,
        //     Frame::Error("This is an Frame::Error, with a Frame:Integer:".to_string()),
        //     Frame::Integer(150)
        // ]);

        use frame::{Frame, Error::Incomplete};

        /// #Example: init frame
        let frame = Frame::Array(vec![
            Frame::Simple("hello ".to_string()),
            Frame::Bulk(Bytes::from("world!")),
        ]);

        /// #Example: serialized Frame to BytesMut
        let serialized = frame.serialize();
        // println!("{:?}", serialized);

        /// #Example: deserialize BytesMut to Frame
        let fame_res = Frame::deserialize(&serialized);
        // println!("{:?}", fame_res.unwrap_or(Frame::Null));


        /// #EXAMPLE: parse Frame from buffer of BytesMut
        // the buffer, can used by the network transportation
        let mut buffer_bytes_mut = BytesMut::with_capacity(64);
        // see more Frame data types here: https://redis.io/docs/reference/protocol-spec/
        let bin_str = b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
        buffer_bytes_mut.put(&bin_str[..]);
        let mut cursor_on_buffer = Cursor::new(&buffer_bytes_mut[..]);
        let frame_res = match Frame::check(&mut cursor_on_buffer) {
            Ok(_) => {
                let len = cursor_on_buffer.position() as usize;
                cursor_on_buffer.set_position(0);
                let frame_res = Frame::parse(&mut cursor_on_buffer);
                buffer_bytes_mut.advance(len);
                Ok(Some(frame_res.unwrap()))
            }

            Err(Incomplete) => Err(Null),

            Err(e) => panic!(),
        };
    }

    #[test]
    fn temp() {
        use data_structure::LogEntry;
        let temp = DataEntry::KeyValue{
            key: "tempKey".to_string(),
            value: Bytes::from("tempValue")
        };
        println!("{:?}", temp.to_frame());
        let temp = DataEntry::from_frame(&temp.to_frame());
        match *temp.unwrap() {
            DataEntry::KeyValue{key,value} => {
                println!("{}, {:?}", key, value);
            }
            _ => {}
        }
        // println!("{:?}", &temp);


        // let LogEntry::SetValue {key, ..} = temp.clone();
        // println!("{:?}", temp);
    }
}

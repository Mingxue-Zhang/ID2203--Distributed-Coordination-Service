

use std::env;
use std::error::Error;

use ddbb_libs::data_structure::{CommandEntry, DataEntry, FrameCast, MessageEntry};
use bytes::{ Bytes };
use tokio::net::TcpStream;
use ddbb_libs::connection::Connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{

    //1. 建立连接

    let mut tcp_stream = TcpStream::connect("127.0.0.1:6142").await?;
    let mut connection = Connection::new(tcp_stream);

    //循环

    //2. 解析命令

    // let env_args = env::args().collect();
    // let args = argrments::parse_args(env_args);
    // if let Some(opt) = args {
    //     println!("我的参数 {:#?}", opt)
    // }

    //3. 发送请求

    let cmd1 = CommandEntry::GetValue {
        key: "x".to_string()
    };

    let cmd2 = CommandEntry::SetValue {
        key: "tempKey_set".to_string(),
        value: Bytes::from("tempValue"),
    };

    match cmd2{
        CommandEntry::GetValue {
            key
        } => {
            // client.set(&key, value).await?;
            // println!("OK");
            let cmd = CommandEntry::GetValue { key };
            connection.write_frame(&cmd.to_frame()).await;
            let res = connection.read_frame().await.unwrap().unwrap();

            match *DataEntry::from_frame(&res).unwrap(){
                DataEntry::KeyValue{key, value} => {
                    println!("{:?}", value)
                }
            }
        },
        CommandEntry::SetValue {
            key,
            value
        } => {
            // client.set(&key, value).await?;
            // println!("OK");
            let cmd = CommandEntry::SetValue { key, value };
            connection.write_frame(&cmd.to_frame()).await;
            let res = connection.read_frame().await.unwrap().unwrap();

            match *MessageEntry::from_frame(&res).unwrap(){
                MessageEntry::Success {msg} => {
                    println!("Receive success msg: {}", msg);
                }

                MessageEntry::Error {err_msg} => {
                    println!("Receive err_msg: {}", err_msg);

                }
            }
        },

        _ => {
            println!("Wrong command!")
        }
    }

    Ok(())
}
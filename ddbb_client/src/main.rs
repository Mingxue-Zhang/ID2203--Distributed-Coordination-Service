//Imports
#![allow(unused)]
use std::env;
use std::error::Error;

//Tokio - used for network stuff
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::mpsc,
};
//Serde - used for serializing (turning into bytes) and deserializing messages
use serde::{Serialize, Deserialize};
use async_stream::try_stream;
use bytes::Bytes;
use std::io::{ErrorKind};
use std::time::Duration;
use tokio_stream::Stream;
use tracing::{debug, instrument};
use ddbb_libs::data_structure::{CommandEntry, DataEntry, FrameCast, MessageEntry};
use ddbb_libs::connection::Connection;

#[tokio::main]
async fn main()  {

    // let (sender_peers, receiver) = mpsc::channel(32);
    // let sender_messages = sender_peers.clone();
    // default cmd
    let mut user_cmd: CommandEntry = CommandEntry::Empty;
    
    //Spawn threads
    // tokio::spawn(async move {
    //     message_sender(user_cmd).await;
    // }); 


    // let env_args = env::args().collect();
    // let args = argrments::parse_args(env_args);
    // if let Some(opt) = args {
    //    println!("my parameter {:#?}", opt)
    // }
    let sign = format!(">>");
    
    //Std:io is required for the read_line method; needs to be imported here in order to not conflict with Tokio
    use std::io::{Write};
    //Loop through and read input from the command line of the client
    loop {
        //Get the input command
        print!("{}", sign);
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(err) => println!("Could not parse input: {}", err)
        }
        //Vectorize the input - check the first word to determine if it is a get or a put message
        let input_vector:Vec<&str> = input.trim().split(" ").collect();
        
        
        // println!("{:?}", input_vector); for testing 
        if input_vector[0] == "get" {
            if input_vector.len() == 2 {
                // sender_messages.send(("get", bincode::serialize(&input).unwrap())).await.unwrap();
                user_cmd = CommandEntry::GetValue { key: input_vector[1].to_string()};
                message_sender(user_cmd).await;
            } else {
                println!(" -> ERROR: Incorrect  command");
            }
            

        }
        else if input_vector[0] == "set" {
            if input_vector.len() == 3 {
                // sender_messages.send(("set", bincode::serialize(&input).unwrap())).await.unwrap();
                user_cmd = CommandEntry::SetValue { key: input_vector[1].to_string(), value: Bytes::from(input_vector[2].to_string()) };
                message_sender(user_cmd).await;
            } else {
                println!(" -> ERROR: Incorrect command");
            }
            
        }
        else{
            //If it is not a put or a get
            println!(" -> ERROR: Unknown command");
        }


    }
}
async fn message_sender(mut user_cmd: CommandEntry) -> Result<(), Box<dyn Error>>{
    let mut tcp_stream = TcpStream::connect("127.0.0.1:6142").await?;
    let mut connection = Connection::new(tcp_stream);
    match user_cmd{
        CommandEntry::Empty => {
            println!("Wrong command!")
        },
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

//The message_receiver function handles the messages sent within the client's code
async fn message_receiver(mut receiver: mpsc::Receiver<(&str, Vec<u8>)>) {
    //Record of the number of peers (i.e. active nodes - 1), default is 0
    let mut number_of_peers: u64 = 0; 
    //Go through messages
    while let Some(action) = receiver.recv().await {
        match (action.0, action.1) {
            //Message from a node with an updated number of peers; update the number set here
            ("peers", updated_number_of_peers) => {
                let deserialized_update: u64 = bincode::deserialize(&updated_number_of_peers).unwrap();
                number_of_peers = deserialized_update;
            },
            //Put or get message
            (_, message) => {
                let mut node = 0;
                let mut deserialized_message: String = bincode::deserialize(&message).unwrap();
                let message_vector:Vec<&str> = deserialized_message.split(" ").collect();

                let mut key: u64 = message_vector[1].trim().parse().expect(" -> ERROR: The key needs to be a number");
                if action.0 == "put" && message_vector.len() == 2 {
                    println!(" -> ERROR: Put message requires a value");
                }
                else if action.0 == "get" || message_vector.len() == 3 {
                    //Loop to see which node gets the message; each node handles up to five keys, other than the last one which handles everything higher
                    //than the key of the second last node
                    for number in 0..key {
                        if number % 5 == 0{
                            node = node + 1;
                        }
                        if number == key || node >= number_of_peers{
                            break;
                        }
                    } 
                    //Connect to the right node
                    let mut address: String = "127.0.0.1:".to_owned();
                    let port_of_node: u64 = 64500 + node;
                    address.push_str(&port_of_node.to_string().to_owned()); 
                    let stream = TcpStream::connect(address).await.unwrap();
                    //Print to the client so that it is possible to see what is going on
                    if action.0 == "put"{
                        println!(" -> Sending put message to node {}", node);
                    }
                    else {
                        println!(" -> Sending get message to node {}", node);
                    }
                    //Send the message
                    let (_reader, mut writer) = tokio::io::split(stream);    
                    let encrypted_message: Vec<u8> = bincode::serialize(&deserialized_message.trim()).unwrap();
                    writer.write_all(&encrypted_message).await.unwrap();
                }
                else{
                    println!(" -> ERROR: Unforeseen error when trying to put or get");
                }
            },
            //If a message type not implemented here is received
            _ => {
                println!(" -> ERROR: Received an unknown message type");
            },
        }
    }
}
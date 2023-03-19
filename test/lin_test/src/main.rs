#![allow(unused)]
use libs::generate_commands;
use log::info;

use std::env::set_var;
use std::time::Instant;

use ddbb_server::ddbb_server::DDBB;
use ddbb_server::omni_paxos_server::op_data_structure::LogEntry;

mod libs;
pub mod configs;

fn main() {
    set_var("RUST_LOG", "debug");


    

}

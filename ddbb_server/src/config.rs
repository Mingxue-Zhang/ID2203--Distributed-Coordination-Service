use std::time::Duration;

/// OmniSIMO configs
pub const RETRIEVE_INTERVAL: u64 = 1;
pub const RECONNECT_INTERVAL: u64 = 200;

/// DDBB configs
pub const LOG_RETRIEVE_INTERVAL: u64 = 20;

/// OmniPaxos configs
pub const BUFFER_SIZE: usize = 10000;
pub const ELECTION_TIMEOUT: Duration = Duration::from_millis(100);
pub const OUTGOING_MESSAGE_PERIOD: Duration = Duration::from_millis(1);
pub const WAIT_LEADER_TIMEOUT: Duration = Duration::from_millis(500);
pub const WAIT_DECIDED_TIMEOUT: Duration = Duration::from_millis(500);

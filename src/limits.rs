use std::time::Duration;

pub const MAX_CONN:     usize = 1;      // ---
pub const MAX_REQ:      usize = 1;      // FastCGI specific vars
pub const MPXS_CONNS:   bool = false;    // ---

pub const MAX_REQ_LENGTH: usize = 48 * 1024;
pub const TCP_LISTENER_ADDR: &'static str = "127.0.0.1:9000";
pub const UNIX_LISTENER_ADDR: &'static str = "/tmp/fcgi/listener";
pub const CONNECTION_TIMEOUT: Duration = Duration::from_mins(2);
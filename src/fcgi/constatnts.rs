pub const MAX_CONN:     usize = 1;      // ---
pub const MAX_REQ:      usize = 5;      // FastCGI specific vars
pub const MPXS_CONNS:   bool = false;    // ---

pub const HEADER_LENGTH: usize = 8;

pub const TCP_LISTENER_ADDR: &'static str = "127.0.0.1:9000";
pub const UNIX_LISTENER_ADDR: &'static str = "/tmp/fcgi/listener";
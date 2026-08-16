use std::{io::Read, os::unix::net::{UnixListener, UnixStream}};

pub struct FcgiUnixListener<Stream: Read, Write> {
    socket: UnixListener
}

impl FcgiUnixListener {
    pub fn new(addr: &'static str) -> Result<Self, &'static str> {
        let listener = UnixListener::bind(addr);
        match listener {
            Ok(socket) => Ok(FcgiUnixListener { socket: socket }),
            //Err(e) => Err("Can't open listener socket" + e.to_string().as_str());
            Err(_) => Err("Can't open listener socket")
        }
    }
}

pub struct FcgiClientUnix {
    socket: UnixStream
}

use std::os::unix::net::{UnixListener, UnixStream};

use crate::fcgi::client::{FcgiClient, FcgiListener};

/*
use std::io::{Error, ErrorKind};
use std::collections::HashSet;
use std::net::IpAddr;

impl FcgiListener<UnixStream, UnixStream> for UnixListener {
    fn accept_fcgi(&self, allowed_ips: Option<HashSet<IpAddr>>) -> Result<UnixStream, std::io::Error> {
        let (listener, addr) = (self as &UnixListener).accept()?;
        match allowed_ips {
            None => Ok(listener),
            Some(h_table) => {
                if h_table.contains(&addr.as_pathname()) { Ok(listener) }
                else { Err(Error::from(ErrorKind::ConnectionRefused)) }
            }
        }
    }
}

pub struct FcgiClientUnix { }

*/

impl FcgiClient for UnixStream { }
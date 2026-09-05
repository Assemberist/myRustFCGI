use std::io::{Error, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::collections::HashSet;
use std::net::IpAddr;

use crate::fcgi::client::{FcgiClient, FcgiListener};

impl FcgiListener<TcpStream> for TcpListener {
    fn accept_fcgi(&self, allowed_ips: Option<HashSet<IpAddr>>) -> Result<TcpStream, std::io::Error> {
        let (listener, addr) = (self as &TcpListener).accept()?;
        match allowed_ips {
            None => Ok(listener),
            Some(h_table) => {
                if h_table.contains(&addr.ip()) { Ok(listener) }
                else { Err(Error::from(ErrorKind::ConnectionRefused)) }
            }
        }
    }
}

impl FcgiClient for TcpStream { }
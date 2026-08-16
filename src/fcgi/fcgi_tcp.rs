use std::io::{Error, ErrorKind};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::collections::HashSet;
use std::net::IpAddr;

use crate::fcgi::{FcgiListener, FcgiClient};

impl FcgiListener<TcpStream, TcpStream> for TcpListener {
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

impl FcgiClient<TcpStream> for TcpStream {
    fn read_fcgi(&self) -> RequestReadResult;
    fn read_fcgi_continue(&self, part: PartialRequest) -> RequestReadResult;

    fn write_fcgi(&mut self, data: FcgiRequest) -> Result<(), &'static str>;
}
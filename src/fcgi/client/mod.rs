use std::net::IpAddr;
use std::collections::HashSet;
use std::result::Result;
use std::io::{Read, Write, Error};

use crate::fcgi::constatnts::HEADER_LENGTH;

pub mod fcgi_tcp;
pub mod fcgi_unix;

pub trait FcgiListener<Client: FcgiClient> {
    fn accept_fcgi(&self, allowed_ips: Option<HashSet<IpAddr>>) -> Result<Client, std::io::Error>;
}

pub trait FcgiClient where Self: Read + Write {
    fn read_fcgi(&mut self) -> Result<Vec<u8>, Error> {
        let mut header = [0; HEADER_LENGTH];

        self.read_exact(&mut header)?;

        let content_length = u16::from_be_bytes([header[4], header[5]]) as usize; // see FcgiHeader::get_content_length()
        let mut buffer = vec![0; content_length + 8];
        buffer.copy_from_slice(&header);

        self.read_exact(&mut buffer[8..])?;

        let pad_size = header[6] as usize;
        if pad_size > 0 {
            let mut pad = vec![0u8; pad_size];
            let _ = self.read_exact(&mut pad);
        }

        Ok(buffer)
    }

    fn read_fcgi_part(&mut self) -> Result<Vec<u8>, Error> {
        let mut header = [0; HEADER_LENGTH];

        let header_len = self.read(&mut header)?;

        if header_len == HEADER_LENGTH {
            let content_length = u16::from_be_bytes([header[4], header[5]]) as usize; // see FcgiHeader::get_content_length()
            let mut buffer = Vec::<u8>::with_capacity(content_length + 8);
            buffer.copy_from_slice(&header);
            
            let data_len = self.read(&mut buffer[8..])?;

            if data_len == content_length {
                let mut pad = vec![0u8; header[6] as usize];
                self.read(&mut pad);
            }

            Ok(buffer)
        }
        else {
            let (head, _) = header.split_at(header_len);
            Ok(Vec::from(head))
        }
    }

    //fn read_fcgi_continue(&self, part: PartialRequest) -> RequestReadResult;
}

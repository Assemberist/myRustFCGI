use std::os::unix::fs::PermissionsExt;
use std::{io::Write, net::TcpListener};

mod fcgi;

use fcgi::variables::get_allowed_ips;
use fcgi::FcgiRequest;
use fcgi::client::{FcgiListener, FcgiClient};

use crate::fcgi::client::FcgiRecord;
use crate::fcgi::{FcgiHeader, FcgiHeaderType};

fn main() -> std::io::Result<()> {
    let allowed_ips = get_allowed_ips();


    let listener = TcpListener::bind("127.0.0.1:9000")?;
    let mut socket = listener.accept_fcgi(allowed_ips)?;


    for _ in 1..4 {
        let mut record = FcgiRecord::void();

        record = socket.read_fcgi()?;
        let header = record.header();

        let header_id = header.get_request_id();
        let header_type = header.get_type();
        let header_length = header.get_content_length();
        
        if let Ok(data) = String::from_utf8(record.data()) {
            println!("id: {header_id}");
            println!("type: {header_type:?}");
            println!("len: {header_length}");
            println!("data: {data}\n\n\n");
        }
    }

        let mut record = socket.read_fcgi()?;
        let header = record.header();

        let header_id = header.get_request_id();
        let header_type = header.get_type();
        let header_length = header.get_content_length();
        let data = record.data_ref();

        println!("id: {header_id}");
        println!("type: {header_type:?}");
        println!("len: {header_length}");
        println!("data: {data:?}\n\n\n");

                // Fcgi write = socket write
        let data = Vec::from("Content-Type: text/html; charset=utf-8\r\n\r\n<html>sos received</html>");
        let request = FcgiHeader::create(FcgiHeaderType::FcgiStdout, header_id, data.len() as u16, 0);
        socket.write(&request[..]);
        socket.write(&data[..]);
        
        let request = FcgiHeader::create(FcgiHeaderType::FcgiEndRequest, header_id, 0 as u16, 0);

        //socket.write(&request[..]); 

    Ok(())
}

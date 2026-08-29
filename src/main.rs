use std::{io::Write, net::TcpListener};

mod fcgi;

use fcgi::{
    FcgiHeader,
    FcgiHeaderType,
    FcgiRequest,
    FcgiRequestStatus
};

use fcgi::variables::get_allowed_ips;
use fcgi::client::{FcgiListener, FcgiClient};

fn main() -> std::io::Result<()> {
    let allowed_ips = get_allowed_ips();

    let listener = TcpListener::bind(fcgi::constatnts::TCP_LISTENER_ADDR)?;
    let mut socket = listener.accept_fcgi(allowed_ips)?;

    // Fcgi write = socket write
    let data: Vec<u8> = vec![1,2,3];
    let _write_result = socket.write(&data[..]); 

    {
        // Fcgi read
        let request = socket.read_fcgi()?;

        // Package content
        let mut header = FcgiRequest::get_header(&request);
        let _data = FcgiRequest::get_data(&request);

        let _proto_ver = header.get_protocol_version();
        let _request_type = header.get_type();
        let _request_id = header.get_request_id();
        let _data_len = header.get_content_length();
        let _pad_len = header.get_padding_length();

        let header_bin = header.as_mut_slice();
        header_bin[0] = 1;

        let _validity = 
            match FcgiRequest::validate_fcgi_request(request) {
                FcgiRequestStatus::IncompleteHeader           => Err("Incomlete header"),
                FcgiRequestStatus::UnsupportedProtocolVersion => Err("Unsupported protocol version"),
                FcgiRequestStatus::IncorrectDataLength        => Err("Incorrect data length"),

                FcgiRequestStatus::Good => Ok(true),
            };
    }

    // Create
    let _header = FcgiHeader::create(FcgiHeaderType::FcgiBeginRequest, 1, 15, 0);
    let request = FcgiRequest::create(FcgiHeaderType::FcgiBeginRequest, 1, 15, 0);

    // Package modification
    let (_header, _data) = FcgiRequest::split_request(request)?;

    Ok(())
}

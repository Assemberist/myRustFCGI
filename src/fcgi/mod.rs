//! Fcgi protocol module

use std::result::Result;
use std::io::Error;

use constatnts::HEADER_LENGTH;

pub mod variables;
pub mod constatnts;
pub mod client;

//! # About the library
//! The library provides FastCGI (FCGI) api for rust applications.
//! The API is extencion upon standard taits.
//!
//! Refer to https://www.mit.edu/~yandros/doc/specs/fcgi-spec.html for detailed protocol description.
//!
//! # High level functionality
//! FCGI program should make several common steps:
//! 1. Process environment variables and command line arguments
//! 2. Open socket-listener
//! 3. Enter loop
//!   - Accept connection
//!   - Read request
//!   - Construct response
//!   - Send response
//!
//! <sub>TODO</sub> Add code example
//!
//! Socket listener realised as FcgiListener trait.
//! Request Read/Write is realised in FcgiClient trait.
//!
//! The library don't provides highlevel abstraction for requests:
//! FCGI Read returns `Vec<u8>`.
//! Standard `Write` trait can be used to send FCGI request.

/// Provides functionality for operation with FCGI requests.
/// FcgiRequest is abstraction over `Vec<u8>`. FCGI Header is 8 first bytes and rest is data block.
pub struct FcgiRequest {}

impl<'a> FcgiRequest {
	/// Return FcgiHeader slice the request.
    pub fn get_header(req: &'a mut Vec<u8>) -> FcgiHeader<'a> { FcgiHeader(&mut req[..HEADER_LENGTH]) }

	/// Return slice of data
    pub fn get_data(req: &'a Vec<u8>) -> &'a [u8] { &req[HEADER_LENGTH..] }
    pub fn get_data_mut(req: &'a mut Vec<u8>) -> &'a mut [u8] { &mut req[HEADER_LENGTH..] }

	/// Split request on Header and Data
    pub fn split_request(mut req: Vec<u8>) -> Result<(Vec<u8>, Vec<u8>), Error>{
        let val = req.split_off(8);
        Ok((val, req))
    }

	/// Checks integrity of FCGI request
    pub fn validate_fcgi_request(req: Vec<u8>) -> FcgiRequestStatus {
        if req.len() < 8 { return FcgiRequestStatus::IncompleteHeader; }
        if req[0] != 1 { return FcgiRequestStatus::UnsupportedProtocolVersion; }

        let content_length = u16::from_be_bytes([req[4], req[5]]) as usize;
        if content_length < req.len() - HEADER_LENGTH { return FcgiRequestStatus::IncorrectDataLength; }

        FcgiRequestStatus::Good
    }

	/// Creates FCGI Header
    pub fn create(  req_type:       FcgiHeaderType,
                    req_id:         u16,
                    content_len:    u16,
                    paddding_len:   u8)
                            -> Vec<u8>
    {
        let mut req = Vec::with_capacity(8 + content_len as usize);

        req[0] = 1;
        req[1] = req_type as u8;
        req[2] = (req_id | 255) as u8;
        req[3] = (req_id >> 8) as u8;
        req[4] = (content_len | 255) as u8;
        req[5] = (content_len >> 8) as u8;
        req[6] = (paddding_len | 255) as u8;

        req
    }
}

/// Status of partial FCGI request reading (Not applicable for blocking reading)
#[repr(u8)]
pub enum FcgiRequestStatus {
    IncompleteHeader,				/// Only part of Header was read
    UnsupportedProtocolVersion,		/// Only FCGI v1 protocol is supported
    IncorrectDataLength,			/// Header specified more data than received
    Good							/// All data received
}

#[repr(u8)]
pub enum FcgiHeaderType {
    FcgiBeginRequest    = 1,
    FcgiAbortRequest    = 2,
    FcgiEndRequest      = 3,
    FcgiParams          = 4,
    FcgiStdin           = 5,
    FcgiStdout          = 6,
    FcgiStderr          = 7,
    FcgiData            = 8,
    FcgiGetValues       = 9,
    FcgiGetValuesResult = 10,
    FcgiUnknownType     = 11
}

/// The struct is abstraction over `Vec<u8>`.
///
/// FCGI Header field specification
///
/// | Size | Field | Shift      |
/// |----|------------------|---|
/// | u8 | version;         | 0 |
/// | u8 | type;            | 1 |
/// | u8 | requestIdB1;     | 2 |
/// | u8 | requestIdB0;     | 3 |
/// | u8 | contentLengthB1; | 4 |
/// | u8 | contentLengthB0; | 5 |
/// | u8 | paddingLength;   | 6 |
/// | u8 | reserved;        | 7 |
///
/// Or
///
/// | Size | Field | Shift      |
/// |----_|-----------------|---|
/// | u8  | version;        | 0 |
/// | u8  | type;           | 1 |
/// | u16 | requestId;      | 2 |
/// | u16 | contentLength;  | 4 |
/// | u8  | paddingLength;  | 6 |
/// | u8  | reserved;       | 7 |
/// 
/// The struct relates on external data, so the struct can be construted on base of an existing data array.
/// Once you created FcgiHeader you can't change slice of data it relates.
pub struct FcgiHeader<'a>(&'a mut [u8]);

impl<'a> FcgiHeader<'a> {
	/// Creates `Vec<u8>` which contains only FCGI header (8 byte).
    pub fn create(  // version:        u8,
                    req_type:       FcgiHeaderType,
                    req_id:         u16,
                    content_len:    u16,
                    padding_len:   u8)
                        -> Vec<u8>
    {
		let req_id_bytes = req_id.to_be_bytes();
		let content_len_bytes = content_len.to_be_bytes();

        vec![
            1,
            req_type as u8,
			req_id_bytes[0],
			req_id_bytes[1],
			content_len_bytes[0],
			content_len_bytes[1],
            padding_len,
			0
        ]
    }

    pub fn get_protocol_version(&self) -> u8    { self.0[0] }
    pub fn get_type(&self) -> FcgiHeaderType {
        match self.0[1] {
            1 =>  FcgiHeaderType::FcgiBeginRequest,
            2 =>  FcgiHeaderType::FcgiAbortRequest,
            3 =>  FcgiHeaderType::FcgiEndRequest,
            4 =>  FcgiHeaderType::FcgiParams,
            5 =>  FcgiHeaderType::FcgiStdin,
            6 =>  FcgiHeaderType::FcgiStdout,
            7 =>  FcgiHeaderType::FcgiStderr,
            8 =>  FcgiHeaderType::FcgiData,
            9 =>  FcgiHeaderType::FcgiGetValues,
            10 => FcgiHeaderType::FcgiGetValuesResult,
            _ =>  FcgiHeaderType::FcgiUnknownType
        }
    }
    pub fn get_request_id(&self) -> u16         { u16::from_be_bytes([self.0[2], self.0[3]]) }
    pub fn get_content_length(&self) -> u16     { u16::from_be_bytes([self.0[4], self.0[5]]) }
    pub fn get_padding_length(&self) -> u8      { self.0[6] }

	pub fn set_type(&mut self, header_type: FcgiHeaderType) { self.0[1] = header_type as u8; }
    pub fn set_request_id(&mut self, id: u16) {
		let bytes = id.to_be_bytes();
		self.0[2] = bytes[0];
		self.0[3] = bytes[1];
	}
    pub fn set_content_length(&mut self, length: u16) {
		let bytes = length.to_be_bytes();
		self.0[4] = bytes[0];
		self.0[5] = bytes[1];
	}
    pub fn set_padding_length(&mut self, pad: u8) { self.0[6] = pad; }
	
    pub fn as_slice(&self) -> &[u8] { self.0 }
    pub fn as_mut_slice(&mut self) -> &mut [u8] { self.0 }
}

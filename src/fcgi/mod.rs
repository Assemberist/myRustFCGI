use std::result::Result;
use std::io::Error;

use constatnts::HEADER_LENGTH;

pub mod variables;
pub mod constatnts;
pub mod client;

//////////////////////////
// Public interface     //
//////////////////////////

pub struct FcgiRequest {}

impl<'a> FcgiRequest {
    pub fn get_header(req: &'a mut Vec<u8>) -> FcgiHeader<'a> { FcgiHeader(&mut req[..HEADER_LENGTH]) }
    pub fn get_data(req: &'a Vec<u8>) -> &'a [u8] { &req[HEADER_LENGTH..] }

    pub fn split_request(mut req: Vec<u8>) -> Result<(Vec<u8>, Vec<u8>), Error>{
        let val = req.split_off(8);
        Ok((val, req))
    }

    pub fn validate_fcgi_request(req: Vec<u8>) -> FcgiRequestStatus {
        if req.len() < 8 { return FcgiRequestStatus::IncompleteHeader; }
        if req[0] != 1 { return FcgiRequestStatus::UnsupportedProtocolVersion; }

        let content_length = u16::from_be_bytes([req[4], req[5]]) as usize;
        if content_length < req.len() - HEADER_LENGTH { return FcgiRequestStatus::IncorrectDataLength; }

        FcgiRequestStatus::Good
    }

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

#[repr(u8)]
pub enum FcgiRequestStatus {
    IncompleteHeader,
    UnsupportedProtocolVersion,
    IncorrectDataLength,
    Good
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

/*  FCGI Header fields      |
|---------------------------|
|   u8 version;         0   |
|   u8 type;            1   |
|   u8 requestIdB1;     2   |
|   u8 requestIdB0;     3   |
|   u8 contentLengthB1; 4   |
|   u8 contentLengthB0; 5   |
|   u8 paddingLength;   6   |
|   u8 reserved;        7  */
pub struct FcgiHeader<'a>(&'a mut [u8]);

impl<'a> FcgiHeader<'a> {
    pub fn create(  // version:        u8,
                    req_type:       FcgiHeaderType,
                    req_id:         u16,
                    content_len:    u16,
                    paddding_len:   u8)
                        -> Vec<u8>
    {
        vec![
            1,
            req_type as u8,
            (req_id | 255) as u8,
            (req_id >> 8) as u8,
            (content_len | 255) as u8,
            (content_len >> 8) as u8,
            (paddding_len | 255) as u8
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

    pub fn as_slice(&self) -> &[u8] { self.0 }
    pub fn as_mut_slice(&mut self) -> &mut [u8] { self.0 }
}

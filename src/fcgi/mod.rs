use std::fmt::Error;
use std::net::{
    IpAddr,
    TcpListener,
    TcpStream,
    ToSocketAddrs,
    SocketAddr
};
use std::collections::HashSet;
use std::result::Result;
//use std::sync::atomic::{AtomicU64, Ordering};
use std::io::{Read, Write};
//use std::str::pattern::StrSearcher;

pub mod fcgi_variables;
pub mod fcgi_tcp;
pub mod fcgi_unix;
pub mod fcgi_limits;

//////////////////////////
// Public interface     //
//////////////////////////

pub trait Listener {}
pub trait FcgiListener<Stream: Read + Write,
                       Client: FcgiClient<Stream>> {
    fn accept_fcgi(&self, allowed_ips: Option<HashSet<IpAddr>>) -> Result<Client, std::io::Error>;
}

pub struct PartialRequest {
    stage:      RequestConsumingStage,
    request:    FcgiRequest,
}

pub trait FcgiClient<Stream: Read + Write> {
    fn read_fcgi(&self) -> RequestReadResult;
    fn read_fcgi_continue(&self, part: PartialRequest) -> RequestReadResult;

    fn write_fcgi(&mut self, data: FcgiRequest) -> Result<(), &'static str>;
}

#[derive(Default)]
struct FcgiHeader{
    bytes: [u8; 8],

    /*
        u8 version;         0
        u8 type;            1
        u8 requestIdB1;     2
        u8 requestIdB0;     3
        u8 contentLengthB1; 4
        u8 contentLengthB0; 5
        u8 paddingLength;   6
        u8 reserved;        7
    */
}

struct FcgiRequest{
    header: FcgiHeader,
    data: Vec<u8>
}

pub enum RequestReadResult {
    PartialRequest(PartialRequest),
    CompleteRequest(FcgiRequest),
    Nothing
}

#[repr(u8)]
#[derive(Debug)]
enum FcgiHeaderType {
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

enum RequestConsumingStage {
    GettingHeader(usize),
    GettingData(usize)
}


impl FcgiHeader {
    fn get_type(&self) -> FcgiHeaderType {
        match self.bytes[1] {
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

    fn get_request_id(&self) -> u16 {
        u16::from_be_bytes([self.bytes[2], self.bytes[3]])
    }

    fn get_content_length(&self) -> u16 {
        u16::from_be_bytes([self.bytes[4], self.bytes[5]])
    }

    fn get_padding_length(&self) -> u8 {
        self.bytes[6]
    }
}

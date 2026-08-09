use std::collections::HashSet;
use std::net::{IpAddr, TcpListener, TcpStream};
use std::os::unix::net::SocketAddr;
use std::thread;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::result::Result;
use std::rc::Rc;
//use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, Duration};

mod limits;
mod fcgi_variables;
mod ringbuffer;
mod planner;

use std::array::from_fn as makearr;
use std::time::Duration;

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

//const HEADER_LENGTH: usize = 8;

#[derive(Default)]
struct Request{
    header: FcgiHeader,
    data: Vec<u8>
}

struct Client {
    id:     usize,
    pos:    usize,
    socket: TcpStream
}

enum RequestConsumingStage {
    GettingHeader(usize),
    GettingData(usize)
}

struct PartialRequest {
    id:         usize,
    pos:        usize,
    stage:      RequestConsumingStage,
    request:    Request,
    start_time: Instant
}

type DataBuffer = ringbuffer::SpscRingBuffer<Request, { limits::MAX_REQ }>;

fn socket_read_write_loop(data_in: &DataBuffer, data_out: &DataBuffer, clients: &[RefCell<Option<Client>>; limits::MAX_CONN]){
    thread::sleep(Duration::from_millis(1));

    let mut partial_requests: [Option<PartialRequest>; limits::MAX_REQ];

    let mut latest_client_id = 0;

    loop {
        let mut was_activity = false;
        let mut i = latest_client_id;

        for i in latest_client_id..latest_client_id + clients.len() {
            let client = clients[i % clients.len()].borrow();
            if client.is_some() {

                was_activity = true;
                latest_client_id = i % clients.len();
                break;
            }
        }
        if !was_activity {
            thread::sleep(Duration::from_micros(100));
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Listener                                                                 ////
////////////////////////////////////////////////////////////////////////////////

fn listener_loop(mut clients: [RefCell<Option<Client>>; limits::MAX_CONN]){
    let allowed_ips = fcgi_variables::get_allowed_ips();
    let listener = TcpListener::bind(limits::TCP_LISTENER_ADDR).unwrap();

    let mut global_id = 0;

    loop {
        if let Some(new_pos) = clients.iter_mut().position(|c| c.get_mut().is_none()) {
            if let Ok((stream, addr)) = listener.accept() {
                if !check_connection(addr.ip(), &allowed_ips) { continue; }

                let new_data = Client {
                    id: global_id,
                    pos: new_pos,
                    socket: stream
                };

                clients[new_pos].replace(Some(new_data));

                global_id = global_id + 1;
            }
        }
        else {
            thread::sleep(Duration::from_micros(100));
        }
    }
}

fn check_connection(addr: IpAddr, allowed_ips: &Option<HashSet<IpAddr>>) -> bool {
    if let Some(allowed) = allowed_ips { allowed.contains(&addr) }
    else { true }
}

fn main() -> std::io::Result<()> {
    let data_in: DataBuffer = ringbuffer::SpscRingBuffer::new();
    let data_out: DataBuffer = ringbuffer::SpscRingBuffer::new();
    let clients: [RefCell<Option<Client>>; limits::MAX_CONN] = makearr(|_| RefCell::new(None));

    thread::scope(|_| socket_read_write_loop(&data_in, &data_out, &clients));
    thread::scope(|_| listener_loop(clients));

    Ok(())
}

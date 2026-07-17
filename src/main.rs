use std::collections::HashSet;
use std::net::{IpAddr, TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex};
use std::result::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, Duration};

mod limits;
mod fcgi_variables;

use std::array::from_fn as makearr;

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

struct Request {
    header: Option<FcgiHeader>,
    data: Vec<u8>,
}

impl Request {
    fn new() -> Self {
        Request { header: None, data: Vec::new() }
    }
}

struct Flags{
	new_clients: 			AtomicU64,
	new_requests: 			AtomicU64,
	running_clients:		AtomicU64,
	client_disconnected: 	AtomicU64,
}

type AM<T> = Arc<Mutex<T>>;

////////////////////////////////////////////////////////////////////////////////
// Listener                                                           ////
////////////////////////////////////////////////////////////////////////////////

fn listener_loop(incoming: AM<[AM<[Request; limits::MAX_REQ]>; limits::MAX_CONN]>,
                 outcoming: AM<[AM<Request>; limits::MAX_CONN]>,
                 flags: Arc<Flags>){
    let allowed_ips = fcgi_variables::get_allowed_ips();
    let listener = TcpListener::bind(limits::TCP_LISTENER_ADDR).unwrap();

    for stream in listener.incoming() {
        if let Some(stream) = check_connection(stream, &allowed_ips) {
            let running_flags = flags.running_clients.load(Ordering::SeqCst);

            for i in 0..limits::MAX_CONN {
                if (running_flags & (1 << i)) == 0  {
                    flags.new_clients.fetch_or(1 << i, Ordering::SeqCst);

                    let arg_incoming = incoming[i].clone();
                    let arg_outcoming = outcoming[i].clone();
                    let arg_flag = flags.clone();
                    thread::spawn(move || client_loop(i, stream, arg_incoming, arg_outcoming, arg_flag));

                    break;
                }
            }
        }
    }
}

fn check_connection(stream: Result<TcpStream, std::io::Error>,
                    allowed_ips: &Option<HashSet<IpAddr>>) -> Option<TcpStream> {
    let stream = stream.ok()?;
    
    if let Some(allowed) = allowed_ips {
        let client_ip = stream.peer_addr().ok()?;
        if !allowed.contains(&client_ip.ip()) { return None; }
    }

    Some(stream)
}

////////////////////////////////////////////////////////////////////////////////
// Client handler                                                           ////
////////////////////////////////////////////////////////////////////////////////

fn client_loop( id:         usize,
                socket:     TcpStream,
                incoming:   AM<[Request; limits::MAX_REQ]>,
                outcoming:  Arc<Mutex<Request>>,
                state:      Arc<Flags>){

    // Init //
    {
        let new_client: [Request; limits::MAX_REQ] = makearr(|_| Request::new());
        let mut nested_client = incoming.lock().unwrap();
        *nested_client = new_client;
    }

    state.running_clients.fetch_or(1 << id, Ordering::SeqCst);

    // Loop //
    let last_activity_time = Instant::now();
    

    loop {
        if Instant::now() - last_activity_time > limits::CONNECTION_TIMEOUT { return; }


        {
            let client_data = incoming.lock().unwrap();
            if let Some(req_number) = client_data.iter().position(|req| req.header.is_none()) {

                //client_data[req_number] = Request { header: Some() };
            }
            else{
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
    // Твоя логика обработки
    //println!("Connection accepted from: {:?}", stream.peer_addr());
    //println!("{} {} {}", id, dataRef.read().unwrap().requests[0].data[0], state.new_client.load(Ordering::SeqCst));
}

////////////////////////////////////////////////////////////////////////////////
// Main                                                                     ////
////////////////////////////////////////////////////////////////////////////////

fn main() -> std::io::Result<()> {
    let in_data: Arc<[AM<[Request; limits::MAX_REQ]>; limits::MAX_CONN]> =
        Arc::new(
            makearr(|_| Arc::new(Mutex::new(
                makearr(|_| Request::new())
            ))));

    let out_data: Arc<[Arc<Mutex<Request>>; limits::MAX_CONN]> =
        Arc::new(makearr(|_| Arc::new(Mutex::new(Request::new()))));

    let flags =
        Arc::new(Flags {
            running_clients:        AtomicU64::new(0), 
            new_requests:           AtomicU64::new(0),
            client_disconnected:    AtomicU64::new(0),
            new_clients:            AtomicU64::new(0),
        });

    let arg_in_data = in_data.clone();
    let arg_out_data = out_data.clone();
    let arg_flags = flags.clone();
    thread::spawn(|| listener_loop(arg_in_data, arg_out_data, arg_flags));

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(())
}

/*fn read_header(mut stream: TcpStream) -> Result<FcgiHeader, Error> {
    let mut header = FcgiHeader { bytes: [0; HEADER_LENGTH] };

    let len = stream.read(&mut header.bytes)?;

    if len != HEADER_LENGTH {
        return Err(Error::new(ErrorKind::InvalidData, "Corrupted header"));
    }

    if header.bytes[0] != 1 {
        return Err(Error::new(ErrorKind::InvalidData, "Only FCGI v1 supported"));
    }

    Ok(header)
}*/

/*pub fn handle_client(stream: TcpStream) -> Result<(), Error> {
    let header = read_header(stream)?;



    Ok(())
}*/

/*const BUFFER_LENGTH: usize = 2048;

pub fn handle_client(mut stream: TcpStream) -> Result<[u8; 2048], Error> {
    loop {
        let header = read_header(stream)?;
        // data + pads
        let data_size = usize::from(header.get_content_length() + u16::from(header.get_padding_length()));
        let mut data: Vec<u8> = Vec::new();
        data.reserve(data_size);

        let len = stream.read(data.as_mut_slice())?;

        if len > BUFFER_LENGTH {
            return Err(Error::new(ErrorKind::FileTooLarge, "Buffer owerflow"));
        }

        let slic = &buff[i..len];
        if slic.len() > 0 {
            println!("{slic:?}");
        }

        let begin : [u8; 8] = [1, 6, 0, 1, 0, 30, 2, 0];
        let resp = b"Content-type: text/html\r\n\r\nsos  ";

        stream.write(&begin)?;
        stream.write(resp)?;
    }

    Ok(buff)
}

pub fn handle_client(mut stream: TcpStream) -> Result<[u8; 2048], Error> {
    let mut buff: [u8; BUFFER_LENGTH] = [0; 2048];
    let len = stream.read(&mut buff)?;

    if len > BUFFER_LENGTH {
        return Err(Error::new(ErrorKind::FileTooLarge, "Buffer owerflow"));
    }

    let mut i : usize = 0;
    while i+7 < len {
        let l = buff[i+4];
        let b = buff[i+5];
        let addr = u16::from_be_bytes([buff[i+4], buff[i+5]]);
        println!("l({l}) + b({b}) = {addr}");
        let req =
            FcgiHeaderParsed{
                version         : buff[i],
                r#type          : buff[i+1],
                requestId       : u16::from_be_bytes([buff[i+2], buff[i+3]]),
                contentLength   : u16::from_be_bytes([buff[i+4], buff[i+5]]),
                paddingLength   : buff[i+6],
                reserved        : buff[i+7],
            };

        i += 8;
        //println!("i = {i}");
        let end = i + usize::from(req.contentLength);
        //println!("end = {end}");

        println!("Header\n{req:?}\n");

        let slic = &buff[i..len];
        if slic.len() > 0 {
            println!("{slic:?}");
        }

        i = end;
        i += usize::from(req.paddingLength);
    }


    let slic = &buff[i..len];
    if slic.len() > 0 {
        println!("{slic:?}");
    }

    let begin : [u8; 8] = [1, 6, 0, 1, 0, 30, 2, 0];
    let resp = b"Content-type: text/html\r\n\r\nsos  ";

    stream.write(&begin)?;
    stream.write(resp)?;

    Ok(buff)
}*/
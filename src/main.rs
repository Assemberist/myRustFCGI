use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::io::{Error, ErrorKind};

const BUFFER_LENGTH : usize = 2048;

#[derive(Debug)]
struct FcgiRequest {
    version         : u8,
    r#type          : u8,
    requestId       : u16,
    contentLength   : u16,
    paddingLength   : u8,
    reserved        : u8,
}

fn handle_client(mut stream: TcpStream) -> Result<[u8; 2048], Error> {
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
            FcgiRequest{
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
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9000")?;

    let s = u16::from_be_bytes([3, 29]);
    println!("{s}");

    // accept connections and process them serially
    for stream in listener.incoming() {
        let line = handle_client(stream?)?;
        //println!("{line:?}");
    }
    Ok(())
}

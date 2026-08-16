use std::net::TcpListener;


mod fcgi;
use fcgi::FcgiListener;
use fcgi::RequestReadResult;

use crate::fcgi::FcgiClient;

////////////////////////////////////////////////////////////////////////////////
// Listener                                                                 ////
////////////////////////////////////////////////////////////////////////////////

/*fn listener_loop(mut clients: [RefCell<Option<Client>>; limits::MAX_CONN]){
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
*/

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(fcgi::fcgi_limits::TCP_LISTENER_ADDR)?;
    let socket = listener.accept_fcgi(None)?;

    match socket.read_fcgi() {
        RequestReadResult::CompleteRequest(req) => {
            ()
        }
        RequestReadResult::PartialRequest(part) => {
            ()
        }
        RequestReadResult::Nothing => (),
    }

    Ok(())
}

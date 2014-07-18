#![feature(phase)]
#[phase(plugin, link)] extern crate log;

extern crate rustuv;
extern crate green;
extern crate debug;

use std::io::net::tcp::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::BufferedStream;

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
  green::start(argc, argv, rustuv::event_loop, main)
}

enum Command {
  UpdatePeerTable(std::io::net::ip::IpAddr, u16),
}

fn main() {
  let listener = TcpListener::bind("127.0.0.1", 9000);
  let mut acceptor = listener.listen();

  let (tx, rx) = channel();

  spawn(proc() {
    loop {
      debug!("{:?}", rx.recv());
    }
  });

  for stream in acceptor.incoming() {
    let commands = tx.clone();
    spawn(proc() {
      match stream {
        Ok(conn) => {
          let mut stream = conn;
            let peer = stream.peer_name().unwrap();
            commands.send(UpdatePeerTable(peer.ip, peer.port));
            //let mut echo = BufferedStream::new(conn);
            //let data = echo.read_to_end().unwrap();
            //match echo.write(data.as_slice()) {
            //  Ok(_) => debug!("close"),
            //  Err(e) => error!("{}", e),
            //};
        },
        Err(e) => error!("{}", e),
      }
    });
  }
}

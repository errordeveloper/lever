#![feature(phase)]
#[phase(plugin, link)] extern crate log;

extern crate rustuv;
extern crate green;
extern crate debug;

use std::io::net::tcp::{TcpListener};
use std::io::{Acceptor, Listener};
use std::io::BufferedStream;
use std::io::net::ip::IpAddr;

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
  green::start(argc, argv, rustuv::event_loop, main)
}

enum PeerState {
  Accepted,
  Closed,
}

enum Command {
  UpdatePeerTable(PeerState, IpAddr, u16),
}

fn main() {
  let listener = TcpListener::bind("127.0.0.1", 9000);
  let mut acceptor = listener.listen();

  let (tx, rx): (Sender<Command>, Receiver<Command>) = channel();

  spawn(proc() {
    loop {
      let command = rx.recv();
      debug!("{:?}", command);
      match command {
        UpdatePeerTable(Accepted, ip, port) => {
          info!("Seen {}:{}", ip, port)
        },
        UpdatePeerTable(Closed, ip, port) => {
          info!("Gone {}:{}", ip, port)
        },
      }
    }
  });



  for stream in acceptor.incoming() {
    let command = tx.clone();
    spawn(proc() {
      match stream {
        Ok(conn) => {
          let mut stream = conn;
          let peer = stream.peer_name().unwrap();
          command.send(UpdatePeerTable(Accepted, peer.ip, peer.port));
          //debug!("{:?}", stream);
          let stream = stream;
          let mut echo = BufferedStream::new(stream);
          loop {
            match echo.read_line() {
              Ok(data) => {
                match echo.write(data.as_bytes()) {
                  Ok(_) => echo.flush(),
                  Err(e) => {
                    error!("{}", e);
                    break;
                  },
                };
              },
              Err(e) => {
                error!("{}", e);
                break;
              },
            }
          }
          command.send(UpdatePeerTable(Closed, peer.ip, peer.port));
        },
        Err(e) => error!("{}", e),
      }
    });
  }
}

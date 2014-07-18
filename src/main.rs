#![feature(phase)]
#[phase(plugin, link)] extern crate log;

extern crate rustuv;
extern crate green;
extern crate debug;

use std::io::net::tcp::{TcpListener};
use std::io::{Acceptor, Listener};
use std::io::BufferedStream;
use std::io::net::ip::SocketAddr;
use std::collections::HashMap;

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
  green::start(argc, argv, rustuv::event_loop, main)
}

enum PeerState {
  Accepted,
  Closed,
}

enum Command {
  UpdatePeerTable(PeerState, SocketAddr),
}

struct PeerInfo {
  conn: bool,
  seen: u64,
  gone: u64,
}

fn main() {
  let listener = TcpListener::bind("127.0.0.1", 9000);
  let mut acceptor = listener.listen();

  let (tx, rx): (Sender<Command>, Receiver<Command>) = channel();

  spawn(proc() {
    let mut peers: HashMap<SocketAddr, PeerInfo> = HashMap::new();
    loop {
      match rx.recv() {
        UpdatePeerTable(Accepted, addr) => {
          peers.insert_or_update_with(addr,
              PeerInfo { conn: true, seen: 1, gone: 0 },
              |_, v| { v.conn = true; v.seen += 1; });
          info!("Seen {}:{}", addr.ip, addr.port);
        },
        UpdatePeerTable(Closed, addr) => {
          info!("Gone {}:{}", addr.ip, addr.port);
          // This shouldn't insert, but we could use `find_with_or_insert_with`
          peers.insert_or_update_with(addr,
              PeerInfo { conn: false, seen: 0, gone: 0 },
              |_, v| { v.conn = false; v.gone += 1; });
        },
      }
      debug!("peers={}", peers.len());
    }
  });

  for stream in acceptor.incoming() {
    let command = tx.clone();
    spawn(proc() {
      match stream {
        Ok(conn) => {
          let mut stream = conn;
          let peer = stream.peer_name().unwrap();
          command.send(UpdatePeerTable(Accepted, peer));
          let stream = stream;
          let mut echo = BufferedStream::new(stream);
          loop {
            match echo.read_line() {
              Ok(data) => {
                let _ = match echo.write(data.as_bytes()) {
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
          command.send(UpdatePeerTable(Closed, peer));
        },
        Err(e) => error!("{}", e),
      }
    });
  }
}

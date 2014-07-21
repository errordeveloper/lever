#![feature(phase)]
#[phase(plugin, link)] extern crate log;

extern crate rustuv;
extern crate green;
extern crate debug;

use std::io::net::tcp::TcpListener;
use std::io::{Acceptor, Listener};
use std::io::BufferedStream;
use std::io::net::ip::{SocketAddr, IpAddr};
use std::collections::{HashMap, HashSet};

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
  unique_sockets: HashSet<SocketAddr>,
}

struct StatsInfo {
  connected: u64,
  unique_addresses: u64,
  unique_sockets_per_address: f64,
  //connection_rate: f64,
  //disconnection_rate: f64,
}

fn main() {
  let listener = TcpListener::bind("127.0.0.1", 9000);
  let mut acceptor = listener.listen();

  let (tx, rx): (Sender<Command>, Receiver<Command>) = channel();

  spawn(proc() {
    let mut peers: HashMap<IpAddr, PeerInfo> = HashMap::new();

    let mut stats = StatsInfo {
      connected: 0,
      unique_addresses: 0,
      unique_sockets_per_address: 0.0,
      //connection_rate: 0,
      //disconnection_rate: 0,
    };

    loop {
      match rx.recv() {
        UpdatePeerTable(Accepted, addr) => {
          stats.connected += 1;

          let mut may_be_new = PeerInfo {
            unique_sockets: HashSet::new(),
          };
          may_be_new.unique_sockets.insert(addr);

          let updater = |_: &IpAddr, v: &mut PeerInfo| {
            v.unique_sockets.insert(addr);
          };
          peers.insert_or_update_with(addr.ip, may_be_new, updater);

          stats.unique_addresses = peers.len() as u64;
          let mut u = 0;
          for p in peers.iter() {
            match p {
              (_, info) => u += info.unique_sockets.len(),
            }
          }
          stats.unique_sockets_per_address = (u as u64 / stats.unique_addresses) as f64;
        },
        UpdatePeerTable(Closed, _) => { stats.connected -= 1;
        },
      }
      info!(" // connected:{} unique_addresses:{} unique_sockets_per_address:{}",
            stats.connected, stats.unique_addresses, stats.unique_sockets_per_address);
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

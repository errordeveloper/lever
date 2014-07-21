#![feature(phase)]
#[phase(plugin, link)] extern crate log;

extern crate uuid;
extern crate debug;

extern crate serialize;
#[phase(plugin, link)]
extern crate hammer;

use std::io::net::tcp::TcpStream;
use std::io::BufferedStream;
use uuid::Uuid;

use std::os;
use hammer::decode_args;
use std::clone::Clone;

#[deriving(Decodable, Show)]
struct HandleOpts {
  host: Option<String>,
  port: Option<u16>,
  threads: Option<uint>,
}

hammer_config!(HandleOpts "Handle is an echo client for TCP tuning")

fn main() {
  let opts: HandleOpts = decode_args(os::args().tail()).unwrap();
  let host: String =
    match opts.host {
      None => "127.0.0.1".to_string(),
      Some(h) => h,
    };
  let port: u16 =
    match opts.port {
      None => 9000,
      Some(p) => p,
    };
  let threads: uint =
    match opts.threads {
      None => 25u,
      Some(t) => t,
    };
  debug!("Connecting {} handle to Lever on {}:{}", threads, host, port);

  for t in range(0u, threads) {
    let (host, port) = (host.clone(), port.clone());
    spawn(proc() {
      match TcpStream::connect(host.as_slice(), port) {
        Err(e) => error!("{}", e),
        Ok(conn) => {
          info!("Connected client #{}!", t);
          let mut echo = BufferedStream::new(conn);
          loop {
            let x = Uuid::new_v4().to_urn_str();
            std::io::timer::sleep(1000);
            let _ = match echo.write(x.as_bytes()) {
              Ok(_) => echo.flush(),
              Err(e) => {
                error!("{}", e);
                break;
              },
            };
          }
        },
      }
    });
  }
}

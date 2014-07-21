#![feature(phase)]
#[phase(plugin, link)] extern crate log;

extern crate uuid;
extern crate debug;

use std::io::net::tcp::TcpStream;
use std::io::BufferedStream;
use uuid::Uuid;

fn main() {
  for i in range(0u, 25) {
    spawn(proc() {
      match TcpStream::connect("127.0.0.1", 9000) {
        Err(e) => error!("{}", e),
        Ok(conn) => {
          info!("Connected client #{}!", i);
          let mut echo = BufferedStream::new(conn);
          loop {
            let x = Uuid::new_v4().to_urn_str();
            debug!("{}", x);
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

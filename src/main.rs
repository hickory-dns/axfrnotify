extern crate futures;
extern crate log;
extern crate tokio_core;
extern crate trust_dns;

use std::net::*;
use std::process::exit;

use tokio_core::reactor::Core;

use trust_dns::client::{ClientFuture, ClientHandle};
use trust_dns::error::*;
use trust_dns::op::{Message, ResponseCode};
use trust_dns::rr::domain;
use trust_dns::rr::{DNSClass, RecordType, RecordSet};
use trust_dns::udp::UdpClientStream;

enum ExitCodes {
  NotifySucceeded,
  NotifyFailed,
  FailedToCreateEvenLoop,
  Error,
}

impl From<ExitCodes> for i32 {
  fn from(code: ExitCodes) -> Self {
    match code {
        ExitCodes::NotifySucceeded => 0,
        ExitCodes::NotifyFailed => 1,
        ExitCodes::FailedToCreateEvenLoop => 101,
        ExitCodes::Error => -1,
    }
  }
}

fn main() {
  let secondary = "127.0.0.1:53";
  let domain_name = "central-device.de".to_string();

  let result = notify(secondary, domain_name);
  match result {
    ExitCodes::FailedToCreateEvenLoop => {
      println!("Failed to create event loop.");
    },
    _ => {}
  }

  exit(result.into());
}

fn notify<A: ToSocketAddrs>(addr: A, domain_name: String) -> ExitCodes {
    let io_loop = Core::new();
    if io_loop.is_err() {
      return ExitCodes::FailedToCreateEvenLoop
    };

    let addr: SocketAddr = addr.to_socket_addrs().unwrap().next().unwrap();
    let name = domain::Name::with_labels(domain_name.split('.').map(|x| x.to_string()).collect());
    let message = send_notify(io_loop.unwrap(), addr, name);

    match message {
        Ok(msg) => {
            match msg.get_response_code() {
                ResponseCode::NoError => ExitCodes::NotifySucceeded,
                _ => ExitCodes::NotifyFailed
            }
        },
        Err(_) => {
            ExitCodes::Error
        }
    }
}

fn send_notify(mut io_loop: Core, addr: SocketAddr, name: domain::Name) -> ClientResult<Message> {
  let (stream, sender) = UdpClientStream::new(addr, io_loop.handle());
  let mut client = ClientFuture::new(stream, sender, io_loop.handle(), None);

  let message = io_loop.run(client.notify(name.clone(), DNSClass::IN, RecordType::A, None::<RecordSet>));

  message
}


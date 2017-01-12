//! axfrnotify sends a NOTIFY request to a secondary name server to initiate a zone refresh for a specific domain name.
#![deny(missing_docs)]

extern crate clap;
extern crate futures;
extern crate tokio_core;
extern crate trust_dns;

use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs};
use std::process::exit;

use clap::{Arg, App};

use tokio_core::reactor::Core;

use trust_dns::client::{ClientFuture, ClientHandle};
use trust_dns::error::ClientResult;
use trust_dns::op::{Message, ResponseCode};
use trust_dns::rr::domain;
use trust_dns::rr::{DNSClass, RecordType, RecordSet};
use trust_dns::udp::UdpClientStream;

/// Crate version
static VERSION: &'static str = env!("CARGO_PKG_VERSION");


/// Struct representing the configuration set by command line parameters and defaults
struct Config {
    retries: u16,
    secondary: String,
    port: u16,
    record_type: RecordType,
    domain_name: String,
    verbose: bool,
}


/// Errors which are reported back to the user
#[derive(PartialEq)]
enum ExitCodes {
    Unknown,
    InputError(String),
    NotifySucceeded,
    NotifyFailed,
    FailedToCreateEvenLoop,
    TransportError(String),
}

/// Converts errors to shell exit codes.
impl From<ExitCodes> for i32 {
    fn from(code: ExitCodes) -> Self {
        match code {
            ExitCodes::Unknown => -2,
            ExitCodes::InputError(_) => -1,
            ExitCodes::NotifySucceeded => 0,
            ExitCodes::NotifyFailed => 1,
            ExitCodes::FailedToCreateEvenLoop => 101,
            ExitCodes::TransportError(_) => 102,
        }
    }
}

fn main() {
    let result = match parse_parameters() {
        Ok(config) => {
            if config.verbose { println!("Trying to send notify of type '{:?}' for domain '{}' to secondary '{}:{}' for at most {} times.", config.record_type, config.domain_name, config.secondary, config.port, config.retries+1) }

            let mut result = ExitCodes::Unknown;
            for _ in 0..config.retries+1 {
                result = notify((&config.secondary[..], config.port), &config.record_type, &config.domain_name);
                if config.verbose { print_failure_message(&result) };
                if result == ExitCodes::NotifySucceeded { break };
            }
            result
        },
        Err(result) => {
            print_failure_message(&result);
            result
        }
    };

    exit(result.into());
}

/// Parses the command line parameters and creates a new `Config`.
fn parse_parameters() -> Result<Config, ExitCodes> {
    let app = App::new("axfrnotify")
        .version(VERSION)
        .about("Send an NOTIFY message to a secondary name server to initiate a zone refresh for a specific domain name.")
        .arg(Arg::with_name("retries")
            .takes_value(true)
            .short("r")
            .long("retries")
            .value_name("retries")
            .help("Set the number of retries if notification fails; default is 0"))
        .arg(Arg::with_name("secondary")
            .takes_value(true)
            .short("s")
            .long("secondary")
            .value_name("IP or host name")
            .help("Set the secondary name server to notify; default is 127.0.0.1"))
        .arg(Arg::with_name("port")
            .takes_value(true)
            .short("p")
            .long("port")
            .value_name("port")
            .help("Set the secondary's port; default is 53"))
        .arg(Arg::with_name("record_type")
            .takes_value(true)
            .short("t")
            .long("type")
            .value_name("record_type")
            .help("Set the record type to send (A, AAAA, CHAME, MX, NS, PTR, SOA, SRV, TXT, ANY, AXFR); default is SOA"))
        .arg(Arg::with_name("domain")
            .takes_value(true)
            .required(true)
            .value_name("domain name")
            .help("Domain name to notify about"))
        .arg(Arg::with_name("verbose")
            .short("v")
            .help("Set verbose output"))
        ;
    let cli_args = app.get_matches();

    let verbose = cli_args.is_present("verbose");
    let secondary = cli_args.value_of("secondary").unwrap_or("127.0.0.1");
    let retries = if let Ok(retries) = cli_args.value_of("retries").unwrap_or("0").parse::<u16>() {
        retries
    } else {
        let msg = format!("'{}' is not a valid number of retries.", cli_args.value_of("retries").unwrap());
        return Err(ExitCodes::InputError(msg));
    };
    let port = if let Ok(port) = cli_args.value_of("port").unwrap_or("53").parse::<u16>() {
        port
    } else {
        let msg = format!("'{}' is not a valid port number.", cli_args.value_of("port").unwrap());
        return Err(ExitCodes::InputError(msg));
    };
    let record_type = if let Ok(record_type) = RecordType::from_str(cli_args.value_of("record_type").unwrap_or("SOA")) {
        record_type
    } else {
        let msg = format!("'{}' is not a valid record type.", cli_args.value_of("record_type").unwrap());
        return Err(ExitCodes::InputError(msg));
    };
    let domain_name = cli_args.value_of("domain").unwrap();

    Ok(Config { retries: retries, secondary: secondary.to_string(), port: port, record_type: record_type, domain_name: domain_name.to_string(), verbose: verbose })
}

/// Translates user facing errors into human readable error messages.
fn print_failure_message(result: &ExitCodes) -> () {
    match *result {
        ExitCodes::Unknown => {
            println!("You broke axfrnotify. Now go and fix it.");
        },
        ExitCodes::InputError(ref msg) => {
            println!("Failed to parse input because {}", msg);
        },
        ExitCodes::NotifySucceeded=> {
            println!("Successfully sent notification and received positive response.");
        },
        ExitCodes::NotifyFailed => {
            println!("Successfully sent notification but received negative response.");
        },
        ExitCodes::FailedToCreateEvenLoop => {
            println!("Failed to create event loop.");
        },
        ExitCodes::TransportError(ref msg)=> {
            println!("Failed to send query or receive response because {}.", msg);
        },
    }
}

/// Starts event loop for `Futures` and converts parameters for executing the notification.
fn notify<A: ToSocketAddrs>(addr: A, record_type: &RecordType, domain_name: &str) -> ExitCodes {
    let io_loop = if let Ok(io_loop) = Core::new() {
        io_loop
    } else {
        return ExitCodes::FailedToCreateEvenLoop
    };

    let addr: SocketAddr = addr.to_socket_addrs().unwrap().next().unwrap();
    let name = domain::Name::with_labels(domain_name.split('.').map(|x| x.to_string()).collect());
    let message = send_notify(io_loop, addr, record_type, name);

    match message {
        Ok(ref msg) if msg.get_response_code() == ResponseCode::NoError => ExitCodes::NotifySucceeded,
        Ok(_) => ExitCodes::NotifyFailed,
        Err(err) => ExitCodes::TransportError(err.description().to_string()),
    }
}

/// Actually sends the notification request.
fn send_notify(mut io_loop: Core, addr: SocketAddr, record_type: &RecordType, name: domain::Name) -> ClientResult<Message> {
    let (stream, sender) = UdpClientStream::new(addr, io_loop.handle());
    let mut client = ClientFuture::new(stream, sender, io_loop.handle(), None);

    io_loop.run(client.notify(name.clone(), DNSClass::IN, *record_type, None::<RecordSet>))
}


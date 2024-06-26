use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, process::ExitCode};

use clap::{ArgAction, Parser};
use client::Client;
use server::Server;

mod client;
mod lib;
mod server;
mod states;

/// A small program to act as a server or client in a game of shut the box.
#[derive(Parser)]
#[command(about, version)]
pub struct CliArgs {
    /// Whether the program should act as a server or client  
    /// Pass "server" for a server & "client" for a client
    role: String,

    /// The IP address to connect to
    #[arg(short = 'a', long = "ip")]
    ip_address: Option<IpAddr>,

    /// The port to connect over
    #[arg(short = 'p', long = "port", default_value_t = 3333)]
    port: u16,

    /// Debug mode, don't enable this unless you're me
    #[arg(short = 'd', long = "debug", default_value_t = false, action=clap::ArgAction::SetTrue)]
    debug: bool,
}

fn main() -> ExitCode {
    let args = CliArgs::parse();
    
    if args.debug {
        println!("-- In debug mode --");
        // Loopback socket address
        let loopback_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3333);
        Server::new(loopback_socket).start();
    }

    // If no IP was given prompt for one
    let ip_address = match args.ip_address {
        Some(val) => {val},
        None => {lib::get_ip_input()},
    };

    let socket_address = SocketAddr::new(ip_address, args.port);

    match args.role.to_ascii_lowercase().as_str() {
        "server" => {
            println!("Starting server on {}:{}", socket_address.ip(), socket_address.port());
            Server::new(socket_address).start()
        }
        "client" => {
            println!("Starting client");
            Client::new(socket_address).start()
        }
        _ => {
            println!("Invalid arg, must be either \"server\" or \"client\". Exiting");
            ExitCode::FAILURE
        }
    }
}

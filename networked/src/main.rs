use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use clap::Parser;

mod client_states;
mod server_state;
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

fn main() {
    let args = CliArgs::parse();

    if args.debug {
        println!("-- In debug mode --");
        // Loopback socket address
        let loopback_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3333);
        server_state::start(loopback_socket);
    }

    // If no IP was given prompt for one
    let ip_address = match args.ip_address {
        Some(val) => val,
        None => networked::get_ip_input(),
    };

    let socket_address = SocketAddr::new(ip_address, args.port);

    match args.role.to_ascii_lowercase().as_str() {
        "server" => {
            println!(
                "Starting server on {}:{}",
                socket_address.ip(),
                socket_address.port()
            );
            server_state::start(socket_address);
        }
        "client" => {
            println!("Starting client");
            client_states::start(socket_address);
        }
        _ => {
            println!("Invalid arg, must be either \"server\" or \"client\". Exiting");
        }
    }
}

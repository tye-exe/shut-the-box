use std::{
    fmt::Debug,
    io::{self, ErrorKind, Read, Write},
    net::{IpAddr, TcpStream},
    sync::mpsc,
    thread,
};

use serde::{de::DeserializeOwned, Serialize};

pub const ETX: char = 0b00000011 as char;

pub fn get_ip_input() -> IpAddr {
    // Loops until valid IP is given
    loop {
        // Prompts for IP address to connect to
        print!("IP to connect to: ");
        io::stdout().flush().expect("Cannot write text to stdout.");

        // Reads the given IP
        let mut data = String::new();
        io::stdin()
            .read_line(&mut data)
            .expect("Cannot read from stdin");

        // Validates IP
        match data.trim().parse::<IpAddr>() {
            Ok(ip_address) => return ip_address,
            Err(_) => {
                eprintln!("Invalid IP. Please try again.")
            }
        }
    }
}

pub fn get_port_input() -> u16 {
    // Loops until valid port is given
    loop {
        // Prompts for port address to connect to
        print!("Port to connect on: ");
        io::stdout().flush().expect("Cannot write text to stdout.");

        // Reads the given IP
        let mut data = String::new();
        io::stdin()
            .read_line(&mut data)
            .expect("Cannot read from stdin");

        // Validates IP
        match data.trim().parse::<u16>() {
            Ok(port) => return port,
            Err(_) => {
                eprintln!("Invalid port. Please try again.")
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ChannelError {
    #[error("`{0}`")]
    BadPacket(String),
    #[error("Error when reading packet: {0}")]
    ReadError(ErrorKind),
}

/// A wrapper struct that receives data from a connection of type T & sends data down a connection of type V
#[derive(Debug)]
pub struct Channels<T, V>
where
    T: DeserializeOwned + Debug + Send,
    V: Serialize + Debug + Send,
{
    pub reading: mpsc::Receiver<Result<T, ChannelError>>,
    pub writing: mpsc::Sender<V>,
}

/// Creates a [`Channels`] struct, which can be used to send and receive data over the given tcp_stream.
pub fn initialize_channels<T, V>(tcp_stream: TcpStream) -> Channels<T, V>
where
    T: DeserializeOwned + Debug + Send + 'static,
    V: Serialize + Debug + Send + 'static,
{
    let peer_addr: String = match tcp_stream.peer_addr() {
        Ok(addr) => addr.to_string(),
        Err(_) => "Unknown".to_string(),
    };

    let (read_sender, read_receiver) = mpsc::channel();
    let (write_sender, write_receiver) = mpsc::channel();

    let mut read_stream = tcp_stream.try_clone().expect("Cannot clone tcp stream.");
    let mut write_stream = tcp_stream;

    // Reading thread
    thread::Builder::new()
        .name(format!("reading for {peer_addr}"))
        .spawn(move || {
            'outer: loop {
                let mut data = Vec::new();

                // Reads until end of message (ETX char is sent)
                loop {
                    let mut buffer = [0u8; 1];

                    match read_stream.read_exact(&mut buffer) {
                        Ok(_) => {}
                        Err(e) if e.kind() == ErrorKind::UnexpectedEof => {}
                        Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                        Err(e) => {
                            if read_sender
                                .send(Err(ChannelError::ReadError(e.kind())))
                                .is_err()
                            {
                                eprintln!("Couldn't send fatal error to self.")
                            };
                            eprintln!("Reading dropped: {} {}", e, e.kind());
                            break 'outer;
                        }
                    };

                    // This char equals end of message.
                    if buffer[0] as char == ETX {
                        break;
                    }

                    data.push(buffer[0] as char);
                }

                let message = String::from_iter(data.iter());
                println!("{}", message);
                let client_message = match serde_yml::from_str(&message).ok() {
                    Some(parsed_packet) => Ok(parsed_packet),
                    None => Err(ChannelError::BadPacket(message)),
                };

                // When the receiver is dropped the thread should terminate
                if read_sender.send(client_message).is_err() {
                    eprintln!("Reading dropped");
                    break;
                };

                // Clears data buffer
                data.clear();
            }
        })
        .expect("Wasn't able to create reading thread");

    // Writing thread
    thread::Builder::new()
        .name(format!("writing for {peer_addr}"))
        .spawn(move || {
            loop {
                let received = write_receiver.recv();
                // When the sender is dropped the thread should terminate
                if received.is_err() {
                    eprintln!("Writer dropped");
                    break;
                }

                let data_to_send = received.unwrap();
                let mut data_to_send = serde_yml::to_string(&data_to_send)
                    .expect("Couldn't serializes Client Message to send.");

                // Adds char for end of message
                data_to_send.push(ETX);

                let write_res = write_stream.write_all(data_to_send.as_bytes());

                if let Err(e) = write_res {
                    eprintln!("Writer dropped: {e}");
                    break;
                }
            }
        })
        .expect("Wasn't able to create writing thread");

    // Wrapper struct
    Channels {
        reading: read_receiver,
        writing: write_sender,
    }
}

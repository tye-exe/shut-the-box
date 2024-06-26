use std::{
    io::{self, Write},
    net::{IpAddr, TcpStream},
    sync::mpsc,
};

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

struct Channels {
    reading: mpsc::Receiver<Result<ClientMessages, Box<ServerError>>>,
    writing: mpsc::Sender<ServerMessages>,
}

pub fn initialize_client(tcp_stream: TcpStream) -> ClientChannels {
    let (read_sender, read_receiver) = mpsc::channel();
    let (write_sender, write_receiver) = mpsc::channel();

    let mut read_stream = tcp_stream.try_clone().expect("Cannot clone tcp stream.");
    let mut write_stream = tcp_stream;

    // Reading thread
    thread::spawn(move || loop {
        let mut data = String::new();

        // Reads the data the client sent from the stream
        let client_message: Result<ClientMessages, Box<ServerError>> = read_stream
            .read_to_string(&mut data)
            .map_err(|_| {
                Box::new(ServerError::BadClientPacket {
                    client_message: (Option::None),
                })
            })
            // Parses the string into a Client Message
            .and_then(|_| {
                serde_yml::from_str(&data).map_err(|_| {
                    Box::new(ServerError::BadClientPacket {
                        client_message: (Some(data.clone())),
                    })
                })
            });

        println!("{:?}", client_message);

        // When the receiver is dropped the thread should terminate
        if read_sender.send(client_message).is_err() {
            break;
        };
    });

    // Writing thread
    thread::spawn(move || loop {
        let received = write_receiver.recv();
        // When the sender is dropped the thread should terminate
        if received.is_err() {
            break;
        }

        let data_to_send: ServerMessages = received.unwrap();
        let data_to_send = serde_yml::to_string(&data_to_send)
            .expect("Couldn't serializes Client Message to send.");

        write_stream
            .write_all(data_to_send.as_bytes())
            .expect("Couldn't send data to Client.");
    });

    // Wrapper struct
    Channels {
        reading: read_receiver,
        writing: write_sender,
    }
}

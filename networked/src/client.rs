use std::{
    error::Error,
    fmt::Display,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    process::{ExitCode, Termination},
    sync::mpsc::{self, TryRecvError},
    thread,
};

use mac_address2::MacAddress;
use networked::ETX;
use serde::de::value;

use crate::states::{self, ClientMessages, ServerMessages};

#[derive(Debug)]
enum ClientError {
    BadServerPacket { server_message: Option<String> },
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "no")
    }
}

impl Error for ClientError {}

enum ClientState {
    Terminate(ExitCode),
    Joining { mac_address: MacAddress },
    WaitingJoinResponse,
    WaitingGameStart { ready: bool },
}

/// Wrapper to hold the read & write threads for a client.
struct ServerChannels {
    reading: mpsc::Receiver<Result<ServerMessages, Box<ClientError>>>,
    writing: mpsc::Sender<ClientMessages>,
}

pub struct Client {
    connection: ServerChannels,
    client_state: ClientState,
}

impl Client {
    pub fn new(socket_address: SocketAddr) -> Client {
        let connection = TcpStream::connect(socket_address)
            .expect("Couldn't connect to server. Did you give the correct address?");

        let mac_address = mac_address2::get_mac_address()
            .expect("Couldn't get Mac address.")
            .expect("Couldn't get Mac address");

        Client {
            connection: Self::threads(connection),
            client_state: ClientState::Joining { mac_address },
        }
    }

    pub fn start(mut self) -> ExitCode {
        loop {
            match &self.client_state {
                ClientState::Terminate(code) => return *code,
                ClientState::Joining { mac_address } => self.connect(*mac_address),
                ClientState::WaitingJoinResponse => self.connect_response(),
                ClientState::WaitingGameStart { .. } => todo!(),
            }
        }
    }

    fn term(&mut self, code: ExitCode, message: &str) {
        print!("{}", message);
        self.client_state = ClientState::Terminate(code);
    }

    fn threads(tcp_stream: TcpStream) -> ServerChannels {
        tcp_stream.set_read_timeout(None).unwrap();
        tcp_stream.set_write_timeout(None).unwrap();
        let (read_sender, read_receiver) = mpsc::channel();
        let (write_sender, write_receiver) = mpsc::channel();

        let mut read_stream = tcp_stream.try_clone().expect("Cannot clone tcp stream.");
        let mut write_stream = tcp_stream;

        // Reading thread
        thread::spawn(move || {
            {
                let mut data = String::new();
                loop {
                    read_stream.read()
                    // Reads the data the client sent from the stream
                    let client_message: Result<ServerMessages, Box<ClientError>> = read_stream
                        .read_to_string(&mut data)
                        .map_err(|_| {
                            Box::new(ClientError::BadServerPacket {
                                server_message: (Option::None),
                            })
                        })
                        // Parses the string into a Client Message
                        .and_then(|_| {
                            serde_yml::from_str(&data).map_err(|_| {
                                Box::new(ClientError::BadServerPacket {
                                    server_message: (Some(data.clone())),
                                })
                            })
                        });

                    // When the receiver is dropped the thread should terminate
                    if read_sender.send(client_message).is_err() {
                        break;
                    };
                }
            };
        });

        // Writing thread
        thread::spawn(move || loop {
            let received = write_receiver.recv();
            // When the sender is dropped the thread should terminate
            if received.is_err() {
                break;
            }

            let data_to_send: ClientMessages = received.unwrap();
            let mut data_to_send = serde_yml::to_string(&data_to_send)
                .expect("Couldn't serializes Client Message to send.");

            // Adds char for ascci ETX to signal end of transmission.
            data_to_send.push(ETX);

            write_stream
                .write_all(data_to_send.as_bytes())
                .expect("Couldn't send data to Client.");
        });

        // Wrapper struct
        ServerChannels {
            reading: read_receiver,
            writing: write_sender,
        }
    }

    fn connect(&mut self, mac_address: MacAddress) {
        let opt_in = ClientMessages::OptInForPlaying(mac_address);
        self.connection.writing.send(opt_in).unwrap();
        self.client_state = ClientState::WaitingJoinResponse;
        println!("Sent join request.")
    }

    fn connect_response(&mut self) {
        let received_response = match self.connection.reading.try_recv() {
            Ok(value) => value,
            Err(TryRecvError::Empty) => {
                return;
            }
            Err(TryRecvError::Disconnected) => {
                self.term(ExitCode::FAILURE, "Server disconnect.");
                return;
            }
        };

        match received_response {
            Ok(ServerMessages::OptInAccept) => {
                println!("Joined game.")
            }
            Ok(ServerMessages::OptInDeny) => {
                self.term(ExitCode::FAILURE, "Server declined join.");
            }
            Ok(..) | Err(_) => {
                self.term(ExitCode::FAILURE, "Server sent bad packet.");
            }
        }
    }
}

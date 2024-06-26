use fastrand::Rng;
use mac_address2::MacAddress;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::process::ExitCode;
use std::sync::mpsc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use crate::server::ServerState::ListeningForClients;
use crate::states::ClientMessages;
use crate::states::ServerMessages;

#[derive(Debug)]
enum ServerError {
    BadClientPacket { client_message: Option<String> },
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "no")
    }
}

impl Error for ServerError {}

/// Contains all the different states the server could be in.
enum ServerState {
    ListeningForClients {
        to_process: Vec<ClientChannels>,
        connected: u8,
        ready: u8,
    },
}

impl PartialEq for ServerState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ListeningForClients { .. }, Self::ListeningForClients { .. }) => true,
        }
    }
}

/// Wrapper to hold the read & write threads to a client.
struct ClientChannels {
    reading: mpsc::Receiver<Result<ClientMessages, Box<ServerError>>>,
    writing: mpsc::Sender<ServerMessages>,
}

pub struct Server {
    // Internal state
    server_state: ServerState,

    // Networking
    listener: TcpListener,

    rng: Rng,
    client_connections: HashMap<MacAddress, ClientChannels>,
}

impl Server {
    /// Constructs a new server to oversee a game.
    pub fn new(socket_address: SocketAddr) -> Server {
        let listener = TcpListener::bind(socket_address)
            .expect("Unable to bind to given address. Is it already in use?");

        Server {
            server_state: ListeningForClients {
                to_process: Default::default(),
                connected: 0,
                ready: 0,
            },
            listener,
            rng: Rng::new(),
            client_connections: HashMap::new(),
        }
    }

    /// Starts the server.
    pub fn start(mut self) -> ExitCode {
        // The server will use it's internal state to determine what action it should perform.
        loop {
            match &self.server_state {
                ListeningForClients { .. } => {
                    self.listening_for_clients();
                    self.register_client();
                }
            }

            sleep(Duration::from_millis(100))
        }
    }

    fn listening_for_clients(&mut self) {
        self.listener
            .set_nonblocking(true)
            .expect("Cannot set non-blocking.");

        let client_channels: ClientChannels = match self.listener.accept() {
            Ok((stream, _addr)) => self.initialize_client(stream),

            // If it's `WouldBlock` then there is no connection to handle.
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => return,

            Err(err) => {
                eprintln!("Listening for client connection failed: {err}");
                panic!()
            }
        };

        // Adds the clients to queue for receiving their mac address before full assignment
        match &mut self.server_state {
            ListeningForClients {
                to_process: vec, ..
            } => {
                vec.push(client_channels);
            }
            _ => unreachable!("Must be of variant ListeningForClients"),
        };
    }

    fn initialize_client(&mut self, tcp_stream: TcpStream) -> ClientChannels {
        let (read_sender, read_receiver) = mpsc::channel();
        let (write_sender, write_receiver) = mpsc::channel();

        let mut read_stream = tcp_stream.try_clone().expect("Cannot clone tcp stream.");
        let mut write_stream = tcp_stream;

        // Reading thread
        thread::spawn(move || loop {
            let mut data = String::new();

            read_stream.read

            // Reads the data the client sent from the stream
            // let client_message: Result<ClientMessages, Box<ServerError>> = read_stream
            //     .read_to_string(&mut data)
            //     .map_err(|_| {
            //         Box::new(ServerError::BadClientPacket {
            //             client_message: (Option::None),
            //         })
            //     })
            //     // Parses the string into a Client Message
            //     .and_then(|_| {
            //         serde_yml::from_str(&data).map_err(|_| {
            //             Box::new(ServerError::BadClientPacket {
            //                 client_message: (Some(data.clone())),
            //             })
            //         })
            //     });

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
        ClientChannels {
            reading: read_receiver,
            writing: write_sender,
        }
    }

    fn register_client(&mut self) {
        // Adds the clients to queue for receiving their mac address before full assignment
        let (clients_to_process, connected_clients) = match &mut self.server_state {
            ListeningForClients {
                to_process,
                connected,
                ..
            } => (to_process, connected),
            _ => unreachable!("Must be of variant ListeningForClients"),
        };

        // Stores the indices of the clients to drop.
        let mut to_remove = Vec::new();
        // Stores the indices of the clients to add.
        let mut to_add = Vec::new();

        for (index, client) in clients_to_process.iter().enumerate() {
            // Tries to read the message from a client
            let received = match client.reading.try_recv() {
                Ok(val) => val,
                e => {
                    println!("{:?}", e);
                    continue;
                }
            };

            // Only an OptIn message is accepted currently.
            match received {
                Ok(val) => {
                    if let ClientMessages::OptInForPlaying(mac_address) = val {
                        to_add.push((index, mac_address));
                        continue;
                    }

                    eprintln!(
                        "A client sent a bad packet, dropping client. Packet: {:?}",
                        val
                    );
                    to_remove.push(index);
                    continue;
                }
                Err(err) => {
                    eprintln!("A client sent a bad packet, dropping client. Packet: {err}");
                    to_remove.push(index);
                    continue;
                }
            };
        }

        // Registers valid clients
        for to_add in to_add {
            let client_channels = clients_to_process.swap_remove(to_add.0);
            client_channels
                .writing
                .send(ServerMessages::OptInAccept)
                .expect("Couldn't accept client");

            self.client_connections.insert(to_add.1, client_channels);
            *connected_clients += 1;
        }

        // Drops the clients that sent bad packets
        for index_to_remove in to_remove {
            let removed_client = clients_to_process.remove(index_to_remove);
            removed_client
                .writing
                .send(ServerMessages::OptInDeny)
                .expect("Couldn't gracefully disconnect from client.");
        }
    }
}

struct Client {}

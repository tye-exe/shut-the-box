use fastrand::Rng;
use mac_address2::MacAddress;
use networked::Channels;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::process::ExitCode;
use std::thread::sleep;
use std::time::Duration;

use crate::server::ServerState::ListeningForClients;
use crate::states::ClientMessages;
use crate::states::ServerMessages;

#[derive(thiserror::Error, Debug)]
enum ServerError {
    #[error("Client sent a back packaet: {client_message:?}")]
    BadClientPacket { client_message: Option<String> },
}

/// Contains all the different states the server could be in.
enum ServerState {
    ListeningForClients {
        to_process: Vec<Channels<ClientMessages, ServerMessages>>,
        previous_connected: u32,
        previous_ready: u32,
    },
}

impl PartialEq for ServerState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ListeningForClients { .. }, Self::ListeningForClients { .. }) => true,
        }
    }
}

pub struct Server {
    // Internal state
    server_state: ServerState,

    // Networking
    listener: TcpListener,

    rng: Rng,
    client_connections: HashMap<MacAddress, Client>,
}

impl Server {
    /// Constructs a new server to oversee a game.
    pub fn new(socket_address: SocketAddr) -> Server {
        let listener = TcpListener::bind(socket_address)
            .expect("Unable to bind to given address. Is it already in use?");

        Server {
            server_state: ListeningForClients {
                to_process: Default::default(),
                previous_connected: 0,
                previous_ready: 0,
            },
            listener,
            rng: Rng::new(),
            client_connections: HashMap::new(),
        }
    }

    /// Starts the server.
    // pub fn start(mut self) -> ExitCode {
    //     // The server will use it's internal state to determine what action it should perform.
    //     loop {
    //         match self.server_state {
    //             ListeningForClients {
    //                 ref mut to_process,
    //                 ref mut previous_connected,
    //                 ref mut previous_ready,
    //             } => {
    //                 self.listening_for_clients(to_process);

    //                 self.register_client(to_process);
    //                 self.clients_ready(to_process, previous_connected, previous_ready);
    //             }
    //         }

    //         sleep(Duration::from_millis(100))
    //     }
    // }

    fn listening_for_clients(
        self,
        clients_to_process: &mut Vec<Channels<ClientMessages, ServerMessages>>,
    ) {
        self.listener
            .set_nonblocking(true)
            .expect("Cannot set non-blocking.");

        let client_channels = match self.listener.accept() {
            Ok((stream, _addr)) => networked::initialize_channels(stream),

            // If it's `WouldBlock` then there is no connection to handle.
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => return,

            Err(err) => {
                eprintln!("Listening for client connection failed: {err}");
                panic!()
            }
        };

        clients_to_process.push(client_channels);
    }

    fn register_client(
        &self,
        clients_to_process: &mut Vec<Channels<ClientMessages, ServerMessages>>,
    ) {
        // Adds the clients to queue for receiving their mac address before full assignment
        // let clients_to_process = match &mut self.server_state {
        //     ListeningForClients { to_process, .. } => to_process,
        //     _ => unreachable!("Must be of variant ListeningForClients"),
        // };

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
                Some(val) => {
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
                None => {
                    eprintln!("A client sent a bad packet, dropping client.");
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

            // self.client_connections
            //     .insert(to_add.1, Client::new(client_channels));
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

    fn clients_ready(
        &self,
        clients_to_process: &mut Vec<Channels<ClientMessages, ServerMessages>>,
        previous_connected: &mut u32,
        previous_ready: &mut u32,
    ) {
        let connected = self.client_connections.len() as u32;

        let ready = self
            .client_connections
            .values()
            .try_fold(0, |acc, e| match e.state {
                ClientStates::Start { ready } => Ok(acc + ready as u32),
                _ => Err(()),
            })
            .ok()
            .expect("A client wasn't in the start state");

        if ready == connected {};

        if connected != *previous_connected || ready != *previous_ready {};
    }
}

struct Client {
    connection: Channels<ClientMessages, ServerMessages>,
    state: ClientStates,
}

enum ClientStates {
    Start { ready: bool },
    Playing,
}

impl Client {
    fn new(connection: Channels<ClientMessages, ServerMessages>) -> Client {
        Client {
            connection,
            state: ClientStates::Start { ready: false },
        }
    }
}

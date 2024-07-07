use core::panic;
use std::{
    any::Any,
    net::{SocketAddr, TcpListener},
    sync::mpsc::{SendError, TryRecvError},
};

use networked::Channels;
type Channel = Channels<ClientMessages, ServerMessages>;

use crate::states::{ClientMessages, ServerMessages};

#[derive(Debug, thiserror::Error)]
enum ServerError {
    #[error("Server closed channels unexpectedly")]
    ChannelsClosed(#[from] SendError<Box<dyn Any>>),
}

pub fn start(socket_addr: SocketAddr) -> ! {
    let mut server = Server::new(socket_addr);
    loop {
        server.listen();
        server.register_client();
        server.clients_ready()
    }
}

struct Server<S> {
    listener: TcpListener,
    clients: Vec<Channel>,
    state: S,
}

struct Listening {
    previous_connected: u32,
    previous_ready: u32,
    to_accept: Vec<Channel>,
    accepted: Vec<(Channel, bool)>,
}

struct Playing {}

impl<S> Server<S> {
    fn write_to_all(&self, server_message: ServerMessages) {
        for channel in &self.clients {
            let send = channel.writing.send(server_message.clone());
            // if send.is_err() {
            //     eprintln!("Failed to send message to a client");
            //     todo!("Drop bad client")
            // };
            match send {
                Ok(_) => {}
                Err(e) => {
                    println!("{e}");
                    panic!("AAAAAAAAA")
                }
            }
        }
        // Ok(())
    }
}

impl Server<Listening> {
    fn new(socket_address: SocketAddr) -> Self {
        let listener = TcpListener::bind(socket_address)
            .expect("Unable to bind to given address. Is it already in use?");

        Server {
            listener,
            clients: Vec::new(),
            state: Listening {
                previous_connected: 0,
                previous_ready: 0,
                to_accept: Vec::new(),
                accepted: Vec::new(),
            },
        }
    }

    fn listen(&mut self) {
        self.listener
            .set_nonblocking(true)
            .expect("Cannot set non-blocking.");

        let client_channels = match self.listener.accept() {
            Ok((stream, _addr)) => networked::initialize_channels(stream),

            // If it's `WouldBlock` then there is no connection to handle.
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return,

            Err(err) => {
                eprintln!("Listening for client connection failed: {err}");
                return;
            }
        };

        self.state.to_accept.push(client_channels);
    }

    fn register_client(&mut self) {
        // Stores the indices of the clients to drop.
        let mut to_remove = Vec::new();
        // Stores the indices of the clients to add.
        let mut to_add = Vec::new();

        for (index, client) in self.state.to_accept.iter().enumerate() {
            let received = match client.reading.try_recv() {
                Ok(val) => val,
                Err(e) => match e {
                    TryRecvError::Empty => continue,
                    TryRecvError::Disconnected => panic!("AAAAAAAAA"),
                },
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
                Err(e) => {
                    eprintln!("A client sent a bad packet, dropping client: {e}");
                    to_remove.push(index);
                    continue;
                }
            };
        }

        // Registers valid clients
        for to_add in to_add {
            let client_channels = self.state.to_accept.swap_remove(to_add.0);
            client_channels
                .writing
                .send(ServerMessages::OptInAccept)
                .expect("Couldn't accept client");

            self.clients.push(client_channels);
            println!("Added client: {}", to_add.1)
        }

        // Drops the clients that sent bad packets
        for index_to_remove in to_remove {
            let removed_client = self.state.to_accept.remove(index_to_remove);
            removed_client
                .writing
                .send(ServerMessages::OptInDeny)
                .expect("Couldn't gracefully disconnect from client.");
        }
    }
    fn clients_ready(&mut self) {
        let connected = self.clients.len() as u32;

        let ready = self
            .state
            .accepted
            .iter()
            .fold(0, |acc, channel| acc + channel.1 as u32);

        // Inform clients of new player connented/ready amount
        if connected != self.state.previous_connected {
            self.write_to_all(ServerMessages::PlayersConnected(connected as u8))
        };
        if ready != self.state.previous_ready {
            self.write_to_all(ServerMessages::PlayersReady(ready as u8))
        };

        // Starts the game
        if ready == connected && connected != 0 {
            todo!("Make game start features :P")
        };
    }
}

// #[cfg(test)]
// mod tests {
//     use std::{
//         net::{Ipv4Addr, TcpStream},
//         thread,
//         time::Duration,
//     };

//     use anyhow::Ok;

//     use super::*;

//     // fn create_socket() -> SocketAddr {

//     // }

//     fn create_server() -> anyhow::Result<Channels<ServerMessages, ClientMessages>> {
//         let socket = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000);
//         thread::spawn(move || start(socket));
//         let channels = networked::initialize_channels(TcpStream::connect(socket)?);
//         anyhow::Ok(channels)
//     }

//     #[test]
//     fn joining() -> anyhow::Result<()> {
//         let channels = create_server()?;

//         channels
//             .writing
//             .send(ClientMessages::OptInForPlaying(MacAddress::default()))
//             .unwrap();

//         let recv: ServerMessages = channels
//             .reading
//             .recv_timeout(Duration::from_secs(5))
//             .unwrap()
//             .unwrap();

//         assert_eq!(recv, ServerMessages::OptInAccept);
//         Ok(())
//     }

//     // #[test]
//     // fn closing_connection() -> anyhow::Result<()> {
//     //     let channels = create_server()?;
//     //     channels
//     //         .writing
//     //         .send(ClientMessages::OptInForPlaying(MacAddress::default()));

//     //     let recv: ServerMessages = channels
//     //         .reading
//     //         .recv_timeout(Duration::from_secs(5))
//     //         .unwrap()
//     //         .unwrap();

//     //     drop(channels);
//     //     drop(recv);

//     //     let channels = create_server()?;

//     //     channels
//     //         .writing
//     //         .send(ClientMessages::OptInForPlaying(MacAddress::default()));
//     //     anyhow::Ok(())
//     // }
// }

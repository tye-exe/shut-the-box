use std::{
    net::{SocketAddr, TcpStream},
    sync::mpsc::{RecvError, SendError},
};

use mac_address2::MacAddress;
use networked::{ChannelError, Channels};

use crate::states::{ClientMessages, ServerMessages};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Server closed client-senting channel unexpectedly")]
    WriteClosed(#[from] SendError<ClientMessages>),
    #[error("Server closed client-reading channel unexpectedly")]
    ReadClosed(#[from] RecvError),
    #[error("Server sent malformed packet: {0}")]
    MalformedPacket(#[from] ChannelError),
    #[error("Server responded with unexpected packet: {0:?}")]
    UnexpectedPacket(ServerMessages),
}

pub fn start(socket_address: SocketAddr) -> Result<(), ClientError> {
    let client = Client::new(socket_address);
    client.connect()?;
    if !client.connect_allowed()? {
        println!("Connection refused.");
        return Ok(());
    };

    let client = Client::<PreGame>::from(client);
    // client.

    Ok(())
}

struct Client<S> {
    connection: Channels<ServerMessages, ClientMessages>,
    state: S,
}

#[derive(Clone, Copy)]
struct Joining {
    // server_address: SocketAddr,
    mac_address: MacAddress,
}

struct PreGame {
    ready: bool,
}

impl<S> Client<S> {
    fn write(&self, client_message: ClientMessages) -> Result<(), ClientError> {
        self.connection.writing.send(client_message)?;
        Ok(())
    }

    fn read(&self) -> Result<ServerMessages, ClientError> {
        match self.connection.reading.recv() {
            Err(e) => Err(e.into()),
            Ok(value) => Ok(value?),
        }
    }
}

impl Client<Joining> {
    fn new(socket_address: SocketAddr) -> Self {
        let connection = TcpStream::connect(socket_address)
            .expect("Couldn't connect to server. Did you give the correct address?");

        let mac_address = mac_address2::get_mac_address()
            .expect("Couldn't get Mac address.")
            .expect("Couldn't get Mac address");

        Client {
            connection: networked::initialize_channels(connection),
            state: Joining {
                // server_address: socket_address,
                mac_address,
            },
        }
    }

    fn connect(&self) -> Result<(), ClientError> {
        let opt_in = ClientMessages::OptInForPlaying(self.state.mac_address);
        self.write(opt_in)?;
        println!("Sent join request.");
        Ok(())
    }

    fn connect_allowed(&self) -> Result<bool, ClientError> {
        match self.read()? {
            ServerMessages::OptInAccept => Ok(true),
            ServerMessages::OptInDeny => Ok(false),
            packet => Err(ClientError::UnexpectedPacket(packet)),
        }
    }
}

impl From<Client<Joining>> for Client<PreGame> {
    fn from(value: Client<Joining>) -> Self {
        todo!()
    }
}

impl Client<PreGame> {}

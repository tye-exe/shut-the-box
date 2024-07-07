use std::{
    error::Error,
    fmt::Display,
    net::{SocketAddr, TcpStream},
    process::ExitCode,
    sync::mpsc::TryRecvError,
};

use mac_address2::MacAddress;
use networked::Channels;

use crate::states::{ClientMessages, ServerMessages};

#[derive(thiserror::Error, Debug)]
enum ClientError {
    #[error("Server sent a back packaet: {server_message:?}")]
    BadServerPacket { server_message: Option<String> },
}

enum ClientState {
    Terminate(ExitCode),
    Joining { mac_address: MacAddress },
    WaitingJoinResponse,
    WaitingGameStart { ready: bool },
}

pub struct Client {
    connection: Channels<ServerMessages, ClientMessages>,
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
            connection: networked::initialize_channels(connection),
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
            Some(ServerMessages::OptInAccept) => {
                println!("Joined game.")
            }
            Some(ServerMessages::OptInDeny) => {
                self.term(ExitCode::FAILURE, "Server declined join.");
            }
            Some(..) | None => {
                self.term(ExitCode::FAILURE, "Server sent bad packet.");
            }
        }
    }
}

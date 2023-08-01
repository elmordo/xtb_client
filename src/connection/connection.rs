use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use thiserror::Error;
use tungstenite::Message;

use crate::connection::socket::{Socket, SocketError};
use crate::connection::SocketConfig;

pub struct ServerConnection {}


impl ServerConnection {
    pub fn new(socket_config: SocketConfig, stream_socket_config: SocketConfig) -> Result<Self, ServerConnectionError> {
        let socket = Socket::new(socket_config)?;
        let stream_socket = Socket::new(stream_socket_config)?;
        todo!()
    }
}


fn wrap_socket(socket: Socket) -> (Sender<SocketControlCommand>, Receiver<SocketNotification>, JoinHandle<()>) {
    let mut socket = socket;
    let (input_sender, input_receiver) = channel();
    let (output_sender, output_receiver) = channel();
    let handle = thread::spawn(move || {
        loop {
            if let Ok(cmd) = input_receiver.try_recv() {
                match cmd {
                    SocketControlCommand::Stop => break,
                    SocketControlCommand::Send(msg) => socket.send(msg)
                        .or_else(|err| {
                            output_sender.send(SocketNotification::SocketError(err))
                        }).unwrap()
                }
            }
        }
        output_sender.send(SocketNotification::Stopped).unwrap();
    });
    (input_sender, output_receiver, handle)
}

enum SocketControlCommand {
    Stop,
    Send(Message),
}

enum SocketNotification {
    Stopped,
    SocketError(SocketError),
}


#[derive(Debug, Error)]
pub enum ServerConnectionError {
    #[error("Socket error")]
    SocketError(SocketError)
}


impl From<SocketError> for ServerConnectionError {
    fn from(value: SocketError) -> Self {
        Self::SocketError(value)
    }
}

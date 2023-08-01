use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use thiserror::Error;
use tungstenite::Message;

use crate::connection::socket::{Socket, SocketError};
use crate::connection::SocketConfig;

pub struct ServerConnection {
    api_service: SocketServiceHandle,
    stream_api_service: SocketServiceHandle
}


impl ServerConnection {
    pub fn new(socket_config: SocketConfig, stream_socket_config: SocketConfig) -> Result<Self, ServerConnectionError> {
        let api_service = Self::make_socket_service(socket_config)?;
        let stream_api_service = Self::make_socket_service(stream_socket_config)?;
        Ok(Self {
            api_service,
            stream_api_service,
        })
    }

    fn make_socket_service(socket_config: SocketConfig) -> Result<SocketServiceHandle, ServerConnectionError> {
        let socket = Socket::new(socket_config)?;
        Ok(SocketServiceHandle::new(socket))
    }
}


struct SocketServiceHandle {
    input: Sender<SocketControlCommand>,
    output: Receiver<SocketNotification>,
    join: JoinHandle<()>
}


impl SocketServiceHandle {
    fn new(socket: Socket) -> Self {
        let (mut service, input, output) = SocketService::new(socket);
        let join = thread::spawn(move || {
            service.start();
        });
        Self {
            input,
            output,
            join,
        }
    }
}


struct SocketService {
    socket: Socket,
    command_input: Receiver<SocketControlCommand>,
    notification_output: Sender<SocketNotification>,
    stop_flag: bool,
}


impl SocketService {
    fn new(socket: Socket) -> (Self, Sender<SocketControlCommand>, Receiver<SocketNotification>) {
        let (cmd_sender, cmd_receiver) = channel();
        let (notification_sender, notification_receiver) = channel();
        let instance = Self {
            socket,
            command_input: cmd_receiver,
            notification_output: notification_sender,
            stop_flag: false,
        };
        (instance, cmd_sender, notification_receiver)
    }

    fn start(&mut self) {
        loop {
            self.receive_messages();
            self.process_commands();
            if self.stop_flag {
                break;
            }
        }
        self.notification_output.send(SocketNotification::Stopped).unwrap();
    }

    fn process_commands(&mut self) {
        while let Ok(cmd) = self.command_input.try_recv() {
            match cmd {
                SocketControlCommand::Stop => self.stop(),
                SocketControlCommand::Send(msg) => self.send_message(msg),
            }
        }
    }

    fn stop(&mut self) {
        self.stop_flag = true;
    }

    fn send_message(&mut self, message: Message) {
        match self.socket.send(message) {
            Err(err) => self.send_notification(SocketNotification::SocketError(err)),
            _ => ()
        }
    }

    fn receive_messages(&mut self) {

    }

    fn send_notification(&mut self, notification: SocketNotification) {
        self.notification_output.send(notification).unwrap();
    }
}


enum SocketControlCommand {
    Stop,
    Send(Message),
}

enum SocketNotification {
    Stopped,
    Message(Message),
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

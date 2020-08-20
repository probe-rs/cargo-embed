use std::fmt::Debug;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;

use serde::{de::DeserializeOwned, Serialize};
use tungstenite::{accept, Error, HandshakeError, Message, WebSocket};

/// The `Updater` trait specifies an interface for a statemachine updater.
/// An `Updater` is basically a self contained unit that runs asynchronously and pushes/receives events to/from mpscs.
pub trait Updater {
    /// Starts the `Updater`.
    /// This should never block and run the `Updater` asynchronously.
    fn start<I, O>(&mut self) -> UpdaterChannel<I, O>
    where
        I: DeserializeOwned + Send + Sync + Debug + 'static,
        O: Serialize + Send + Sync + Debug + 'static;
    /// Stops the `Updater` if currently running.
    /// Returns `Ok` if everything went smooth during the run of the `Updater`.
    /// Returns `Err` if something went wrong during the run of the `Updater`.
    fn stop(&mut self) -> Result<(), ()>;
}

/// A complete channel to an updater.
/// Rx and tx naming is done from the user view of the channel, not the `Updater` view.
pub struct UpdaterChannel<I, O>
where
    I: DeserializeOwned + Send + Sync + Debug + 'static,
    O: Serialize + Send + Sync + Debug + 'static,
{
    /// The rx where the user reads data from.
    rx: Receiver<I>,
    /// The tx where the user sends data to.
    tx: Sender<O>,
}

impl<I, O> UpdaterChannel<I, O>
where
    I: DeserializeOwned + Send + Sync + Debug + 'static,
    O: Serialize + Send + Sync + Debug + 'static,
{
    /// Creates a new `UpdaterChannel` where crossover is done internally.
    ///
    /// The argument naming is done from the `Updater`s view. Where as the member naming is done from a user point of view.
    pub fn new(rx: Sender<O>, tx: Receiver<I>) -> Self {
        Self { rx: tx, tx: rx }
    }

    /// Returns the rx end of the channel.
    pub fn rx(&mut self) -> &mut Receiver<I> {
        &mut self.rx
    }

    /// Returns the tx end of the channel.
    pub fn tx(&mut self) -> &mut Sender<O> {
        &mut self.tx
    }
}

/// An updater which receives and sends it's updates from and to a websocket.
/// It supports concurrent connections from multiple clients and handles disconnects and errors gracefully.
pub struct WebsocketUpdater {
    connection_string: String,
    thread_handle: Option<(JoinHandle<()>, Sender<()>)>,
}

impl WebsocketUpdater {
    /// Creates a new websocket updater.
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
            thread_handle: None,
        }
    }

    /// Writes a message to all connected websockets and removes websockets that are no longer connected.
    fn write_to_all_sockets<I>(sockets: &mut Vec<(WebSocket<TcpStream>, SocketAddr)>, update: &I)
    where
        I: Serialize + Send + Sync + Debug + 'static,
    {
        let mut to_remove = vec![];
        for (i, (socket, addr)) in sockets.iter_mut().enumerate() {
            match socket.write_message(Message::Text(serde_json::to_string(update).unwrap())) {
                Ok(_) => (),
                Err(Error::ConnectionClosed) => {
                    log::info!("Socket connection to {} was closed", addr);
                    to_remove.push(i);
                }
                Err(tungstenite::Error::Io(err)) => {
                    if err.kind() == std::io::ErrorKind::WouldBlock {
                    } else {
                        log::error!(
                            "Writing to websocket at {} experienced an error: {:?}",
                            addr,
                            err
                        )
                    }
                }
                Err(err) => log::error!(
                    "Writing to websocket at {} experienced an error: {:?}",
                    addr,
                    err
                ),
            }
        }

        // Remove all closed websockets.
        for i in to_remove.into_iter().rev() {
            sockets.swap_remove(i);
        }
    }

    /// Reads all messages from all connected websockets and removes websockets that are no longer connected.
    fn read_from_all_sockets<I>(
        sockets: &mut Vec<(WebSocket<TcpStream>, SocketAddr)>,
        sender: Sender<I>,
    ) where
        I: DeserializeOwned + Send + Sync + Debug + 'static,
    {
        let mut to_remove = vec![];
        for (i, (socket, addr)) in sockets.iter_mut().enumerate() {
            match socket.read_message() {
                Ok(msg) => match msg {
                    // For now we handle text messages only.
                    Message::Text(string) => match serde_json::from_str(&string) {
                        Ok(update) => {
                            log::debug!("Parsed JSON: {:#?}", update);
                            let _ = sender.send(update);
                        }
                        Err(error) => {
                            log::error!("Failed to parse JSON: {:#?}", error);
                        }
                    },
                    _ => (),
                },
                Err(tungstenite::Error::ConnectionClosed) => {
                    log::info!("Socket connection to {} was closed", addr);
                    to_remove.push(i);
                }
                Err(tungstenite::Error::Io(err)) => {
                    if err.kind() == std::io::ErrorKind::WouldBlock {
                    } else {
                        log::error!(
                            "Reading from websocket at {} experienced an error: {:?}",
                            addr,
                            err
                        )
                    }
                }
                Err(err) => log::error!(
                    "Reading from websocket at {} experienced an error: {:?}",
                    addr,
                    err
                ),
            }
        }

        // Remove all closed websockets.
        for i in to_remove.into_iter().rev() {
            sockets.swap_remove(i);
        }
    }
}

impl Updater for WebsocketUpdater {
    fn start<I, O>(&mut self) -> UpdaterChannel<I, O>
    where
        I: DeserializeOwned + Send + Sync + Debug + 'static,
        O: Serialize + Send + Sync + Debug + 'static,
    {
        let mut sockets = Vec::new();

        let (rx, inbound) = channel::<O>();
        let (outbound, tx) = channel::<I>();
        let (halt_tx, halt_rx) = channel::<()>();

        log::info!("Opening websocket on '{}'", self.connection_string);
        let server = TcpListener::bind(&self.connection_string).unwrap();
        server.set_nonblocking(true).unwrap();

        self.thread_handle = Some((
            spawn(move || {
                let mut incoming = server.incoming();
                loop {
                    // If a halt was requested, cease operations.
                    if halt_rx.try_recv().is_ok() {
                        return ();
                    }

                    // Handle new incomming connections.
                    match incoming.next() {
                        Some(Ok(stream)) => {
                            // Assume we always get a peer addr, so this unwrap is fine.
                            let addr = stream.peer_addr().unwrap();
                            // Try accepting the websocket.
                            match accept(stream) {
                                Ok(mut websocket) => {
                                    // Make sure we operate in nonblocking mode.
                                    // Is is required so read does not block forever.
                                    websocket.get_mut().set_nonblocking(true).unwrap();
                                    log::info!("Accepted a new websocket connection from {}", addr);
                                    sockets.push((websocket, addr));
                                }
                                Err(HandshakeError::Interrupted(_)) => {}
                                Err(HandshakeError::Failure(err)) => log::error!(
                                    "Accepting a new websocket experienced an error: {:?}",
                                    err
                                ),
                            }
                        }
                        Some(Err(err)) => {
                            if err.kind() == std::io::ErrorKind::WouldBlock {
                            } else {
                                log::error!(
                                    "Connecting to a websocket experienced an error: {:?}",
                                    err
                                )
                            }
                        }
                        None => {
                            log::error!("The TCP listener iterator was exhausted. Shutting down websocket listener.");
                            return ();
                        }
                    }

                    // Read at max one new message from each socket.
                    Self::read_from_all_sockets(&mut sockets, outbound.clone());

                    // Send at max one pending message to each socket.
                    match inbound.try_recv() {
                        Ok(update) => {
                            Self::write_to_all_sockets(&mut sockets, &update);
                        }
                        _ => (),
                    }

                    // Pause the current thread to not use CPU for no reason.
                    sleep(Duration::from_micros(100));
                }
            }),
            halt_tx,
        ));

        UpdaterChannel::new(rx, tx)
    }

    fn stop(&mut self) -> Result<(), ()> {
        let thread_handle = self.thread_handle.take();
        match thread_handle.map(|h| {
            // If we have a running thread, send the request to stop it and then wait for a join.
            // If this unwrap fails the thread has already been destroyed.
            // This cannot be assumed under normal operation conditions. Even with normal fault handling this should never happen.
            // So this unwarp is fine.
            h.1.send(()).unwrap();
            h.0.join()
        }) {
            Some(Err(err)) => {
                log::error!("An error occured during thread execution: {:?}", err);
                Err(())
            }
            _ => Ok(()),
        }
    }
}

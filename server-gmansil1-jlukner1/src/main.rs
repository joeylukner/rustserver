mod connection;
pub mod error;
pub mod frame;
pub mod rw;

use connection::Connection;
use error::ReadError;

use std::io::{self, Write};

use crate::{
    frame::Frame::{self, *},
    rw::WriteFrame,
};

use std::{collections::VecDeque, net::TcpListener};

fn nonblocking<T>(res: io::Result<T>) -> io::Result<Option<T>> {
    match res {
        Ok(value) => Ok(Some(value)),
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
        Err(e) => Err(e),
    }
}

pub struct Server {
    pub lst: Vec<Connection>,
    pub listener: TcpListener,
    pub messages: VecDeque<Vec<u8>>,
    pub next_id: u32,
}

impl Server {
    pub fn new(addr: &str) -> io::Result<Server> {
        let listen = TcpListener::bind(addr)?;
        listen.set_nonblocking(true)?;
        Ok(Server {
            lst: Vec::new(),
            listener: listen,
            messages: VecDeque::new(),
            next_id: 0,
        })
    }
    pub fn enqueue_message(&mut self, fr: &Frame<'_>) {
        let mut vec: Vec<u8> = Vec::new();
        vec.write_frame(fr).unwrap();
        self.messages.push_back(vec);
    }
    pub fn accept_clients(&mut self) -> io::Result<()> {
        while let Some((stream, addr)) = nonblocking(self.listener.accept())? {
            stream.set_nonblocking(true)?;
            match Connection::establish(stream, self.next_id) {
                Some(con) => {
                    self.enqueue_message(&Slice(&[Simple("JOIN"), Simple(&con.name)]));
                    self.lst.push(con);
                    self.next_id += 1;
                }
                None => {
                    println!("{addr} timed out during connection")
                }
            }
        }
        Ok(())
    }
    pub fn is_pending(e: &ReadError) -> bool {
        match e {
            ReadError::Parse(_) => false,
            ReadError::Io(e) => e.kind() == io::ErrorKind::WouldBlock,
        }
    }
    pub fn is_exhausted(e: &ReadError) -> bool {
        match e {
            ReadError::Parse(_) => false,
            ReadError::Io(e) => e.kind() == io::ErrorKind::WriteZero,
        }
    }
    pub fn handle_frame(&mut self, name: &str, fr: &Frame<'_>) {
        match fr.as_array() {
            Some([Frame::Simple("MSG"), Frame::Simple(message)]) => {
                self.enqueue_message(&Slice(&[
                    Simple("MSG"),
                    Slice(&[Simple(name), Simple(": "), Simple(message)]),
                ]))
            }

            _ => println!("Not a message, sorry!"),
        }
    }
    pub fn read_frames(&mut self) {
        let mut connections = std::mem::take(&mut self.lst);

        connections.retain_mut(|conn| match conn.reader.read_frame() {
            Ok(guard) => {
                self.handle_frame(&conn.name, &guard.frame());
                true
            }
            Err(e) if Server::is_pending(&e) => true,
            Err(e) => {
                if Server::is_exhausted(&e) {
                    self.enqueue_message(&Slice(&[Simple("LEAVE"), Simple(&conn.name)]));
                    false
                } else {
                    println!("This disconnection wasn't intentional!");
                    self.enqueue_message(&Slice(&[Simple("LEAVE"), Simple(&conn.name)]));
                    false
                }
            }
        });

        self.lst = connections;
    }
    pub fn broadcast_messages(&mut self) -> io::Result<()> {
        while let Some(message) = self.messages.pop_front() {
            for connection in self.lst.iter_mut() {
                connection.reader.get_mut().write_all(&message)?;
            }
        }
        Ok(())
    }
}
fn main() -> io::Result<()> {
    let ip = local_ip_address::local_ip().unwrap();

    println!("IP: {ip:?}");

    let mut server = Server::new(&format!("{ip}:6379"))?;
    // The event loop is literally a loop
    loop {
        server.accept_clients()?;
        server.read_frames();
        server.broadcast_messages()?;
    }
}

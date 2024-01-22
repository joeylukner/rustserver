use crate::{frame::Frame::*, rw::FrameReader};

use std::{net::TcpStream, thread::sleep, time::Duration};

pub struct Connection {
    pub reader: FrameReader<TcpStream>,
    pub name: String,
    pub id: u32,
}

impl Connection {
    pub fn establish(reader: TcpStream, id: u32) -> Option<Self> {
        let mut reader = FrameReader::new(reader);

        sleep(Duration::from_millis(50));

        let name =
            if let [Simple("JOIN"), Simple(name)] = reader.read_frame().ok()?.frame().as_array()? {
                name.to_string()
            } else {
                return None;
            };

        Some(Connection { reader, name, id })
    }
}

use std::io::Write;
use std::net::TcpStream;

pub struct Queue {
    pub connection: TcpStream,
}

impl Queue {
    pub fn new(addr: &str) -> Result<Self, std::io::Error> {
        let connection = TcpStream::connect(addr)?;

        Ok(Self { connection })
    }

    pub fn publish(&mut self, message: &[u8]) {
        self.connection.write(&[0]).unwrap();
        self.connection.write(message).unwrap();
    }

    pub fn subscribe(&mut self) {
        self.connection.write(&[1]).unwrap();
    }
}

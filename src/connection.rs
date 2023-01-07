use tokio::net::TcpStream;
use bytes::BytesMut;
use anyhow::Result;
use tracing::info;
use crate::frame::Frame;

#[derive(Debug)]
pub struct Connection {
    stream:TcpStream,
    buff : BytesMut
}

impl Connection {
    pub fn new(stream :TcpStream) -> Self {
        Self {
            stream,
            buff:BytesMut::with_capacity(1024 * 4)
        }
    }

    pub async fn read_frame(&self) -> Result<Option<Frame>> {

        Ok(None)
    }

    pub fn parse_frame(&mut self) -> Result<()> {
        
    }
}
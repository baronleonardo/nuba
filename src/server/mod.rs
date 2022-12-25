use std::{collections::VecDeque, net::SocketAddr};
use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt} };
use super::message::Message;

mod error;
use error::ServerError;

pub struct Server
{
    listener: TcpListener,
    stream: Option<TcpStream>,
    addr: Option<SocketAddr>,
    queue: VecDeque<Message>
}

impl Server
{
    const BUF_SIZE: usize = 8192;
    const QUEUE_SIZE: usize = 10;

    pub async fn new(addr: &str) -> tokio::io::Result<Server>
    {
        let listener = TcpListener::bind(addr).await?;

        let queue = VecDeque::<Message>::with_capacity(Self::QUEUE_SIZE);
        let server = Server { listener, stream: None, addr: None, queue };
        Ok(server)
    }

    pub async fn accept(&mut self) -> tokio::io::Result<()>
    {
        if let Some(stream) = self.stream.as_mut()
        {
            stream.shutdown().await?;
        }

        let (stream, addr) = self.listener.accept().await?;
        self.stream = Some(stream);
        self.addr = Some(addr);

        Ok(())
    }

    pub async fn read(&mut self) -> Result<usize, ServerError>
    {
        match self.stream.as_mut()
        {
            Some(stream) => {
                let mut buf: Vec<u8> = vec![0; Self::BUF_SIZE];
                let streamed_bytes = stream.read_buf(&mut buf).await?;
                self.queue.push_back(Message::from_buf(buf).await?);
                Ok(streamed_bytes)
            },

            None => Err(
                ServerError::IO(
                    tokio::io::Error::new(tokio::io::ErrorKind::NotConnected, "Not connected")
                )
            )
        }
    }

    pub async fn parse(&mut self)
    {

    }
}
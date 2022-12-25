use std::net::SocketAddr;
use tokio::{net::{TcpListener, TcpStream}, io::{BufReader, AsyncReadExt, AsyncWriteExt}};

mod server;
mod message;

use server::Server;

// async fn handle_socket(stream: TcpStream, _socket_address: SocketAddr)
// {
//     const BUF_SIZE: usize = 8192;
//     const EOF: u8 = 0;

//     tokio::spawn(async move{
//         let mut buf_reader = BufReader::new(stream);
//         let mut buf = vec![0; BUF_SIZE];

//         loop
//         {
//             match buf_reader.read_buf(&mut buf).await {
//                 Ok(0) => continue,
//                 Ok(1) if buf[0] == EOF => return,
//                 Ok(_) => print!("{}", String::from_utf8(buf.clone()).unwrap_or_default()),
//                 Err(e) => {
//                     eprintln!("failed to read from socket; err = {:?}", e);
//                     return;
//                 }
//             };
//         }
//     });
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>>
// {
//     let listener = TcpListener::bind("127.0.0.1:8080").await?;

//     loop {
//         let (socket, socket_address) = listener.accept().await?;
//         println!("-- New Connection: {}", socket_address.to_string());
//         handle_socket(socket, socket_address).await;
//     }
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let server = Server::new("localhost:8080").await?;

    Ok(())
}
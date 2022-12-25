use std::{net::{TcpListener, TcpStream}, io::{BufReader, Read}};

fn handle_client(stream: TcpStream)
{
    let mut buf_reader = BufReader::new(stream);
    let mut buf: Vec<u8> = Vec::with_capacity(8192);

    loop
    {
        buf_reader.read(&mut buf).unwrap();
        println!("{}", String::from_utf8_lossy(&buf));
    }
}

fn main() -> std::io::Result<()>
{
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    // accept connections and process them serially
    for stream in listener.incoming()
    {
        handle_client(stream?);
    }
    Ok(())
}
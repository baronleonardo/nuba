use nuba::server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let server = Server::new("127.0.0.1:3000").await?;
    Ok(server.run().await?)
}


pub mod dominion;
pub mod network;

use dominion::bookmaker::*;
use network::ws_client::*;

#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    let stream = connect().await?;
    let mut bookmaker = Bookmaker::new(stream);
    bookmaker.listen().await?;    
    Ok(())
}

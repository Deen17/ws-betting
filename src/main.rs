

pub mod dominion;
pub mod network;
pub mod utils;

use utils::logger::*;
use dominion::bookmaker::*;
use network::ws_client::*;

#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    initialize_logging(Some("log/log.log")).unwrap();
    let stream = connect().await?;
    let mut bookmaker = Bookmaker::new(stream);
    bookmaker.listen().await?;    
    Ok(())
}



pub mod dominion;
pub mod network;

use dominion::vote::*;
use network::ws_client::*;

#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    let stream = connect().await?;
    let mut voting_system = VotingSystem::new(stream);
    voting_system.listen().await?;    
    Ok(())
}

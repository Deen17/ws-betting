

use crate::network::{
    ws_client::*,
    message::*
};
use tokio::stream::StreamExt;
use tokio_tungstenite::tungstenite::Message as tMessage;
use serde_json;

pub struct VotingSystem {
    ws: WSStream
}

impl VotingSystem{
    pub fn new(ws: WSStream) -> Self{
        Self{ws}
    }

    pub async fn listen(mut self) -> Result<() , Box<dyn std::error::Error + Send + Sync>>{
        while let Some(msg) = self.ws.next().await {
            let msg = msg?;
            if let tMessage::Text(data) = msg{
                let (msg_type, payload) = data.split_at(data.find(" ").unwrap());
                match msg_type {
                    "NAMES" => {
                        let dict: NamesDict = serde_json::from_str(payload)?;
                        println!("NAMES:\n{:#?}", dict);
                        // TODO: populate active users with entries in dict
                    },
                    _ => {println!("{}", payload);}
                };
            }
        }
        Ok(())
    }
}
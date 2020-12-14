

use crate::network::{
    ws_client::*,
    message::*
};
use crate::dominion::user::User;
use tokio::stream::StreamExt;
use tokio_tungstenite::tungstenite::Message as tMessage;
use serde_json;
use std::collections::HashMap;

pub struct VotingSystem {
    ws: WSStream,
    users: HashMap<String, User>
}

impl VotingSystem{
    pub fn new(ws: WSStream) -> Self{
        Self{
            ws,
            users: HashMap::new()
        }
    }

    pub async fn listen(&mut self) -> Result<() , Box<dyn std::error::Error + Send + Sync>>{
        while let Some(msg) = self.ws.next().await {
            let msg = msg?;
            if let tMessage::Text(data) = msg{
                let (msg_type, payload) = data.split_at(data.find(" ").unwrap());
                match msg_type {
                    "NAMES" => {
                        let dict: NamesDict = serde_json::from_str(payload)?;
                        dict.users
                            .into_iter()
                            .for_each(|UserData::One{nick,features}| {
                                self.users.insert(nick.clone(), UserData::One{nick,features}.into());
                            });
                        println!("NAMES:\n{:#?}", self.users);
                    },
                    _ => {println!("{}", data);}
                };
            }
        }
        Ok(())
    }
}
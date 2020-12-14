

use crate::network::{
    ws_client::*,
    message::*
};
use crate::dominion::user::User;
use tokio::stream::{StreamExt};
use futures_util::sink::SinkExt;
use tokio_tungstenite::tungstenite::Message as tMessage;
use serde_json;
use std::collections::HashMap;
use crate::dominion::fs::*;
use log::*;

pub struct Bookmaker {
    ws: WSStream,
    users: HashMap<String, User>,
    bets: HashMap<String, (usize,usize)>
}

impl Bookmaker{
    pub fn new(ws: WSStream) -> Self{
        Self{
            ws,
            users: HashMap::new(),
            bets: HashMap::new()
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
                    },
                    "JOIN"=> {
                        let join_msg: JoinDict = serde_json::from_str(payload)?;
                        self.users.insert(join_msg.nick.clone(),  join_msg.into());
                    },
                    "QUIT" =>{
                        let join_msg: QuitDict = serde_json::from_str(payload)?;
                        self.users.remove(&join_msg.nick);
                    }
                    "PRIVMSG" => {
                        let pm: PrivateMessage = serde_json::from_str(payload)?;
                        println!("pm: {:?}\n", pm);
                        self.private_command(pm.nick,pm.data.as_str()).await?;
                    }
                    _ => {println!("{}", data);}
                };
            }
        }
        Ok(())
    }

    async fn private_command(&mut self, nick: String, msg: &str)
    -> Result<() , Box<dyn std::error::Error + Send + Sync>>
    {
        let (msg_type, payload) = msg
            .split_at(
                msg.find(|c| c == ' ' || c == '\n' || c == '\0').unwrap_or(msg.len()));
        match msg_type.to_ascii_lowercase().as_str(){
            "points" => {
                let points: usize = get_points(nick.as_str())?;
                let res = format!("Your points: {}", points);
                info!("{}", res);
                self.send("PRIVMSG", &res).await?;
            }
            "help" => {
                let res = "Commands: help,points,vote <amount>";
                info!("{}", res);
                self.send("PRIVMSG", res).await?;
            }
            "bet" => {
                // handle if bet is not going on right now

                // handle payload
                let points: usize = get_points(&nick)?;
                // TODO: handle bets
                let (_choice, current_bet) = self.bets.get(&nick).unwrap_or(&(0,0));
                let res = match payload.trim().parse::<usize>(){
                    Ok(amt) if amt + current_bet > points =>{
                        "You tried to bet too many points."
                    }
                    Ok(amt) if amt > 0=>{
                        *self.bets.entry(nick).or_insert(0) += amt;
                        "Bet Placed!"
                    }
                    _ => {
                        "Error parsing <amount>. Should be a positive integer"
                    }
                };
                self.send("PRIVMSG", res).await?;
                println!("{}", res);
            }
            _ => {
                self.send("PRIVMSG", "Error: Unknown Command. Try using \"help\"").await?;
                println!("Error: Unknown Command. Try using \"help\"");
            }
        };

        Ok(())
    }


    async fn send(&mut self, msg_type: &str, payload: &str)
    -> std::result::Result<(), tokio_tungstenite::tungstenite::Error>{
        let msg = format!("{} {}", msg_type, payload);
        self.ws.send(tMessage::text(msg)).await
    }
}
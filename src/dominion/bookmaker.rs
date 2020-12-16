

use crate::network::{
    ws_client::*,
    message::*,
    config::NICK
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
    bets: HashMap<String, (usize,usize)>,
    in_progress: bool
}

impl Bookmaker{
    pub fn new(ws: WSStream) -> Self{
        Self{
            ws,
            users: HashMap::new(),
            bets: HashMap::new(),
            in_progress: false
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
        let mut iter = payload.trim().split(' ').take(2);
        let choice_str = iter.next();
        let amt_str = iter.next();
        let res: String = match msg_type.to_ascii_lowercase().as_str(){
            "points" => {
                let points: usize = get_points(nick.as_str())?;
                format!("Your points: {}", points)
            }
            "help" => {
                "Hi FeelsOkayMan I'm a bookkeeper for betting in D.GG, but right now, I'm still a work in progress. Commands: help,points,bet <1 or 2> <amount>,odds, (if privileged) start, (if privileged) cancel, (if privileged) call <1 or 2>".into()
            }
            "odds" if self.in_progress => "No bets in progress.".into(),
            "odds" if self.bets.len() == 0 => "No bets yet.".into(),
            "odds" => {
                format!("{}",self.odds())
            }
            "start" if self.in_progress => {
                "betting already in progress".into()
            }
            "start" => {
                "unimplemented".into()
            }
            "cancel" => {
                "unimplemented".into()
            }
            "call" if !self.in_progress=> {
                "Betting not in progress".into()
            }
            "call" if choice_str.is_none() ||amt_str.is_none() | 
                choice_str.unwrap().parse::<usize>().is_err() |
                (choice_str.unwrap().parse::<usize>().unwrap() > 2)
                => "Usage: call <1 or 2>".into(),
            "call" => {
                let winner = choice_str.unwrap().parse::<usize>()?;

                "unimplemented".into()
            }
            "bet" if !self.in_progress => "Bet not currently in progress.".into(),
            "bet" if choice_str.is_none() ||amt_str.is_none() | 
                (choice_str.unwrap().parse::<usize>().is_err() || amt_str.unwrap().parse::<usize>().is_err()) |
                (choice_str.unwrap().parse::<usize>().unwrap() > 2)
                => "Usage: bet <1 or 2> <amt: positive integer>".into(),
            "bet"  => {
                // format: bet <choice> <amt>
                let amt: usize = amt_str.unwrap().parse()?;
                let choice: usize = choice_str.unwrap().parse()?;
                let points: usize = get_points(&nick)?;
                // TODO: handle bets
                let (old_choice, current_bet) = self.bets.get(&nick).unwrap_or(&(0,0));

                if old_choice != &choice {
                    "You can't change your bet once it's placed".into()
                } 
                else if amt + current_bet > points {
                    "You tried to bet too many points.".into()
                }
                else if amt > 0{
                    let (_, cur) =self.bets.entry(nick.clone()).or_insert((choice, 0));
                    *cur += amt;
                    "Bet Placed!".into()
                }
                else {
                    "Error placing bet".into()
                }
            }
            _ => {
                "Error: Unknown Command. Try using help".into()
            }
        };

        self.send("PRIVMSG" , &format!("{{\"nick\":\"{}\",\"data\":\"{}\"}}",nick,res)).await?;
        Ok(())
    }


    async fn send(&mut self, msg_type: &str, payload: &str)
    -> std::result::Result<(), tokio_tungstenite::tungstenite::Error>{
        let msg = format!("{} {}", msg_type, payload);
        self.ws.send(tMessage::text(msg)).await
    }
}
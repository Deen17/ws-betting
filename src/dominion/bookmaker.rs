

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
use commitlog::*;
use std::io::Result as ioResult;

pub struct Bookmaker {
    ws: WSStream,
    users: HashMap<String, User>,
    bets: HashMap<String, (usize,usize)>,
    commits: CommitLog,
    in_progress: bool
}

impl Bookmaker{
    pub fn new(ws: WSStream) -> Self{
        Self{
            ws,
            users: HashMap::new(),
            bets: HashMap::new(),
            commits: create_commit_log(),
            in_progress: false
        }
    }

    fn odds(&self) -> (f32,f32){
        let one: usize = self.bets.values().filter(|(choice, _)| *choice == 1).map(|(_,amt)| amt).sum();
        let two: usize = self.bets.values().filter(|(choice, _)| *choice == 2).map(|(_,amt)| amt).sum();
        if self.bets.is_empty() || one == 0 || two  == 0 {return (0f32,0f32)}
        let total = one+two;
        (
            (total / one) as f32,
            (total / two) as f32
        )
    }

    fn cancel(&mut self) {
        self.bets.clear();
        self.in_progress = false;
    }

    /// assumes that a query for nick will always work
    fn points<S>(&self, nick: S) -> usize
        where S: Into<String>  {
        self.users.get(&nick.into()).map(|user| user.points).unwrap()
    }

    // winner should only be 1 or 2
    fn payout(&mut self, winner: usize) -> ioResult< Vec<(String, usize)>> {
        let odds = self.odds();
        if odds.0 == 0f32 || odds.1 == 0f32 {
            self.cancel();
            return Err( 
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "no bets placed yet"));
        }
        let mut results: Vec<(String, usize)> = Vec::with_capacity(self.bets.len());
        self
            .bets
            .iter()
            .filter(|(_, (choice, _))| *choice == winner)
            .for_each(|(nick, (_, bet))| results.push((nick.clone(), self.points(nick) + (*bet as f32 * if winner == 1 {odds.0} else {odds.1}) as usize)));      // TODO: fix the entire system to use f32 or f64?
        self
            .bets
            .iter()
            .filter(|(_, (choice, _))| *choice != winner)
            .for_each(|(nick, (_, bet))| results.push((nick.clone(), self.points(nick).checked_sub(*bet).unwrap_or(0))));     // TODO: fix the entire system to use f32 or f64?
        let encoded: String = serde_json::to_string(&results).unwrap();
        self.commits.append_msg(encoded).unwrap();
        for (nick, val) in results.iter(){
            set_points(nick,*val)?;
        }
        Ok(results)
    }

    pub fn payout_commit(&mut self) {
        if let Some(latest_offset) = self.commits.last_offset(){
            let results: Vec<(String, usize)> = serde_json::from_str(&String::from_utf8_lossy(&self.commits.read(latest_offset, ReadLimit::default()).unwrap().into_bytes())).unwrap();
            for (nick, val) in results.iter(){
                set_points(nick,*val).unwrap(); // can fail if a nick does not exist in points directory
            }
        }
    }

    pub async fn listen(&mut self) -> Result<() , Box<dyn std::error::Error + Send + Sync>>{
        // first, do the last commit.
        self.payout_commit();
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
                // format!("{}",self.odds())
                let odds = self.odds();
                format!("Odds: {}:{}", odds.0, odds.1)
            }
            "start" if self.in_progress => {
                "betting already in progress".into()
            }
            "start" => {
                self.in_progress =  true;
                "Betting has started".into()
            }
            "cancel" => {
                self.cancel();
                "Betting has been cancelled".into()
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
                let response: String = match self.payout(winner) {
                    Ok(results) => {
                        for (better, val) in results.iter() {
                            let bet: &(usize, usize) = self.bets.get(better).unwrap();
                            let odds = self.odds();
                            let winning_odds = if winner == 1 {odds.0} else {odds.1};
                            let indiv_res: String = 
                                format!("You {} {} points. You now have {} points.",
                                if bet.0 == winner {"won"} else {"lost"},
                                if bet.0 == winner {(bet.1 as f32 * winning_odds) as usize} else {bet.1},
                                val);
                            self.send_pm(better.to_string(), indiv_res).await?;
                        }
                        let biggest_winner = self.bets
                            .iter()
                            .filter(|(_, (c,_))| *c == winner)
                            .max_by(|x,y| (x.1.1).cmp(&y.1.1))
                            .unwrap();
                        let r = format!("Bet finished: Biggest Winner: {} with a bet of {}", biggest_winner.0.clone(), biggest_winner.1.1);
                        // clear bets
                        self.cancel();
                        r
                    },
                    Err(e) if e.kind() == std::io::ErrorKind::Other => {
                        "No bets placed on at least one side yet. Try again later".into()
                    },
                    Err(e) => {
                        error!("Calling the bet failed. last commit: {:?}", self.commits.last_offset());
                        error!("Error: {:?}", e);
                        panic!()
                    }
                };
                response
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
                    debug!("Bet placed by {} for {}", nick, amt);
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

        self.send_pm(nick,res).await?;
        Ok(())
    }

    async fn send_pm(&mut self, nick: String, res: String)
    -> Result<() , Box<dyn std::error::Error + Send + Sync>>{
        self.send("PRIVMSG" , &format!("{{\"nick\":\"{}\",\"data\":\"{}\"}}",nick,res)).await?;
        Ok(())
    }

    async fn send(&mut self, msg_type: &str, payload: &str)
    -> std::result::Result<(), tokio_tungstenite::tungstenite::Error>{
        let msg = format!("{} {}", msg_type, payload);
        debug!("Send: {}", msg);
        self.ws.send(tMessage::text(msg)).await
    }
}


use serde::{Deserialize, Serialize};
// use serde_json;
use std::collections::HashMap;
use std::time::Instant;


// #[derive(Serialize,Deserialize,Debug,Clone)]
// pub enum Message{
//     Names(String),
// }

#[derive(Serialize,Deserialize,Debug)]
pub struct NamesDict {
    connectioncount:    usize,
    pub users:          Vec<UserData>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum UserData {
    One {nick: String, features: Vec<String>}
}


#[derive(Serialize,Deserialize,Debug)]
pub struct JoinDict {
    pub nick:       String,
    pub features:   Vec<String>,
    timestamp:      usize
}

#[derive(Serialize,Deserialize,Debug)]
pub struct QuitDict {
    pub nick:       String,
    pub features:   Vec<String>,
    timestamp:      usize
}

#[derive(Serialize,Deserialize,Debug)]
pub struct PrivateMessage {
    pub messageid:   usize,
    pub  nick:       String,
    pub timestamp:   usize,
    pub data:        String
}


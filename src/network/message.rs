

use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;


#[derive(Serialize,Deserialize,Debug,Clone)]
pub enum Message{
    Names(String),
}

#[derive(Serialize,Deserialize,Debug)]
pub struct NamesDict {
    connectioncount:    usize,
    pub users:              Vec<UserData>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum UserData {
    One {nick: String, features: Vec<String>}
}
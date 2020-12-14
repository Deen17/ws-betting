use crate::network::message::UserData;
use crate::dominion::fs::{get_points,set_points};


#[derive(Debug)]
pub struct User {
    features: Vec<String>,
    points: usize
}

impl From<UserData> for User {
    fn from(user: UserData) -> Self {
        let UserData::One {nick, features} = user;
        let points = match get_points(nick.as_str()){
            Ok(p)=> p,
            Err(_) => {
                set_points(nick.as_str(), 0).unwrap();
                0
            }
        };
        Self {
            features,
            points
        }
    }
}
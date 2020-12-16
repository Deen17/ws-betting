use crate::network::message::{UserData,JoinDict};
use crate::dominion::fs::{get_points,set_points};
use crate::network::config::INITIAL_POINTS;

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
                set_points(nick.as_str(), INITIAL_POINTS).unwrap();
                INITIAL_POINTS
            }
        };
        Self {
            features,
            points
        }
    }
}

impl From<JoinDict> for User {
    fn from(user: JoinDict) -> Self {
        let points = match get_points(user.nick.as_str()){
            Ok(p)=> p,
            Err(_) => {
                set_points(user.nick.as_str(), INITIAL_POINTS).unwrap();
                INITIAL_POINTS
            }
        };
        Self {
            features: user.features,
            points
        }
    }
}

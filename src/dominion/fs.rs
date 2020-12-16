

use std::fs::{read_to_string,write};
// use std::io::prelude::*;
use std::io::Result as ioResult;
use commitlog::*;

pub fn get_points(nick: &str) -> ioResult<usize>{
    let points = read_to_string(format!("points/{}", nick))?
        .parse()
        .or(
            Err(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData, 
                    format!("Could not parse points/{}", nick)
                )
            )
        )?;
    Ok(points)
}

pub fn set_points(nick: &str, points: usize) -> ioResult<()>{
    write(format!("points/{}", nick),format!("{}", points))
}

pub fn add_points(nick: &str, points: usize) -> ioResult<()>{
    let cur = get_points(nick)?;
    write(format!("points/{}", nick),format!("{}", points+cur))
}

pub fn remove_points(nick: &str, points: usize) -> ioResult<()>{
    let cur = get_points(nick)?;
    if points > cur {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "arithmetic overflow"))
    }
    write(format!("points/{}", nick),format!("{}", cur - points))
}

pub fn create_commit_log() -> CommitLog{
    let opts = LogOptions::new("commits");
    let log = CommitLog::new(opts).unwrap();

    log
}
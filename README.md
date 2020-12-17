# ws-betting

## Description
This is a project that implements the idea of channel points and betting of these points in a websocket-based channel.

## Setup

Install [rust.](https://www.rust-lang.org/tools/install)

Make `commits` folder and `log` folder.

Make file in src/network/config.rs

In it, specify the following parameters:

    pub static AUTHTOKEN: &str = "your DGG login key";

    pub static NICK: &str = "BOT NAME"; // the dgg name of your bot

    pub static DEV: &str = "DEV NAME"; // the developer's name for testing

    pub static INITIAL_POINTS: usize = 500; // could be anything

Run in release mode
> cargo run --release

## Usage

A user can use the following commands in the chat by whispering the bot:

| Command     | Description | Privileged? |
| ----------- | ----------- | ----------- |
| help      | lists the commands one can use   | No |
| points   | states the amount of points you own | No |
| odds   | states the current odds of the current bet | No |
| bet | Bet some of your points | No |
| start | starts a round of betting  | yes |
| cancel | cancels the current round of betting, no side effects  | yes |
| call | calls the winner and distributes points | yes |

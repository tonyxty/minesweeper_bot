#![feature(str_split_once)]

use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;

use futures::StreamExt;
use hyper::{Client, Uri};
use hyper::client::HttpConnector;
use hyper_socks2::SocksConnector;
use telegram_bot::*;
use telegram_bot::connector::Connector;
use telegram_bot::connector::hyper::{default_connector, HyperConnector};

use crate::coop_game::CoopGame;
use crate::Error::BotError;
use crate::game::{Coord, Game};
use crate::minesweeper::Minesweeper;
use crate::othello_game::OthelloGame;

mod mine_field;
mod minesweeper;
mod grid_game;
mod game;
mod coop_game;
mod othello;
mod othello_game;

fn parse_coord(s: Option<&str>) -> Option<Coord> {
    let mut iter = s?.split_whitespace();
    let row = str::parse::<usize>(iter.next()?).ok()?;
    let column = str::parse::<usize>(iter.next()?).ok()?;
    Some((row, column))
}

enum Error {
    BotError(telegram_bot::Error),
    NoCommand,
    InvalidCoord,
    MessageTooOld,
    NoSuchGame,
}

impl From<telegram_bot::Error> for Error {
    fn from(error: telegram_bot::Error) -> Self {
        BotError(error)
    }
}


fn find_command<'a>(data: &'a str, entities: &[MessageEntity]) -> Option<&'a str> {
    for entity in entities {
        if entity.kind == MessageEntityKind::BotCommand {
            return Some(&data[entity.offset as usize..(entity.offset + entity.length) as usize]);
        }
    }
    None
}

fn filter_command<'a>(command: &'a str, bot_name: &str, is_private_chat: bool) -> Option<&'a str> {
    if let Some((command, name)) = command.split_once('@') {
        if is_private_chat || name == bot_name {
            Some(command)
        } else {
            None
        }
    } else {
        if is_private_chat {
            Some(command)
        } else {
            None
        }
    }
}

fn create_game(data: &str, entities: &[MessageEntity], user: &User) -> Option<(Box<dyn Game>, String, InlineKeyboardMarkup)> {
    if data.starts_with("/mine") {
        let (game, text, inline_keyboard) = CoopGame::create(Minesweeper::from_command(data));
        Some((Box::new(game), text, inline_keyboard))
    } else if data.starts_with("/othello") {
        let (game, text, inline_keyboard) = OthelloGame::from_message(data, entities, user)?;
        Some((Box::new(game), text, inline_keyboard))
    } else {
        None
    }
}


struct GameManager<'a> {
    api: &'a Api,
    bot_name: String,
    running_games: HashMap<(ChatId, MessageId), Box<dyn Game>>,
}

impl<'a> GameManager<'a> {
    async fn new(api: &'a Api) -> GameManager<'a> {
        let me = api.send(GetMe).await.unwrap();
        Self {
            api,
            bot_name: me.username.unwrap(),
            running_games: HashMap::new(),
        }
    }

    async fn handle_update(&mut self, update: Result<Update, telegram_bot::Error>) -> Result<(), Error> {
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, ref entities, .. } = message.kind {
                let is_private_chat = match message.chat {
                    MessageChat::Private(_) => { true }
                    _ => { false }
                };
                let command = filter_command(
                    find_command(data, entities).ok_or(Error::NoCommand)?,
                    &self.bot_name, is_private_chat).ok_or(Error::NoCommand)?;

                if command.starts_with("/stats") {
                    let text = format!("{} running games.", self.running_games.len());
                    self.api.send(message.text_reply(text)).await?;
                } else if let Some((game, text, inline_keyboard)) = create_game(data, entities, &message.from) {
                    let reply = self.api.send(message
                        .text_reply(text)
                        .reply_markup(inline_keyboard)).await?;
                    if let MessageOrChannelPost::Message(reply) = reply {
                        self.running_games.insert((reply.chat.id(), reply.id), game);
                    }
                } else {
                    self.api.send(message.text_reply("Command not understood.")).await?;
                }
            }
        } else if let UpdateKind::CallbackQuery(query) = update.kind {
            self.api.send(query.acknowledge()).await?;
            let coord = parse_coord(query.data.as_ref().map(String::as_str)).ok_or(Error::InvalidCoord)?;
            if let MessageOrChannelPost::Message(message) = query.message.ok_or(Error::MessageTooOld)? {
                let game = self.running_games.get_mut(&(message.chat.id(), message.id)).ok_or(Error::NoSuchGame)?;
                if let Some(result) = game.interact(coord, &query.from) {
                    if result.game_end {
                        self.running_games.remove(&(message.chat.id(), message.id));
                    }
                    result.reply_to(self.api, &message).await?;
                }
            }
        }
        Ok(())
    }
}

fn socks5_connector(addr: String) -> Box<dyn Connector> {
    let mut connector = HttpConnector::new();
    connector.enforce_http(false);
    Box::new(
        HyperConnector::new(Client::builder().build(SocksConnector {
            proxy_addr: Uri::try_from(addr).unwrap(),
            auth: None,
            connector,
        }.with_tls().unwrap()))
    )
}

#[tokio::main]
async fn main() {
    let token = env::var("API_TOKEN").unwrap();
    let connector = env::var("PROXY")
        .map_or_else(|_| default_connector().unwrap(), socks5_connector);

    let api = Api::with_connector(token, connector);
    let mut stream = api.stream();

    let mut manager: GameManager = GameManager::new(&api).await;

    while let Some(update) = stream.next().await {
        let _ = manager.handle_update(update).await;
    }
}

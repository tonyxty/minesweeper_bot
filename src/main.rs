#![feature(box_syntax)]
#![feature(bool_to_option)]
#![feature(option_result_contains)]

use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;

use futures::StreamExt;
use hyper::client::{Client, HttpConnector};
use hyper::Uri;
use hyper_socks2::SocksConnector;
use telegram_bot::*;
use telegram_bot::connector::Connector;
use telegram_bot::connector::hyper::{default_connector, HyperConnector};
use thiserror::Error;

use crate::coop_game::CoopGame;
use crate::game::Game;
use crate::minesweeper::game::Game as MinesweeperGame;
use othello::game::Game as OthelloGame;

mod minesweeper;
mod grid_game;
mod game;
mod coop_game;
mod othello;

#[derive(Error, Debug)]
enum Error {
    #[error("telegram bot API encountered error")]
    BotError(#[from] telegram_bot::Error),
    #[error("no command given or command not understood")]
    NoCommand,
    #[error("received invalid coordinates")]
    InvalidCoord,
    #[error("message too old")]
    MessageTooOld,
    #[error("message is not a game")]
    NoSuchGame,
}


fn parse_command<'a>(command: &'a str, bot_name: &str, is_private_chat: bool) -> Option<&'a str> {
    if let Some((command, name)) = command.split_once('@') {
        (is_private_chat || name == bot_name).then_some(command)
    } else {
        is_private_chat.then_some(command)
    }
}

fn create_game(data: &str, entities: &[MessageEntity], user: &User) -> Option<(Box<dyn Game>, String, InlineKeyboardMarkup)> {
    if data.starts_with("/mine") {
        let (game, text, inline_keyboard) = CoopGame::create(MinesweeperGame::from_message(data));
        Some((box game, text, inline_keyboard))
    } else if data.starts_with("/othello") {
        let (game, text, inline_keyboard) = OthelloGame::from_message(data, entities, user)?;
        Some((box game, text, inline_keyboard))
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

    async fn handle_update(
        &mut self, update: Result<Update, telegram_bot::Error>
    ) -> Result<(), Error> {
        let update = update?;
        match update.kind {
            UpdateKind::Message(message) => {
                if let MessageKind::Text { data, entities, .. } = &message.kind {
                    let is_private_chat = matches!(message.chat, MessageChat::Private(_));

                    let command = entities.iter()
                        .find(|x| x.kind == MessageEntityKind::BotCommand)
                        .map(|x| &data[x.offset as usize .. (x.offset + x.length) as usize])
                        .ok_or(Error::NoCommand)?;
                    let command = parse_command(command, &self.bot_name, is_private_chat)
                        .ok_or(Error::NoCommand)?;

                    if command == "/stats" {
                        let text = format!("{} running games.", self.running_games.len());
                        self.api.send(message.text_reply(text)).await?;
                    } else if command == "/del" {
                        if let Some(reply_to) = message.reply_to_message {
                            self.running_games.remove(&(reply_to.to_source_chat(), reply_to.to_message_id()))
                                .ok_or(Error::NoSuchGame)?;
                            self.api.send(reply_to.delete()).await?;
                        }
                    } else if let Some((game, text, inline_keyboard)) = create_game(data, entities, &message.from) {
                        let mut reply = message.text_reply(text);
                        reply.reply_markup(inline_keyboard);
                        let reply = self.api.send(reply).await?;
                        if let MessageOrChannelPost::Message(reply) = reply {
                            self.running_games.insert((reply.chat.id(), reply.id), game);
                        }
                    } else {
                        self.api.send(message.text_reply("Command not understood.")).await?;
                    }
                }
            }
            UpdateKind::CallbackQuery(query) => {
                self.api.send(query.acknowledge()).await?;
                let coord = query.data.ok_or(Error::InvalidCoord)?
                    .parse().map_err(|_| Error::InvalidCoord)?;
                let message = query.message.ok_or(Error::MessageTooOld)?;
                if let MessageOrChannelPost::Message(message) = message {
                    let game = self.running_games.get_mut(&(message.chat.id(), message.id))
                        .ok_or(Error::NoSuchGame)?;
                    let result = game.interact(coord, &query.from).unwrap_or_default();
                    if result.game_end {
                        self.running_games.remove(&(message.chat.id(), message.id));
                    }
                    result.reply_to(self.api, &message).await?;
                }
            }
            _ => (),
        }
        Ok(())
    }
}

fn socks5_connector(addr: String) -> Box<dyn Connector> {
    let mut connector = HttpConnector::new();
    connector.enforce_http(false);
    box HyperConnector::new(Client::builder().build(SocksConnector {
        proxy_addr: Uri::try_from(addr).unwrap(),
        auth: None,
        connector,
    }.with_tls().unwrap()))
}

#[tokio::main]
async fn main() {
    let token = env::var("API_TOKEN").unwrap();
    let connector = env::var("PROXY")
        .map_or_else(|_| default_connector().unwrap(), socks5_connector);

    let api = Api::with_connector(token, connector);
    let mut stream = api.stream();

    let mut manager = GameManager::new(&api).await;

    while let Some(update) = stream.next().await {
        if let Err(e) = manager.handle_update(update).await {
            eprintln!("{:?}", e);
        }
    }
}

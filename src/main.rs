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
use crate::game::{Coord, Game};
use crate::grid_game::GridGame;
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

struct GameManager<'a> {
    api: &'a Api,
    running_games: HashMap<(ChatId, MessageId), Box<dyn Game>>,
}

impl<'a> GameManager<'a> {
    fn new(api: &'a Api) -> GameManager<'a> {
        Self {
            api,
            running_games: HashMap::new(),
        }
    }

    async fn handle_update(&mut self, update: Result<Update, Error>) -> Result<(), Error> {
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, ref entities, .. } = message.kind {
                if data.starts_with("/mine") {
                    let game = Minesweeper::from_command(data);
                    let reply = self.api.send(message
                        .text_reply(game.get_text())
                        .reply_markup(game.to_inline_keyboard())).await?;
                    if let MessageOrChannelPost::Message(reply) = reply {
                        self.running_games.insert((reply.chat.id(), reply.id), Box::new(CoopGame::new(game)));
                    }
                } else if data.starts_with("/othello") {
                    if let Some((game, text, inline_keyboard)) = OthelloGame::from_message(data, entities, &message.from) {
                        let reply = self.api.send(message
                            .text_reply(text)
                            .reply_markup(inline_keyboard)).await?;
                        if let MessageOrChannelPost::Message(reply) = reply {
                            self.running_games.insert((reply.chat.id(), reply.id), Box::new(game));
                        }
                    }
                }
            }
        } else if let UpdateKind::CallbackQuery(query) = update.kind {
            self.api.send(query.acknowledge()).await?;
            if let Some(coord) = parse_coord(query.data.as_ref().map(String::as_str)) {
                if let MessageOrChannelPost::Message(message) = query.message.unwrap() {
                    if let Some(game) = self.running_games.get_mut(&(message.chat.id(), message.id)) {
                        if let Some(result) = game.interact(coord, &query.from) {
                            if result.game_end {
                                self.running_games.remove(&(message.chat.id(), message.id));
                            }
                            let _ = result.reply_to(self.api, &message).await;
                        }
                    }
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

    let mut manager: GameManager = GameManager::new(&api);

    while let Some(update) = stream.next().await {
        let _ = manager.handle_update(update).await;
    }
}

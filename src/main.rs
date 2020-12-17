use std::collections::HashMap;
use std::env;

use futures::StreamExt;
use telegram_bot::*;

use crate::game::{Coord, Game};
use crate::grid_game::GridGame;
use crate::minesweeper::Minesweeper;
use crate::coop_game::CoopGame;
use telegram_bot::connector::Connector;
use telegram_bot::connector::hyper::{HyperConnector, default_connector};
use hyper::{Client, Uri};
use hyper_socks2::SocksConnector;
use hyper::client::HttpConnector;
use std::convert::TryFrom;

mod mine_field;
mod minesweeper;
mod grid_game;
mod game;
mod coop_game;

fn parse_coord(s: Option<&str>) -> Option<Coord> {
    let mut iter = s?.split_whitespace();
    let row = str::parse::<usize>(iter.next()?).ok()?;
    let column = str::parse::<usize>(iter.next()?).ok()?;
    Some((row, column))
}

struct GameManager {
    running_games: HashMap<(ChatId, MessageId), Box<dyn Game>>,
}

impl GameManager {
    fn new() -> GameManager {
        Self {
            running_games: HashMap::new(),
        }
    }

    async fn handle_update(&mut self,
                           api: &Api,
                           update: Result<Update, Error>) -> Result<(), Error> {
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { data: ref command, .. } = message.kind {
                if command.starts_with("/mine") {
                    let game = Minesweeper::from_command(command);
                    let reply = api.send(message
                        .text_reply(game.get_text())
                        .reply_markup(game.to_inline_keyboard())).await?;
                    if let MessageOrChannelPost::Message(reply) = reply {
                        self.running_games.insert((reply.chat.id(), reply.id), Box::new(CoopGame::new(game)));
                    }
                }
            }
        } else if let UpdateKind::CallbackQuery(query) = update.kind {
            api.send(query.acknowledge()).await?;
            if let Some(coord) = parse_coord(query.data.as_ref().map(String::as_str)) {
                if let MessageOrChannelPost::Message(message) = query.message.unwrap() {
                    if let Some(game) = self.running_games.get_mut(&(message.chat.id(), message.id)) {
                        if let Some(result) = game.interact(coord, &query.from) {
                            if result.game_end {
                                self.running_games.remove(&(message.chat.id(), message.id));
                            }
                            let _ = result.reply_to(api, &message).await;
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

    let mut manager: GameManager = GameManager::new();

    while let Some(update) = stream.next().await {
        let _ = manager.handle_update(&api, update).await;
    }
}

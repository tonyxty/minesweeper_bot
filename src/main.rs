use std::collections::HashMap;

use futures::{StreamExt};
use telegram_bot::*;

use crate::grid_game::{Coord, GridGame, GameState};
use crate::minesweeper::Minesweeper;
use std::env;

mod mine_field;
mod minesweeper;
mod grid_game;

fn parse_coord(s: Option<&str>) -> Option<Coord> {
    let mut iter = s?.split_whitespace();
    let row = str::parse::<usize>(iter.next()?).ok()?;
    let column = str::parse::<usize>(iter.next()?).ok()?;
    Some((row, column))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("API_TOKEN").unwrap();
    let api = Api::new(token);
    let mut stream = api.stream();

    let mut running_games = HashMap::new();

    while let Some(update) = stream.next().await {
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { data: ref command, .. } = message.kind {
                if command.starts_with("/mine") {
                    let game = Minesweeper::from_command(command);
                    let reply = api.send(message
                        .text_reply(game.get_text())
                        .reply_markup(game.to_inline_keyboard())).await?;
                    if let MessageOrChannelPost::Message(reply) = reply {
                        running_games.insert((reply.chat.id(), reply.id), game);
                    }
                }
            }
        } else if let UpdateKind::CallbackQuery(query) = update.kind {
            api.send(query.acknowledge()).await?;
            if let Some(coord) = parse_coord(query.data.as_ref().map(String::as_str)) {
                if let MessageOrChannelPost::Message(message) = query.message.unwrap() {
                    if let Some(game) = running_games.get_mut(&(message.chat.id(), message.id)) {
                        let changed = game.interact(coord);
                        if changed {
                            let keyboard_markup = game.to_inline_keyboard();
                            match game.get_state() {
                                GameState::Normal => {
                                    api.send(message.edit_reply_markup(Some(keyboard_markup))).await?;
                                }
                                GameState::Solved => {
                                    api.send(message.edit_text("Solved!").reply_markup(keyboard_markup)).await?;
                                    running_games.remove(&(message.chat.id(), message.id));
                                }
                                GameState::GameOver => {
                                    api.send(message.edit_text("Game over!").reply_markup(keyboard_markup)).await?;
                                    running_games.remove(&(message.chat.id(), message.id));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

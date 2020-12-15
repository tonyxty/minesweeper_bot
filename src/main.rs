use std::collections::HashMap;
use std::env;

use futures::StreamExt;
use telegram_bot::*;

use crate::grid_game::{Coord, GridGame, CoopGame};
use crate::minesweeper::Minesweeper;

mod mine_field;
mod minesweeper;
mod grid_game;

fn parse_coord(s: Option<&str>) -> Option<Coord> {
    let mut iter = s?.split_whitespace();
    let row = str::parse::<usize>(iter.next()?).ok()?;
    let column = str::parse::<usize>(iter.next()?).ok()?;
    Some((row, column))
}

async fn handle_update(api: &Api,
                       running_games: &mut HashMap<(ChatId, MessageId), CoopGame>,
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
                    running_games.insert((reply.chat.id(), reply.id), CoopGame::new(Box::new(game)));
                }
            }
        }
    } else if let UpdateKind::CallbackQuery(query) = update.kind {
        api.send(query.acknowledge()).await?;
        if let Some(coord) = parse_coord(query.data.as_ref().map(String::as_str)) {
            if let MessageOrChannelPost::Message(message) = query.message.unwrap() {
                if let Some(game) = running_games.get_mut(&(message.chat.id(), message.id)) {
                    if let Some(result) = game.interact(coord, &query.from) {
                        if result.is_game_end() {
                            running_games.remove(&(message.chat.id(), message.id));
                        }
                        let _ = result.reply_to(api, &message).await;
                    }
                }
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = env::var("API_TOKEN").unwrap();
    let api = Api::new(token);
    let mut stream = api.stream();

    let mut running_games = HashMap::new();

    while let Some(update) = stream.next().await {
        let _ = handle_update(&api, &mut running_games, update).await;
    }
}

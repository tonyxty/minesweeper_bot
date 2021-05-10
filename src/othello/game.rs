use telegram_bot::{InlineKeyboardButton, InlineKeyboardMarkup, MessageEntity, MessageEntityKind, User, UserId};

use crate::game::{self, Coord, InteractResult};
use super::board::Board;

pub struct Game {
    board: Board,
    first_player: String,
    second_player: (UserId, String),
}

impl Game {
    pub fn from_message<'a>(
        data: &str, entities: impl IntoIterator<Item=&'a MessageEntity>, user: &User
    ) -> Option<(Self, String, InlineKeyboardMarkup)> {
        let first_player = entities.into_iter()
            .find_map(|e| (e.kind == MessageEntityKind::Mention).then(|| {
                let start = (e.offset + 1) as usize;
                let end = (e.offset + e.length) as usize;
                data[start..end].to_owned()
            }))?;
        let second_player = (
            user.id,
            user.username.to_owned().unwrap_or_else(|| user.first_name.to_owned())
        );
        let game = Game {
            board: Board::new(),
            first_player,
            second_player,
        };
        let text = game.get_text();
        let inline_keyboard = game.to_inline_keyboard();
        Some((game, text, inline_keyboard))
    }

    fn get_text(&self) -> String {
        let scores = self.board.get_score();
        let mut text = format!("{} {} vs {} {}", self.first_player, scores.0, scores.1, self.second_player.1);

        if self.board.game_over {
            use std::cmp::Ordering::*;
            match u32::cmp(&scores.0, &scores.1) {
                Less => {
                    text += "\nWinner: ";
                    text += self.second_player.1.as_str();
                }
                Equal => {
                    text += "\nDraw game."
                }
                Greater => {
                    text += "\nWinner: ";
                    text += self.first_player.as_str();
                }
            }
        } else if self.board.player {
            text += " ⚪";
        } else {
            text.insert_str(0, "⚫ ");
        }
        text
    }

    fn to_inline_keyboard(&self) -> InlineKeyboardMarkup {
        self.board.iter()
            .enumerate()
            .map(|(i, row)| row.iter()
                .enumerate()
                .map(|(j, &p)| InlineKeyboardButton::callback(to_string(p), format!("{} {}", i, j)))
                .collect()
            ).collect::<Vec<Vec<_>>>().into()
    }

    fn is_current_player(&self, user: &User) -> bool {
        if !self.board.player {
            user.username.contains(&self.first_player)
        } else {
            user.id == self.second_player.0
        }
    }
}

impl game::Game for Game {
    fn interact(&mut self, coord: Coord, user: &User) -> Option<InteractResult> {
        (self.is_current_player(user) && self.board.play(coord)).then_some(
            InteractResult {
                update_text: Some(self.get_text()),
                update_board: Some(self.to_inline_keyboard()),
                game_end: self.board.game_over,
            }
        )
    }
}

fn to_string<'a>(piece: Option<bool>) -> &'a str {
    match piece {
        None => " ",
        Some(true) => "⚪",
        Some(false) => "⚫",
    }
}

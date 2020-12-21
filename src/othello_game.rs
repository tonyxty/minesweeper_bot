use telegram_bot::{InlineKeyboardButton, InlineKeyboardMarkup, MessageEntity, MessageEntityKind, User, UserId};

use crate::game::{Game, InteractResult};
use crate::othello::Othello;

pub struct OthelloGame {
    game: Othello,
    first_player: String,
    second_player: (UserId, String),
}

impl OthelloGame {
    pub fn from_message(data: &str, entities: &[MessageEntity], user: &User) -> Option<(Self, String, InlineKeyboardMarkup)> {
        let mut first_player = None;
        for entity in entities {
            if entity.kind == MessageEntityKind::Mention {
                let start = (entity.offset + 1) as usize;
                let end = (entity.offset + entity.length) as usize;
                first_player = Some(data[start..end].into());
                break;
            }
        }
        let first_player = first_player?;
        let second_player = (user.id, user.username.clone().unwrap_or_else(|| user.first_name.clone()));
        let game = OthelloGame {
            game: Othello::new(),
            first_player,
            second_player,
        };
        let text = game.get_text();
        let inline_keyboard = game.to_inline_keyboard();
        Some((game, text, inline_keyboard))
    }

    fn get_text(&self) -> String {
        let scores = self.game.get_score();
        let mut text = format!("{} {} vs {} {}", self.first_player, scores.0, scores.1, self.second_player.1);

        if self.game.is_game_over() {
            if scores.0 > scores.1 {
                text += "\nWinner: ";
                text += self.first_player.as_str();
            } else if scores.0 < scores.1 {
                text += "\nWinner: ";
                text += self.second_player.1.as_str();
            } else {
                text += "\nDraw game."
            }
        } else {
            if self.game.get_current_player() {
                text += " ⚪";
            } else {
                text.insert_str(0, "⚫ ");
            }
        }
        text
    }

    fn to_inline_keyboard(&self) -> InlineKeyboardMarkup {
        let mut inline_keyboard = InlineKeyboardMarkup::new();
        for i in 0..8 {
            inline_keyboard.add_row(self.game.iter_row(i)
                .enumerate()
                .map(|(j, p)| InlineKeyboardButton::callback(to_char(*p).to_string(), format!("{} {}", i, j)))
                .collect());
        }
        inline_keyboard
    }
}

impl Game for OthelloGame {
    fn interact(&mut self, coord: (usize, usize), user: &User) -> Option<InteractResult> {
        let player = self.game.get_current_player();
        if (!player && user.username.as_ref() == Some(&self.first_player)) || (player && user.id == self.second_player.0) {
            if self.game.play(coord) {
                Some(InteractResult {
                    update_text: Some(self.get_text()),
                    update_board: Some(self.to_inline_keyboard()),
                    game_end: self.game.is_game_over(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn to_char(piece: Option<bool>) -> char {
    match piece {
        None => { ' ' }
        Some(b) => { if b { '⚪' } else { '⚫' } }
    }
}

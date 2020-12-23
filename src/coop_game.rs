use std::collections::HashMap;

use telegram_bot::{User, InlineKeyboardMarkup};

use crate::game::{Coord, Game, InteractResult};
use crate::grid_game::{GameState, GridGame};

pub struct CoopGame<T: GridGame> {
    game: T,
    interactions: HashMap<String, u32>,
}

impl<T: GridGame> CoopGame<T> {
    pub fn create(game: T) -> (Self, String, InlineKeyboardMarkup) {
        let text = game.get_text();
        let inline_keyboard = game.to_inline_keyboard();
        (Self {
            game,
            interactions: HashMap::new(),
        }, text, inline_keyboard)
    }
}

impl<T: GridGame> Game for CoopGame<T> {
    fn interact(&mut self, coord: Coord, user: &User) -> Option<InteractResult> {
        if self.game.interact(coord) {
            let username = user.username.as_ref().unwrap_or(&user.first_name);
            let value = self.interactions.get_mut(username);
            if let Some(x) = value {
                *x += 1;
            } else {
                self.interactions.insert(username.into(), 1);
            }

            let keyboard_markup = self.game.to_inline_keyboard();
            let state = self.game.get_state();
            if state == GameState::Normal {
                Some(InteractResult {
                    update_text: Some(self.game.get_text()),
                    update_board: Some(keyboard_markup),
                    game_end: false,
                })
            } else {
                let mut summary = String::with_capacity(self.interactions.len() * 10);
                let mut largest_count = 0;
                let mut top_contributor = "";
                for (name, count) in self.interactions.iter() {
                    summary += format!("{} - {} moves\n", name, count).as_str();
                    if *count > largest_count {
                        largest_count = *count;
                        top_contributor = name;
                    }
                }
                let count = self.interactions.get(username).unwrap();
                if *count == largest_count {
                    if state == GameState::Solved {
                        summary += format!("{} has won the game!", username).as_str();
                    } else {
                        summary += format!("Boom, {} is dead!", username).as_str();
                    }
                } else {
                    if state == GameState::Solved {
                        summary += format!("{} has snatched it from {}!", username, top_contributor).as_str();
                    } else {
                        summary += format!("{} has ruined it for {}!", username, top_contributor).as_str();
                    }
                }

                Some(InteractResult {
                    update_text: Some(summary),
                    update_board: Some(keyboard_markup),
                    game_end: true,
                })
            }
        } else {
            None
        }
    }
}

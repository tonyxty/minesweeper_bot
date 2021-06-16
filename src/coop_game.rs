use std::collections::HashMap;

use itertools::Itertools;
use telegram_bot::{User, InlineKeyboardMarkup};

use crate::game::{Coord, Game, InteractResult};
use crate::grid_game::{GameState, GridGame};

// Wraps a cooperative game and implements interaction stats
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
        self.game.interact(coord).then_some({
            let username = user.username.as_ref().unwrap_or(&user.first_name);
            *self.interactions.entry(username.to_owned()).or_default() += 1;

            let keyboard_markup = self.game.to_inline_keyboard();
            let state = self.game.get_state();
            if state == GameState::Normal {
                InteractResult {
                    update_text: Some(self.game.get_text()),
                    update_board: Some(keyboard_markup),
                    game_end: false,
                }
            } else {
                let mut largest_count = 0;
                let mut top_contributor = "";
                let mut summary = self.interactions.iter()
                    .map(|(name, &count)| {
                        if count > largest_count {
                            largest_count = count;
                            top_contributor = name;
                        }
                        format!("{} - {} moves", name.as_str(), count)
                    }).join("\n") + "\n";
                let &count = self.interactions.get(username).unwrap();
                if count == largest_count {
                    // It's the top contributor acting
                    if state == GameState::Solved {
                        summary += format!("{} has won the game!", username).as_str();
                    } else {
                        summary += format!("Boom, {} is dead!", username).as_str();
                    }
                } else if state == GameState::Solved {
                    // Someone else interfered
                    summary += format!("{} has snatched it from {}!", username, top_contributor).as_str();
                } else {
                    summary += format!("{} has ruined it for {}!", username, top_contributor).as_str();
                }

                InteractResult {
                    update_text: Some(summary),
                    update_board: Some(keyboard_markup),
                    game_end: true,
                }
            }
        })
    }
}

use telegram_bot::*;
use std::collections::HashMap;

pub type Coord = (usize, usize);

#[derive(Eq, PartialEq)]
pub enum GameState {
    Normal,
    Solved,
    GameOver,
}

pub trait GridGame {
    fn get_state(&self) -> GameState;
    fn get_text(&self) -> String;
    fn interact(&mut self, coord: Coord) -> bool;
    fn to_inline_keyboard(&self) -> InlineKeyboardMarkup;
}


pub struct CoopGame {
    game: Box<dyn GridGame>,
    interactions: HashMap<String, u32>,
}

pub struct InteractResult {
    update_text: Option<String>,
    update_board: Option<InlineKeyboardMarkup>,
    game_end: bool,
}

impl InteractResult {
    pub fn is_game_end(&self) -> bool {
        self.game_end
    }

    pub async fn reply_to(self, api: &Api, message: &Message) -> Result<Message, Error> {
        if let Some(text) = self.update_text {
            if let Some(board) = self.update_board {
                api.send(message.edit_text(text).reply_markup(board)).await
            } else {
                api.send(message.edit_text(text)).await
            }
        } else {
            let board = self.update_board.unwrap();
            api.send(message.edit_reply_markup(Some(board))).await
        }
    }
}

impl CoopGame {
    pub fn new(game: Box<dyn GridGame>) -> Self {
        Self {
            game,
            interactions: HashMap::new(),
        }
    }

    pub fn interact(&mut self, coord: Coord, user: &User) -> Option<InteractResult> {
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
                    update_text: None,
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

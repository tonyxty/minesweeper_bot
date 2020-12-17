use telegram_bot::*;

use crate::game::Coord;

#[derive(Eq, PartialEq)]
pub enum GameState {
    Normal,
    Solved,
    GameOver,
}

pub trait GridGame {
    fn get_state(&self) -> GameState;
    fn get_text(&self) -> String;
    fn to_inline_keyboard(&self) -> InlineKeyboardMarkup;
    fn interact(&mut self, coord: Coord) -> bool;
}

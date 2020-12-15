use telegram_bot::InlineKeyboardMarkup;

pub type Coord = (usize, usize);

#[derive(Clone)]
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

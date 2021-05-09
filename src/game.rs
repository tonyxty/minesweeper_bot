use std::str::FromStr;
use std::num::ParseIntError;

use telegram_bot::*;
use thiserror::Error;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Coord(pub u32, pub u32);

#[derive(Error, Debug)]
pub enum ParseCoordError {
    #[error("not enough components for a coordinate")]
    NotEnoughComponents,
    #[error("cannot parse input as integer")]
    ParseInt(#[from] ParseIntError),
}

impl FromStr for Coord {
    type Err = ParseCoordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut i = s.split_whitespace();
        let row = i.next().ok_or(ParseCoordError::NotEnoughComponents)?.parse()?;
        let column = i.next().ok_or(ParseCoordError::NotEnoughComponents)?.parse()?;
        Ok(Coord(row, column))
    }
}

pub struct InteractResult {
    pub update_text: Option<String>,
    pub update_board: Option<InlineKeyboardMarkup>,
    pub game_end: bool,
}

pub trait Game {
    fn interact(&mut self, coord: Coord, user: &User) -> Option<InteractResult>;
}


impl InteractResult {
    pub async fn reply_to(self, api: &Api, message: &Message) -> Result<(), Error> {
        if let Some(text) = self.update_text {
            if let Some(board) = self.update_board {
                api.send(message.edit_text(text).reply_markup(board)).await.map(|_| ())
            } else {
                api.send(message.edit_text(text)).await.map(|_| ())
            }
        } else if let Some(board) = self.update_board {
            api.send(message.edit_reply_markup(Some(board))).await.map(|_| ())
        } else {
            Ok(())
        }
    }
}

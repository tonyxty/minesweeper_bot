use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::num::ParseIntError;
use std::str::FromStr;

use itertools::iproduct;
use telegram_bot::*;
use thiserror::Error;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Coord(pub i32, pub i32);

impl Add for Coord {
    type Output = Coord;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0, self.1 + other.1)
    }
}

impl AddAssign for Coord {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
        self.1 += other.1;
    }
}

impl Sub for Coord {
    type Output = Coord;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0, self.1 - other.1)
    }
}

impl SubAssign for Coord {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
        self.1 -= other.1;
    }
}

impl Coord {
    pub const DIRECTIONS: [Coord; 8] = [
        Coord(-1, -1),
        Coord(-1, 0),
        Coord(-1, 1),
        Coord(0, -1),
        Coord(0, 1),
        Coord(1, -1),
        Coord(1, 0),
        Coord(1, 1),
    ];
}

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

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Size(pub u32, pub u32);

impl Size {
    pub fn contains(self, coord: Coord) -> bool {
        (0..self.0 as i32).contains(&coord.0) && (0..self.1 as i32).contains(&coord.1)
    }

    pub fn index(self, coord: Coord) -> usize {
        ((coord.0 as u32 * self.1) + coord.1 as u32) as _
    }

    pub fn size(self) -> u32 {
        self.0 * self.1
    }

    pub fn valid_indices(self) -> impl Iterator<Item=Coord> {
        iproduct!(0 .. self.0 as _, 0 .. self.1 as _)
            .map(|(i, j)| Coord(i, j))
    }
}

#[derive(Default)]
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
                api.send(message.edit_text(text).reply_markup(board)).await?;
            } else {
                api.send(message.edit_text(text)).await?;
            }
        } else if let Some(board) = self.update_board {
            api.send(message.edit_reply_markup(Some(board))).await?;
        }
        Ok(())
    }
}

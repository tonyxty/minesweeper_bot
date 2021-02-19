use std::char;
use std::cmp;

use telegram_bot::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::game::Coord;
use crate::grid_game::{GameState, GridGame};
use crate::grid_game::GameState::{GameOver, Normal, Solved};
use crate::mine_field::{Cell, MineField};

#[derive(Eq, PartialEq)]
pub enum MinesweeperModes {
    Classic,
    NoFlag,
}

pub struct Minesweeper {
    field: MineField,
    mode: MinesweeperModes,
}

impl MinesweeperModes {
    fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "noflag" => Some(Self::NoFlag),
            "classic" => Some(Self::Classic),
            _ => None
        }
    }
}

impl Minesweeper {
    pub fn from_message(data: &str) -> Self {
        // constraints:
        // 2 <= rows <= 10
        // 2 <= columns <= 8
        // 1 <= mines < rows * columns
        let mut args = [10, 8, 0];
        let mut mode = MinesweeperModes::Classic;
        let mut i = 0;

        for arg in data.split_whitespace().skip(1) {
            if let Some(game_mode) = MinesweeperModes::parse(arg) {
                mode = game_mode;
            } else if i < 3 {
                if let Ok(number) = arg.parse() {
                    args[i] = number;
                    i += 1;
                }
            }
        }

        let rows = cmp::min(args[0], 10);
        let columns = cmp::min(args[1], 8);
        let mines = if args[2] < 1 { rows * columns / 10 } else { args[2] };
        Self {
            field: MineField::new(rows, columns, mines),
            mode,
        }
    }
}

impl GridGame for Minesweeper {
    fn get_state(&self) -> GameState {
        let stats = self.field.get_stats();
        if stats.exploded > 0 {
            GameOver
        } else if stats.uncovered_blank + self.field.get_mines() == self.field.get_rows() * self.field.get_columns() {
            Solved
        } else {
            Normal
        }
    }

    fn get_text(&self) -> String {
        format!("{} x {}\n{} left / {} mines", self.field.get_rows(), self.field.get_columns(),
                self.field.get_stats().covered_mine, self.field.get_mines())
    }

    fn to_inline_keyboard(&self) -> InlineKeyboardMarkup {
        let mut inline_keyboard = InlineKeyboardMarkup::new();
        for i in 0..self.field.get_rows() {
            inline_keyboard.add_row(self.field.iter_row(i)
                .enumerate()
                .map(|(j, c)| InlineKeyboardButton::callback(to_char(c).to_string(), format!("{} {}", i, j)))
                .collect());
        }
        inline_keyboard
    }

    fn interact(&mut self, coord: Coord) -> bool {
        if !self.field.is_initialized() {
            self.field.initialize(coord);
        }
        if self.field.get(coord).is_covered() {
            self.field.uncover(coord);
            true
        } else {
            self.mode == MinesweeperModes::Classic && self.field.uncover_around(coord)
        }
    }
}

fn to_char(cell: &Cell) -> char {
    if cell.is_exploded() {
        '💣'
    } else if cell.is_covered() {
        '■'
    } else if cell.is_mine() {
        '🚩'
    } else if cell.get_value() == 0 {
        ' '
    } else {
        char::from_digit(cell.get_value(), 10).unwrap()
    }
}

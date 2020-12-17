use std::char;

use telegram_bot::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::game::Coord;
use crate::grid_game::{GameState, GridGame};
use crate::grid_game::GameState::{GameOver, Normal, Solved};
use crate::mine_field::{Cell, MineField};

pub struct Minesweeper(MineField);

fn parse_number(s: Option<&str>) -> Option<usize> {
    str::parse(s?).ok()
}

impl Minesweeper {
    pub fn from_command(command: &str) -> Self {
        // constraints:
        // 2 <= rows <= 10
        // 2 <= columns <= 8
        // 1 <= mines < rows * columns
        let mut iter = command.split_whitespace().skip(1);
        let mut rows = parse_number(iter.next()).unwrap_or(10);
        if rows > 10 {
            rows = 10;
        }
        let mut columns = parse_number(iter.next()).unwrap_or(8);
        if columns > 8 {
            columns = 8;
        }
        let mines = parse_number(iter.next()).unwrap_or_else(|| rows * columns / 10);
        Self(MineField::new(rows, columns, mines))
    }
}

impl GridGame for Minesweeper {
    fn get_state(&self) -> GameState {
        let stats = self.0.get_stats();
        if stats.exploded > 0 {
            GameOver
        } else if stats.uncovered_blank + self.0.get_mines() == self.0.get_rows() * self.0.get_columns() {
            Solved
        } else {
            Normal
        }
    }

    fn get_text(&self) -> String {
        format!("{}x{} {} mines", self.0.get_rows(), self.0.get_columns(), self.0.get_mines())
    }

    fn to_inline_keyboard(&self) -> InlineKeyboardMarkup {
        let mut inline_keyboard = InlineKeyboardMarkup::new();
        for i in 0..self.0.get_rows() {
            inline_keyboard.add_row(self.0.iter_row(i)
                .enumerate()
                .map(|(j, c)| InlineKeyboardButton::callback(to_char(c).to_string(), format!("{} {}", i, j)))
                .collect());
        }
        inline_keyboard
    }

    fn interact(&mut self, coord: Coord) -> bool {
        if !self.0.is_initialized() {
            self.0.initialize(coord);
        }
        if self.0.get(coord).is_covered() {
            self.0.uncover(coord);
            true
        } else {
            self.0.uncover_around(coord)
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

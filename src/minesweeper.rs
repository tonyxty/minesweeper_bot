use std::char;

use telegram_bot::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::grid_game::{GameState, GridGame};
use crate::grid_game::GameState::{GameOver, Normal, Solved};
use crate::mine_field::{Cell, MineField};

pub struct Minesweeper {
    field: MineField,
}

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
        Self {
            field: MineField::new(rows, columns, mines)
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
        format!("{}x{} {} mines", self.field.get_rows(), self.field.get_columns(), self.field.get_mines())
    }

    fn interact(&mut self, coord: (usize, usize)) -> bool {
        if !self.field.is_initialized() {
            self.field.initialize(coord);
        }
        if self.field.get(coord).is_covered() {
            self.field.uncover(coord);
            true
        } else {
            self.field.uncover_around(coord)
        }
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

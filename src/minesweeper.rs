use std::str::FromStr;

use telegram_bot::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::game::Coord;
use crate::grid_game::{GameState, GridGame};
use crate::grid_game::GameState::{GameOver, Normal, Solved};
use crate::mine_field::{Cell, MineField, State, CellValue};

#[derive(Eq, PartialEq)]
pub enum MinesweeperMode {
    Classic,
    NoFlag,
}

pub struct Minesweeper {
    field: MineField,
    mode: MinesweeperMode,
}

impl FromStr for MinesweeperMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "noflag" => Ok(Self::NoFlag),
            "classic" => Ok(Self::Classic),
            _ => Err(()),
        }
    }
}

impl Minesweeper {
    pub fn from_message(data: &str) -> Self {
        // constraints:
        // 2 <= rows <= 10
        // 2 <= columns <= 8
        // 1 <= mines < rows * columns
        let mut args = Vec::new();
        let mut mode = MinesweeperMode::Classic;

        for arg in data.split_whitespace().skip(1) {
            if let Ok(game_mode) = arg.parse() {
                mode = game_mode;
            } else if let Ok(num) = arg.parse() {
                args.push(num);
                if args.len() >= 3 { break; }
            }
        }

        let rows = args.get(0).copied().unwrap_or(10).min(10);
        let columns = args.get(1).copied().unwrap_or(8).min(8);
        let mines = args.get(2).copied().unwrap_or_else(|| rows * columns / 10);
        Self {
            field: MineField::new(rows, columns, mines),
            mode,
        }
    }
}

impl GridGame for Minesweeper {
    fn get_state(&self) -> GameState {
        let stats = &self.field.stats;
        if stats.exploded > 0 {
            GameOver
        } else if stats.uncovered_blank + self.field.mines == self.field.rows * self.field.columns {
            Solved
        } else {
            Normal
        }
    }

    fn get_text(&self) -> String {
        format!("{} x {}\n{} left / {} mines", self.field.rows, self.field.columns,
            self.field.stats.covered_mine, self.field.mines)
    }

    fn to_inline_keyboard(&self) -> InlineKeyboardMarkup {
        self.field.iter()
            .enumerate()
            .map(|(i, row)| row.iter()
                .enumerate()
                .map(|(j, c)| InlineKeyboardButton::callback(to_string(c), format!("{} {}", i, j)))
                .collect())
            .collect::<Vec<Vec<_>>>().into()
    }

    fn interact(&mut self, coord: Coord) -> bool {
        if !self.field.initialized {
            self.field.initialize(coord);
        }
        if self.field[coord].state == State::Covered {
            self.field.uncover(coord);
            true
        } else {
            self.mode == MinesweeperMode::Classic && self.field.uncover_around(coord)
        }
    }
}

fn to_string<'a>(cell: &Cell) -> &'a str {
    use State::*;
    use CellValue::*;
    match cell.state {
        Covered => "■",
        Exploded => "💣",
        Uncovered => match cell.value {
            Mine => "🚩",
            Number(n) => {
                let n = n as usize;
                &" 123456789"[n..n+1]
            },
        }
    }
}

use std::collections::vec_deque::VecDeque;
use std::iter;

use itertools::iproduct;

use crate::game::Coord;

// In our UI there is no flagging; if a cell is numbered k and has exactly k uncovered neighbors
// and the player decides to "uncover-around" it, then all neighbors will be uncovered as mines.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum State {
    Covered,
    Uncovered,
    Exploded,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CellValue {
    Mine,
    Number(u32),
}

#[derive(Clone)]
pub struct Cell {
    pub value: CellValue,
    pub state: State,
}

use CellValue::*;
use State::*;
impl Default for Cell {
    fn default() -> Self {
        Self { value: Number(0), state: Covered }
    }
}

fn neighborhood_of(coord: Coord, rows: u32, columns: u32) -> impl Iterator<Item=Coord> {
    iproduct!(-1..=1, -1..=1)
        .filter_map(move |(i, j)|
            (i != 0 || j != 0).then_some((coord.0 as i32 + i, coord.1 as i32 + j)))
        .filter_map(move |(row, column)|
            ((0..rows as i32).contains(&row) && (0..columns as i32).contains(&column))
                .then_some(Coord(row as u32, column as u32)))
}

// "state" (win/loss) is not part of the MineField struct because we may support other modes of
// deciding game outcome, such as Multiple Lives or Tap in Windows 10 Minesweeper daily challenges.
// Instead we provide an interface to access the current stats across the mine field.
pub struct MineFieldStats {
    pub uncovered_blank: u32,
    pub covered_mine: u32,
    pub exploded: u32,
}

pub struct MineField {
    pub initialized: bool,
    field: Vec<Cell>,
    rows: u32,
    columns: u32,
    mines: u32,
    stats: MineFieldStats,
}

impl MineField {
    pub fn new(rows: u32, columns: u32, mines: u32) -> Self {
        let rows = rows.max(2);
        let columns = columns.max(2);
        let mines = mines.clamp(1, rows * columns - 1);
        Self {
            initialized: false,
            field: vec![Cell::default(); (columns * rows) as _],
            rows,
            columns,
            mines,
            stats: MineFieldStats {
                uncovered_blank: 0,
                covered_mine: mines,
                exploded: 0,
            },
        }
    }

    pub fn get_rows(&self) -> u32 {
        self.rows
    }

    pub fn get_columns(&self) -> u32 {
        self.columns
    }

    pub fn get_mines(&self) -> u32 {
        self.mines
    }

    pub fn get_stats(&self) -> &MineFieldStats {
        &self.stats
    }

    fn get_index(&self, coord: Coord) -> usize {
        (coord.0 * self.columns + coord.1) as _
    }

    pub fn get(&self, coord: Coord) -> &Cell {
        &self.field[self.get_index(coord)]
    }

    fn iter_neighborhood(&self, center: Coord) -> impl Iterator<Item=&Cell> {
        neighborhood_of(center, self.rows, self.columns)
            .map(move |i| &self.field[self.get_index(i)])
    }

    pub fn iter_row(&self, row: u32) -> impl Iterator<Item=&Cell> {
        let index = (row * self.columns) as usize;
        self.field[index .. index + self.columns as usize].iter()
    }

    pub fn initialize(&mut self, avoid: Coord) {
        let avoid_index = self.get_index(avoid);
        let mut rng = rand::thread_rng();
        for mut i in rand::seq::index::sample(&mut rng, (self.columns * self.rows - 1) as _, self.mines as _).into_iter() {
            if i >= avoid_index {
                i += 1;
            }
            self.field[i].value = Mine;
        }
        for i in 0..self.rows {
            for j in 0..self.columns {
                let coord = Coord(i, j);
                let index = self.get_index(coord);
                if self.field[index].value != Mine {
                    let value = self.iter_neighborhood(coord)
                        .filter(|c| c.value == Mine)
                        .count() as u32;
                    self.field[index].value = Number(value);
                }
            }
        }
        self.initialized = true;
    }

    // primitive actions
    fn reveal(&mut self, coords: impl Iterator<Item=Coord>) {
        // flood-fill
        // reveal all adjacent cells if the current cell has a value of 0
        let mut queue = VecDeque::with_capacity((self.columns * self.rows) as _);
        queue.extend(coords);
        while let Some(coord) = queue.pop_front() {
            let index = self.get_index(coord);
            if self.field[index].state == Covered {
                self.field[index].state = Uncovered;
                if self.field[index].value == Mine {
                    self.stats.covered_mine -= 1;
                } else {
                    self.stats.uncovered_blank += 1;
                }
                if self.field[index].value == Number(0) {
                    queue.extend(neighborhood_of(coord, self.rows, self.columns)
                        .filter(|&i| self.get(i).state == Covered));
                }
            }
        }
    }

    fn reveal_around(&mut self, coord: Coord) {
        self.reveal(neighborhood_of(coord, self.rows, self.columns));
    }

    // simple actions
    pub fn uncover(&mut self, coord: Coord) {
        let index = self.get_index(coord);
        if self.field[index].value == Mine {
            self.stats.exploded += 1;
            self.field[index].state = Exploded;
        } else {
            self.reveal(iter::once(coord));
        }
    }

    // uncovers around cell, returns true if the field has changed
    pub fn uncover_around(&mut self, coord: Coord) -> bool {
        let index = self.get_index(coord);
        match self.field[index].value {
            Mine => false,
            Number(value) => {
                // count the number of adjacent covered cells and adjacent uncovered mine cells
                // there are certainly iterator chains that can do this in one statement but
                // a loop seems more readable
                let mut covered = 0;
                let mut uncovered_mines = 0;
                for c in self.iter_neighborhood(coord) {
                    if c.state == Uncovered {
                        if c.value == Mine { uncovered_mines += 1; }
                    } else {
                        covered += 1;
                    }
                }
                if covered == 0 {
                    false
                } else if uncovered_mines == value || covered + uncovered_mines == value {
                    // reveal all adjacent cells
                    self.reveal_around(coord);
                    true
                } else {
                    false
                }
            }
        }
    }
}

use std::collections::vec_deque::VecDeque;
use std::iter;
use std::ops::{Index, IndexMut};

use crate::game::{Coord, Size};

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

fn neighborhood_of(coord: Coord, size: Size) -> impl Iterator<Item=Coord> {
    Coord::DIRECTIONS.iter()
        .map(move |&d| coord + d)
        .filter(move |&c| size.contains(c))
}

// "state" (win/loss) is not part of the MineField struct because we may support other modes of
// deciding game outcome, such as Multiple Lives or Tap in Windows 10 Minesweeper daily challenges.
// Instead we provide an interface to access the current stats across the mine field.
pub struct MineFieldStats {
    pub uncovered_blank: u32,
    pub covered_mine: u32,
    pub exploded: u32,
}

pub struct Field {
    pub initialized: bool,
    pub size: Size,
    pub mines: u32,
    pub stats: MineFieldStats,
    data: Box<[Cell]>,
}

impl Index<Coord> for Field {
    type Output = Cell;

    fn index(&self, index: Coord) -> &Self::Output {
        &self.data[self.size.index(index)]
    }
}

impl IndexMut<Coord> for Field {
    fn index_mut(&mut self, index: Coord) -> &mut Self::Output {
        let i = self.size.index(index);
        &mut self.data[i]
    }
}

impl Field {
    pub fn new(rows: u32, columns: u32, mines: u32) -> Self {
        let rows = rows.max(2);
        let columns = columns.max(2);
        let mines = mines.clamp(1, rows * columns - 1);
        Self {
            initialized: false,
            data: vec![Cell::default(); (columns * rows) as _].into(),
            size: Size(rows, columns),
            mines,
            stats: MineFieldStats {
                uncovered_blank: 0,
                covered_mine: mines,
                exploded: 0,
            },
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=&[Cell]> {
        self.data.chunks(self.size.1 as _)
    }

    fn iter_neighborhood(&self, center: Coord) -> impl Iterator<Item=&Cell> {
        neighborhood_of(center, self.size).map(move |i| &self[i])
    }

    pub fn initialize(&mut self, avoid: Coord) {
        let avoid_index = self.size.index(avoid);
        let mut rng = rand::thread_rng();
        for mut i in rand::seq::index::sample(&mut rng, (self.size.size() - 1) as _, self.mines as _) {
            if i >= avoid_index {
                i += 1;
            }
            self.data[i].value = Mine;
        }
        for coord in self.size.valid_indices() {
            if self[coord].value != Mine {
                let value = self.iter_neighborhood(coord)
                    .filter(|c| c.value == Mine)
                    .count() as u32;
                self[coord].value = Number(value);
            }
        }
        self.initialized = true;
    }

    // primitive actions
    fn reveal(&mut self, coords: impl Iterator<Item=Coord>) {
        // flood-fill
        // reveal all adjacent cells if the current cell has a value of 0
        let mut queue = VecDeque::with_capacity(self.size.size() as _);
        queue.extend(coords);
        while let Some(coord) = queue.pop_front() {
            if self[coord].state == Covered {
                self[coord].state = Uncovered;
                if self[coord].value == Mine {
                    self.stats.covered_mine -= 1;
                } else {
                    self.stats.uncovered_blank += 1;
                }
                if self[coord].value == Number(0) {
                    queue.extend(neighborhood_of(coord, self.size)
                        .filter(|&i| self[i].state == Covered));
                }
            }
        }
    }

    fn reveal_around(&mut self, coord: Coord) {
        self.reveal(neighborhood_of(coord, self.size));
    }

    // simple actions
    pub fn uncover(&mut self, coord: Coord) {
        if self[coord].value == Mine {
            self.stats.exploded += 1;
            self[coord].state = Exploded;
        } else {
            self.reveal(iter::once(coord));
        }
    }

    // uncovers around cell, returns true if the field has changed
    pub fn uncover_around(&mut self, coord: Coord) -> bool {
        match self[coord].value {
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

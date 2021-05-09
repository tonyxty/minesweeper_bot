use std::collections::vec_deque::VecDeque;
use std::iter::once;

use crate::game::Coord;

// In our UI there is no flagging & unflagging; a cell with mine is uncovered when the player
// decided to "uncover-around" an adjacent cell.  But we use an enum here for extensibility.
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

struct NeighborhoodCoordIterator {
    rows: usize,
    columns: usize,
    center: Coord,
    current: Coord,
    exhausted: bool,
}

impl NeighborhoodCoordIterator {
    pub fn new(rows: usize, columns: usize, center: Coord) -> Self {
        let mut current = center;
        if current.0 > 0 { current.0 -= 1; }
        if current.1 > 0 { current.1 -= 1; }
        Self {
            rows,
            columns,
            center,
            current,
            exhausted: false,
        }
    }

    fn move_next(&mut self) {
        let center = &self.center;
        let current = &mut self.current;
        current.1 += 1; // move to next column
        if current.1 > center.1 + 1 || current.1 >= self.columns {
            // if exhausted current row
            current.0 += 1; // move to next row
            if current.0 > center.0 + 1 || current.0 >= self.rows {
                // if all rows exhausted
                self.exhausted = true;
                return;
            }
            current.1 = if center.1 > 0 { center.1 - 1 } else { 0 } // move to first column
        }
    }
}

impl Iterator for NeighborhoodCoordIterator {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            None
        } else {
            if self.current == self.center {
                self.move_next();
                if self.exhausted {
                    return None;
                }
            }
            let ret = self.current;
            self.move_next();
            Some(ret)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // TODO: this can be improved
        (3, Some(8))
    }
}

// "state" (win/loss) is not part of the MineField struct because we may support other modes of
// deciding game outcome, such as Multiple Lives or Tap in Windows 10 Minesweeper daily challenges.
// Instead we provide an interface to access the current stats across the mine field.
pub struct MineFieldStats {
    pub uncovered_blank: usize,
    pub covered_mine: usize,
    pub exploded: usize,
}

pub struct MineField {
    initialized: bool,
    field: Vec<Cell>,
    rows: usize,
    columns: usize,
    mines: usize,
    stats: MineFieldStats,
}

impl MineField {
    pub fn new(rows: usize, columns: usize, mines: usize) -> Self {
        let rows = rows.max(2);
        let columns = columns.max(2);
        let mines = mines.clamp(1, rows * columns - 1);
        Self {
            initialized: false,
            field: Vec::new(),
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

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn get_rows(&self) -> usize {
        self.rows
    }

    pub fn get_columns(&self) -> usize {
        self.columns
    }

    pub fn get_mines(&self) -> usize {
        self.mines
    }

    pub fn get_stats(&self) -> &MineFieldStats {
        &self.stats
    }

    fn get_index(&self, coord: Coord) -> usize {
        coord.0 * self.columns + coord.1
    }

    pub fn get(&self, coord: Coord) -> &Cell {
        &self.field[self.get_index(coord)]
    }

    fn iter_neighborhood(&self, center: Coord) -> impl Iterator<Item=&Cell> {
        NeighborhoodCoordIterator::new(self.rows, self.columns, center)
            .map(move |i| &self.field[self.get_index(i)])
    }

    pub fn iter_row(&self, row: usize) -> impl Iterator<Item=&Cell> {
        let index = row * self.columns;
        self.field[index..index + self.columns].iter()
    }

    pub fn initialize(&mut self, avoid: Coord) {
        self.field = vec![Cell::default(); self.columns * self.rows];
        let avoid_index = self.get_index(avoid);
        let mut rng = rand::thread_rng();
        for mut i in rand::seq::index::sample(&mut rng, self.columns * self.rows - 1, self.mines).into_iter() {
            if i >= avoid_index {
                i += 1;
            }
            self.field[i].value = Mine;
        }
        for i in 0..self.rows {
            for j in 0..self.columns {
                let coord = (i, j);
                let index = self.get_index(coord);
                if self.field[index].value != Mine {
                    let value = self.iter_neighborhood(coord)
                        .filter(|c| c.value != Mine)
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
        let mut queue = VecDeque::with_capacity(self.columns * self.rows);
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
                    queue.extend(NeighborhoodCoordIterator::new(self.rows, self.columns, coord)
                        .filter(|&i| self.get(i).state == Covered));
                }
            }
        }
    }

    fn reveal_around(&mut self, coord: Coord) {
        self.reveal(NeighborhoodCoordIterator::new(self.rows, self.columns, coord));
    }

    // simple actions
    pub fn uncover(&mut self, coord: Coord) {
        let index = self.get_index(coord);
        if self.field[index].value == Mine {
            self.stats.exploded += 1;
            self.field[index].state = Exploded;
        } else {
            self.reveal(once(coord));
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

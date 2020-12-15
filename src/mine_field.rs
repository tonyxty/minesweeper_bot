use std::collections::vec_deque::VecDeque;
use std::iter::once;

use crate::mine_field::CellStatus::{Covered, Exploded, Uncovered};
use crate::grid_game::Coord;

// In our UI there is no flagging & unflagging; a cell with mine is uncovered when the player
// decided to "uncover-around" an adjacent cell.  But we use an enum here for extensibility.
#[derive(Clone, Eq, PartialEq)]
enum CellStatus {
    Covered,
    Uncovered,
    Exploded,
}

#[derive(Clone)]
pub struct Cell {
    value: i32, // -1 for mines
    status: CellStatus,
}

impl Cell {
    pub fn new() -> Self {
        Self { value: 0, status: Covered }
    }

    pub fn is_mine(&self) -> bool {
        self.value < 0
    }

    pub fn get_value(&self) -> u32 {
        self.value as u32
    }

    pub fn is_covered(&self) -> bool {
        self.status == Covered
    }

    pub fn is_uncovered(&self) -> bool {
        self.status == Uncovered
    }

    pub fn is_exploded(&self) -> bool {
        self.status == Exploded
    }

    fn set_mine(&mut self) {
        self.value = -1;
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
            return None;
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
    pub exploded: usize,
}

impl MineFieldStats {
    fn new() -> Self {
        MineFieldStats {
            uncovered_blank: 0,
            exploded: 0,
        }
    }
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
        let rows = if rows < 2 { 2 } else { rows };
        let columns = if columns < 2 { 2 } else { columns };
        let mut mines = mines;
        if mines < 1 {
            mines = 1;
        } else if mines >= rows * columns {
            mines = rows * columns - 1;
        }
        Self {
            initialized: false,
            field: vec![Cell::new(); columns * rows],
            rows,
            columns,
            mines,
            stats: MineFieldStats::new(),
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
        let avoid_index = self.get_index(avoid);
        let mut rng = rand::thread_rng();
        for mut i in rand::seq::index::sample(&mut rng, self.columns * self.rows - 1, self.mines).into_iter() {
            if i >= avoid_index {
                i += 1;
            }
            self.field[i].set_mine();
        }
        for i in 0..self.rows {
            for j in 0..self.columns {
                let coord = (i, j);
                let index = self.get_index(coord);
                if !self.field[index].is_mine() {
                    let value = self.iter_neighborhood(coord)
                        .filter(|c| c.is_mine())
                        .count() as i32;
                    self.field[index].value = value;
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
            if self.field[index].is_covered() {
                self.field[index].status = Uncovered;
                if !self.field[index].is_mine() {
                    self.stats.uncovered_blank += 1;
                }
                if self.field[index].value == 0 {
                    queue.extend(NeighborhoodCoordIterator::new(self.rows, self.columns, coord)
                        .filter(|i| self.get(*i).is_covered()));
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
        if self.field[index].is_mine() {
            self.stats.exploded += 1;
            self.field[index].status = Exploded;
        } else {
            self.reveal(once(coord));
        }
    }

    pub fn uncover_around(&mut self, coord: Coord) -> bool {
        let index = self.get_index(coord);
        if self.field[index].is_mine() {
            return false;
        }
        // count the number of adjacent covered cells and adjacent uncovered mine cells
        // there are certainly iterator chains that can do this in one statement
        // but a loop seems more readable
        let mut covered = 0;
        let mut uncovered_mines = 0;
        for c in self.iter_neighborhood(coord) {
            if c.is_uncovered() {
                uncovered_mines += c.is_mine() as u32;
            } else {
                covered += 1;
            }
        }
        if covered == 0 {
            return false;
        }
        let value = self.field[index].get_value();
        if uncovered_mines == value || covered + uncovered_mines == value {
            // reveal all adjacent cells
            self.reveal_around(coord);
            true
        } else {
            false
        }
    }
}

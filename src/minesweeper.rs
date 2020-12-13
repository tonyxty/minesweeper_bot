use std::collections::VecDeque;
use std::iter::once;

use crate::minesweeper::CellStatus::{Covered, Exploded, Uncovered};
use crate::minesweeper::InteractResult::{Changed, GameOver, Solved, Unchanged};

pub type Coord = (usize, usize);

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
    value: i32,
    // -1 for mines
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
    width: usize,
    height: usize,
    center: Coord,
    current: Coord,
    exhausted: bool,
}

impl NeighborhoodCoordIterator {
    pub fn new(width: usize, height: usize, center: Coord) -> Self {
        let mut current = center;
        if current.0 > 0 { current.0 -= 1; }
        if current.1 > 0 { current.1 -= 1; }
        Self {
            width,
            height,
            center,
            current,
            exhausted: false,
        }
    }

    fn move_next(&mut self) {
        let center = &self.center;
        let current = &mut self.current;
        current.1 += 1; // move to next column
        if current.1 > center.1 + 1 || current.1 >= self.width {
            // if exhausted current row
            current.0 += 1; // move to next row
            if current.0 > center.0 + 1 || current.0 >= self.height {
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

pub struct MineField {
    initialized: bool,
    field: Vec<Cell>,
    width: usize,
    height: usize,
    mines: usize,
    uncovered: usize,
}

impl MineField {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        Self {
            initialized: false,
            field: vec![Cell::new(); width * height],
            width,
            height,
            mines,
            uncovered: 0,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn get_index(&self, coord: Coord) -> usize {
        coord.0 * self.width + coord.1
    }

    fn get_coord(&self, index: usize) -> Coord {
        (index / self.width, index % self.width)
    }

    fn get(&self, coord: Coord) -> &Cell {
        &self.field[self.get_index(coord)]
    }

    fn iter_neighborhood(&self, center: Coord) -> impl Iterator<Item=&Cell> {
        NeighborhoodCoordIterator::new(self.width, self.height, center)
            .map(move |i| &self.field[self.get_index(i)])
    }

    pub fn iter_row(&self, row: usize) -> impl Iterator<Item=&Cell> {
        let index = row * self.width;
        self.field[index..index + self.width].iter()
    }

    pub fn initialize(&mut self, avoid: Coord) {
        let avoid_index = self.get_index(avoid);
        let mut rng = rand::thread_rng();
        for mut i in rand::seq::index::sample(&mut rng, self.width * self.height - 1, self.mines).into_iter() {
            if i >= avoid_index {
                i += 1;
            }
            self.field[i].set_mine();
        }
        for i in 0..self.height {
            for j in 0..self.width {
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

    pub fn is_solved(&self) -> bool {
        self.uncovered + self.mines == self.width * self.height
    }

    // simple actions
    pub fn uncover(&mut self, coord: Coord) -> bool {
        let index = self.get_index(coord);
        if self.field[index].is_mine() {
            self.field[index].status = Exploded;
            false   // boom, you're dead!
        } else {
            self.reveal(once(coord));
            true
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

    fn reveal(&mut self, coords: impl Iterator<Item=Coord>) {
        // flood-fill
        // reveal all adjacent cells if the current cell has a value of 0
        let mut queue = VecDeque::with_capacity(self.width * self.height);
        queue.extend(coords);
        while let Some(coord) = queue.pop_front() {
            let index = self.get_index(coord);
            if self.field[index].is_covered() {
                self.field[index].status = Uncovered;
                if !self.field[index].is_mine() {
                    self.uncovered += 1;
                }
                if self.field[index].value == 0 {
                    queue.extend(NeighborhoodCoordIterator::new(self.width, self.height, coord)
                        .filter(|i| self.get(*i).is_covered()));
                }
            }
        }
    }

    fn reveal_around(&mut self, coord: Coord) {
        self.reveal(NeighborhoodCoordIterator::new(self.width, self.height, coord));
    }
}


#[derive(Eq, PartialEq)]
pub enum InteractResult {
    Unchanged,
    Changed,
    Solved,
    GameOver,
}

impl MineField {
    // "compound" actions
    pub fn interact(&mut self, coord: Coord) -> InteractResult {
        if !self.is_initialized() {
            self.initialize(coord);
        }
        if self.get(coord).is_covered() {
            if self.uncover(coord) {
                Changed
            } else {
                GameOver
            }
        } else {
            if self.uncover_around(coord) {
                if self.is_solved() {
                    Solved
                } else {
                    Changed
                }
            } else {
                Unchanged
            }
        }
    }
}

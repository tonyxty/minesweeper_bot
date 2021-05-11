use std::ops::{Index, IndexMut};

use crate::game::{Coord, Size};

#[derive(Default)]
pub struct Board {
    pub player: bool,
    pub game_over: bool,
    data: [[Option<bool>; 8]; 8],
}

impl Index<Coord> for Board {
    type Output = Option<bool>;

    fn index(&self, index: Coord) -> &Self::Output {
        &self.data[index.0 as usize][index.1 as usize]
    }
}

impl IndexMut<Coord> for Board {
    fn index_mut(&mut self, index: Coord) -> &mut Self::Output {
        &mut self.data[index.0 as usize][index.1 as usize]
    }
}

impl Board {
    pub fn new() -> Self {
        let mut data: [[Option<bool>; 8]; 8] = Default::default();
        data[3][3] = Some(true);
        data[3][4] = Some(false);
        data[4][3] = Some(false);
        data[4][4] = Some(true);
        Self {
            player: false,
            data,
            game_over: false,
        }
    }

    pub fn get_score(&self) -> (u32, u32) {
        let mut scores = [0, 0];
        for row in &self.data {
            for &i in row {
                if let Some(i) = i {
                    scores[i as usize] += 1;
                }
            }
        }
        (scores[0], scores[1])
    }

    pub fn iter(&self) -> impl Iterator<Item=&[Option<bool>; 8]> {
        self.data.iter()
    }

    fn find_anchor(&self, mut coord: Coord, direction: Coord, player: bool) -> Option<Coord> {
        let mut valid = false;
        while Size(8, 8).contains(coord) {
            coord += direction;
            match self[coord] {
                None => { return None; }
                Some(p) => {
                    if p == player {
                        break;
                    } else {
                        valid = true;
                    }
                }
            }
        }
        valid.then_some(coord)
    }

    fn capture(&mut self, coord: Coord, direction: Coord) -> bool {
        let anchor = self.find_anchor(coord, direction, self.player);
        if let Some(mut anchor) = anchor {
            while anchor != coord {
                anchor -= direction;
                self[anchor] = Some(self.player);
            }
            true
        } else {
            false
        }
    }

    fn has_move(&self, player: bool) -> bool {
        Size(8, 8).valid_indices()
            .any(|c| self[c] == None && Coord::DIRECTIONS.iter()
                .any(|&d| self.find_anchor(c, d, player).is_some())
            )
    }

    pub fn play(&mut self, coord: Coord) -> bool {
        let mut valid = false;
        for &direction in &Coord::DIRECTIONS {
            valid |= self.capture(coord, direction)
        }
        if valid {
            self[coord] = Some(self.player);
            if self.has_move(!self.player) {
                // If the opposing player has a move
                self.player = !self.player;
            } else if !self.has_move(self.player) {
                // If neither player has a move
                self.game_over = true;
            }
        }
        valid
    }
}

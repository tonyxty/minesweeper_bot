use crate::game::Coord;

#[derive(Default)]
pub struct Othello {
    player: bool,
    board: [[Option<bool>; 8]; 8],
    game_over: bool,
}

impl Othello {
    pub fn new() -> Self {
        let mut board: [[Option<bool>; 8]; 8] = Default::default();
        board[3][3] = Some(true);
        board[3][4] = Some(false);
        board[4][3] = Some(false);
        board[4][4] = Some(true);
        Self {
            player: false,
            board,
            game_over: false,
        }
    }

    pub fn get_current_player(&self) -> bool {
        self.player
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn get_score(&self) -> (u32, u32) {
        let mut scores = [0, 0];
        for row in self.board.iter() {
            for i in row {
                if let Some(i) = *i {
                    scores[i as usize] += 1;
                }
            }
        }
        (scores[0], scores[1])
    }

    pub fn iter_row(&self, row: usize) -> impl Iterator<Item=&Option<bool>> {
        self.board[row].iter()
    }

    const DIRECTIONS: [(i32, i32); 8] = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];

    fn find_anchor(&self, coord: (i32, i32), direction: (i32, i32), player: bool) -> Option<(i32, i32)> {
        let mut anchor = coord;
        let mut valid = false;
        loop {
            anchor.0 += direction.0;
            anchor.1 += direction.1;
            if anchor.0 < 0 || anchor.0 >= 8 || anchor.1 < 0 || anchor.1 >= 8 {
                return None;
            }
            match self.board[anchor.0 as usize][anchor.1 as usize] {
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
        return if valid { Some(anchor) } else { None };
    }

    fn capture(&mut self, coord: (i32, i32), direction: (i32, i32)) -> bool {
        let anchor = self.find_anchor(coord, direction, self.player);
        if let Some(mut anchor) = anchor {
            loop {
                anchor.0 -= direction.0;
                anchor.1 -= direction.1;
                if anchor == coord { break; }
                self.board[anchor.0 as usize][anchor.1 as usize] = Some(self.player);
            }
            true
        } else {
            false
        }
    }

    fn has_move(&self, player: bool) -> bool {
        for i in 0..8 {
            for j in 0..8 {
                if self.board[i as usize][j as usize] == None {
                    for direction in Self::DIRECTIONS.iter() {
                        if self.find_anchor((i, j), *direction, player).is_some() {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn play(&mut self, coord: Coord) -> bool {
        let coord_i32 = (coord.0 as i32, coord.1 as i32);
        let mut valid = false;
        for direction in Self::DIRECTIONS.iter() {
            valid |= self.capture(coord_i32, *direction)
        }
        if valid {
            self.board[coord.0][coord.1] = Some(self.player);
            if self.has_move(!self.player) {
                self.player = !self.player;
            } else if !self.has_move(self.player) {
                self.game_over = true;
            }
        }
        valid
    }
}

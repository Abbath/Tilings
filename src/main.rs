use rand::Rng;
use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq)]
enum Orientation {
    Top = 1,
    Bottom = 2,
    Left = 3,
    Right = 4,
}

struct Tile {
    coord: (usize, usize),
    orientation: Orientation,
}

struct Diamond {
    size: usize,
    data: Vec<i64>,
    tiles: HashMap<usize, Tile>,
    tile_id: usize,
}

impl Diamond {
    pub fn new() -> Diamond {
        let mut d = Diamond {
            size: 2,
            data: vec![0; 4],
            tiles: HashMap::new(),
            tile_id: 1,
        };
        d.tile();
        d
    }
    fn at_ref(&mut self, m: usize, n: usize) -> &mut i64 {
        &mut self.data[m * self.size + n]
    }
    fn at(&self, m: usize, n: usize) -> i64 {
        self.data[m * self.size + n]
    }
    fn fill(&mut self) {
        for i in 0..self.size / 2 {
            for j in 0..self.size / 2 {
                if i + j < self.size / 2 - 1 {
                    *self.at_ref(i, j) = -1;
                    *self.at_ref(self.size - i - 1, j) = -1;
                    *self.at_ref(i, self.size - j - 1) = -1;
                    *self.at_ref(self.size - i - 1, self.size - j - 1) = -1;
                }
            }
        }
    }
    fn clear_square(&mut self, i: usize, j: usize) {
        *self.at_ref(i, j) = 0;
        *self.at_ref(i + 1, j) = 0;
        *self.at_ref(i, j + 1) = 0;
        *self.at_ref(i + 1, j + 1) = 0;
    }
    fn extend(&mut self) {
        let new_size = self.size + 2;
        let mut new_data = vec![0; new_size * new_size];
        for i in 0..self.size {
            for j in 0..self.size {
                if self.at(i, j) != -1 {
                    new_data[(i + 1) * new_size + j + 1] = self.at(i, j);
                }
            }
        }
        self.data = new_data;
        self.size = new_size;
        self.fill();
        for (_, tile) in self.tiles.iter_mut() {
            tile.coord = (tile.coord.0 + 1, tile.coord.1 + 1);
        }
    }
    fn find_square(&self) -> Option<(usize, usize)> {
        for i in 0..self.size - 1 {
            for j in 0..self.size - 1 {
                if self.at(i, j) == 0
                    && self.at(i + 1, j) == 0
                    && self.at(i, j + 1) == 0
                    && self.at(i + 1, j + 1) == 0
                {
                    return Some((i, j));
                }
            }
        }
        None
    }
    fn tile_square(&mut self, c: (usize, usize)) {
        let mut rng = rand::thread_rng();
        let dir = rng.gen::<u64>() % 2;
        if dir == 0 {
            *self.at_ref(c.0, c.1) = self.tile_id as i64;
            *self.at_ref(c.0, c.1 + 1) = self.tile_id as i64;
            self.tiles.insert(
                self.tile_id,
                Tile {
                    coord: c,
                    orientation: Orientation::Top,
                },
            );
            self.tile_id += 1;
            *self.at_ref(c.0 + 1, c.1) = self.tile_id as i64;
            *self.at_ref(c.0 + 1, c.1 + 1) = self.tile_id as i64;
            self.tiles.insert(
                self.tile_id,
                Tile {
                    coord: (c.0 + 1, c.1),
                    orientation: Orientation::Bottom,
                },
            );
            self.tile_id += 1;
        } else {
            *self.at_ref(c.0, c.1) = self.tile_id as i64;
            *self.at_ref(c.0 + 1, c.1) = self.tile_id as i64;
            self.tiles.insert(
                self.tile_id,
                Tile {
                    coord: c,
                    orientation: Orientation::Left,
                },
            );
            self.tile_id += 1;
            *self.at_ref(c.0, c.1 + 1) = self.tile_id as i64;
            *self.at_ref(c.0 + 1, c.1 + 1) = self.tile_id as i64;
            self.tiles.insert(
                self.tile_id,
                Tile {
                    coord: (c.0, c.1 + 1),
                    orientation: Orientation::Right,
                },
            );
            self.tile_id += 1;
        }
    }
    fn eliminate_stuck_tiles(&mut self) {
        for i in 0..self.size - 1 {
            for j in 0..self.size - 1 {
                if self.at(i, j) > 0 {
                    let tile_id = self.at(i, j) as usize;
                    if self.tiles[&tile_id].orientation == Orientation::Bottom
                        && self.at(i + 1, j) > 0
                    {
                        let tile_id_2 = self.at(i + 1, j) as usize;
                        if self.tiles[&tile_id_2].orientation == Orientation::Top {
                            self.tiles.remove(&tile_id);
                            self.tiles.remove(&tile_id_2);
                            self.clear_square(i, j);
                        }
                    } else if self.tiles[&tile_id].orientation == Orientation::Right
                        && self.at(i, j + 1) > 0
                    {
                        let tile_id_2 = self.at(i, j + 1) as usize;
                        if self.tiles[&tile_id_2].orientation == Orientation::Left {
                            self.tiles.remove(&tile_id);
                            self.tiles.remove(&tile_id_2);
                            self.clear_square(i, j);
                        }
                    }
                }
            }
        }
    }
    fn move_tiles(&mut self) {
        let mut to_move: Vec<(usize, (usize, usize), Orientation)> = Vec::new();
        for (id, tile) in self.tiles.iter_mut() {
            let c = tile.coord;
            to_move.push((*id, c, tile.orientation));
            match tile.orientation {
                Orientation::Top => {
                    tile.coord = (c.0 - 1, c.1);
                }
                Orientation::Bottom => {
                    tile.coord = (c.0 + 1, c.1);
                }
                Orientation::Left => {
                    tile.coord = (c.0, c.1 - 1);
                }
                Orientation::Right => {
                    tile.coord = (c.0, c.1 + 1);
                }
            }
        }
        for m in to_move {
            let (id, (i, j), o) = m;
            match o {
                Orientation::Top => {
                    if self.at(i, j) == id as i64 {
                        *self.at_ref(i, j) = 0;
                    }
                    if self.at(i, j + 1) == id as i64 {
                        *self.at_ref(i, j + 1) = 0;
                    }
                    *self.at_ref(i - 1, j) = id as i64;
                    *self.at_ref(i - 1, j + 1) = id as i64;
                }
                Orientation::Bottom => {
                    if self.at(i, j) == id as i64 {
                        *self.at_ref(i, j) = 0;
                    }
                    if self.at(i, j + 1) == id as i64 {
                        *self.at_ref(i, j + 1) = 0;
                    }
                    *self.at_ref(i + 1, j) = id as i64;
                    *self.at_ref(i + 1, j + 1) = id as i64;
                }
                Orientation::Left => {
                    if self.at(i, j) == id as i64 {
                        *self.at_ref(i, j) = 0;
                    }
                    if self.at(i + 1, j) == id as i64 {
                        *self.at_ref(i + 1, j) = 0;
                    }
                    *self.at_ref(i, j - 1) = id as i64;
                    *self.at_ref(i + 1, j - 1) = id as i64;
                }
                Orientation::Right => {
                    if self.at(i, j) == id as i64 {
                        *self.at_ref(i, j) = 0;
                    }
                    if self.at(i + 1, j) == id as i64 {
                        *self.at_ref(i + 1, j) = 0;
                    }
                    *self.at_ref(i, j + 1) = id as i64;
                    *self.at_ref(i + 1, j + 1) = id as i64;
                }
            }
        }
    }
    fn tile(&mut self) {
        while let Some(c) = self.find_square() {
            self.tile_square(c)
        }
    }
    pub fn step(&mut self) {
        self.eliminate_stuck_tiles();
        self.extend();
        self.move_tiles();
        self.tile();
    }
    pub fn generate(&mut self, n: u64) {
        for _ in 0..n {
            self.step();
        }
    }
    pub fn print(&self) {
        for i in 0..self.size {
            for j in 0..self.size {
                if self.at(i, j) > 0 {
                    print!(
                        " {} ",
                        match self.tiles[&(self.at(i, j) as usize)].orientation {
                            Orientation::Top => {
                                "T"
                            }
                            Orientation::Bottom => {
                                "B"
                            }
                            Orientation::Left => {
                                "L"
                            }
                            Orientation::Right => {
                                "R"
                            }
                        }
                    );
                } else {
                    print!("   ");
                }
            }
            println!();
        }
        println!();
    }
    pub fn print_debug(&self) {
        for i in 0..self.size {
            for j in 0..self.size {
                print!("{:4} ", self.at(i, j))
            }
            println!();
        }
        println!();
    }
}

fn main() {
    let mut x = Diamond::new();
    for _ in 1..20 {
        x.step();
        x.print();
    }
    x.generate(10);
    x.print();
}

use std::collections::HashMap;
use rand::Rng;

#[derive(Copy, Clone, PartialEq)]
enum Orientation {
    Top = 1,
    Bottom = 2,
    Left = 3,
    Right = 4
}

struct Tile {
    coord: (usize, usize),
    orientation: Orientation
}

struct Diamond {
    size: usize,
    data: Vec<i64>,
    tiles: HashMap<usize, Tile>,
    tile_id: usize
} 

impl Diamond {
    pub fn new(a: usize) -> Diamond {
        let size = if a % 2 != 0 {
            a + 1
        }else{
            if a == 0 {
                2
            }else{
                a
            }
        };
        Diamond{size: size, data: vec![0; a * a], tiles: HashMap::new(), tile_id: 1}
    }
    pub fn at_ref(&mut self, m: usize, n: usize) -> &mut i64 {
        &mut self.data[m*self.size+n]
    }
    pub fn at(&self, m: usize, n: usize) -> i64 {
        self.data[m*self.size+n]
    }
    pub fn fill(&mut self) {
        for i in 0..self.size/2 {
            for j in 0..self.size/2 {
                if i + j < self.size / 2 - 1 {
                    *self.at_ref(i, j) = -1;
                    *self.at_ref(self.size - i - 1, j) = -1;
                    *self.at_ref(i, self.size - j - 1) = -1;
                    *self.at_ref(self.size - i - 1, self.size - j - 1) = -1;
                }
            }
        }
    }
    pub fn clear_square(&mut self, i: usize, j: usize) {
        *self.at_ref(i, j) = 0;
        *self.at_ref(i+1, j) = 0;
        *self.at_ref(i, j+1) = 0;
        *self.at_ref(i+1, j+1) = 0;
    }
    pub fn extend(&mut self) {
        let new_size = self.size + 2;
        let mut new_data = vec![0; new_size * new_size];
        for i in 0..self.size {
            for j in 0..self.size {
                if self.at(i, j) != -1 {
                    new_data[(i+1)*new_size + j + 1] = self.at(i, j);
                }
            }
        }
        self.data = new_data;
        self.size = new_size;
        self.fill();
        for (_, tile) in self.tiles.iter_mut() {
            tile.coord = (tile.coord.0 + 1, tile.coord.1 + 1 );
        }
    }
    pub fn find_square(&self) -> Option<(usize, usize)> {
        for i in 0..self.size - 1 {
            for j in 0..self.size - 1 {
                if self.at(i, j) == 0 && self.at(i+1, j) == 0 && self.at(i, j+1) == 0 && self.at(i+1, j+1) == 0 {
                    return Some((i, j));
                }    
            }
        }
        None
    }
    pub fn tile_square(&mut self, c: (usize, usize)) {
        let mut rng = rand::thread_rng();
        let dir = rng.gen::<u64>() % 2;
        if dir == 0 {
            *self.at_ref(c.0, c.1) = self.tile_id as i64;
            *self.at_ref(c.0, c.1+1) = self.tile_id as i64;
            self.tiles.insert(self.tile_id, Tile{coord: c, orientation: Orientation::Top});
            self.tile_id += 1;
            *self.at_ref(c.0+1, c.1) = self.tile_id as i64;
            *self.at_ref(c.0+1, c.1+1) = self.tile_id as i64;
            self.tiles.insert(self.tile_id, Tile{coord: (c.0+1, c.1), orientation: Orientation::Bottom});
            self.tile_id += 1;   
        }else{
            *self.at_ref(c.0, c.1) = self.tile_id as i64;
            *self.at_ref(c.0+1, c.1) = self.tile_id as i64;
            self.tiles.insert(self.tile_id, Tile{coord: c, orientation: Orientation::Left});
            self.tile_id += 1;
            *self.at_ref(c.0, c.1+1) = self.tile_id as i64;
            *self.at_ref(c.0+1, c.1+1) = self.tile_id as i64;
            self.tiles.insert(self.tile_id, Tile{coord: (c.0, c.1+1), orientation: Orientation::Right});
            self.tile_id += 1;
        }
    }
    pub fn eliminate_stuck_tiles(&mut self) {
        for i in 0..self.size - 1 {
            for j in 0..self.size - 1 {
                if self.at(i, j) > 0 {
                    let tile_id = self.at(i, j) as usize;
                    if self.tiles[&tile_id].orientation == Orientation::Bottom {
                        if self.at(i+1, j) > 0 {
                            let tile_id_2 = self.at(i+1, j) as usize;
                            if self.tiles[&tile_id_2].orientation == Orientation::Top {
                                self.tiles.remove(&tile_id);
                                self.tiles.remove(&tile_id_2);
                                self.clear_square(i, j);
                            }
                        }
                    } else if self.tiles[&tile_id].orientation == Orientation::Right {
                        if self.at(i, j+1) > 0 {
                            let tile_id_2 = self.at(i, j+1) as usize;
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
    }
    pub fn move_tiles(&mut self) {
        let mut to_move : Vec<(usize, (usize, usize), Orientation)> = Vec::new();
        for (id, tile) in self.tiles.iter_mut() {
            let c = tile.coord;
            to_move.push((*id, c, tile.orientation));
            match tile.orientation {
                Orientation::Top => {
                    tile.coord = (c.0-1, c.1);
                },
                Orientation::Bottom => {
                    tile.coord = (c.0+1, c.1);
                },
                Orientation::Left => {
                    tile.coord = (c.0, c.1-1);
                },
                Orientation::Right => {
                    tile.coord = (c.0, c.1+1);
                }
            }
        }
        for m in to_move {
            let (id, (i, j), o) = m;
            match o {
                Orientation::Top => {
                    if self.at(i,j) == id as i64 {
                        *self.at_ref(i, j) = 0;
                    }
                    if self.at(i,j+1) == id as i64 {
                        *self.at_ref(i, j+1) = 0;
                    }
                    *self.at_ref(i-1, j) = id as i64;
                    *self.at_ref(i-1, j+1) = id as i64;
                },
                Orientation::Bottom => {
                    if self.at(i,j) == id as i64 {
                        *self.at_ref(i, j) = 0;
                    }
                    if self.at(i,j+1) == id as i64 {
                        *self.at_ref(i, j+1) = 0;
                    }
                    *self.at_ref(i+1, j) = id as i64;
                    *self.at_ref(i+1, j+1) = id as i64;
                },
                Orientation::Left => {
                    if self.at(i,j) == id as i64 {
                        *self.at_ref(i, j) = 0;
                    }
                    if self.at(i+1,j) == id as i64 {
                        *self.at_ref(i+1, j) = 0;
                    }
                    *self.at_ref(i, j-1) = id as i64;
                    *self.at_ref(i+1, j-1) = id as i64;
                },
                Orientation::Right => {
                    if self.at(i,j) == id as i64 {
                        *self.at_ref(i, j) = 0;
                    }
                    if self.at(i+1,j) == id as i64 {
                        *self.at_ref(i+1, j) = 0;
                    }
                    *self.at_ref(i, j+1) = id as i64;
                    *self.at_ref(i+1, j+1) = id as i64;
                }
            }
        }
    }
    pub fn tile(&mut self) {
        loop {
            match self.find_square() {
                Some(c) => {
                    self.tile_square(c)
                },
                None => {
                    break
                }
            }
        }
    }
    pub fn print(&self) {
        for i in 0..self.size {
            for j in 0..self.size {
                if self.at(i, j) > 0 {
                    print!(" {} ", match self.tiles[&(self.at(i, j) as usize)].orientation {
                        Orientation::Top => {
                            "T"
                        },
                        Orientation::Bottom => {
                            "B"
                        },
                        Orientation::Left => {
                            "L"
                        },
                        Orientation::Right => {
                            "R"
                        }
                    });
                }else{
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

fn tilings(m: u64, n: u64) -> u64 {
    let mut prod = 1.0;
    for j in 1..=f64::ceil(m as f64 / 2.0) as u64 {
        let mut p = 1.0;
        for k in 1..=f64::ceil(n as f64 / 2.0) as u64 {
            p *= 4.0
                * (j as f64 * std::f64::consts::PI / (m as f64 + 1.0))
                    .cos()
                    .powf(2.0)
                + 4.0
                    * (k as f64 * std::f64::consts::PI / (n as f64 + 1.0))
                        .cos()
                        .powf(2.0);
        }
        prod *= p;
    }
    return prod as u64;
}

fn main() {
    // let mut s1 = String::new();
    // let mut s2 = String::new();
    // let stdin = io::stdin();
    // let mut handle = stdin.lock();
    // handle.read_line(&mut s1).expect("NO SHIT!");
    // handle.read_line(&mut s2).expect("NO SHIT!");
    // let m = s1.trim().parse::<u64>().expect("NOT A NUMBER!");
    // let n = s2.trim().parse::<u64>().expect("NOT A NUMBER!");
    // println!("{}", tilings(m, n));
    let mut x = Diamond::new(2);
    x.fill();
    x.tile();
    for i in 1..20 {
        x.eliminate_stuck_tiles();
        // x.print();
        x.extend();
        x.move_tiles();
        x.tile();
        // x.print();
        // x.print();
        x.print();
    }
    // x.tile();
    // x.print();
}

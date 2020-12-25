use rand::Rng;
use raqote::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

#[derive(Copy, Clone, PartialEq)]
enum Orientation {
    Top = 1,
    Bottom = 2,
    Left = 3,
    Right = 4,
}

#[derive(Copy, Clone)]
struct Tile {
    coord: (usize, usize),
    orientation: Orientation,
}

struct Diamond {
    size: usize,
    data: Vec<i64>,
    tiles: HashMap<usize, Tile>,
    tile_id: usize,
    free_ids: VecDeque<usize>,
    current_row: usize,
}

impl Diamond {
    pub fn new() -> Diamond {
        let mut d = Diamond {
            size: 2,
            data: vec![0; 4],
            tiles: HashMap::new(),
            tile_id: 1,
            free_ids: VecDeque::new(),
            current_row: 0,
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
    fn find_square(&mut self) -> Option<(usize, usize)> {
        for i in self.current_row..self.size - 1 {
            for j in 0..self.size - 1 {
                if self.at(i, j) == 0
                    && self.at(i + 1, j) == 0
                    && self.at(i, j + 1) == 0
                    && self.at(i + 1, j + 1) == 0
                {
                    self.current_row = i;
                    return Some((i, j));
                }
            }
        }
        self.current_row = 0;
        None
    }
    fn next_tile_id(&mut self) -> usize {
        if !self.free_ids.is_empty() {
            let tid = self.free_ids.pop_front().unwrap();
            tid
        } else {
            let tid = self.tile_id;
            self.tile_id += 1;
            tid
        }
    }
    fn tile_square(&mut self, c: (usize, usize)) {
        let mut rng = rand::thread_rng();
        let dir = rng.gen::<u64>() % 2;
        if dir == 0 {
            let tid = self.next_tile_id();
            *self.at_ref(c.0, c.1) = tid as i64;
            *self.at_ref(c.0, c.1 + 1) = tid as i64;
            self.tiles.insert(
                tid,
                Tile {
                    coord: c,
                    orientation: Orientation::Top,
                },
            );
            let tid = self.next_tile_id();
            *self.at_ref(c.0 + 1, c.1) = tid as i64;
            *self.at_ref(c.0 + 1, c.1 + 1) = tid as i64;
            self.tiles.insert(
                tid,
                Tile {
                    coord: (c.0 + 1, c.1),
                    orientation: Orientation::Bottom,
                },
            );
        } else {
            let tid = self.next_tile_id();
            *self.at_ref(c.0, c.1) = tid as i64;
            *self.at_ref(c.0 + 1, c.1) = tid as i64;
            self.tiles.insert(
                tid,
                Tile {
                    coord: c,
                    orientation: Orientation::Left,
                },
            );
            let tid = self.next_tile_id();
            *self.at_ref(c.0, c.1 + 1) = tid as i64;
            *self.at_ref(c.0 + 1, c.1 + 1) = tid as i64;
            self.tiles.insert(
                tid,
                Tile {
                    coord: (c.0, c.1 + 1),
                    orientation: Orientation::Right,
                },
            );
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
                            self.free_ids.push_back(tile_id);
                            self.free_ids.push_back(tile_id_2);
                        }
                    } else if self.tiles[&tile_id].orientation == Orientation::Right
                        && self.at(i, j + 1) > 0
                    {
                        let tile_id_2 = self.at(i, j + 1) as usize;
                        if self.tiles[&tile_id_2].orientation == Orientation::Left {
                            self.tiles.remove(&tile_id);
                            self.tiles.remove(&tile_id_2);
                            self.clear_square(i, j);
                            self.free_ids.push_back(tile_id);
                            self.free_ids.push_back(tile_id_2);
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
    pub fn draw(&self, fname: &str, tile_size: usize) {
        let mut dt = DrawTarget::new(
            (self.size * tile_size) as i32,
            (self.size * tile_size) as i32,
        );
        let mut drawn: HashSet<i64> = HashSet::new();
        for i in 0..self.size {
            for j in 0..self.size {
                if self.at(i, j) > 0 {
                    if drawn.contains(&self.at(i, j)) {
                        continue;
                    }
                    let tile = self.tiles[&(self.at(i, j) as usize)];
                    let (src, w, h) = match tile.orientation {
                        Orientation::Top => (
                            Source::Solid(SolidSource {
                                r: 255,
                                g: 0,
                                b: 0,
                                a: 255,
                            }),
                            2,
                            1,
                        ),
                        Orientation::Bottom => (
                            Source::Solid(SolidSource {
                                r: 0,
                                g: 0,
                                b: 255,
                                a: 255,
                            }),
                            2,
                            1,
                        ),
                        Orientation::Left => (
                            Source::Solid(SolidSource {
                                r: 255,
                                g: 255,
                                b: 0,
                                a: 255,
                            }),
                            1,
                            2,
                        ),
                        Orientation::Right => (
                            Source::Solid(SolidSource {
                                r: 0,
                                g: 255,
                                b: 0,
                                a: 255,
                            }),
                            1,
                            2,
                        ),
                    };
                    let mut pb = PathBuilder::new();
                    pb.move_to((j * tile_size) as f32, (i * tile_size) as f32);
                    pb.line_to((j * tile_size) as f32, ((i + h) * tile_size) as f32);
                    pb.line_to(((j + w) * tile_size) as f32, ((i + h) * tile_size) as f32);
                    pb.line_to(((j + w) * tile_size) as f32, (i * tile_size) as f32);
                    let path = pb.finish();
                    dt.stroke(
                        &path,
                        &Source::Solid(SolidSource {
                            r: 0,
                            g: 0,
                            b: 0,
                            a: 255,
                        }),
                        &StrokeStyle::default(),
                        &DrawOptions::new(),
                    );
                    dt.fill_rect(
                        (j * tile_size) as f32 + 1.0,
                        (i * tile_size) as f32 + 1.0,
                        (tile_size * w) as f32 - 2.0,
                        (tile_size * h) as f32 - 2.0,
                        &src,
                        &DrawOptions::new(),
                    );
                    drawn.insert(self.at(i, j));
                } else {
                    dt.fill_rect(
                        (i * tile_size) as f32,
                        (j * tile_size) as f32,
                        tile_size as f32,
                        tile_size as f32,
                        &Source::Solid(SolidSource {
                            r: 128,
                            g: 128,
                            b: 128,
                            a: 255,
                        }),
                        &DrawOptions::new(),
                    )
                }
            }
        }
        dt.write_png(fname).expect("FAILED TO SAVE AN IMAGE!");
    }
}

fn main() {
    let mut x = Diamond::new();
    for _ in 1..20 {
        x.step();
        x.print();
    }
    x.generate(108);
    // x.print();
    x.draw("test.png", 16);
}

use clap::Clap;
use image::{Rgba, RgbaImage};
use image::imageops::{resize, FilterType};
use imageproc::drawing::{draw_filled_rect_mut, draw_hollow_rect_mut};
use imageproc::rect::Rect;
use rand::Rng;
use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::num::ParseIntError;

type Coords = (usize, usize);

#[derive(Copy, Clone, PartialEq)]
enum Orientation {
    Top = 1,
    Bottom = 2,
    Left = 3,
    Right = 4,
}

#[derive(Copy, Clone)]
struct Tile {
    coord: Coords,
    orientation: Orientation,
}

struct Diamond {
    size: usize,
    data: Vec<i64>,
    tiles: HashMap<usize, Tile>,
    tile_id: usize,
    free_ids: VecDeque<usize>,
    current_square: Coords,
}

impl Diamond {
    pub fn new() -> Diamond {
        let mut d = Diamond {
            size: 2,
            data: vec![0; 4],
            tiles: HashMap::new(),
            tile_id: 1,
            free_ids: VecDeque::new(),
            current_square: (0, 0),
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
        self.tiles.par_iter_mut().for_each(|(_, tile)| {
            tile.coord = (tile.coord.0 + 1, tile.coord.1 + 1);
        });
    }
    fn find_square(&mut self) -> Option<Coords> {
        for i in self.current_square.0..self.size - 1 {
            for j in (if i == self.current_square.0 {
                self.current_square.1
            } else {
                0
            })..self.size - 1
            {
                if self.at(i, j) == 0
                    && self.at(i + 1, j) == 0
                    && self.at(i, j + 1) == 0
                    && self.at(i + 1, j + 1) == 0
                {
                    self.current_square = (i, j);
                    return Some((i, j));
                }
            }
        }
        self.current_square = (0, 0);
        None
    }
    fn next_tile_id(&mut self) -> usize {
        if !self.free_ids.is_empty() {
            self.free_ids.pop_front().unwrap()
        } else {
            let tid = self.tile_id;
            self.tile_id += 1;
            tid
        }
    }
    fn tile_square(&mut self, c: Coords) {
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
        let mut to_move: Vec<(usize, Coords, Orientation)> = Vec::new();
        for (id, tile) in self.tiles.iter_mut() {
            to_move.push((*id, tile.coord, tile.orientation));
        }
        self.tiles.par_iter_mut().for_each(|(_, tile)| {
            let c = tile.coord;
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
        });
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn print_debug(&self) {
        for i in 0..self.size {
            for j in 0..self.size {
                print!("{:4} ", self.at(i, j))
            }
            println!();
        }
        println!();
    }
    fn int_to_color(&self, c: u32) -> Rgba<u8> {
        Rgba([
            (c >> 24 & 0xff) as u8,
            (c >> 16 & 0xff) as u8,
            (c >> 8 & 0xff) as u8,
            (c & 0xff) as u8,
        ])
    }
    pub fn draw_image(&self, fname: &str, ts: usize, colors: &Colors) {
        let tile_size = if ts > 16 {ts / 2} else {ts};
        let mut im = RgbaImage::new(
            (self.size * tile_size) as u32,
            (self.size * tile_size) as u32,
        );
        let mut drawn: HashSet<i64> = HashSet::new();
        let gray = Rgba([128, 128, 128, 255]);
        let black = Rgba([0, 0, 0, 255]);
        for i in 0..self.size {
            for j in 0..self.size {
                if self.at(i, j) > 0 {
                    if drawn.contains(&self.at(i, j)) {
                        continue;
                    }
                    let tile = self.tiles[&(self.at(i, j) as usize)];
                    let (src, w, h) = match tile.orientation {
                        Orientation::Top => (self.int_to_color(colors.top), 2, 1),
                        Orientation::Bottom => (self.int_to_color(colors.bottom), 2, 1),
                        Orientation::Left => (self.int_to_color(colors.left), 1, 2),
                        Orientation::Right => (self.int_to_color(colors.right), 1, 2),
                    };
                    draw_hollow_rect_mut(
                        &mut im,
                        Rect::at((j * tile_size) as i32, (i * tile_size) as i32)
                            .of_size((w * tile_size) as u32, (h * tile_size) as u32),
                        black,
                    );
                    draw_filled_rect_mut(
                        &mut im,
                        Rect::at((j * tile_size) as i32 + 1, (i * tile_size) as i32 + 1)
                            .of_size((w * tile_size) as u32 - 2, (h * tile_size) as u32 - 2),
                        src,
                    );
                    drawn.insert(self.at(i, j));
                } else {
                    draw_filled_rect_mut(
                        &mut im,
                        Rect::at((j * tile_size) as i32, (i * tile_size) as i32)
                            .of_size(tile_size as u32, tile_size as u32),
                        gray,
                    );
                }
            }
        }
        if ts > 16 {
            let (w, h) = (im.width(), im.height());
            im = resize(&mut im, w*2, h*2, FilterType::Nearest);
        }
        im.save(fname).expect("FAILED TO SAVE AN IMAGE!");
    }
}

fn parse_hex(input: &str) -> Result<u32, ParseIntError> {
    u32::from_str_radix(input, 16)
}

struct Colors {
    top: u32,
    bottom: u32,
    left: u32,
    right: u32,
}

impl Colors {
    pub fn new(t: u32, b: u32, l: u32, r: u32) -> Colors {
        Colors {
            top: t,
            bottom: b,
            left: l,
            right: r,
        }
    }
}

#[derive(Clap)]
#[clap(version = "1.0", author = "Abbath")]
struct Opts {
    #[clap(short('n'), long, default_value = "256")]
    steps: u64,
    #[clap(short, long, default_value = "test.png")]
    filename: String,
    #[clap(short('s'), long, default_value = "16", validator(|x| if x.parse::<usize>().unwrap_or(0) > 0 {Ok(())} else {Err("Must be >0")}))]
    tile_size: usize,
    #[clap(short, long, default_value = "ff0000ff", parse(try_from_str = parse_hex))]
    top_color: u32,
    #[clap(short, long, default_value = "0000ffff", parse(try_from_str = parse_hex))]
    bottom_color: u32,
    #[clap(short, long, default_value = "ffff00ff", parse(try_from_str = parse_hex))]
    left_color: u32,
    #[clap(short, long, default_value = "00ff00ff", parse(try_from_str = parse_hex))]
    right_color: u32,
}

fn main() {
    let opts: Opts = Opts::parse();
    let mut x = Diamond::new();
    println!("Generating...");
    x.generate(opts.steps);
    println!("Rendering...");
    x.draw_image(
        &opts.filename,
        opts.tile_size,
        &Colors::new(
            opts.top_color,
            opts.bottom_color,
            opts.left_color,
            opts.right_color,
        ),
    );
}

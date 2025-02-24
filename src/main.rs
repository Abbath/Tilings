use actix_extract_multipart::*;
use actix_web::{App, HttpResponse, HttpServer, get, http::header, post, web};
use clap::Parser;
use image::imageops::{FilterType, resize};
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_hollow_rect_mut};
use imageproc::rect::Rect;
use progressing::{Baring, mapping::Bar as MappingBar};
use rand::{Rng, random};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::Cursor;
use std::num::ParseIntError;
use std::ops::Range;

type Coords = (usize, usize);

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
enum Direction {
    T = 1,
    B = 2,
    L = 3,
    R = 4,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
struct Tile {
    pos: Coords,
    dir: Direction,
}

#[derive(Serialize, Deserialize)]
struct Diamond {
    size: usize,
    capacity: usize,
    origin: Coords,
    data: Vec<usize>,
    tiles: HashMap<usize, Tile>,
    tile_id: usize,
    free_ids: VecDeque<usize>,
    current_square: Coords,
    p: f64,
}

enum ImageAction {
    Save(String),
    Return,
}

#[derive(Clone)]
enum EmbeddableImage {
    FileName(String),
    FileBytes(Vec<u8>),
}

impl Diamond {
    pub fn new(p: f64, size: usize) -> Diamond {
        let n = size / 2 - 1;
        let corner = n * (n + 1) / 2;
        Diamond {
            size: 0,
            capacity: size,
            origin: (size / 2, size / 2),
            data: vec![0; size * size - corner * 4],
            tiles: HashMap::new(),
            tile_id: 1,
            free_ids: VecDeque::new(),
            current_square: (0, 0),
            p,
        }
    }
    fn to_offset(&self, i: usize, j: usize) -> usize {
        let j = j - self.half_span(i, self.capacity);
        let s = self.capacity / 2;
        if i < s {
            i * (1 + i) + j
        } else {
            let s2 = self.capacity - i;
            s * 2 * (1 + s) - s2 * (1 + s2) + j
        }
    }
    fn at_ref(&mut self, m: usize, n: usize) -> &mut usize {
        let l = self.to_offset(m + self.origin.0, n + self.origin.1);
        &mut self.data[l]
    }
    fn at(&self, m: usize, n: usize) -> usize {
        self.data[self.to_offset(m + self.origin.0, n + self.origin.1)]
    }
    fn clear_square(&mut self, i: usize, j: usize) {
        *self.at_ref(i, j) = 0;
        *self.at_ref(i + 1, j) = 0;
        *self.at_ref(i, j + 1) = 0;
        *self.at_ref(i + 1, j + 1) = 0;
    }
    fn half_span(&self, i: usize, size: usize) -> usize {
        let s = size / 2;
        if i < s { s - 1 - i } else { s - size + i }
    }
    fn span(&self, i: usize) -> Range<usize> {
        let s = self.size / 2;
        if i < s {
            Range {
                start: s - 1 - i,
                end: self.size - s + 1 + i,
            }
        } else {
            Range {
                start: s - self.size + i,
                end: 2 * self.size - s - i,
            }
        }
    }
    fn extend(&mut self) {
        self.size += 2;
        self.origin.0 -= 1;
        self.origin.1 -= 1;
        self.tiles.par_iter_mut().for_each(|(_, tile)| {
            tile.pos = (tile.pos.0 + 1, tile.pos.1 + 1);
        });
    }
    fn find_square(&mut self) -> Option<Coords> {
        (self.current_square.0..self.size - 1)
            .find_map(|i| {
                let Range { start: b, end: e } = self.span(i);
                ((if i == self.current_square.0 {
                    self.current_square.1
                } else {
                    b
                })..e - 1)
                    .find_map(|j| {
                        if self.at(i, j) == 0
                            && self.at(i + 1, j) == 0
                            && self.at(i, j + 1) == 0
                            && self.at(i + 1, j + 1) == 0
                        {
                            self.current_square = (i, j);
                            Some((i, j))
                        } else {
                            None
                        }
                    })
            })
            .or_else(|| {
                self.current_square = (0, self.span(0).start + 1);
                None
            })
    }
    fn next_tile_id(&mut self) -> usize {
        self.free_ids.pop_front().unwrap_or({
            let tid = self.tile_id;
            self.tile_id += 1;
            tid
        })
    }
    fn tile_square(&mut self, c: Coords, img: &Option<image::DynamicImage>) {
        let mut rng = rand::thread_rng();
        let predicate: bool = match img {
            Some(im) => {
                let pix = im.get_pixel(c.1 as u32, c.0 as u32).0[0];
                if pix < 128 {
                    true
                } else if (128..=192).contains(&pix) {
                    let dir: u64 = rng.r#gen::<u64>() % 2;
                    dir == 0
                } else {
                    false
                }
            }
            None => {
                let dir: f64 = rng.gen_range(0.0..=1.0);
                dir < self.p
            }
        };
        let tid = self.next_tile_id();
        *self.at_ref(c.0, c.1) = tid;
        if predicate {
            *self.at_ref(c.0, c.1 + 1) = tid;
            self.tiles.insert(
                tid,
                Tile {
                    pos: c,
                    dir: Direction::T,
                },
            );
            let tid = self.next_tile_id();
            *self.at_ref(c.0 + 1, c.1) = tid;
            *self.at_ref(c.0 + 1, c.1 + 1) = tid;
            self.tiles.insert(
                tid,
                Tile {
                    pos: (c.0 + 1, c.1),
                    dir: Direction::B,
                },
            );
        } else {
            *self.at_ref(c.0 + 1, c.1) = tid;
            self.tiles.insert(
                tid,
                Tile {
                    pos: c,
                    dir: Direction::L,
                },
            );
            let tid = self.next_tile_id();
            *self.at_ref(c.0, c.1 + 1) = tid;
            *self.at_ref(c.0 + 1, c.1 + 1) = tid;
            self.tiles.insert(
                tid,
                Tile {
                    pos: (c.0, c.1 + 1),
                    dir: Direction::R,
                },
            );
        }
    }
    fn remove_two_tiles(&mut self, tid1: usize, tid2: usize, i: usize, j: usize) {
        self.tiles.remove(&tid1);
        self.tiles.remove(&tid2);
        self.clear_square(i, j);
        self.free_ids.push_back(tid1);
        self.free_ids.push_back(tid2);
    }
    fn eliminate_stuck_tiles(&mut self) {
        if self.size == 0 {
            return;
        }
        (0..self.size - 1).for_each(|i| {
            let Range { start: b, end: e } = self.span(i);
            (b..e - 1).for_each(|j| {
                if self.at(i, j) > 0 {
                    let tile_id = self.at(i, j);
                    if self.tiles[&tile_id].dir == Direction::B && j > b && self.at(i + 1, j) > 0 {
                        let tile_id_2 = self.at(i + 1, j);
                        if self.tiles[&tile_id_2].dir == Direction::T {
                            self.remove_two_tiles(tile_id, tile_id_2, i, j)
                        }
                    } else if self.tiles[&tile_id].dir == Direction::R && self.at(i, j + 1) > 0 {
                        let tile_id_2 = self.at(i, j + 1);
                        if self.tiles[&tile_id_2].dir == Direction::L {
                            self.remove_two_tiles(tile_id, tile_id_2, i, j)
                        }
                    }
                }
            });
        });
    }
    fn move_tiles(&mut self) {
        let to_move: Vec<(usize, Tile)> =
            self.tiles.iter().map(|(id, tile)| (*id, *tile)).collect();
        self.tiles.par_iter_mut().for_each(|(_, tile)| {
            let c = tile.pos;
            match tile.dir {
                Direction::T => {
                    tile.pos = (c.0 - 1, c.1);
                }
                Direction::B => {
                    tile.pos = (c.0 + 1, c.1);
                }
                Direction::L => {
                    tile.pos = (c.0, c.1 - 1);
                }
                Direction::R => {
                    tile.pos = (c.0, c.1 + 1);
                }
            }
        });
        to_move.iter().for_each(|(id, tile)| {
            let Tile {
                pos: (i, j),
                dir: o,
            } = tile;
            match o {
                Direction::T => {
                    if self.at(*i, *j) == *id {
                        *self.at_ref(*i, *j) = 0;
                    }
                    if self.at(*i, *j + 1) == *id {
                        *self.at_ref(*i, j + 1) = 0;
                    }
                    *self.at_ref(i - 1, *j) = *id;
                    *self.at_ref(i - 1, j + 1) = *id;
                }
                Direction::B => {
                    if self.at(*i, *j) == *id {
                        *self.at_ref(*i, *j) = 0;
                    }
                    if self.at(*i, j + 1) == *id {
                        *self.at_ref(*i, j + 1) = 0;
                    }
                    *self.at_ref(i + 1, *j) = *id;
                    *self.at_ref(i + 1, j + 1) = *id;
                }
                Direction::L => {
                    if self.at(*i, *j) == *id {
                        *self.at_ref(*i, *j) = 0;
                    }
                    if self.at(i + 1, *j) == *id {
                        *self.at_ref(i + 1, *j) = 0;
                    }
                    *self.at_ref(*i, j - 1) = *id;
                    *self.at_ref(i + 1, j - 1) = *id;
                }
                Direction::R => {
                    if self.at(*i, *j) == *id {
                        *self.at_ref(*i, *j) = 0;
                    }
                    if self.at(i + 1, *j) == *id {
                        *self.at_ref(i + 1, *j) = 0;
                    }
                    *self.at_ref(*i, j + 1) = *id;
                    *self.at_ref(i + 1, j + 1) = *id;
                }
            }
        });
    }
    fn tile(&mut self, embed: &Option<EmbeddableImage>) {
        let im = if let Some(ef) = embed {
            let img = match ef {
                EmbeddableImage::FileName(fname) => {
                    image::open(fname).unwrap_or_else(|_| panic!("NO IMAGE {}!", &fname))
                }
                EmbeddableImage::FileBytes(data) => {
                    image::load_from_memory(data).unwrap_or_else(|_| panic!("NO IMAGE IN REQUEST!"))
                }
            };
            let img = img.grayscale().resize_exact(
                self.size as u32,
                self.size as u32,
                FilterType::Nearest,
            );
            Some(img)
        } else {
            None
        };
        while let Some(c) = self.find_square() {
            self.tile_square(c, &im)
        }
    }
    pub fn step(&mut self, embed: &Option<EmbeddableImage>) {
        self.eliminate_stuck_tiles();
        self.extend();
        self.move_tiles();
        self.tile(embed);
    }
    pub fn generate(&mut self, n: usize, embed: Option<EmbeddableImage>) {
        let mut progress_bar = MappingBar::with_range(0, n);
        progress_bar.set_len(32);
        progress_bar.set(0_usize);
        (0..n).for_each(|i| {
            progress_bar.set(i + 1);
            if progress_bar.has_progressed_significantly() {
                print!("\r{}", progress_bar);
            }
            if i == n - 1 {
                self.step(&embed);
            } else {
                self.step(&None);
            }
        });
        println!();
    }
    #[allow(dead_code)]
    pub fn print(&self) {
        (0..self.size).for_each(|i| {
            (0..self.size).for_each(|j| {
                if self.at(i, j) > 0 {
                    print!(
                        " {} ",
                        match self.tiles[&(self.at(i, j))].dir {
                            Direction::T => {
                                "T"
                            }
                            Direction::B => {
                                "B"
                            }
                            Direction::L => {
                                "L"
                            }
                            Direction::R => {
                                "R"
                            }
                        }
                    );
                } else {
                    print!("   ");
                }
            });
            println!();
        });
        println!();
    }
    #[allow(dead_code)]
    pub fn print_debug(&self) {
        (0..self.size).for_each(|i| {
            (0..self.size).for_each(|j| print!("{:4} ", self.at(i, j)));
            println!();
        });
        println!();
    }
    pub fn draw_image(&self, ts: usize, colors: &Colors, action: ImageAction) -> Option<Vec<u8>> {
        let tile_size = if ts > 16 { ts / 2 } else { ts };
        let mut im = RgbaImage::new(
            (self.size * tile_size) as u32,
            (self.size * tile_size) as u32,
        );
        draw_filled_rect_mut(
            &mut im,
            Rect::at(0, 0).of_size(
                (self.size * tile_size) as u32,
                (self.size * tile_size) as u32,
            ),
            Rgba([128, 128, 128, 255]),
        );
        let mut progress_bar = MappingBar::with_range(0, self.tiles.len());
        self.tiles.values().enumerate().for_each(|(counter, tile)| {
            let (i, j) = tile.pos;
            let (src, w, h) = match tile.dir {
                Direction::T => (colors.top, 2, 1),
                Direction::B => (colors.bottom, 2, 1),
                Direction::L => (colors.left, 1, 2),
                Direction::R => (colors.right, 1, 2),
            };
            draw_hollow_rect_mut(
                &mut im,
                Rect::at((j * tile_size) as i32, (i * tile_size) as i32)
                    .of_size((w * tile_size) as u32, (h * tile_size) as u32),
                colors.grid,
            );
            draw_filled_rect_mut(
                &mut im,
                Rect::at((j * tile_size) as i32 + 1, (i * tile_size) as i32 + 1)
                    .of_size((w * tile_size) as u32 - 2, (h * tile_size) as u32 - 2),
                src,
            );
            progress_bar.set(counter + 1);
            if progress_bar.has_progressed_significantly() {
                print!("\r{}", progress_bar);
            }
        });
        println!();
        if ts > 16 {
            im = resize(&im, im.width() * 2, im.height() * 2, FilterType::Nearest);
        }
        match action {
            ImageAction::Save(s) => {
                im.save(s).expect("FAILED TO SAVE AN IMAGE!");
                None
            }
            ImageAction::Return => {
                let mut bytes: Vec<u8> = Vec::new();
                match DynamicImage::ImageRgba8(im)
                    .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
                {
                    Ok(()) => Some(bytes),
                    Err(_) => None,
                }
            }
        }
    }
}

fn parse_hex(input: &str) -> Result<u32, ParseIntError> {
    u32::from_str_radix(input, 16)
}

struct Colors {
    top: Rgba<u8>,
    bottom: Rgba<u8>,
    left: Rgba<u8>,
    right: Rgba<u8>,
    grid: Rgba<u8>,
}

impl Colors {
    pub fn new(t: u32, b: u32, l: u32, r: u32, g: u32) -> Colors {
        Colors {
            top: Colors::int_to_color(t),
            bottom: Colors::int_to_color(b),
            left: Colors::int_to_color(l),
            right: Colors::int_to_color(r),
            grid: Colors::int_to_color(g),
        }
    }
    pub fn default() -> Colors {
        Colors {
            top: Rgba([255, 0, 0, 255]),
            bottom: Rgba([0, 0, 255, 255]),
            left: Rgba([255, 255, 0, 255]),
            right: Rgba([0, 255, 0, 255]),
            grid: Rgba([0, 0, 0, 255]),
        }
    }
    fn int_to_color(c: u32) -> Rgba<u8> {
        Rgba([
            (c >> 24 & 0xff) as u8,
            (c >> 16 & 0xff) as u8,
            (c >> 8 & 0xff) as u8,
            (c & 0xff) as u8,
        ])
    }
}

#[derive(Parser)]
#[command(version = "1.0", author = "Abbath")]
struct Opts {
    #[arg(short('n'), long, default_value = "256")]
    steps: usize,
    #[arg(short, long, default_value = "test.png")]
    filename: String,
    #[arg(short('s'), long, default_value = "8")]
    tile_size: usize,
    #[arg(short, long, default_value = "ff0000ff", value_parser = parse_hex)]
    top_color: u32,
    #[arg(short, long, default_value = "0000ffff", value_parser = parse_hex)]
    bottom_color: u32,
    #[arg(short, long, default_value = "ffff00ff", value_parser = parse_hex)]
    left_color: u32,
    #[arg(short, long, default_value = "00ff00ff", value_parser = parse_hex)]
    right_color: u32,
    #[arg(short, long, default_value = "000000ff", value_parser = parse_hex)]
    grid_color: u32,
    #[arg(short('c'), long)]
    random_colors: bool,
    #[arg(short('a'), long)]
    save_all_steps: bool,
    #[arg(short('w'), long)]
    web: bool,
    #[arg(short('i'), long)]
    input: Option<String>,
    #[arg(short('o'), long)]
    output: Option<String>,
    #[arg(short('p'), long, default_value = "0.5")]
    probability: f64,
    #[arg(short('e'), long)]
    embed: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Params {
    fname: Option<File>,
    steps: usize,
    size: usize,
    p: usize,
}

#[post("/")]
async fn index_post(params: Multipart<Params>) -> HttpResponse {
    let mut x = Diamond::new(params.p as f64 / 100.0f64, params.steps * 2);
    x.generate(
        params.steps,
        params
            .fname
            .as_ref()
            .map(|f| EmbeddableImage::FileBytes(f.data().to_vec())),
    );
    let f = x
        .draw_image(params.size, &Colors::default(), ImageAction::Return)
        .expect("IMAGE IS NOT HERE!");
    HttpResponse::Ok()
        .append_header(header::ContentDisposition::attachment("image.png"))
        .content_type("image/png")
        .body(f)
}

#[get("/")]
async fn index_get() -> HttpResponse {
    let html = r#"<!DOCTYPE html>
    <html>
    <body>

    <h2>Tilings</h2>

    <form method="post" action="/" enctype="multipart/form-data">
      <label for="fname">Image:</label><br>
      <input type="file" id="fname" name="fname" accept="image/png, image/jpeg"><br>
      <label for="lname">Steps:</label><br>
      <input type="number" id="steps" name="steps" value="256"><br>
      <label for="lname">Size:</label><br>
      <input type="number" id="size" name="size" value="4"><br>
      <label for="lname">Probability (%):</label><br>
      <input type="number" id="p" name="p" value="50" min="0" max="100"><br><br>
      <input type="submit" value="Submit">
    </form>

    </body>
    </html>"#;
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[get("/{steps}/{size}")]
async fn index(path: web::Path<(usize, usize)>) -> HttpResponse {
    let (steps, size) = path.into_inner();
    let mut x = Diamond::new(0.5, steps * 2);
    x.generate(steps, None);
    let f = x
        .draw_image(size, &Colors::default(), ImageAction::Return)
        .expect("IMAGE IS NOT HERE!");
    HttpResponse::Ok().content_type("image/png").body(f)
}

#[actix_web::main]
async fn amain() -> std::io::Result<()> {
    let port = 3000;
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(index_get)
            .service(index_post)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

fn random_color() -> u32 {
    let r: u8 = random();
    let g: u8 = random();
    let b: u8 = random();
    let a: u8 = 255;
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | a as u32
}

fn main() {
    let opts: Opts = Opts::parse();
    if opts.web {
        amain().unwrap_or_else(|s| panic!("SOMETHING WENT WRONG {}!", s));
        return;
    }
    let mut x = match opts.input {
        Some(input) => {
            let content = std::fs::read_to_string(&input)
                .unwrap_or_else(|err| panic!("COULD NOT LOAD FILE {} WITH ERROR {}!", input, err));
            serde_json::from_str(&content)
                .unwrap_or_else(|err| panic!("COULD NOT PARSE FILE {} WITH ERROR {}!", input, err))
        }
        None => Diamond::new(opts.probability, opts.steps * 2),
    };
    let colors: Colors = if opts.random_colors {
        Colors::new(
            random_color(),
            random_color(),
            random_color(),
            random_color(),
            random_color(),
        )
    } else {
        Colors::new(
            opts.top_color,
            opts.bottom_color,
            opts.left_color,
            opts.right_color,
            opts.grid_color,
        )
    };
    if opts.save_all_steps {
        for i in 0..opts.steps {
            x.step(&None);
            x.draw_image(
                opts.tile_size,
                &colors,
                ImageAction::Save(format!("{}_{}.png", opts.filename, i + 1)),
            );
        }
    } else {
        println!("Generating...");
        x.generate(opts.steps, opts.embed.map(EmbeddableImage::FileName));
        println!("Rendering...");
        x.draw_image(opts.tile_size, &colors, ImageAction::Save(opts.filename));
        println!("Done.");
    }
    if let Some(output) = opts.output {
        let serialized = serde_json::to_string(&x).unwrap();
        if output == "--" {
            println!("{}", serialized);
        } else {
            std::fs::write(&output, serialized)
                .unwrap_or_else(|err| panic!("COULD NOT SAVE FILE {} {}!", output, err));
        }
    }
}

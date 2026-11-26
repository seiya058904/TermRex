use crate::game::*;
const PLANE: usize = 155 * 68;
static ATLAS: &[u8; PLANE * 2] = include_bytes!("../assets/chromium.bits");
const DOTS: [u8; 8] = [1, 8, 2, 16, 4, 32, 64, 128];

pub struct Raster {
    pub cols: usize,
    pub rows: usize,
    pub cells: Vec<u8>,
    pixels: Vec<u8>,
    xmap: [usize; 600],
    ymap: [usize; 150],
}
impl Raster {
    pub fn new(cols: usize) -> Self {
        let cols = cols.clamp(1, 300);
        let rows = (cols + 4) / 8;
        let mut r = Self {
            cols,
            rows: rows.max(1),
            cells: vec![0; cols * rows.max(1)],
            pixels: vec![0; 600 * 150],
            xmap: [0; 600],
            ymap: [0; 150],
        };
        for (i, x) in r.xmap.iter_mut().enumerate() {
            *x = i * cols * 2 / 600;
        }
        for (i, y) in r.ymap.iter_mut().enumerate() {
            *y = i * r.rows * 4 / 150;
        }
        r
    }
    fn sprite(&mut self, sx: i32, sy: i32, w: i32, h: i32, x: i32, y: i32) {
        let left = 0.max(-x);
        let right = w.min(600 - x);
        for dy in 0.max(-y)..h.min(150 - y) {
            for dx in left..right {
                let ax = (sx + dx) as usize;
                let index = (sy + dy) as usize * 155 + ax / 8;
                let bit = 1 << (ax % 8);
                if ATLAS[index] & bit != 0 {
                    self.pixels[((y + dy) * 600 + x + dx) as usize] =
                        u8::from(ATLAS[index + PLANE] & bit != 0);
                }
            }
        }
    }
    pub fn draw(&mut self, g: &Game) {
        self.pixels.fill(0);
        for &(x, y) in &g.clouds {
            self.sprite(86, 2, 46, 14, x as i32, y);
        }
        if g.night > 0. {
            let phase = ((g.score().max(700) / 700 - 1) % 7) as usize;
            let drift = g.night_ticks as i32;
            let moon_x = 600 - (52 + drift / 4) % 640;
            self.sprite(
                484 + [140, 120, 100, 60, 40, 20, 0][phase],
                2,
                if phase == 3 { 40 } else { 20 },
                40,
                moon_x,
                26,
            );
            for (i, (x, y)) in [(120, 32), (365, 20), (540, 51)].into_iter().enumerate() {
                self.sprite(
                    645,
                    2 + 9 * i as i32,
                    9,
                    9,
                    (x - drift * 3 / 10).rem_euclid(610) - 9,
                    y,
                );
            }
        }
        for i in 0..2 {
            self.sprite(
                2 + g.terrain[i] * 600,
                54,
                600,
                12,
                i as i32 * 600 - g.ground_x,
                127,
            );
        }
        for ob in &g.obstacles {
            let (sx, w, h) = match ob.kind {
                Species::Small => (228, 17, 35),
                Species::Large => (332, 25, 50),
                Species::Bird => (134 + ((g.frame / 10) % 2) as i32 * 46, 46, 40),
            };
            self.sprite(
                sx + w * ob.count * (ob.count - 1) / 2,
                2,
                w * ob.count,
                h,
                ob.x,
                ob.y,
            );
        }
        let (sx, w) = match g.state {
            State::Ready => (if g.frame % 186 < 10 { 44 } else { 0 }, 44),
            State::Crashed => (220, 44),
            _ if g.airborne() => (0, 44),
            _ if g.duck() => (264 + ((g.frame * 2 / 15) % 2) as i32 * 59, 59),
            _ => (88 + ((g.frame / 5) % 2) as i32 * 44, 44),
        };
        self.sprite(848 + sx, 2, w, 47, REX_X, g.y);
        self.cells.fill(0);
        for (y, line) in self.pixels.as_chunks::<600>().0.iter().enumerate() {
            let py = self.ymap[y];
            let row = py / 4 * self.cols;
            for (x, &ink) in line.iter().enumerate() {
                if ink != 0 {
                    let px = self.xmap[x];
                    self.cells[row + px / 2] |= DOTS[(py % 4) * 2 + px % 2];
                }
            }
        }
    }
}
pub fn colors(night: f64) -> ([u8; 3], [u8; 3]) {
    let day = [[55., 59., 54.], [244., 244., 233.]];
    let dark = [[224., 227., 211.], [31., 34., 32.]];
    let blend = |row: usize| {
        std::array::from_fn(|i| (day[row][i] + (dark[row][i] - day[row][i]) * night).round() as u8)
    };
    (blend(0), blend(1))
}
pub fn hud(g: &Game) -> String {
    let value = if g.flash > 0 && ((72 - g.flash) / 12).is_multiple_of(2) {
        "     ".to_string()
    } else {
        format!("{:05}", g.score())
    };
    format!("HI {:05}  {}", g.best, value)
}

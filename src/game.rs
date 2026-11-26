// Geometry and movement adapted from Chromium 90. See assets/LICENSE.chromium.
pub const GROUND: i32 = 93;
pub const REX_X: i32 = 50;
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Ready,
    Running,
    Paused,
    Crashed,
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Key {
    Jump,
    Down,
    Pause,
    Restart,
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Species {
    Small,
    Large,
    Bird,
}
use Species::*;
#[derive(Clone, Copy, Debug)]
pub struct Obstacle {
    pub kind: Species,
    pub x: i32,
    pub y: i32,
    pub count: i32,
    pub offset: f64,
    gap: i32,
}
type Box4 = (i32, i32, i32, i32);
const RUN: &[Box4] = &[
    (22, 0, 17, 16),
    (1, 18, 30, 9),
    (10, 35, 14, 8),
    (1, 24, 29, 5),
    (5, 30, 21, 4),
    (9, 34, 15, 4),
];
const DUCK: &[Box4] = &[(1, 18, 55, 25)];
impl Obstacle {
    pub fn width(&self) -> i32 {
        self.count
            * match self.kind {
                Small => 17,
                Large => 25,
                Bird => 46,
            }
    }
    pub fn hits(&self, y: i32, duck: bool) -> bool {
        // Chromium trims a one-pixel border on both broad-phase rectangles.
        let height = match self.kind {
            Small => 35,
            Large => 50,
            Bird => 40,
        };
        if self.x >= REX_X + 42
            || self.x + self.width() - 2 <= REX_X
            || self.y >= y + 45
            || self.y + height - 2 <= y
        {
            return false;
        }
        let parts: &[Box4] = match self.kind {
            Small => &[(0, 7, 5, 27), (4, 0, 6, 34), (10, 4, 7, 14)],
            Large => &[(0, 12, 7, 38), (8, 0, 7, 49), (13, 10, 10, 38)],
            Bird => &[
                (15, 15, 16, 5),
                (18, 21, 24, 6),
                (2, 14, 4, 3),
                (6, 10, 4, 7),
                (10, 8, 6, 9),
            ],
        };
        for (index, &(mut bx, by, mut bw, bh)) in parts.iter().enumerate() {
            if self.count > 1 {
                if index == 1 {
                    bw = self.width() - parts[0].2 - parts[2].2;
                }
                if index == 2 {
                    bx = self.width() - bw;
                }
            }
            let (bx, by) = (bx + self.x, by + self.y);
            for &(rx, ry, rw, rh) in if duck { DUCK } else { RUN } {
                let (rx, ry) = (REX_X + rx, y + ry);
                if rx < bx + bw && rx + rw > bx && ry < by + bh && ry + rh > by {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Clone)]
pub struct Game {
    pub state: State,
    pub frame: u64,
    pub y: i32,
    pub speed: f64,
    pub distance: f64,
    pub best: u32,
    pub obstacles: Vec<Obstacle>,
    pub clouds: [(f64, i32); 3],
    pub ground_x: i32,
    pub terrain: [i32; 2],
    pub night: f64,
    pub night_ticks: u32,
    pub flash: u32,
    pub chime: bool,
    velocity: f64,
    airborne: bool,
    jump_held: bool,
    pilot_release: u64,
    down: bool,
    diving: bool,
    rng: u32,
    history: [Option<Species>; 2],
}
impl Game {
    pub fn new(seed: u32, best: u32) -> Self {
        Self {
            state: State::Ready,
            frame: 0,
            y: GROUND,
            speed: 6.,
            distance: 0.,
            best,
            obstacles: Vec::new(),
            clouds: [(97., 36), (332., 54), (545., 26)],
            ground_x: 0,
            terrain: [0, 1],
            night: 0.,
            night_ticks: 0,
            flash: 0,
            chime: false,
            velocity: 0.,
            airborne: false,
            jump_held: false,
            pilot_release: 0,
            down: false,
            diving: false,
            rng: seed.max(1),
            history: [None, None],
        }
    }
    fn rand(&mut self, n: u32) -> u32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x % n
    }
    pub fn score(&self) -> u32 {
        (self.distance.ceil() * 0.025).round() as u32
    }
    pub fn duck(&self) -> bool {
        self.down && !self.airborne
    }
    pub fn airborne(&self) -> bool {
        self.airborne
    }
    pub fn clear_keys(&mut self) {
        self.jump_held = false;
        self.down = false;
    }
    pub fn key(&mut self, k: Key, pressed: bool) {
        if k == Key::Restart && pressed {
            *self = Self::new(self.rng, self.best.max(self.score()));
            self.state = State::Running;
            return;
        }
        if k == Key::Pause && pressed {
            self.state = match self.state {
                State::Running => State::Paused,
                State::Paused => State::Running,
                s => s,
            };
            self.clear_keys();
            return;
        }
        if self.state == State::Paused || self.state == State::Crashed {
            return;
        }
        match k {
            Key::Jump => {
                if pressed && !self.jump_held && !self.airborne && !self.down {
                    self.state = State::Running;
                    self.airborne = true;
                    self.diving = false;
                    self.velocity = -10. - self.speed / 10.;
                }
                if !pressed && self.airborne && self.y < GROUND - 30 {
                    self.velocity = self.velocity.max(-5.);
                }
                self.jump_held = pressed;
            }
            Key::Down => {
                if pressed && !self.down && self.airborne {
                    self.diving = true;
                    self.velocity = 1.;
                }
                self.down = pressed;
            }
            _ => (),
        }
    }
    fn spawn(&mut self) {
        let mut choices = vec![Small, Large];
        if self.speed >= 8.5 {
            choices.push(Bird);
        }
        if self.history[0] == self.history[1] {
            choices.retain(|k| Some(*k) != self.history[0]);
        }
        let kind = choices[self.rand(choices.len() as u32) as usize];
        self.history = [self.history[1], Some(kind)];
        let count = if kind == Small || (kind == Large && self.speed >= 7.) {
            1 + self.rand(3) as i32
        } else {
            1
        };
        let y = match kind {
            Small => 105,
            Large => 90,
            Bird => [100, 75, 50][self.rand(3) as usize],
        };
        let offset = if kind == Bird {
            if self.rand(2) == 0 {
                -0.8
            } else {
                0.8
            }
        } else {
            0.
        };
        let mut ob = Obstacle {
            kind,
            x: 600,
            y,
            count,
            offset,
            gap: 0,
        };
        let gap =
            (ob.width() as f64 * self.speed + if kind == Bird { 90. } else { 72. }).round() as i32;
        ob.gap = gap + self.rand(1 + gap as u32 / 2) as i32;
        self.obstacles.push(ob);
    }
    pub fn tick(&mut self) {
        self.chime = false;
        if self.state == State::Ready {
            self.frame += 1;
            return;
        }
        if self.state != State::Running {
            return;
        }
        self.frame += 1;
        if self.airborne {
            self.y += (self.velocity * (if self.diving { 3. } else { 1. }) + 0.5).floor() as i32;
            self.velocity += 0.6;
            if self.y < 30 || (!self.jump_held && self.y < GROUND - 30) {
                self.velocity = self.velocity.max(-5.);
            }
            if self.y > GROUND {
                self.y = GROUND;
                self.airborne = false;
                self.diving = false;
            }
        }
        self.ground_x += self.speed.floor() as i32;
        if self.ground_x >= 600 {
            self.ground_x -= 600;
            self.terrain = [self.terrain[1], self.rand(2) as i32];
        }
        for i in 0..self.clouds.len() {
            self.clouds[i].0 -= self.speed * 0.08;
            if self.clouds[i].0 < -46. {
                self.clouds[i] = (600. + self.rand(70) as f64, 25 + self.rand(42) as i32);
            }
        }
        for ob in &mut self.obstacles {
            ob.x -= (self.speed + ob.offset).floor() as i32;
        }
        self.obstacles.retain(|o| o.x + o.width() > 0);
        if self.distance >= 1080.
            && self
                .obstacles
                .last()
                .is_none_or(|o| o.x + o.width() + o.gap <= 600)
        {
            self.spawn();
        }
        if self.obstacles.iter().any(|o| o.hits(self.y, self.duck())) {
            self.state = State::Crashed;
            self.best = self.best.max(self.score());
            return;
        }
        let old = self.score();
        self.distance += self.speed;
        self.speed = (self.speed + 0.001).min(13.);
        let score = self.score();
        self.flash = self.flash.saturating_sub(1);
        if score / 100 > old / 100 {
            self.flash = 6 * 12;
            self.chime = true;
        }
        let dark = score >= 700 && score % 700 < 200;
        self.night = (self.night + if dark { 0.035 } else { -0.035 }).clamp(0., 1.);
        if score / 700 > old / 700 {
            self.night_ticks = 0;
        }
        if self.night > 0. {
            self.night_ticks += 1;
        }
    }
    // Demo drives the same keys and collision checks as the native game.
    pub fn pilot(&mut self) {
        if self.state != State::Running {
            return;
        }
        if self.airborne {
            self.key(Key::Jump, self.frame < self.pilot_release);
            return;
        }
        self.key(Key::Jump, false);
        let Some(ob) = self
            .obstacles
            .iter()
            .find(|o| o.x + o.width() > REX_X + 2)
            .copied()
        else {
            self.key(Key::Down, false);
            return;
        };
        if ob.kind == Bird && ob.y <= 75 {
            self.key(Key::Down, ob.x < REX_X + 110);
            return;
        }
        self.key(Key::Down, false);
        let step = (self.speed + ob.offset).floor() as i32;
        if ob.x > REX_X + step * 22 {
            return;
        }
        for hold in [4, 7, 10, 16, 24] {
            let mut trial = self.clone();
            trial.key(Key::Jump, true);
            let mut frames = 0;
            while trial.airborne && trial.state == State::Running && frames < 60 {
                if frames == hold {
                    trial.key(Key::Jump, false);
                }
                trial.tick();
                frames += 1;
            }
            if trial.state == State::Running && ob.x + ob.width() - frames * step <= REX_X {
                self.pilot_release = self.frame + hold as u64;
                self.key(Key::Jump, true);
                break;
            }
        }
    }
}

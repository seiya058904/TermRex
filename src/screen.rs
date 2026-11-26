use std::{
    fmt::Write as _,
    io::{self, Write},
};
use termrex_core::{game::*, render::*};
pub struct Screen {
    previous: Vec<String>,
    size: (usize, usize),
    tone: (u8, u8),
    raster: Raster,
    restore: &'static str,
}
impl Screen {
    pub fn open(enter: &str, restore: &'static str) -> io::Result<Self> {
        let s = Self {
            previous: vec![],
            size: (0, 0),
            tone: (0, 0),
            raster: Raster::new(1),
            restore,
        };
        send(&format!("\x1b[?1049h\x1b[?25l{enter}"))?;
        Ok(s)
    }
    pub fn fits((w, h): (usize, usize)) -> bool {
        w >= 81 && h >= 14
    }
    pub fn paint(
        &mut self,
        g: &Game,
        size: (usize, usize),
        mute: bool,
        demo: bool,
    ) -> io::Result<()> {
        let (w, h) = size;
        let resized = size != self.size;
        let mut out = String::new();
        if resized {
            // Erase only previously occupied rows when the viewport moves.
            let (_, old_h) = self.size;
            let top = old_h.saturating_sub(self.previous.len()) / 2;
            for row in top..(top + self.previous.len()).min(h) {
                let _ = write!(out, "\x1b[{};1H\x1b[0m\x1b[2K", row + 1);
            }
            self.previous.clear();
            self.size = size;
            self.raster = Raster::new(
                w.saturating_sub(1)
                    .min(h.saturating_sub(3) * 8)
                    .clamp(1, 300),
            );
        }
        let fits = Self::fits(size);
        let cols = if fits {
            self.raster.cols
        } else {
            w.saturating_sub(1)
        };
        let mut lines = Vec::new();
        if fits {
            self.raster.draw(g);
            lines.push(format!("{:>width$}", hud(g), width = cols));
            for row in self.raster.cells.chunks_exact(cols) {
                lines.push(
                    row.iter()
                        .map(|&v| {
                            if v == 0 {
                                ' '
                            } else {
                                char::from_u32(0x2800 + u32::from(v)).unwrap()
                            }
                        })
                        .collect(),
                );
            }
            let status = match g.state {
                State::Ready => "SPACE to run",
                State::Paused => "Paused  /  P to resume",
                State::Crashed => "Game over  /  R to restart",
                State::Running if demo => "Autopilot",
                State::Running => "SPACE jump   S duck",
            };
            lines.push(format!(
                "{:<width$}",
                format!(
                    "  {status}    P pause   Q quit   M sound {}",
                    if mute { "off" } else { "on" }
                ),
                width = cols
            ));
        } else {
            lines.push(
                "Resize to 81 x 14 or larger   Q quit"
                    .chars()
                    .take(cols)
                    .collect(),
            );
        }
        let (fg, bg) = colors(g.night);
        let tone = (fg[0], bg[0]);
        let changed = resized || tone != self.tone;
        let (top, left) = (
            h.saturating_sub(lines.len()) / 2,
            w.saturating_sub(cols) / 2,
        );
        for (i, line) in lines.iter().enumerate() {
            if changed || self.previous.get(i) != Some(line) {
                if !out.contains("38;2;") {
                    let _ = write!(
                        out,
                        "\x1b[38;2;{};{};{};48;2;{};{};{}m",
                        fg[0], fg[1], fg[2], bg[0], bg[1], bg[2]
                    );
                }
                let _ = write!(out, "\x1b[{};{}H{}", top + i + 1, left + 1, line);
            }
        }
        self.previous = lines;
        self.tone = tone;
        if !out.is_empty() {
            send(&out)?
        }
        Ok(())
    }
}
impl Drop for Screen {
    fn drop(&mut self) {
        let _ = send(&format!("{}\x1b[0m\x1b[?25h\x1b[?1049l", self.restore));
    }
}
pub fn send(s: &str) -> io::Result<()> {
    let mut out = io::stdout().lock();
    out.write_all(s.as_bytes())?;
    out.flush()
}

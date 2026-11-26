mod platform;
mod screen;
use platform::{Button, Event, Terminal};
use screen::Screen;
use std::{
    env, fs,
    io::{self, IsTerminal, Write},
    path::PathBuf,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use termrex_core::game::{Game, Key, State};
fn score_path() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else {
        env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|v| PathBuf::from(v).join(".local/state")))
    };
    base.map(|p| p.join("termrex").join("best"))
}
fn read_best() -> u32 {
    score_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}
fn save_best(value: u32) -> io::Result<()> {
    let Some(p) = score_path() else { return Ok(()) };
    fs::create_dir_all(p.parent().unwrap())?;
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(p.with_extension("lock"))?;
    lock.lock()?;
    if value > read_best() {
        let tmp = p.with_extension("tmp");
        let mut file = fs::File::create(&tmp)?;
        writeln!(file, "{value}")?;
        file.sync_all()?;
        drop(file);
        fs::rename(tmp, p)?;
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("termrex: {e}");
        std::process::exit(1);
    }
}
fn run() -> io::Result<()> {
    let mut demo = false;
    let mut mute = false;
    let mut measure = false;
    for a in env::args().skip(1) {
        match a.as_str() {
            "--demo" => demo = true,
            "--mute" => mute = true,
            "--benchmark" => {
                demo = true;
                measure = true;
                mute = true
            }
            "--license" => {
                println!(
                    "{}\n{}",
                    include_str!("../LICENSE"),
                    include_str!("../assets/LICENSE.chromium")
                );
                return Ok(());
            }
            "--help" | "-h" => {
                println!("TermRex 0.1.0\ntermrex [--mute] [--demo | --benchmark]\nSpace/Up/W jump, Down/S duck or dive, P pause, R restart, M mute, Q/Ctrl+C quit.\n--benchmark drives a real terminal run to 1001 points and reports frame timing.\n--license prints the Chromium attribution.");
                return Ok(());
            }
            _ => return Err(io::Error::other(format!("unknown option {a}"))),
        }
    }
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::other("open an interactive terminal to play"));
    }
    let mut t = Terminal::open()?;
    let (enter, leave) = t.protocol();
    let mut screen = Screen::open(enter, leave)?;
    let seed = if demo {
        4577
    } else {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos()
    };
    let mut g = Game::new(seed, if demo { 0 } else { read_best() });
    if demo {
        g.key(Key::Restart, true)
    }
    let step = Duration::from_secs_f64(1. / 60.);
    let start = Instant::now();
    let mut deadline = start;
    let mut times = Vec::new();
    let mut previous = start;
    let mut expiry: [Option<Instant>; 6] = [None; 6];
    let mut previous_size = t.size()?;
    let mut frames = 0u64;
    let mut dropped = 0;
    'running: loop {
        let now = Instant::now();
        let size = t.size()?;
        let fits = Screen::fits(size);
        if !fits && Screen::fits(previous_size) {
            g.clear_keys();
            expiry.fill(None);
        }
        previous_size = size;
        let mut events = t.poll()?;
        for b in [
            Button::Jump,
            Button::Down,
            Button::Pause,
            Button::Restart,
            Button::Mute,
        ] {
            let slot = &mut expiry[b as usize];
            if slot.is_some_and(|end| now >= end) {
                *slot = None;
                events.insert(0, Event::Key(b, false));
            }
        }
        for event in events {
            let (b, pressed) = match event {
                Event::Blur => {
                    g.clear_keys();
                    expiry.fill(None);
                    continue;
                }
                Event::Key(k, p) => {
                    expiry[k as usize] = None;
                    (k, p)
                }
                #[cfg(unix)]
                Event::Pulse(k, timeout) => {
                    let slot = &mut expiry[k as usize];
                    let held = slot.is_some();
                    *slot = Some(now + Duration::from_millis(timeout));
                    if held {
                        continue;
                    }
                    (k, true)
                }
            };
            match b {
                Button::Quit if pressed => break 'running,
                Button::Mute if pressed => mute = !mute,
                Button::Quit | Button::Mute => (),
                _ if fits => g.key(
                    match b {
                        Button::Jump => Key::Jump,
                        Button::Down => Key::Down,
                        Button::Pause => Key::Pause,
                        _ => Key::Restart,
                    },
                    pressed,
                ),
                _ => (),
            }
        }
        if fits {
            if demo {
                if g.state == State::Crashed {
                    if measure {
                        return Err(io::Error::other("benchmark pilot crashed"));
                    }
                    g.key(Key::Restart, true);
                }
                g.pilot();
            }
            let before = g.state;
            g.tick();
            if before != State::Crashed && g.state == State::Crashed && !demo {
                save_best(g.best)?;
            }
            if g.chime && !mute {
                screen::send("\x07")?
            }
        }
        screen.paint(&g, size, mute, demo)?;
        if measure && fits {
            if frames > 0 {
                times.push(now.duration_since(previous).as_secs_f64() * 1000.)
            }
            frames += 1;
        }
        previous = now;
        if measure && g.score() > 1000 {
            break;
        }
        deadline += step;
        let now = Instant::now();
        if deadline < now {
            let lag = now - deadline;
            if lag >= step {
                dropped += lag.as_nanos() / step.as_nanos();
                deadline = now;
            }
        }
        let wake = deadline
            .checked_sub(Duration::from_micros(500))
            .unwrap_or(deadline);
        thread::sleep(wake.saturating_duration_since(Instant::now()));
        while Instant::now() < deadline {
            std::hint::spin_loop();
        }
    }
    drop(screen);
    drop(t);
    if !demo {
        save_best(g.score().max(g.best))?;
    }
    if measure && !times.is_empty() {
        times.sort_by(f64::total_cmp);
        println!(
            "score={} frames={} fps={:.2} p99_ms={:.2} max_ms={:.2} missed_intervals={dropped}",
            g.score(),
            frames,
            frames as f64 / start.elapsed().as_secs_f64(),
            times[times.len() * 99 / 100],
            times[times.len() - 1]
        );
    }
    Ok(())
}

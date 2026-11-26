use super::*;
use std::io;
type Handle = *mut std::ffi::c_void;
#[repr(C)]
#[derive(Default, Copy, Clone)]
struct Record {
    kind: u16,
    pad: u16,
    data: [u32; 4],
}
#[repr(C)]
#[derive(Default)]
struct Info {
    size: [i16; 2],
    cursor: [i16; 2],
    attr: u16,
    window: [i16; 4],
    max: [i16; 2],
}
#[link(name = "kernel32")]
extern "system" {
    fn GetStdHandle(which: u32) -> Handle;
    fn GetConsoleMode(h: Handle, v: *mut u32) -> i32;
    fn SetConsoleMode(h: Handle, v: u32) -> i32;
    fn GetConsoleScreenBufferInfo(h: Handle, i: *mut Info) -> i32;
    fn GetNumberOfConsoleInputEvents(h: Handle, n: *mut u32) -> i32;
    fn ReadConsoleInputW(h: Handle, buf: *mut Record, n: u32, got: *mut u32) -> i32;
    fn GetConsoleOutputCP() -> u32;
    fn SetConsoleOutputCP(cp: u32) -> i32;
}
fn check(ok: i32) -> io::Result<()> {
    if ok == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
pub struct Terminal {
    input: Handle,
    out: Handle,
    im: u32,
    om: u32,
    cp: u32,
    held: [bool; 256],
}
impl Terminal {
    pub fn open() -> io::Result<Self> {
        unsafe {
            let mut t = Self {
                input: GetStdHandle(-10i32 as u32),
                out: GetStdHandle(-11i32 as u32),
                im: 0,
                om: 0,
                cp: GetConsoleOutputCP(),
                held: [false; 256],
            };
            // Read both modes before constructing a guard that owns any restoration.
            let mut im = 0;
            let mut om = 0;
            let modes = check(GetConsoleMode(t.input, &mut im))
                .and_then(|_| check(GetConsoleMode(t.out, &mut om)));
            if let Err(err) = modes {
                std::mem::forget(t);
                return Err(err);
            }
            t.im = im;
            t.om = om;
            check(SetConsoleMode(t.input, (im | 0x88) & !(0x247)))?;
            check(SetConsoleMode(t.out, om | 5))?;
            check(SetConsoleOutputCP(65001))?;
            Ok(t)
        }
    }
    pub fn size(&self) -> io::Result<(usize, usize)> {
        unsafe {
            let mut i = Info::default();
            check(GetConsoleScreenBufferInfo(self.out, &mut i))?;
            Ok((
                (i.window[2] - i.window[0] + 1) as usize,
                (i.window[3] - i.window[1] + 1) as usize,
            ))
        }
    }
    pub fn poll(&mut self) -> io::Result<Vec<Event>> {
        unsafe {
            let mut n = 0;
            check(GetNumberOfConsoleInputEvents(self.input, &mut n))?;
            let mut events = Vec::new();
            if n == 0 {
                return Ok(events);
            }
            let mut records = [Record::default(); 64];
            let mut got = 0;
            check(ReadConsoleInputW(
                self.input,
                records.as_mut_ptr(),
                n.min(64),
                &mut got,
            ))?;
            for r in &records[..got as usize] {
                if r.kind == 16 && r.data[0] == 0 {
                    self.held.fill(false);
                    events.push(Event::Blur);
                }
                if r.kind != 1 {
                    continue;
                }
                let vk = (r.data[1] >> 16) as usize;
                if vk >= 256 {
                    continue;
                }
                let down = r.data[0] != 0;
                if vk == 67 && r.data[3] & 12 != 0 && down {
                    events.push(Event::Key(Button::Quit, true));
                    continue;
                }
                if self.held[vk] == down {
                    continue;
                }
                let map = |v: usize| {
                    button(match v {
                        38 => 57352,
                        40 => 57353,
                        _ => v as u32,
                    })
                };
                let Some(b) = map(vk) else { continue };
                self.held[vk] = down;
                if !self
                    .held
                    .iter()
                    .enumerate()
                    .any(|(v, &h)| h && v != vk && map(v) == Some(b))
                {
                    events.push(Event::Key(b, down));
                }
            }
            Ok(events)
        }
    }
    pub fn protocol(&self) -> (&'static str, &'static str) {
        ("", "")
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        unsafe {
            SetConsoleOutputCP(self.cp);
            SetConsoleMode(self.input, self.im);
            SetConsoleMode(self.out, self.om);
        }
    }
}

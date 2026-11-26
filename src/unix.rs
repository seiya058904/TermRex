use super::*;
use std::{
    io,
    io::Read,
    os::raw::{c_int, c_ulong},
    sync::atomic::{AtomicBool, Ordering},
};
#[repr(C)]
struct PollFd {
    fd: c_int,
    events: i16,
    revents: i16,
}
extern "C" {
    fn tcgetattr(fd: c_int, state: *mut Termios) -> c_int;
    fn cfmakeraw(state: *mut Termios);
    fn tcsetattr(fd: c_int, action: c_int, state: *const Termios) -> c_int;
    fn poll(fds: *mut PollFd, count: usize, timeout: c_int) -> c_int;
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    fn signal(sig: c_int, handler: usize) -> usize;
}
static STOP: AtomicBool = AtomicBool::new(false);
extern "C" fn stop(_: c_int) {
    STOP.store(true, Ordering::Relaxed);
}
// Opaque storage for libc's termios. Darwin LP64 uses 72 bytes and glibc
// x86_64 uses 60. libc alone interprets the fields, including control chars.
#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct Termios([u8; 256]);
impl Termios {
    fn current() -> io::Result<Self> {
        let mut state = Self([0; 256]);
        if unsafe { tcgetattr(0, &mut state) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(state)
    }
    fn apply(&self) -> io::Result<()> {
        loop {
            if unsafe { tcsetattr(0, 0, self) } == 0 {
                return Ok(());
            }
            let e = io::Error::last_os_error();
            if e.kind() != io::ErrorKind::Interrupted {
                return Err(e);
            }
        }
    }
}
pub struct Terminal {
    original: Termios,
    pending: Vec<u8>,
    enhanced: bool,
    signals: Vec<(i32, usize)>,
    held: Vec<(u32, Button)>,
}
impl Terminal {
    pub fn open() -> io::Result<Self> {
        let mut t = Self {
            original: Termios::current()?,
            pending: vec![],
            enhanced: false,
            signals: vec![],
            held: vec![],
        };
        for sig in [1, 2, 3, 15] {
            let old = unsafe { signal(sig, stop as *const () as usize) };
            if old != usize::MAX {
                t.signals.push((sig, old));
            }
        }
        let mut raw = t.original;
        unsafe { cfmakeraw(&mut raw) };
        raw.apply()?;
        Ok(t)
    }
    pub fn size(&self) -> io::Result<(usize, usize)> {
        #[cfg(target_os = "linux")]
        const SIZE: c_ulong = 0x5413;
        #[cfg(not(target_os = "linux"))]
        const SIZE: c_ulong = 0x40087468;
        let mut value = [0u16; 4];
        if unsafe { ioctl(1, SIZE, value.as_mut_ptr()) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok((value[1] as usize, value[0] as usize))
    }
    pub fn protocol(&self) -> (&'static str, &'static str) {
        ("\x1b[>11u\x1b[?u\x1b[?1004h", "\x1b[?1004l\x1b[<u")
    }
    pub fn poll(&mut self) -> io::Result<Vec<Event>> {
        if STOP.load(Ordering::Relaxed) {
            return Ok(vec![Event::Key(Button::Quit, true)]);
        }
        let mut fd = PollFd {
            fd: 0,
            events: 1,
            revents: 0,
        };
        if unsafe { poll(&mut fd, 1, 0) } > 0 {
            if fd.revents & 0x38 != 0 {
                return Ok(vec![Event::Key(Button::Quit, true)]);
            }
            let mut buf = [0; 256];
            let n = io::stdin().read(&mut buf)?;
            self.pending.extend_from_slice(&buf[..n]);
        }
        let mut events = vec![];
        while !self.pending.is_empty() {
            let b = self.pending[0];
            if b != 27 {
                self.pending.remove(0);
                if let Some(k) = button(b.into()) {
                    events.push(Event::Pulse(
                        k,
                        match k {
                            Button::Jump if b == 32 => 100,
                            Button::Jump => 310,
                            Button::Down => 360,
                            _ => 480,
                        },
                    ));
                }
                continue;
            }
            if self.pending.len() < 2 {
                break;
            }
            if !matches!(self.pending[1], b'[' | b'O') {
                self.pending.remove(0);
                continue;
            }
            let Some(end) =
                (2..self.pending.len()).find(|&i| (64..=126).contains(&self.pending[i]))
            else {
                if self.pending.len() > 64 {
                    self.pending.clear();
                }
                break;
            };
            let seq: Vec<_> = self.pending.drain(..=end).collect();
            let args = std::str::from_utf8(&seq[2..end]).unwrap_or("");
            if seq[end] == b'u' && args.starts_with('?') {
                self.enhanced = args[1..].parse::<u32>().unwrap_or(0) & 2 != 0;
                continue;
            }
            let mut fields = args.split(';');
            let code = match seq[end] {
                b'A' => 57352,
                b'B' => 57353,
                b'u' => fields
                    .next()
                    .and_then(|s| s.split(':').next()?.parse().ok())
                    .unwrap_or(0),
                b'O' => {
                    self.held.clear();
                    events.push(Event::Blur);
                    continue;
                }
                _ => continue,
            };
            let modes = if seq[end] == b'u' {
                fields.next()
            } else {
                args.split(';').nth(1)
            }
            .unwrap_or("1");
            let mut m = modes.split(':');
            let mods = m.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
            let kind = m.next().unwrap_or("1");
            let Some(k) = (if code == 99 && mods.saturating_sub(1) & 4 != 0 {
                Some(Button::Quit)
            } else {
                button(code)
            }) else {
                continue;
            };
            if !self.enhanced && seq[end] != b'u' && !args.contains(':') {
                // Enhanced terminals can send a legacy arrow press followed by a CSI release.
                events.push(Event::Pulse(k, if k == Button::Jump { 310 } else { 360 }));
            } else if kind == "3" {
                self.held.retain(|(c, _)| *c != code);
                if !self.held.iter().any(|v| v.1 == k) {
                    events.push(Event::Key(k, false));
                }
            } else if !self.held.iter().any(|v| v.0 == code) {
                let first = !self.held.iter().any(|v| v.1 == k);
                self.held.push((code, k));
                if first {
                    events.push(Event::Key(k, true));
                }
            }
        }
        Ok(events)
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.original.apply();
        #[cfg(target_os = "macos")]
        {
            // XNU sets PENDIN when returning to canonical input. FIONREAD runs
            // ttypend under the restored discipline without consuming shell input.
            const FIONREAD: c_ulong = 0x4004667f;
            let mut available: c_int = 0;
            unsafe {
                ioctl(0, FIONREAD, &mut available);
            }
        }
        for &(s, old) in &self.signals {
            unsafe {
                signal(s, old);
            }
        }
    }
}

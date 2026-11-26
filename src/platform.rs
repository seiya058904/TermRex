#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Button {
    Jump,
    Down,
    Pause,
    Restart,
    Mute,
    Quit,
}
#[derive(Debug)]
pub enum Event {
    Key(Button, bool),
    #[cfg(unix)]
    Pulse(Button, u64),
    Blur,
}
pub fn button(code: u32) -> Option<Button> {
    use Button::*;
    Some(match code {
        32 | 87 | 119 | 57352 => Jump,
        83 | 115 | 57353 => Down,
        80 | 112 => Pause,
        82 | 114 => Restart,
        77 | 109 => Mute,
        81 | 113 | 3 => Quit,
        _ => return None,
    })
}
#[cfg(windows)]
#[path = "windows.rs"]
mod os;
#[cfg(unix)]
#[path = "unix.rs"]
mod os;
pub use os::Terminal;

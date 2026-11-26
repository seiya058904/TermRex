pub mod game;
pub mod render;

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::{game::*, render::*};
    use std::cell::RefCell;
    thread_local! {static SIM:RefCell<(Game,Raster)>=RefCell::new((Game::new(1,0),Raster::new(150)));}
    #[no_mangle]
    pub extern "C" fn init(seed: u32, best: u32, cols: u32) {
        SIM.with(|s| *s.borrow_mut() = (Game::new(seed, best), Raster::new(cols as usize)));
    }
    #[no_mangle]
    pub extern "C" fn input(key: u32, pressed: u32) {
        SIM.with(|s| {
            s.borrow_mut().0.key(
                match key {
                    0 => Key::Jump,
                    1 => Key::Down,
                    2 => Key::Pause,
                    _ => Key::Restart,
                },
                pressed != 0,
            )
        });
    }
    #[no_mangle]
    pub extern "C" fn step(auto: u32) {
        SIM.with(|s| {
            let mut s = s.borrow_mut();
            if auto != 0 {
                s.0.pilot();
            }
            s.0.tick();
        });
    }
    #[no_mangle]
    pub extern "C" fn draw() -> *const u8 {
        SIM.with(|s| {
            let (g, r) = &mut *s.borrow_mut();
            r.draw(g);
            r.cells.as_ptr()
        })
    }
    #[no_mangle]
    pub extern "C" fn stat(k: u32) -> u32 {
        SIM.with(|s| {
            let s = s.borrow();
            let g = &s.0;
            let (fg, bg) = colors(g.night);
            match k {
                0 => g.score(),
                1 => g.best,
                2 => g.state as u32,
                3 => g.flash,
                4 => s.1.rows as u32,
                5 => u32::from_be_bytes([0, fg[0], fg[1], fg[2]]),
                _ => u32::from_be_bytes([0, bg[0], bg[1], bg[2]]),
            }
        })
    }
}

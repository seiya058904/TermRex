use termrex_core::{game::*, render::*};
fn apex(hold: usize) -> i32 {
    let mut g = Game::new(1, 0);
    g.key(Key::Jump, true);
    let mut apex = GROUND;
    for i in 0..60 {
        if i == hold {
            g.key(Key::Jump, false);
        }
        g.tick();
        apex = apex.min(g.y);
    }
    assert_eq!(g.y, GROUND);
    apex
}
#[test]
fn variable_jump_and_dive() {
    assert!(apex(1) - apex(60) > 15);
    let mut g = Game::new(1, 0);
    g.key(Key::Jump, true);
    for _ in 0..6 {
        g.tick();
    }
    let y = g.y;
    g.key(Key::Down, true);
    g.tick();
    assert_eq!(g.y, y + 3);
    for _ in 0..30 {
        g.tick();
    }
    assert!(g.duck());
    g.key(Key::Down, false);
    assert!(!g.duck());
}
#[test]
fn milestones_and_rendering() {
    let mut g = Game::new(4577, 0);
    g.key(Key::Restart, true);
    let mut chimes = 0;
    let mut r = Raster::new(120);
    let mut levels = vec![];
    while g.score() < 1500 {
        let old = g.score();
        g.pilot();
        g.tick();
        assert_ne!(g.state, State::Crashed, "score {} {:?}", old, g.obstacles);
        if g.chime {
            chimes += 1;
            assert_eq!(g.score() % 100, 0);
            assert_eq!(g.flash, 72);
        }
        if old != g.score() && [699, 710, 899, 911, 1410].contains(&g.score()) {
            levels.push(g.night);
        }
        if g.frame.is_multiple_of(120) {
            r.draw(&g);
            assert!(r.cells.iter().any(|v| *v != 0));
        }
    }
    assert_eq!(chimes, 15);
    assert_eq!(levels, vec![0., 1., 1., 0., 1.]);
}
#[test]
fn pause_restart_and_collision() {
    let mut g = Game::new(42, 31);
    g.key(Key::Restart, true);
    g.key(Key::Pause, true);
    g.tick();
    assert_eq!(g.distance, 0.);
    g.key(Key::Pause, true);
    while g.state == State::Running {
        g.tick();
    }
    assert!(g.best >= g.score());
    let best = g.best;
    g.key(Key::Restart, true);
    assert_eq!((g.score(), g.best, g.speed), (0, best, 6.));
}

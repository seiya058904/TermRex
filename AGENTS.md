# TermRex

Zero-dependency Rust port of the Chromium **90.0.4430.212** offline dinosaur for native terminals and WebAssembly (`termrex_core` `rlib`+`cdylib` + `termrex` binary).

## Structure & Ownership
- `src/game.rs`: Fixed 60 Hz simulation, variable jump/dive physics, overlapping hitboxes, obstacle spawning, night/flash milestones, and `pilot` autopilot.
- `src/render.rs`: Projects the 600×150 logical frame onto 2×4 Unicode Braille cells using embedded 1-bit masks (`assets/chromium.bits`).
- `src/lib.rs`: `wasm32` C-ABI exports (`init`, `input`, `step`, `draw`, `stat`) consumed by `web/app.js`.
- `src/main.rs`, `src/screen.rs`, `src/platform.rs` (`src/windows.rs`, `src/unix.rs`): Native loop, diffed row writes, raw/VT input, RAII `Drop` terminal restoration, and locked atomic best-score persistence (`%LOCALAPPDATA%\termrex\best` or `$XDG_STATE_HOME/termrex/best`).

## Invariants
- Keep zero external crate dependencies; keep `game.rs` and `render.rs` identical across native and `wasm32`.
- Preserve Chromium 90.0.4430.212 physics/geometry and keep `LICENSE` (BSD-3-Clause) + `assets/LICENSE.chromium` intact (`--license` embeds both; CI copies both to `web/`).
- Minimum terminal viewport is `81×14`; `Screen` and `Terminal` `Drop` guards must always restore cursor, alt-screen, and console/termios state.
- If `assets/chromium.png` changes, regenerate `assets/chromium.bits` with `python tools/atlas.py` (requires Pillow).
- `web/termrex.wasm` is gitignored and built dynamically by `.github/workflows/pages.yml`. Only rebuild and copy it locally when serving `web/` via `http.server` (`cargo build --locked --release --lib --target wasm32-unknown-unknown`, then copy `target/wasm32-unknown-unknown/release/termrex_core.wasm` to `web/termrex.wasm`).

## Verification
- `cargo test --locked`: unit tests and `tests/runner.rs`.
- Native console lifecycle: `python tests/windows_console.py target/release/termrex.exe` (Windows) or `python3 tests/posix_console.py target/release/termrex` (POSIX).
- `cargo run --release -- --benchmark`: real terminal run to 1001 points reporting FPS and frame intervals.
- `.github/workflows/pages.yml` builds all 4 native binaries + WASM, runs console tests, and publishes `web/` with `SHA256SUMS.txt`.

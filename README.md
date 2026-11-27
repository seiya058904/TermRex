<!-- project:start -->
<div align="center">

<h1>🦖 TermRex</h1>

<p><strong>The Chromium offline dinosaur, in your terminal.</strong></p>

<p>
  A zero-dependency native Rust executable that turns the original desert sprites into 2×4 Unicode Braille dots.<br>
  The playable website runs the exact same simulation and pixel renderer through WebAssembly.
</p>

<p>
  <img src="https://img.shields.io/badge/Rust-zero%20deps-d9e54a?style=flat-square&amp;labelColor=242620" alt="Zero-dependency Rust">
  <img src="https://img.shields.io/badge/size-~255%20KB%20exe%20%C2%B7%20~49%20KB%20wasm-86b865?style=flat-square&amp;labelColor=242620" alt="About 255 KB native executable and 49 KB WebAssembly module">
  <img src="https://img.shields.io/badge/display-2%C3%974%20Braille-e7e8dd?style=flat-square&amp;labelColor=242620" alt="2 by 4 Braille dot cells">
  <a href="https://github.com/seiya058904/TermRex/actions/workflows/pages.yml"><img src="https://github.com/seiya058904/TermRex/actions/workflows/pages.yml/badge.svg" alt="Build, verify and publish"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-BSD--3--Clause-a9ae9b?style=flat-square&amp;labelColor=242620" alt="BSD 3-Clause"></a>
</p>

<p>
  <a href="https://seiya058904.github.io/TermRex/"><strong>▶ Play Live Demo</strong></a>
  &nbsp;·&nbsp;
  <a href="#run-locally">⬇ Run Locally</a>
  &nbsp;·&nbsp;
  <a href="#highlights">✨ Highlights</a>
  &nbsp;·&nbsp;
  <a href="#controls">⌨️ Controls</a>
  &nbsp;·&nbsp;
  <a href="#inside-the-runner">⚙️ Inside the Runner</a>
  &nbsp;·&nbsp;
  <a href="#development">🛠️ Development</a>
</p>

<p><sub>PURE RUST &nbsp;·&nbsp; ZERO CRATE DEPENDENCIES &nbsp;·&nbsp; ~255 KB WINDOWS EXE / ~49 KB WASM &nbsp;·&nbsp; 60 HZ FIXED TICK &nbsp;·&nbsp; WINDOWS · LINUX · MACOS · BROWSER</sub></p>

</div>
<!-- project:end -->

<p align="center">
  <a href="https://seiya058904.github.io/TermRex/">
    <img src="web/terminal.gif" width="952" alt="A real TermRex Windows console run. A Braille dinosaur jumps over a cactus as the ground scrolls and the score advances.">
  </a>
  <br>
  <sub>LIVE WINDOWS CONSOLE CAPTURE (CASCADIA MONO) &nbsp;·&nbsp; SELECT <strong>PLAY DEMO</strong> IN THE BROWSER TO TAKE OVER FROM AUTOPILOT</sub>
</p>

---

> **No game engine. No external crates. No separate runtime.**
>
> A **~255 KB** portable Windows binary (and **~49 KB** WebAssembly engine) built with LTO, stripped symbols, and 1-bit sprite masks—rendering a 600 × 150 desert scene straight into terminal Braille cells at 60 Hz, with identical Rust logic powering the [live browser demo](https://seiya058904.github.io/TermRex/).

> [!NOTE]
> TermRex is an independent port of the Chromium offline runner (**90.0.4430.212**). Visual atlas, geometry, and movement rules are adapted from Chromium's pinned `offline.js` and 1× sprite sheet, with upstream notices preserved in [assets/LICENSE.chromium](assets/LICENSE.chromium).

<a id="highlights"></a>
## ✨ Highlights

<table>
  <tr>
    <td width="50%" valign="top">
      <h3>⠁ 2×4 Braille Pixel Engine</h3>
      <p><sub>600×150 LOGICAL FRAME · ADAPTIVE COLUMNS</sub></p>
      <p>Projects the original 1× sprite sheet (idle blink, run, duck, jump, crash, cacti, birds, clouds, terrain, and 7 moon phases) onto Unicode Braille dot cells. Wider terminals preserve finer detail.</p>
    </td>
    <td width="50%" valign="top">
      <h3>⚡ Chromium-Faithful Physics</h3>
      <p><sub>60 HZ FIXED TICK · VARIABLE JUMP · DIVE</sub></p>
      <p>Starts at 6 px/tick, accelerates by 0.001, and caps at 13. Early key release trims upward velocity, pressing down in midair triggers a 3× dive, and overlapping head/body/leg boxes handle close passes.</p>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <h3>🖥️ Safe Terminal Lifecycle</h3>
      <p><sub>DIFFED ROW WRITES · RAII RESTORATION</sub></p>
      <p>Batches only changed rows per frame and pauses cleanly if resized below <code>81 × 14</code>. Dropping the screen or input guards restores the shell on exit, I/O error, or panic unwinding.</p>
    </td>
    <td width="50%" valign="top">
      <h3>🌐 Shared Core in WebAssembly</h3>
      <p><sub>NATIVE + WASM PARITY · AUTOPILOT DEMO</sub></p>
      <p>The exact same <code>termrex_core</code> crate compiles to <code>wasm32-unknown-unknown</code> for the browser demo, complete with autopilot takeover and touch controls.</p>
    </td>
  </tr>
</table>

<a id="run-locally"></a>
## 📦 Run Locally

Download a portable binary and run it in a **UTF-8 terminal** with a Braille-capable monospace font, at least **81 columns × 14 rows**.

| Platform | Binary | Run |
| --- | --- | --- |
| **Windows x64** | [termrex.exe](https://seiya058904.github.io/TermRex/downloads/termrex.exe) | `.\termrex.exe` |
| **Linux x64** *(glibc, Ubuntu 22.04+)* | [termrex-linux-x64](https://seiya058904.github.io/TermRex/downloads/termrex-linux-x64) | `chmod +x termrex-linux-x64 && ./termrex-linux-x64` |
| **macOS Apple silicon** | [termrex-macos-arm64](https://seiya058904.github.io/TermRex/downloads/termrex-macos-arm64) | `chmod +x termrex-macos-arm64 && ./termrex-macos-arm64` |
| **macOS Intel** | [termrex-macos-x64](https://seiya058904.github.io/TermRex/downloads/termrex-macos-x64) | `chmod +x termrex-macos-x64 && ./termrex-macos-x64` |

<sub>The downloads are unsigned. Every Pages deployment builds its binaries from the same commit as the live demo and publishes <a href="https://seiya058904.github.io/TermRex/downloads/SHA256SUMS.txt">SHA-256 checksums</a>.</sub>

Or install from source with Rust **1.89+** (`--locked`):

```sh
cargo install --locked --git https://github.com/seiya058904/TermRex
termrex
```

<a id="controls"></a>
## ⌨️ Controls & CLI

| Key | Action |
| --- | --- |
| <kbd>Space</kbd>, <kbd>↑</kbd>, <kbd>W</kbd> | Start or jump. Release early for a shorter arc. |
| <kbd>↓</kbd>, <kbd>S</kbd> | Duck while grounded. Press in midair to dive. |
| <kbd>P</kbd> | Pause or resume. |
| <kbd>R</kbd> | Start a fresh run. |
| <kbd>M</kbd> | Toggle the milestone bell (or pass `--mute`). |
| <kbd>Q</kbd>, <kbd>Ctrl+C</kbd> | Quit and restore the terminal. |

<details>
<summary><strong>Key-release behavior &amp; CLI flags</strong></summary>

- **Key release:** Windows uses native console key events. On macOS and Linux, terminals supporting the kitty keyboard protocol report press, repeat, and release. On press-only terminals, `Space` performs a short jump, `↑`/`W` performs a full jump, and repeated down events renew a short duck timeout.
- **CLI flags:**
  - `termrex --demo` — watch the built-in autopilot run.
  - `termrex --mute` — start without the milestone bell.
  - `termrex --benchmark` — drive real keyboard actions to 1001 points and report loop FPS, 99th-percentile/max frame intervals, and missed deadlines.
  - `termrex --license` — print both BSD 3-Clause and Chromium notices.

</details>

<a id="inside-the-runner"></a>
## ⚙️ Inside the Runner

| Component | Details |
| --- | --- |
| **Movement** | 60 Hz fixed tick; speed `6 → 13` (`+0.001/tick`); jump `-10` with speed adjustment; gravity `0.6`; early-release cap `-5`; `3×` dive multiplier. |
| **Milestones** | Every `100` points flashes the score 3×. Every `700` points triggers a fading night cycle (~`200` displayed points). Birds spawn from speed `8.5`. |
| **Score Storage** | Native best scores use a locked, atomically replaced file at `%LOCALAPPDATA%\termrex\best` or `$XDG_STATE_HOME/termrex/best` (default `~/.local/state/termrex/best`). Browser scores stay local; autopilot runs are unsaved. |

<a id="development"></a>
## 🛠️ Development

```sh
cargo test --locked
cargo run --release -- --demo
cargo run --release -- --benchmark
```

To rebuild and serve the WebAssembly site locally:

```sh
rustup target add wasm32-unknown-unknown
cargo build --locked --release --lib --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/termrex_core.wasm web/termrex.wasm
python -m http.server --directory web
```

<a id="credits"></a>
## 📄 Credits & License

- **Upstream source:** Chromium **90.0.4430.212** ([offline.js](https://github.com/chromium/chromium/blob/90.0.4430.212/components/neterror/resources/offline.js) and [1× sprite sheet](https://github.com/chromium/chromium/blob/90.0.4430.212/components/neterror/resources/images/default_100_percent/offline/100-offline-sprite.png)).
- **Licenses:** Original project code is licensed under [BSD 3-Clause](LICENSE); Chromium assets retain their [upstream notice](assets/LICENSE.chromium). Run `termrex --license` to view both.

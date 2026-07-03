# Plan: browser-based preview of ttysvr via WASM + a tailnet web server

Goal: run ttysvr's screensavers in a browser, served from a host on the
tailscale network, so changes made in remote coding sessions can be previewed
by pulling a branch and running one command.

## 1. How ttysvr works today, and what the "web mode" is

ttysvr is a Bevy app that renders scenes with normal Bevy cameras, then
converts the rendered frames to unicode cells and draws them to the terminal:

- `bevy_ratatui` (currently 0.7) owns the terminal: crossterm context, input
  events, and the `RatatuiContext` draw surface.
- `bevy_ratatui_camera` (currently 0.8.2) attaches to a camera, copies the
  GPU render target back to the CPU each frame, and exposes a ratatui widget.
- The app runs headless: `WinitPlugin` is disabled and `ScheduleRunnerPlugin`
  drives the loop.

The browser path exists in newer versions of this same stack:

- `bevy_ratatui` **0.9+** added a `windowed` feature (built on
  [`soft_ratatui`](https://crates.io/crates/soft_ratatui), a software renderer
  that rasterizes the ratatui buffer with an embedded monospace font). Instead
  of drawing to a terminal, the TUI buffer is drawn into a normal Bevy window
  — and Bevy windows run in the browser via WASM (winit → canvas).
- `bevy_ratatui_camera` 0.16 already forwards this: it has a
  `windowed = ["bevy_ratatui/windowed"]` feature.

So the web preview is: **compile ttysvr to `wasm32-unknown-unknown` with the
`windowed` feature, render the ratatui buffer into a canvas, and serve the
bundle over HTTP on the tailnet.** The preview shows the real cell-for-cell
TUI output (same conversion strategies), just rasterized in a canvas instead
of a terminal.

## 2. Version upgrades required

The newest fully-aligned released stack is **Bevy 0.17** (bevy_ratatui 0.11
targets Bevy 0.18, but bevy_ratatui_camera has not released a 0.18-compatible
version yet — its `main` is still Bevy 0.17):

| crate               | current                | target       | notes |
|---------------------|------------------------|--------------|-------|
| bevy                | 0.15                   | **0.17**     | two-release migration (0.15→0.16→0.17) |
| bevy_ratatui        | 0.7                    | **0.10**     | `windowed` feature exists since 0.9 |
| bevy_ratatui_camera | 0.8.2                  | **0.16**     | has `windowed` passthrough feature |
| ratatui             | 0.29                   | **0.29**     | unchanged (0.30 only needed for bevy_ratatui 0.11+) |
| avian2d             | git `main` (0.15-era)  | **0.4.x** (crates.io) | avian2d 0.4 targets bevy 0.17; drop the git pin |
| rand / rand_chacha  | 0.8 / 0.3              | **0.9 / 0.9**| bevy_math 0.17 uses rand 0.9, so `sample_interior(&mut rng)` requires rand 0.9 types |
| clap                | 4.5                    | unchanged    | native-only (gated off for wasm) |

A later pass can move to bevy 0.18 + bevy_ratatui 0.11 + ratatui 0.30 once
bevy_ratatui_camera releases 0.18 support; nothing in this plan blocks that.

### Code migrations (0.15 → 0.17), by file

- **`src/lib.rs`**
  - rand 0.9: `rand::distributions::Standard` → `rand::distr::StandardUniform`;
    `rng.gen_range(..)` → `rng.random_range(..)`.
  - Plugin setup must become conditional (terminal vs windowed/web) — see §4.
- **`src/common.rs`**
  - `bevy::utils::error` is gone; systems can return `Result` natively since
    0.16 — drop the `.map(error)` wrapper and return `bevy::prelude::Result`.
  - `Query::get_single()` was deprecated in 0.16 and removed in 0.17 →
    `single()` (returns `Result`).
  - `EventWriter::send/send_default` → `write/write_default` (renamed 0.16,
    removed 0.17).
  - Keyboard/mouse exit systems consume crossterm-based events from
    bevy_ratatui — terminal-only; gate them (`#[cfg]`/feature) and add a
    no-op or "reload/switch variant" behavior for web.
- **`src/logo.rs`**
  - `OrthographicProjection` is no longer a component in 0.16+ — spawn
    `Projection::Orthographic(OrthographicProjection { .. })` (bubbles.rs
    already uses this form).
  - rand 0.9 renames as above.
- **`src/maze.rs`**
  - `rand::thread_rng()` → `rand::rng()`; slice `choose` now comes from
    `rand::seq::IndexedRandom`.
  - `camera.single_mut()` returns `Result` → handle it.
  - `Msaa::Sample8`: clamp to `Sample4` on web (WebGPU/WebGL2 typically
    support max 4x).
- **`src/bubbles.rs`**
  - avian2d 0.4: verify `PhysicsPlugins::default().with_length_unit(..)`,
    `Gravity`, `LockedAxes`, `Friction::new` against the 0.2→0.4 migration
    notes (API has been mostly stable; expect small renames at most).
  - rand 0.9 renames.
- **`src/main.rs` / `src/args.rs`**
  - Unchanged for native. On wasm there is no CLI: gate clap/args and the
    TMOUT init/cancel branches to `not(target_arch = "wasm32")`.
- **`Cargo.lock` / packaging**
  - Removing the avian git dependency restores `cargo install --locked`
    friendliness.

## 3. The one real blocker: GPU readback on WASM

`bevy_ratatui_camera`'s readback (`src/camera_image_pipe.rs::send_image_buffer`)
does:

```rust
buffer_slice.map_async(MapMode::Read, ..);
render_device.poll(PollType::wait());   // blocks until GPU done
r.recv().expect(..);                    // blocks on channel
```

On native this is fine. On `wasm32-unknown-unknown` **it deadlocks**: the
browser cannot block the main thread — `poll(wait)` cannot actually wait on
web, `map_async` callbacks are only delivered by the browser event loop after
the current task yields, so the `recv()` never returns.

**Fix (upstream, in bevy_ratatui_camera — which we maintain):** make the
readback asynchronous across frames instead of synchronous within one:

- Option A (preferred): replace the custom render node + blocking channel
  with Bevy's built-in `bevy::render::gpu_readback::Readback` component /
  `ReadbackComplete` events, which already implement the non-blocking
  map-then-harvest-next-frame pattern and are web-compatible.
- Option B (surgical): keep the render node, but split `send_image_buffer`:
  frame N issues `copy_texture_to_buffer` + `map_async`; frame N+1 (or later)
  checks the map result with `try_recv`, copies the mapped range, unmaps.
  Native keeps the fast path behind `#[cfg(not(target_arch = "wasm32"))]` if
  the one-frame latency matters (for a screensaver it does not).

Ship as `bevy_ratatui_camera` 0.17 (or 0.16.1), or consume via a
`[patch.crates-io]` git dependency in ttysvr until released. This is the only
piece of the stack that needs actual new code rather than migration.

Renderer backend: build the web bundle with Bevy's `webgpu` feature as the
primary target (Chrome/Edge/Firefox/Safari all ship WebGPU now; buffer
readback is fully supported). WebGL2 readback works in wgpu but with known
warnings/perf caveats — treat it as a fallback build flag, not the default.

## 4. Restructuring ttysvr for a second target

Add cargo features so one codebase builds both ways:

```toml
[features]
default = ["terminal"]
terminal = ["bevy_ratatui/crossterm"]          # today's behavior
windowed = ["bevy_ratatui/windowed", "bevy_ratatui_camera/windowed"]
```

- `AppPlugin` branches on the feature:
  - terminal: `DefaultPlugins` minus `WinitPlugin`/`LogPlugin` +
    `ScheduleRunnerPlugin` + `RatatuiPlugins::default()` (unchanged).
  - windowed: full `DefaultPlugins` with a `WindowPlugin` config (on wasm:
    `canvas: Some("#ttysvr".into())`, `fit_canvas_to_parent: true`) +
    `RatatuiPlugins` in windowed mode. No `ScheduleRunnerPlugin` — winit
    drives the loop (requestAnimationFrame in the browser).
- `draw_scene_system` is unchanged — `RatatuiContext::draw` is the same API
  in both modes; the widget renders identically.
- New `#[cfg(target_arch = "wasm32")]` entry point in `main.rs`: skip clap,
  read variant/subvariant/background from the URL query string via
  `web_sys::window().location().search()` (e.g. `?variant=maze&sub=hedge&bg=1e1e2e`),
  falling back to random — mirroring the CLI semantics.
- Embedded assets (`embedded://`) work on wasm as-is — no asset HTTP fetching
  needed, the PNGs ride inside the .wasm binary.
- rand on wasm: getrandom 0.3 needs the `wasm_js` backend
  (`getrandom = { version = "0.3", features = ["wasm_js"] }` for wasm targets
  plus `--cfg getrandom_backend="wasm_js"` rustflags in `.cargo/config.toml`,
  scoped to `[target.wasm32-unknown-unknown]`).
- A nice side effect: `cargo run --features windowed` gives a native windowed
  preview too (useful for quick local iteration without a terminal).

## 5. The web server

Two layers — a build/dev tool and a way to reach it over the tailnet:

1. **Trunk** as the bundler + dev server (`web/index.html` + `Trunk.toml`):
   - `Trunk.toml` sets `serve.addresses = ["0.0.0.0"]`, a fixed port (e.g.
     8080), `--release` profile with `wasm-opt`, and the
     `RUSTFLAGS`/target config above.
   - `trunk serve` watches the source tree and rebuilds on change — exactly
     the "preview changes" loop wanted here: on the tailnet host,
     `git pull && trunk serve`, then open `http://<host>.<tailnet>.ts.net:8080`
     (or the raw tailscale IP) from any device on the tailnet. Optionally
     `tailscale serve 8080` to get HTTPS with a tailnet cert (WebGPU is happy
     on plain-HTTP non-localhost origins in Chrome only via flags, so the
     tailscale-serve HTTPS route is the robust choice).
   - `web/index.html`: full-viewport canvas `#ttysvr`, dark background, and a
     tiny variant-picker strip of links (`?variant=bubbles`, `?variant=logo&sub=dvd`,
     …) so switching variants is a page reload, matching the CLI model.
2. **One-command wrapper** in-repo so nobody remembers trunk flags:
   - a `just`/shell script or tiny `xtask` (`cargo xtask serve [--port]`)
     that checks trunk is installed (`cargo binstall trunk`), then runs
     `trunk serve` with the right flags. Documented in README under a new
     "web preview" section.
3. **CI**: add a `cargo build --target wasm32-unknown-unknown --no-default-features
   --features windowed` check job so web support can't silently rot.

## 6. Phased implementation

**Phase 1 — engine upgrade at parity (no web yet).**
Bump to bevy 0.17 / bevy_ratatui 0.10 / bevy_ratatui_camera 0.16 / avian2d
0.4 / rand 0.9; apply the per-file migrations in §2. Verify: all three
variants (and subvariants) render and exit correctly in a real terminal.
This lands as its own commit(s) — it is valuable independent of the web work.

**Phase 2 — windowed feature, native.**
Add the `terminal`/`windowed` features and plugin branching; verify
`cargo run --features windowed -- maze` shows the TUI in a native window.
This proves the soft_ratatui path before wasm enters the picture.

**Phase 3 — wasm-compatible readback in bevy_ratatui_camera.**
Implement §3 (async readback) on a branch of bevy_ratatui_camera; consume via
`[patch.crates-io]`; verify the bevy_ratatui_camera examples run in-browser
with WebGPU; upstream + release, then drop the patch.

**Phase 4 — ttysvr web target + server.**
wasm entry point, query-string settings, `web/index.html`, `Trunk.toml`,
`.cargo/config.toml`, serve script, README docs, CI job. Verify end-to-end:
`trunk serve` on a tailnet host, all variants render in Chrome and Firefox,
resize reflows the TUI grid.

## 7. Risks / open questions

- **Readback on WebGL2** is flaky (wgpu warnings, driver variance) — WebGPU
  is the supported target; WebGL2 stays experimental.
- **avian on wasm**: avian2d works on wasm (single-threaded); if anything
  misbehaves, the bubbles variant is the only consumer and can temporarily
  swap physics for simple integration.
- **MSAA**: cap at 4x (or off) for web builds; maze currently requests 8x.
- **Performance**: soft_ratatui rasterizes the whole grid per frame on the
  CPU — fine for a screensaver-sized grid, but keep the canvas cell count
  bounded (the autoresize multipliers may deserve web-specific tuning).
- **bevy_ratatui_camera 0.16 windowed input**: in windowed mode input arrives
  as Bevy `ButtonInput`, not crossterm `KeyEvent`s — exit-on-input stays
  terminal-only.
- **Fallback (zero-migration) option**, if the wasm readback fix stalls:
  stream the *native* binary to the browser instead — run ttysvr in a PTY
  behind `ttyd`/xterm.js on the tailnet host. Loses the pure-WASM elegance
  but needs no upgrades at all; worth keeping in the back pocket.

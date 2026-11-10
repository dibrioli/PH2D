# Vendored winit 0.30.13 + Wayland `suspended` becomes `WindowEvent::Occluded`

- **Upstream:** crates.io `winit` 0.30.13 (https://github.com/rust-windowing/winit), copied verbatim from the
  registry source (minus `Cargo.lock`, `Cargo.toml.orig`, `.cargo-ok`, `.cargo_vcs_info.json`).
  **Licence:** Apache-2.0 (`LICENSE`).
- **Wired by** `[patch.crates-io]` in the workspace `Cargo.toml`; excluded from the workspace (an empty
  `[workspace]` appended to `Cargo.toml`; `rustfmt.toml` disables formatting; `.typos.toml` skips it).

## What changed against 0.30.13 (every hunk marked `PH2D (VENDOR.md)`)

- `src/platform_impl/linux/wayland/window/state.rs`: `WindowState::is_suspended()` — the last configure
  carried `xdg_toplevel.state.suspended` (xdg-shell v6; sctk 0.19 binds up to 6).
- `src/platform_impl/linux/wayland/state.rs`: the `configure` handler compares `is_suspended()` before and
  after `WindowState::configure`; a change sets `WindowCompositorUpdate::occluded = Some(now)`.
- `src/platform_impl/linux/wayland/event_loop/mod.rs`: `occluded` is delivered as
  `WindowEvent::Occluded(now)` (after `Resized`, before `CloseRequested`).

## Why (Painter missão 4K, `docs/Painter/missao_4k/BACKLOG.md` B23.6)

In 0.30.13 the only `Occluded` on Linux is X11's `VisibilityNotify`; on Wayland the `suspended` state only
skips the resize. The shell's on-demand loop (`ph2d_app_host::quadro::Laco::oculta`) never learned that the
window was minimized under KWin Wayland, so the hidden window kept presenting. The xdg_toplevel lives on
winit's own sctk queue: a second Wayland connection cannot see another client's surface states, hence the
patch. `pre_present_notify` was refused: minimized there are no frame callbacks, so the Documento producers
(paper drying) would stop and change the bytes. Gate: `ph2d-editor-core`
`o_wayland_suspenso_chega_a_shell_como_occluded`. Leaves when upstream emits `Occluded` for `suspended`.

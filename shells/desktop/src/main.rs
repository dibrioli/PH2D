#![forbid(unsafe_code)]

//! Desktop shell — winit 0.30 + wgpu + ECS + sprite render + M6+M7+M12.
//!
//! Run with: `cargo run -p ph2d-host-desktop`
//!
//! Layered subsystems (each gated to keep the demo bootable even if
//! one fails — never crash the shell over an integration-demo issue):
//! - **M5** SpriteRenderer + 1000-sprite Vogel spiral with bouncing motion
//! - **M6** AssetDb loads 16 real PNG files from `assets/sprites/` (auto-
//!   generated on first launch) and composes them into a 256×256
//!   RGBA8 atlas. Falls back to procedural dummy if anything fails.
//! - **M7** ScriptHost with placeholder Luau script; per-frame gc_step
//!   keeps the GC budget warm for future script-driven gameplay.
//! - **M12** editor data layer: `ZenMode` (Tab toggle), `ToastQueue`
//!   (T key adds info toast), theme switch (M key flips Dark↔Light),
//!   and a `FloatingPanel` (Procreate-style selection demo). Visible
//!   via window title since Vello widget paint requires sharing the
//!   wgpu Surface with the sprite pipeline (see `integration.rs`).
//!
//! M8 add-on (gamepad path):
//! - gilrs adapter (`gilrs_adapter` module) pumps gamepad events into
//!   [`ph2d_input::InputState`] each frame, BEFORE sim tick.
//! - Connection / button / axis events log to the terminal at the
//!   `[Nms]` timestamp prefix used by the rest of the shell.
//! - Axis logs filter dead-zone jitter (only `|value| > 0.25`).
//! - Pencil events are wired through the abstraction but not produced
//!   by this shell (iPad shell will, in M9+).
//!
//! Out of scope here:
//! - **M8 → ScriptHost**: `InputState` lives on `App`, but routing into
//!   `ph2d.input` Luau snapshot is a follow-up.
//! - **M11** Vello text/widget overlay — needs surface-sharing pass.

mod app_state;
mod atlas_loader;
mod cursor_pos;
mod forwarding;
mod hero_bridge;
mod hero_intents;
mod image_import;
mod init;
mod input_dispatch;
mod input_handlers;
mod input_log;
mod integration;
mod keymap;
mod name_unique;
mod render_loop;
mod sim_populate;
mod theme;
mod winit_host;

pub(crate) use app_state::{
    App, AppGfx, HeroLive, ImageEditSnapshot, ImageEditTransaction, is_image_edit_tool,
    palette_visible_tool_indices,
};

// forwarding::* moved to input_dispatch.rs (PR 9b).
// cursor_pos::live_cursor_in_window + image_import::import_image_at_camera
// moved to input_handlers.rs (Wave 3.2 stage B).
use input_log::log_input_event;
// keymap::winit_to_editor_keycode moved to input_dispatch.rs (PR 9b).
// theme::parse_theme_env moved to init.rs (PR 9c).
use winit_host::LoggingHandler;

use ph2d_core::{FixedStep, Vec2, install_panic_hook, panic};
use ph2d_ecs::scene::build_hierarchy_snapshot;
use ph2d_ecs::{Component, SimComponent, SimWorld, Transform};
use ph2d_editor::paint::Paint;
// NodeId surfaces in our `dragging` field; re-exported by ph2d-editor.
use ph2d_editor::NodeId;
use ph2d_host::{HostHandler, Lifecycle, Modifiers, PlatformHost};
use ph2d_input::InputState;
use ph2d_render::SpriteRenderer;
use std::time::Instant;

mod gilrs_adapter;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::ModifiersState;
use winit::window::WindowId;

pub(crate) const SPRITE_COUNT: u32 = 1000;
/// Half-extent of the bouncing world in meters. Camera default has
/// `height_world = 10`, so [-5, 5] in Y is exactly the visible region;
/// X depends on aspect (narrower than visible at 4:3+).
pub(crate) const WORLD_HALF: f32 = 5.0;

// ADR-0025: `Position(Vec2)` removed — `Transform` from `ph2d_ecs` is
// the canonical pose component now. `Velocity` stays local to the
// demo because no other crate consumes it yet (M14.2+ may promote it).
#[derive(Component, Copy, Clone, Debug)]
pub(crate) struct Velocity(pub(crate) Vec2);
impl SimComponent for Velocity {}

impl App {
    fn new() -> Self {
        let gilrs = match gilrs::Gilrs::new() {
            Ok(g) => {
                let pads: Vec<String> = g
                    .gamepads()
                    .map(|(id, pad)| format!("[{:?}] {}", id, pad.name()))
                    .collect();
                if pads.is_empty() {
                    println!("gilrs: initialized; no gamepads connected yet");
                } else {
                    println!("gilrs: detected {} gamepad(s):", pads.len());
                    for p in &pads {
                        println!("  {p}");
                    }
                }
                Some(g)
            }
            Err(e) => {
                eprintln!("gilrs init failed (continuing without gamepad): {e}");
                None
            }
        };
        Self {
            window: None,
            host: None,
            gfx: None,
            handler: LoggingHandler::new(),
            fixed_step: FixedStep::default(),
            last_frame: Instant::now(),
            pending_resize: None,
            modifiers: ModifiersState::default(),
            last_pointer: (0.0, 0.0),
            dragging: None,
            title_dirty: true,
            gilrs,
            input: InputState::new(),
            pan_anchor: None,
            eyedropper_dragging: false,
            last_cursor: (0.0, 0.0),
            hovered_files: Vec::new(),
            pending_drops: Vec::new(),
            frame_cpu_ms_ewma: 1.0, // optimistic baseline; reseeds on
            // the first frame's measurement
            pivot_content_center: None,
            rubber_band: None,
            pending_single_replace: None,
            group_drag_starts: Vec::new(),
            cycle_pick_world: None,
            cycle_pick_hits: Vec::new(),
            cycle_pick_idx: 0,
            cycle_pick_count: 0,
            last_bgremoval_pushed_entity: None,
            last_color_equalization_pushed_entity: None,
            color_equalization_previews: std::collections::BTreeMap::new(),
            last_upscale_pushed_entity: None,
            upscale_preview: None,
            bgremoval_preview: None,
            bgremoval_preview_gpu: None,
            last_painter_pushed_entity: None,
            painter_preview: None,
            painter_commit_requested: false,
            committed_vector_pen_paths: Vec::new(),
            frame_ms_ewma: 16.7, // ~60 Hz baseline so the first
                                 // frame's status bar doesn't display
                                 // a wild value while the EWMA seeds.
        }
    }

    /// Pump every queued gilrs event into the [`InputState`] and log
    /// the salient ones. Press / release / axis-change all logged at
    /// elapsed-ms timestamps so behavior is auditable from the
    /// terminal without an explicit debug overlay.
    fn pump_gamepad(&mut self) {
        let Some(g) = self.gilrs.as_mut() else {
            return;
        };
        // begin_frame snapshots last-frame held buttons so
        // pressed()/released() return correct edge-trigger values.
        self.input.begin_frame();
        while let Some(gilrs::Event { event, time: _, .. }) = g.next_event() {
            match event {
                gilrs::EventType::Connected => {
                    println!("[{:>6}ms] gamepad connected", self.handler.elapsed_ms());
                }
                gilrs::EventType::Disconnected => {
                    println!("[{:>6}ms] gamepad disconnected", self.handler.elapsed_ms());
                }
                _ => {
                    if let Some(translated) = gilrs_adapter::translate(event) {
                        self.input.apply_event(translated);
                        log_input_event(self.handler.elapsed_ms(), &translated);
                    }
                }
            }
        }
    }

    fn convert_modifiers(state: ModifiersState) -> Modifiers {
        Modifiers {
            shift: state.shift_key(),
            ctrl: state.control_key(),
            alt: state.alt_key(),
            meta: state.super_key(),
        }
    }

    fn timestamp_ns() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    }

    /// Snapshot `InputState` into the ScriptHost's `ph2d.input` table
    /// so Luau can read held buttons / axis values via the canonical
    /// `gamepad.held.<button>` and `gamepad.axis.<axis>` keys.
    /// Cleared and rebuilt every frame — keys absent from this frame
    /// resolve to `nil` on the Luau side (per the M8 ph2d_input
    /// resolves test).
    fn push_input_to_script(&self) {
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        let Some(host) = gfx.script.as_ref() else {
            return;
        };
        host.clear_input();
        for button in self.input.gamepad.iter_held() {
            let key = format!("gamepad.held.{}", button.as_lua_key());
            host.provide_input(&key, 1.0);
        }
        for (axis, value) in self.input.gamepad.iter_axes() {
            let key = format!("gamepad.axis.{}", axis.as_lua_key());
            host.provide_input(&key, value as f64);
        }
    }

    /// Per-frame render orchestration — body lifted to
    /// [`crate::render_loop`] (Wave 3.1 stage C). See its module docs
    /// for the rationale + the split-impl pattern.
    fn render_frame(&mut self) {
        self.run_render_frame();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // PR 9c of the convention-by-discovery migration: subsystem
        // boot pipeline lives in `init::build_initial_state`. This
        // method now only wires the produced state into `self` and
        // fires lifecycle hooks. See
        // `docs/Migracao/2026-05-convention-by-discovery.md`.
        let (window, host, gfx) = init::build_initial_state(&self.handler, event_loop);
        let size = gfx.surface.size();
        let scale = host.scale_factor();
        self.window = Some(window);
        self.host = Some(host);
        self.gfx = Some(gfx);
        self.handler.on_lifecycle(Lifecycle::Foreground);
        self.handler.on_resize(size, scale);
        self.title_dirty = true;
        if let Some(host_ref) = self.host.as_ref() {
            host_ref.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // PR 9b of the convention-by-discovery migration: per-arm
        // handlers live in `input_dispatch::App::on_<arm>`. This
        // method is a pure dispatch table — adding a new arm is one
        // line here + one method there. See
        // `docs/Migracao/2026-05-convention-by-discovery.md`.
        match event {
            WindowEvent::CloseRequested => self.on_close_request(event_loop),
            WindowEvent::Resized(size) => self.on_resized(size),
            WindowEvent::HoveredFile(path) => self.on_hovered_file(path),
            WindowEvent::HoveredFileCancelled => self.on_hovered_file_cancelled(),
            WindowEvent::DroppedFile(path) => self.on_dropped_file(path),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.on_scale_factor_changed(scale_factor)
            }
            WindowEvent::ModifiersChanged(mods) => self.on_modifiers_changed(mods),
            WindowEvent::Ime(winit::event::Ime::Commit(text)) => self.on_ime_commit(text),
            WindowEvent::CursorMoved { position, .. } => self.on_cursor_moved(position),
            WindowEvent::MouseWheel { delta, .. } => self.on_mouse_wheel(delta),
            WindowEvent::MouseInput { state, button, .. } => self.on_mouse_input(state, button),
            WindowEvent::KeyboardInput { event, .. } => self.on_keyboard_input(event),
            WindowEvent::RedrawRequested => self.render_frame(),
            _ => {}
        }
    }
}

/// Floor for `pixels_per_meter` used inside the import math; below
/// this a single sprite would span kilometers and break camera math.
/// The UI clamps to a higher floor (`MIN_PIXELS_PER_METER = 1.0` in
/// `ph2d_editor::project`) but defense-in-depth here keeps the shell
/// safe even if a future config path skips that clamp.
pub(crate) const EPS_PIXELS_PER_METER: f32 = 0.01;

/// When the image-edit undo slot is being overwritten by a new edit,
/// release every pre-edit Individual texture across the previous
/// transaction's entries (multi-sprite Apply leaves N entries; the
/// single-sprite case degenerates to N=1). Atlas-backed pre-sources
/// don't need release — they share the texture via the asset_db.
/// No-op when the slot is empty.
pub(crate) fn drop_undo_pre_sources_if_individual(
    renderer: &mut SpriteRenderer,
    slot: &mut Option<ImageEditTransaction>,
) {
    if let Some(prev) = slot.take() {
        for entry in prev.entries {
            if let ph2d_render::SpriteSource::Individual { texture_id } = entry.pre_source {
                renderer.individual_mut().release(texture_id);
            }
        }
    }
}

/// Commit `entries` (one per sprite the multi-sprite Apply touched) as
/// the new undo transaction, releasing the previous transaction's
/// pre-edit individual textures. No-op when `entries.is_empty()` (no
/// sprite actually changed → nothing to undo). The transaction label
/// comes from the first entry; per-drain code pushes the same label on
/// every entry it appends, so all N entries agree by construction.
pub(crate) fn commit_image_edit_transaction(
    renderer: &mut SpriteRenderer,
    slot: &mut Option<ImageEditTransaction>,
    entries: Vec<ImageEditSnapshot>,
) {
    if entries.is_empty() {
        return;
    }
    let label = entries[0].label;
    drop_undo_pre_sources_if_individual(renderer, slot);
    *slot = Some(ImageEditTransaction { entries, label });
}

/// Floor for the world-space side length of an imported sprite.
/// Guarantees the quad is selectable even if the user picks an
/// absurd `pixels_per_meter` value combined with a 1-pixel image.
pub(crate) const MIN_SPRITE_SIZE: f32 = 0.001;

/// Query the live cursor position relative to `window` in physical
/// pixels (top-left origin). Returns `None` if the platform path
/// fails; callers fall back to a cached value.
///
/// Existence rationale: winit 0.30 on macOS does not emit
/// `CursorMoved` during external file drag operations, so by the
/// time `DroppedFile` fires the cached cursor is stale. We bypass
/// the event stream by asking CoreGraphics for the live cursor
/// directly. Other platforms reach here only as a no-op stub.
/// Resolve the editor theme from a name (typically read from the
/// `PH2D_THEME` env var), falling back to [`Theme::Forge`] for
/// missing/invalid values. Recognised names match `Theme::id()`
/// (`forge`, `workshop`, `sunstone`, `blueprint`).
fn main() {
    install_panic_hook();
    let event_loop = EventLoop::new().expect("create EventLoop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    println!("PH2D desktop shell starting (close window or Cmd+Q to exit)…");
    event_loop.run_app(&mut app).expect("event loop crashed");
    println!("PH2D desktop shell exited cleanly.");
}

#[cfg(test)]
mod theme_env_tests {
    use crate::theme::resolve_theme;
    use ph2d_tokens::Theme;

    #[test]
    fn unset_defaults_to_forge() {
        assert_eq!(resolve_theme(None), Theme::Forge);
    }

    #[test]
    fn known_names_resolve() {
        assert_eq!(resolve_theme(Some("workshop")), Theme::Workshop);
        assert_eq!(resolve_theme(Some("sunstone")), Theme::Sunstone);
        assert_eq!(resolve_theme(Some("blueprint")), Theme::Blueprint);
        assert_eq!(resolve_theme(Some("forge")), Theme::Forge);
    }

    #[test]
    fn unknown_falls_back_to_default() {
        assert_eq!(resolve_theme(Some("dracula")), Theme::Forge);
    }
}

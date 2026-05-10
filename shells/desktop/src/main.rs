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

mod integration;

use bumpalo::Bump;
use ph2d_asset::AssetDb;
use ph2d_core::{FixedStep, Vec2, install_panic_hook, panic};
use ph2d_ecs::{Component, PresentWorld, SimComponent, SimWorld};
use ph2d_editor::paint::{Paint, PaintCtx};
use ph2d_editor::zones::Rect as EditorRect;
use ph2d_editor::{
    BrushTool, HeroScreen, Layout as EditorLayout, MoveTool, PanelControl, PanelEvent, Toast,
    ToastQueue, ToolRegistry, WidgetEvent, ZenMode, paint_hero_screen,
};
// NodeId surfaces in our `dragging` field; re-exported by ph2d-editor.
use ph2d_editor::NodeId;
use ph2d_gpu::{AcquireError, GpuContext, SurfaceContext};
use ph2d_host::{
    CloseAction, HostHandler, KeyEvent, KeyKind, Lifecycle, Modifiers, PlatformHost, PointerEvent,
    PointerKind, PointerSource, WindowSize,
};
use ph2d_input::{Event as InputEvent, InputState};
use ph2d_render::{Camera2d, RenderInstance, Sprite, SpriteRenderer, TextureAtlas, VelloPass};
use ph2d_script::ScriptHost;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Color as VelloColor, VectorScene};
use std::cell::Cell;
use std::sync::Arc;
use std::time::Instant;

mod gilrs_adapter;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent as WinitKeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::{Window, WindowId};

const SPRITE_COUNT: u32 = 1000;
/// Half-extent of the bouncing world in meters. Camera default has
/// `height_world = 10`, so [-5, 5] in Y is exactly the visible region;
/// X depends on aspect (narrower than visible at 4:3+).
const WORLD_HALF: f32 = 5.0;

#[derive(Component, Copy, Clone, Debug)]
struct Position(Vec2);
impl SimComponent for Position {}

#[derive(Component, Copy, Clone, Debug)]
struct Velocity(Vec2);
impl SimComponent for Velocity {}

struct WinitHost {
    window: Arc<Window>,
    scale: Cell<f32>,
}

impl WinitHost {
    fn new(window: Arc<Window>) -> Self {
        let scale = window.scale_factor() as f32;
        Self {
            window,
            scale: Cell::new(scale),
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }
}

impl PlatformHost for WinitHost {
    fn request_redraw(&self) {
        self.window.request_redraw();
    }
    fn window_size(&self) -> WindowSize {
        let size = self.window.inner_size();
        WindowSize::new(size.width, size.height)
    }
    fn scale_factor(&self) -> f32 {
        self.scale.get()
    }
}

struct LoggingHandler {
    started_at: Instant,
}
impl LoggingHandler {
    fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
    fn elapsed_ms(&self) -> u128 {
        self.started_at.elapsed().as_millis()
    }
}
impl HostHandler for LoggingHandler {
    fn on_resize(&mut self, size: WindowSize, scale_factor: f32) {
        println!(
            "[{:>6}ms] resize: {}x{} @ {:.2}x scale",
            self.elapsed_ms(),
            size.width,
            size.height,
            scale_factor
        );
    }
    fn on_lifecycle(&mut self, kind: Lifecycle) {
        println!("[{:>6}ms] lifecycle: {:?}", self.elapsed_ms(), kind);
    }
    fn on_pointer(&mut self, event: PointerEvent) {
        if matches!(event.kind, PointerKind::Down | PointerKind::Up) {
            println!(
                "[{:>6}ms] pointer {:?} {:?} ({:.0}, {:.0}) p={:.2}",
                self.elapsed_ms(),
                event.source,
                event.kind,
                event.x,
                event.y,
                event.pressure
            );
        }
    }
    fn on_key(&mut self, event: KeyEvent) {
        println!(
            "[{:>6}ms] key {:?} keycode={} mods={:?}",
            self.elapsed_ms(),
            event.kind,
            event.keycode,
            event.modifiers
        );
    }
    fn on_close_request(&mut self) -> CloseAction {
        println!("[{:>6}ms] close requested → Close", self.elapsed_ms());
        CloseAction::Close
    }
}

/// Holds every initialized-after-`resumed` resource. Bundling them into
/// a single `Option<AppGfx>` lets us destructure into per-field `&mut`
/// borrows in `render_frame()` — split-borrowing through a method
/// chain on individual `Option<...>` fields would be awkward.
struct AppGfx {
    surface: SurfaceContext,
    renderer: SpriteRenderer,
    sim: SimWorld,
    present: PresentWorld,
    camera: Camera2d,
    /// M6 — set when PNG fixtures loaded successfully; held so the
    /// AssetDb keeps `Arc<Asset>` alive for hot-reload follow-ups.
    asset_db: AssetDb,
    /// M6 — true when the atlas was composed from real PNG files (vs the
    /// procedural dummy fallback). Surfaced in window title.
    atlas_is_real: bool,
    /// M7 — Luau VM with placeholder script loaded. Per-frame gc_step
    /// keeps the GC budget warm; set/get bindings ready for follow-up
    /// gameplay work.
    script: Option<ScriptHost>,
    /// M12 editor data layer + M11 widget paint pass.
    theme: Theme,
    zen: ZenMode,
    toasts: ToastQueue,
    /// Registered editor tools. Keys 1/2 switch active tool; the
    /// active tool's `build_panel()` is painted each frame as the
    /// FloatingPanel that shows in the bottom-center of the canvas.
    tools: ToolRegistry,
    /// 4-zone editor layout (ADR-0023 §3). Sized from window each
    /// resize; the M11 paint pass walks this to draw zone backdrops.
    layout: EditorLayout,
    /// Vello pipeline + intermediate texture + blitter for the
    /// widget paint pass. Runs AFTER the sprite pass on the same
    /// surface frame, so widgets sit on top of game content.
    vello_pass: VelloPass,
    /// Reused [`VectorScene`] — encoded fresh each frame; allocations
    /// pool inside Vello so this is cheap.
    vector_scene: VectorScene,
    /// parley font + layout context (heavy state). Threaded through
    /// `PaintCtx` so future text passes don't re-load fonts.
    text_system: TextSystem,
    /// Hero screen (`02-editor-main` mockup) — populated when
    /// `PH2D_HERO_SCREEN=1`. Owns the [`WidgetStore`] + [`HitIndex`]
    /// so input pipeline (ADR-0024) can route pointer/key events
    /// through `dispatch_*`.
    hero_screen: Option<HeroScreen>,
    /// Per-frame arena for [`WidgetEvent`]s emitted by the hero
    /// dispatcher. Reset at end-of-frame.
    hero_arena: Bump,
}

struct App {
    window: Option<Arc<Window>>,
    host: Option<WinitHost>,
    gfx: Option<AppGfx>,
    handler: LoggingHandler,
    fixed_step: FixedStep,
    last_frame: Instant,
    pending_resize: Option<WindowSize>,
    modifiers: ModifiersState,
    last_pointer: (f32, f32),
    /// Set to `Some(node_id)` when the user pressed inside a draggable
    /// widget (Slider) — subsequent pointer-move events continue to
    /// fire SetValue until pointer-up clears this. None for click-only
    /// widgets (Toggle, RadioGroup, ColorSwatch).
    dragging: Option<NodeId>,
    title_dirty: bool,
    /// gilrs context (M8). `None` if init failed (e.g. Linux without
    /// /dev/input read perms in CI sandboxes — we degrade gracefully
    /// instead of crashing the renderer).
    gilrs: Option<gilrs::Gilrs>,
    /// Input snapshot pumped by the gilrs adapter each frame.
    input: InputState,
}

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

    /// M12 demo control router.
    ///   Tab — toggle ZenMode (debounced 30 frames)
    ///   M   — flip theme Dark↔Light
    ///   T   — push info toast
    ///   1   — activate Brush tool
    ///   2   — activate Move tool
    fn handle_editor_key(&mut self, code: KeyCode) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        match code {
            KeyCode::Tab if gfx.zen.try_toggle() => {
                let msg = if gfx.zen.is_active() {
                    "Zen mode ON (zones collapsed)"
                } else {
                    "Zen mode OFF (zones restored)"
                };
                gfx.toasts.push(Toast::info(msg));
                self.title_dirty = true;
            }
            KeyCode::KeyM => {
                gfx.theme = gfx.theme.next();
                gfx.toasts
                    .push(Toast::info(format!("Theme → {}", gfx.theme.id())));
                self.title_dirty = true;
            }
            KeyCode::KeyT => {
                gfx.toasts.push(Toast::info("Toast key (T) pressed"));
                self.title_dirty = true;
            }
            KeyCode::Digit1 if gfx.tools.set_active(&ph2d_editor::ToolId::new("brush")) => {
                gfx.toasts.push(Toast::info("Tool → Brush"));
                self.title_dirty = true;
            }
            KeyCode::Digit2 if gfx.tools.set_active(&ph2d_editor::ToolId::new("move")) => {
                gfx.toasts.push(Toast::info("Tool → Move"));
                self.title_dirty = true;
            }
            _ => {}
        }
    }

    /// Hit-test the active tool's panel at `(px, py)` and dispatch a
    /// [`PanelEvent`] into the tool. `is_press` distinguishes the
    /// initial mouse-down (which may start a drag) from continued
    /// move-while-dragging (which only updates an in-progress slider).
    fn dispatch_panel_pointer(&mut self, px: f32, py: f32, is_press: bool) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        if gfx.zen.is_active() {
            return; // panels hidden
        }
        let Some(tool) = gfx.tools.active() else {
            return;
        };
        let panel = tool.build_panel();
        let viewport = EditorRect::new(
            0.0,
            0.0,
            gfx.surface.size().width as f32,
            gfx.surface.size().height as f32,
        );
        let widget_rects = panel.control_widget_rects(viewport);

        // Existing drag → re-emit SetValue against the same node. Done
        // even if pointer left the original cell (slider-style "live
        // drag" feel).
        if let Some(dragging_id) = self.dragging
            && let Some((idx, ctrl)) = panel
                .controls
                .iter()
                .enumerate()
                .find(|(_, c)| matches!(c, PanelControl::Slider(s) if s.id == dragging_id))
            && let Some(rect) = widget_rects.get(idx)
            && let PanelControl::Slider(_) = ctrl
        {
            let v = ((px - rect.x) / rect.w).clamp(0.0, 1.0) as f64;
            if let Some(active) = gfx.tools.active_mut() {
                active.handle_panel_event(PanelEvent::SetValue(dragging_id, v));
            }
            return;
        }

        if !is_press {
            return; // not a click and not a drag — nothing to do
        }

        // Find the cell containing (px, py).
        let Some((idx, _)) = widget_rects
            .iter()
            .enumerate()
            .find(|(_, r)| r.contains(px, py))
        else {
            return;
        };
        let ctrl = &panel.controls[idx];
        let rect = widget_rects[idx];

        let event = match ctrl {
            PanelControl::Slider(s) => {
                self.dragging = Some(s.id);
                let v = ((px - rect.x) / rect.w).clamp(0.0, 1.0) as f64;
                Some(PanelEvent::SetValue(s.id, v))
            }
            PanelControl::Toggle(t) => Some(PanelEvent::Toggle(t.id, !t.on)),
            PanelControl::RadioGroup(g) if !g.options.is_empty() => {
                // Horizontal split — pick option by which sub-rect
                // contains the pointer.
                let opt_w = rect.w / g.options.len() as f32;
                let opt_idx = (((px - rect.x) / opt_w) as usize).min(g.options.len() - 1);
                Some(PanelEvent::SelectOption(
                    g.id,
                    g.options[opt_idx].value.clone(),
                ))
            }
            PanelControl::ColorSwatch(s) => Some(PanelEvent::Click(s.id)),
            PanelControl::Action(_) | PanelControl::RadioGroup(_) => None,
        };

        if let Some(event) = event
            && let Some(active) = gfx.tools.active_mut()
        {
            active.handle_panel_event(event);
            self.title_dirty = true;
        }
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

    /// Compose the M6 atlas from real PNG files. Generates fixtures on first
    /// launch; subsequent launches reuse the on-disk files. Any failure
    /// bubbles a String — caller falls back to the dummy atlas.
    fn try_load_real_atlas(
        gpu: &GpuContext,
        asset_db: &AssetDb,
        dir: &std::path::Path,
    ) -> Result<TextureAtlas, String> {
        let created = integration::ensure_demo_assets_exist(dir)
            .map_err(|e| format!("ensure_demo_assets_exist({}): {e}", dir.display()))?;
        if created > 0 {
            println!(
                "M6: generated {created} demo PNG fixtures in {}",
                dir.display()
            );
        }
        let ids = integration::load_demo_assets(asset_db, dir)?;
        let rgba = integration::compose_atlas_rgba(asset_db, &ids)?;
        Ok(TextureAtlas::from_rgba8(
            gpu,
            integration::ATLAS_PX,
            integration::ATLAS_PX,
            &rgba,
        ))
    }

    /// Spawn `SPRITE_COUNT` sprites on a Vogel (golden-angle) spiral
    /// with pseudo-random velocities derived from index — fully
    /// deterministic, no PRNG dep.
    fn populate_sim(sim: &mut SimWorld) {
        for i in 0..SPRITE_COUNT {
            let f = i as f32;
            let angle = f * 2.399_963_2; // golden angle (rad)
            let r = (f / SPRITE_COUNT as f32).sqrt() * (WORLD_HALF - 0.5);
            let pos = Vec2::new(r * angle.cos(), r * angle.sin());
            // Velocity in m/s; both axes seeded by independent index hashes
            // so motion isn't correlated with the spiral pattern.
            let vx = ((f * 12.9898).sin() * 43758.547).fract() * 3.0 - 1.5;
            let vy = ((f * 78.233).sin() * 12345.678).fract() * 3.0 - 1.5;
            sim.world_mut().spawn((
                Position(pos),
                Velocity(Vec2::new(vx, vy)),
                Sprite {
                    atlas_index: i % 16,
                    size: [0.18, 0.18],
                    tint: [1.0, 1.0, 1.0, 1.0],
                },
            ));
        }
    }

    fn render_frame(&mut self) {
        // Pump gamepad events first so InputState reflects the latest
        // state by the time sim/extract run. Order: input → script
        // input snapshot → sim → extract → render.
        self.pump_gamepad();
        self.push_input_to_script();

        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let AppGfx {
            surface,
            renderer,
            sim,
            present,
            camera,
            asset_db,
            atlas_is_real,
            script,
            theme,
            zen,
            toasts,
            tools,
            layout,
            vello_pass,
            vector_scene,
            text_system,
            hero_screen,
            hero_arena,
        } = gfx;
        let Some(host) = self.host.as_ref() else {
            return;
        };

        // M12 per-frame ticks: ZenMode debounce cooldown + ToastQueue
        // TTL decay. Both are pure data-layer (no Vello paint here).
        zen.tick();
        let prev_toasts = toasts.len();
        toasts.tick();
        if toasts.len() != prev_toasts {
            self.title_dirty = true;
        }

        // M7 per-frame GC step. Cheap (p99 ≤ 10µs target per the M7
        // gate test) — keeps the Luau heap from accumulating between
        // future scripted ticks. Errors here are non-fatal; just log.
        if let Some(host) = script
            && let Err(e) = host.gc_step()
        {
            eprintln!("M7 gc_step error: {e}");
        }

        // Apply coalesced resize once per frame.
        if let Some(size) = self.pending_resize.take() {
            surface.resize(size);
            // Layout + Vello intermediate must follow surface size.
            *layout = EditorLayout::new(size.width as f32, size.height as f32);
            vello_pass.ensure_size(surface.gpu(), (size.width, size.height));
            self.handler.on_resize(size, host.scale_factor());
            self.title_dirty = true;
        }

        // Drive fixed-step accumulator.
        let now = Instant::now();
        let wall_dt = now.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = now;
        let report = self.fixed_step.advance(wall_dt);
        if report.dropped_secs > 0.0 {
            eprintln!(
                "warn: dropped {:.3}s of sim time (max_substeps cap)",
                report.dropped_secs
            );
        }
        panic::set_frame_id(self.fixed_step.tick_count());

        // Sim tick: bouncing motion. Single substep per frame for the
        // M5 demo (we don't yet honor the FixedStep substep count for
        // gameplay — that lands in M10 with the physics integrator).
        let dt = self.fixed_step.fixed_dt() as f32;
        {
            let mut q = sim.world_mut().query::<(&mut Position, &mut Velocity)>();
            for (mut pos, mut vel) in q.iter_mut(sim.world_mut()) {
                let mut p = pos.0;
                let mut v = vel.0;
                p += v * dt;
                if p.x.abs() > WORLD_HALF {
                    v.x = -v.x;
                    p.x = p.x.clamp(-WORLD_HALF, WORLD_HALF);
                }
                if p.y.abs() > WORLD_HALF {
                    v.y = -v.y;
                    p.y = p.y.clamp(-WORLD_HALF, WORLD_HALF);
                }
                pos.0 = p;
                vel.0 = v;
            }
        }

        // Extract: rebuild RenderInstance set in PresentWorld each
        // frame. Per ADR-0021: this is the only legal sim → present
        // bridge. Pre-build the QueryState because `bevy_ecs::query()`
        // needs `&mut World`, but inside `extract!` the sim handle is
        // immutable.
        let atlas = renderer.atlas();
        let mut sim_q = sim.world_mut().query::<(&Position, &Sprite)>();
        ph2d_ecs::extract!(*sim => *present, |sim_w, present_w| {
            present_w.clear_entities();
            for (pos, spr) in sim_q.iter(sim_w) {
                present_w.spawn(RenderInstance {
                    world_pos: [pos.0.x, pos.0.y],
                    size: spr.size,
                    atlas_uv: atlas.dummy_uv(spr.atlas_index),
                    tint: spr.tint,
                });
            }
        });

        // Animated background tint (proves the sim clock drives the
        // frame). Subtle so the sprites stay readable.
        let t = self.fixed_step.tick_count() as f64 * self.fixed_step.fixed_dt();
        let r = (t.sin() * 0.05 + 0.05).clamp(0.0, 1.0);
        let g = ((t + 2.094).sin() * 0.05 + 0.05).clamp(0.0, 1.0);
        let b = ((t + 4.188).sin() * 0.05 + 0.05).clamp(0.0, 1.0);

        let window_size = surface.size();
        // M11: build the widget scene up-front (no GPU work yet — just
        // VectorScene encoding). Done outside acquire_frame so an
        // Occluded/Timeout doesn't waste the encoder.
        let viewport = EditorRect::new(
            0.0,
            0.0,
            window_size.width as f32,
            window_size.height as f32,
        );
        vector_scene.reset();
        let mut paint_ctx = PaintCtx {
            theme: *theme,
            viewport,
            text: text_system,
        };

        // Opt-in hero screen mode: when `PH2D_HERO_SCREEN=1` was set
        // at startup the AppGfx owns a HeroScreen with a retained
        // WidgetStore (ADR-0024). Paint reads + writes its hit_index
        // each frame; pointer/key events are forwarded to it from
        // window_event handlers via `hero_screen.handle_*`.
        if let Some(hero) = hero_screen.as_mut() {
            paint_hero_screen(hero, viewport, vector_scene, paint_ctx.text);
            toasts.paint(vector_scene, &mut paint_ctx);
            // Drain frame-local arena AFTER the dispatch + paint pass
            // so any events emitted earlier this frame are still alive
            // for downstream consumers — wired in Phase A+ (currently
            // events are logged, not acted on).
            hero_arena.reset();
        } else {
            layout.paint(vector_scene, &mut paint_ctx);

            // Tool palette in the CREATE zone (top-right). Always-visible
            // chips; click switches active tool. Hidden in Zen mode by
            // virtue of `tool_palette_rects` returning empty there.
            let palette_rects = layout.tool_palette_rects(tools.tools().len());
            let active_id = tools.active().map(|t| t.id());
            let palette_icons: Vec<(EditorRect, &str, bool)> = palette_rects
                .iter()
                .zip(tools.tools().iter())
                .map(|(r, tool)| {
                    let is_active = active_id.as_ref() == Some(&tool.id());
                    (*r, tool.label(), is_active)
                })
                .collect();
            ph2d_editor::paint_tool_palette_icons(
                paint_ctx.text,
                vector_scene,
                &palette_icons,
                paint_ctx.theme,
            );

            if !zen.is_active()
                && let Some(active) = tools.active()
            {
                // Active tool's panel — built fresh each frame; cheap.
                let panel = active.build_panel();
                panel.paint(vector_scene, &mut paint_ctx);
            }
            toasts.paint(vector_scene, &mut paint_ctx);
        }

        match surface.acquire_frame() {
            Ok(frame) => {
                // Pass 1: sprite renderer clears + draws game content.
                renderer.render(
                    &frame,
                    present,
                    camera,
                    window_size,
                    wgpu::Color { r, g, b, a: 1.0 },
                );
                // Pass 2: Vello widgets composite over the surface.
                // bg_color = TRANSPARENT so sprite content shows
                // through where the editor scene is empty (the canvas
                // Center zone is the whole sprite layer).
                if let Err(e) = vello_pass.render(
                    surface.gpu(),
                    vector_scene.inner(),
                    frame.view(),
                    (window_size.width, window_size.height),
                    VelloColor::TRANSPARENT,
                ) {
                    eprintln!("M11 vello_pass.render error: {e}");
                }
                // FrameTarget presents on Drop.
            }
            Err(AcquireError::AwaitingReconfigure) => {
                surface.reconfigure_after_lost();
            }
            Err(AcquireError::Occluded) => {}
            Err(AcquireError::Timeout) => {}
            Err(AcquireError::Other(s)) => {
                eprintln!("acquire_frame other error: {s}");
            }
        }

        // Window title carries editor state. Refresh only when state
        // actually changes — winit set_title triggers a platform call.
        if self.title_dirty {
            let tool_label = tools.active().map(|t| t.label()).unwrap_or("none");
            let title = format!(
                "PH2D — M5+M6+M7+M11+M12 demo | sprites={SPRITE_COUNT} | atlas={} ({} assets) \
                 | script={} | theme={:?} | zen={} | toasts={} | tool={}",
                if *atlas_is_real { "PNG" } else { "dummy" },
                asset_db.len_assets(),
                if script.is_some() { "ok" } else { "off" },
                theme,
                if zen.is_active() { "on" } else { "off" },
                toasts.len(),
                tool_label,
            );
            host.window().set_title(&title);
            self.title_dirty = false;
        }

        host.request_redraw();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("PH2D — desktop shell (M5 — 1000 sprites)")
            .with_inner_size(winit::dpi::LogicalSize::new(1024, 768));
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("create_window must succeed"),
        );
        let host = WinitHost::new(window.clone());
        let size = host.window_size();
        let scale = host.scale_factor();

        let instance = GpuContext::default_instance();
        let raw_surface = instance
            .create_surface(window.clone())
            .expect("create_surface");
        let gpu = GpuContext::new(instance, Some(&raw_surface)).expect("GpuContext::new");
        let surface = SurfaceContext::new(gpu, raw_surface, size).expect("SurfaceContext::new");

        // M6: try to compose the atlas from real PNG files on disk.
        // Auto-generates 16 procedural fixtures on first launch so the
        // demo is self-contained (no committed binary fixtures). Any
        // failure logs and falls back to the M5 procedural dummy —
        // the shell must boot regardless of asset-pipeline issues.
        let asset_db = AssetDb::new();
        let assets_dir = integration::demo_assets_dir();
        let (atlas, atlas_is_real) =
            match Self::try_load_real_atlas(surface.gpu(), &asset_db, &assets_dir) {
                Ok(atlas) => {
                    println!(
                        "[{:>6}ms] M6: real atlas composed from {} ({} assets cached)",
                        self.handler.elapsed_ms(),
                        assets_dir.display(),
                        asset_db.len_assets()
                    );
                    (atlas, true)
                }
                Err(e) => {
                    eprintln!(
                        "[{:>6}ms] M6 fallback to dummy atlas: {e}",
                        self.handler.elapsed_ms()
                    );
                    (TextureAtlas::dummy(surface.gpu()), false)
                }
            };
        let renderer = SpriteRenderer::new(
            surface.gpu().clone(),
            surface.format(),
            atlas,
            SPRITE_COUNT.next_power_of_two(),
        );

        let mut sim = SimWorld::new();
        Self::populate_sim(&mut sim);
        let present = PresentWorld::new();
        let camera = Camera2d::default();

        // M7: ScriptHost. Failure here is also non-fatal (script is
        // a placeholder; full sim-driving lands in M12+ editor panel).
        let script = match integration::init_script_host() {
            Ok(host) => {
                println!(
                    "[{:>6}ms] M7: ScriptHost initialized (placeholder script loaded)",
                    self.handler.elapsed_ms()
                );
                Some(host)
            }
            Err(e) => {
                eprintln!(
                    "[{:>6}ms] M7 ScriptHost failed: {e} — continuing without scripting",
                    self.handler.elapsed_ms()
                );
                None
            }
        };

        // M12 + M11: editor data layer + Vello widget paint pass.
        // ZenMode/ToastQueue/ToolRegistry model state, Layout computes
        // the 4 zones, VelloPass renders all widgets onto the surface
        // AFTER the sprite pass.
        let theme = parse_theme_env();
        eprintln!("[ph2d] theme = {}", theme.id());
        let zen = ZenMode::new();
        let mut toasts = ToastQueue::new();
        toasts.push(Toast::success("Editor data layer wired (M12)"));
        toasts.push(Toast::info("Press 1=Brush, 2=Move, Tab=Zen"));
        let mut tools = ToolRegistry::new();
        tools.register(Box::new(BrushTool::default()));
        tools.register(Box::new(MoveTool::default()));
        let layout = EditorLayout::new(size.width as f32, size.height as f32);
        let vello_pass =
            match VelloPass::new(surface.gpu(), surface.format(), (size.width, size.height)) {
                Ok(p) => {
                    println!(
                        "[{:>6}ms] M11: VelloPass initialized ({}×{} intermediate)",
                        self.handler.elapsed_ms(),
                        size.width,
                        size.height
                    );
                    p
                }
                Err(e) => {
                    // Pass init failure is fatal here — the demo's whole
                    // point is showing the editor over the canvas.
                    panic!("VelloPass::new failed: {e}");
                }
            };
        let vector_scene = VectorScene::new();
        let text_system = TextSystem::new();

        // Hero screen mode opt-in via env var (set ONCE at startup,
        // not per frame). When set, the editor's default 4-zone
        // chrome is replaced with the `02-editor-main` mockup
        // composition, and pointer/key events flow through the
        // ADR-0024 interaction pipeline.
        let hero_screen = if std::env::var("PH2D_HERO_SCREEN").as_deref() == Ok("1") {
            Some(HeroScreen::new(NodeId(1)).theme(theme))
        } else {
            None
        };

        self.window = Some(window);
        self.host = Some(host);
        self.gfx = Some(AppGfx {
            surface,
            renderer,
            sim,
            present,
            camera,
            asset_db,
            atlas_is_real,
            script,
            theme,
            zen,
            toasts,
            tools,
            layout,
            vello_pass,
            vector_scene,
            text_system,
            hero_screen,
            hero_arena: Bump::with_capacity(4096),
        });
        self.handler.on_lifecycle(Lifecycle::Foreground);
        self.handler.on_resize(size, scale);
        self.title_dirty = true;
        if let Some(host) = self.host.as_ref() {
            host.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => match self.handler.on_close_request() {
                CloseAction::Close => {
                    self.handler.on_lifecycle(Lifecycle::WillTerminate);
                    event_loop.exit();
                }
                CloseAction::Cancel => {}
            },

            WindowEvent::Resized(size) => {
                self.pending_resize = Some(WindowSize::new(size.width, size.height));
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(host) = &self.host {
                    host.scale.set(scale_factor as f32);
                    if let Some(gfx) = self.gfx.as_ref() {
                        self.pending_resize = Some(gfx.surface.size());
                    }
                }
            }

            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.last_pointer = (position.x as f32, position.y as f32);
                let evt = PointerEvent {
                    x: self.last_pointer.0,
                    y: self.last_pointer.1,
                    pressure: 1.0,
                    kind: PointerKind::Move,
                    source: PointerSource::Mouse,
                    button: ph2d_host::PointerButton::Primary,
                    timestamp_ns: Self::timestamp_ns(),
                };
                self.handler.on_pointer(evt);
                forward_to_hero(self.gfx.as_mut(), evt);
                // Drag-in-progress: forward pointer to active tool
                // panel hit-test → updates slider value continuously.
                if self.dragging.is_some() {
                    self.dispatch_panel_pointer(self.last_pointer.0, self.last_pointer.1, false);
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (x * 16.0, y * 16.0),
                    winit::event::MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
                };
                let evt = ph2d_host::WheelEvent {
                    x: self.last_pointer.0,
                    y: self.last_pointer.1,
                    delta_x: dx,
                    delta_y: dy,
                    modifiers: Self::convert_modifiers(self.modifiers),
                    timestamp_ns: Self::timestamp_ns(),
                };
                forward_wheel_to_hero(self.gfx.as_mut(), evt);
            }

            WindowEvent::MouseInput { state, button, .. } => {
                let kind = match state {
                    ElementState::Pressed => PointerKind::Down,
                    ElementState::Released => PointerKind::Up,
                };
                let mapped_button = match button {
                    winit::event::MouseButton::Left => ph2d_host::PointerButton::Primary,
                    winit::event::MouseButton::Right => ph2d_host::PointerButton::Secondary,
                    winit::event::MouseButton::Middle => ph2d_host::PointerButton::Middle,
                    _ => ph2d_host::PointerButton::Primary,
                };
                let evt = PointerEvent {
                    x: self.last_pointer.0,
                    y: self.last_pointer.1,
                    pressure: 1.0,
                    kind,
                    source: PointerSource::Mouse,
                    button: mapped_button,
                    timestamp_ns: Self::timestamp_ns(),
                };
                self.handler.on_pointer(evt);
                forward_to_hero(self.gfx.as_mut(), evt);
                match state {
                    ElementState::Pressed => {
                        // Mirror-sidebar chip takes precedence over the
                        // panel hit-test (different zone, no overlap).
                        let mut consumed = false;
                        if let Some(gfx) = self.gfx.as_mut()
                            && !gfx.zen.is_active()
                            && let Some(btn) = gfx.layout.mirror_button_rect()
                            && btn.contains(self.last_pointer.0, self.last_pointer.1)
                        {
                            gfx.layout.mirror_sidebar();
                            gfx.toasts.push(Toast::info(format!(
                                "Sidebar → {:?}",
                                gfx.layout.sidebar_side
                            )));
                            self.title_dirty = true;
                            consumed = true;
                        }
                        // Tool palette icon click — switch active tool.
                        if !consumed
                            && let Some(gfx) = self.gfx.as_mut()
                            && !gfx.zen.is_active()
                        {
                            let palette = gfx.layout.tool_palette_rects(gfx.tools.tools().len());
                            let hit_idx = palette
                                .iter()
                                .position(|r| r.contains(self.last_pointer.0, self.last_pointer.1));
                            if let Some(idx) = hit_idx {
                                let tool_id = gfx.tools.tools()[idx].id();
                                let tool_label = gfx.tools.tools()[idx].label().to_string();
                                if gfx.tools.set_active(&tool_id) {
                                    gfx.toasts.push(Toast::info(format!("Tool → {tool_label}")));
                                    self.title_dirty = true;
                                }
                                consumed = true;
                            }
                        }
                        if !consumed {
                            // Mouse down — start hit-test against active panel.
                            self.dispatch_panel_pointer(
                                self.last_pointer.0,
                                self.last_pointer.1,
                                true,
                            );
                        }
                    }
                    ElementState::Released => {
                        // End any drag-in-progress.
                        self.dragging = None;
                    }
                }
            }

            WindowEvent::KeyboardInput {
                event:
                    WinitKeyEvent {
                        physical_key,
                        state,
                        repeat,
                        ref text,
                        ..
                    },
                ..
            } => {
                let keycode = match physical_key {
                    winit::keyboard::PhysicalKey::Code(code) => code as u32,
                    winit::keyboard::PhysicalKey::Unidentified(_) => 0,
                };
                let kind = match (state, repeat) {
                    (ElementState::Pressed, false) => KeyKind::Down,
                    (ElementState::Pressed, true) => KeyKind::Repeat,
                    (ElementState::Released, _) => KeyKind::Up,
                };
                self.handler.on_key(KeyEvent {
                    keycode,
                    modifiers: Self::convert_modifiers(self.modifiers),
                    kind,
                    timestamp_ns: Self::timestamp_ns(),
                });

                // Hero pipeline (ADR-0024): translate winit's
                // physical KeyCode into the editor's KEY_* constants
                // and route to the focused widget.
                if state == ElementState::Pressed
                    && let PhysicalKey::Code(code) = physical_key
                    && let Some(editor_keycode) = winit_to_editor_keycode(code)
                {
                    forward_key_to_hero(
                        self.gfx.as_mut(),
                        KeyEvent {
                            keycode: editor_keycode,
                            modifiers: Self::convert_modifiers(self.modifiers),
                            kind,
                            timestamp_ns: Self::timestamp_ns(),
                        },
                    );
                }
                // Printable text from this key event (winit already
                // resolved layout + dead-keys + shift). Send each
                // char through the text-input dispatcher so focused
                // TextInput/NumberInput/Combobox buffers update.
                if state == ElementState::Pressed
                    && let Some(s) = text.as_ref()
                {
                    for ch in s.chars() {
                        if !ch.is_control() {
                            forward_text_to_hero(self.gfx.as_mut(), ch);
                        }
                    }
                }

                // M12 demo controls (only on key Down, no repeat).
                if matches!((state, repeat), (ElementState::Pressed, false))
                    && let PhysicalKey::Code(code) = physical_key
                {
                    self.handle_editor_key(code);
                }
            }

            WindowEvent::RedrawRequested => {
                self.render_frame();
            }

            _ => {}
        }
    }
}

fn log_input_event(elapsed_ms: u128, event: &InputEvent) {
    match event {
        InputEvent::GamepadButtonDown(b) => {
            println!(
                "[{:>6}ms] gamepad button down: {}",
                elapsed_ms,
                b.as_lua_key()
            );
        }
        InputEvent::GamepadButtonUp(b) => {
            println!(
                "[{:>6}ms] gamepad button up:   {}",
                elapsed_ms,
                b.as_lua_key()
            );
        }
        InputEvent::GamepadAxis { axis, value } => {
            // Spam-prone: log only when |value| > 0.25 to skip
            // dead-zone jitter.
            if value.abs() > 0.25 {
                println!(
                    "[{:>6}ms] gamepad axis {} = {:+.2}",
                    elapsed_ms,
                    axis.as_lua_key(),
                    value
                );
            }
        }
        InputEvent::Pencil(_) => {
            // No iPad shell yet; pencil events can't originate here.
        }
    }
}

/// Forward a pointer event to the hero screen's interaction
/// dispatcher when the hero is active. Drains emitted
/// [`WidgetEvent`]s into `HeroScreen::apply_event` (consumed events
/// drive hero-level state mutations) and logs unconsumed ones to
/// stderr for the developer to verify wiring.
fn forward_to_hero(gfx: Option<&mut AppGfx>, event: PointerEvent) {
    let Some(gfx) = gfx else { return };
    let Some(hero) = gfx.hero_screen.as_mut() else {
        return;
    };
    // Snapshot events before applying — apply_event may mutate hero,
    // but the events slice itself lives in the arena (immutable view).
    let snapshot: Vec<WidgetEvent> = hero.handle_pointer(event, &gfx.hero_arena).to_vec();
    for e in snapshot {
        // Eyedropper pick — read the rendered pixel at the click
        // position from vello_pass's intermediate texture and apply
        // it to the picker. Only the host can do this (the dispatch
        // has no GPU access); intercept before `apply_event`.
        if let WidgetEvent::EyedropperPick { parent, px, py } = e {
            if let Some([r, g, b, a]) = gfx.vello_pass.read_pixel(gfx.surface.gpu(), px, py) {
                hero.store
                    .set_blender_value(parent, ph2d_tokens::ColorValue::from_rgba8(r, g, b, a));
            }
            continue;
        }
        if !hero.apply_event(e) {
            eprintln!("[hero] unhandled event: {e:?}");
        }
    }
}

/// Forward a translated [`KeyEvent`] (with editor-canonical
/// `keycode` from [`winit_to_editor_keycode`]) into the hero
/// dispatcher so focused widgets see Tab/Enter/Backspace/arrows etc.
fn forward_key_to_hero(gfx: Option<&mut AppGfx>, event: KeyEvent) {
    let Some(gfx) = gfx else { return };
    let Some(hero) = gfx.hero_screen.as_mut() else {
        return;
    };
    let snapshot: Vec<WidgetEvent> = hero.handle_key(event, &gfx.hero_arena).to_vec();
    for e in snapshot {
        if !hero.apply_event(e) {
            eprintln!("[hero] unhandled key event: {e:?}");
        }
    }
}

/// Forward a wheel / trackpad scroll into the hero dispatcher.
/// Routes to whichever panel registered its rect under the cursor.
fn forward_wheel_to_hero(gfx: Option<&mut AppGfx>, event: ph2d_host::WheelEvent) {
    let Some(gfx) = gfx else { return };
    let Some(hero) = gfx.hero_screen.as_mut() else {
        return;
    };
    let _ = hero.handle_wheel(event, &gfx.hero_arena);
}

/// Forward a single printable character into the hero text-input
/// dispatcher (focused TextInput/NumberInput/Combobox buffer).
fn forward_text_to_hero(gfx: Option<&mut AppGfx>, ch: char) {
    let Some(gfx) = gfx else { return };
    let Some(hero) = gfx.hero_screen.as_mut() else {
        return;
    };
    let snapshot: Vec<WidgetEvent> = hero.handle_text_input(ch, &gfx.hero_arena).to_vec();
    for e in snapshot {
        if !hero.apply_event(e) {
            eprintln!("[hero] unhandled text-input event: {e:?}");
        }
    }
}

/// Map a winit [`KeyCode`] into the editor's canonical KEY_*
/// constants (the values `dispatch_key` matches against). Returns
/// `None` for keys the editor pipeline doesn't currently consume.
fn winit_to_editor_keycode(code: KeyCode) -> Option<u32> {
    use ph2d_editor::interaction::{
        KEY_ARROW_DOWN, KEY_ARROW_LEFT, KEY_ARROW_RIGHT, KEY_ARROW_UP, KEY_BACKSPACE, KEY_ENTER,
        KEY_ESCAPE, KEY_KEY_A, KEY_SPACE, KEY_TAB,
    };
    Some(match code {
        KeyCode::Tab => KEY_TAB,
        KeyCode::Enter | KeyCode::NumpadEnter => KEY_ENTER,
        KeyCode::Space => KEY_SPACE,
        KeyCode::Escape => KEY_ESCAPE,
        KeyCode::Backspace => KEY_BACKSPACE,
        KeyCode::ArrowUp => KEY_ARROW_UP,
        KeyCode::ArrowDown => KEY_ARROW_DOWN,
        KeyCode::ArrowLeft => KEY_ARROW_LEFT,
        KeyCode::ArrowRight => KEY_ARROW_RIGHT,
        KeyCode::KeyA => KEY_KEY_A,
        _ => return None,
    })
}

/// Resolve the editor theme from a name (typically read from the
/// `PH2D_THEME` env var), falling back to [`Theme::ForgeSdf`] for
/// missing/invalid values. Recognised names match `Theme::id()`
/// (`forge-sdf`, `paint-studio`, `sunstone`, `blueprint`).
fn resolve_theme(name: Option<&str>) -> Theme {
    match name {
        None => Theme::ForgeSdf,
        Some("forge-sdf") => Theme::ForgeSdf,
        Some("paint-studio") => Theme::PaintStudio,
        Some("sunstone") => Theme::Sunstone,
        Some("blueprint") => Theme::Blueprint,
        Some(other) => {
            eprintln!(
                "[ph2d] PH2D_THEME={other:?} not recognized; falling back to forge-sdf. Valid: forge-sdf, paint-studio, sunstone, blueprint."
            );
            Theme::ForgeSdf
        }
    }
}

fn parse_theme_env() -> Theme {
    resolve_theme(std::env::var("PH2D_THEME").ok().as_deref())
}

fn main() {
    install_panic_hook();
    let event_loop = EventLoop::new().expect("create EventLoop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    println!("PH2D desktop shell starting (1000 sprites; close window to exit)…");
    event_loop.run_app(&mut app).expect("event loop crashed");
    println!("PH2D desktop shell exited cleanly.");
}

#[cfg(test)]
mod theme_env_tests {
    use super::*;

    #[test]
    fn unset_defaults_to_forge_sdf() {
        assert_eq!(resolve_theme(None), Theme::ForgeSdf);
    }

    #[test]
    fn known_names_resolve() {
        assert_eq!(resolve_theme(Some("paint-studio")), Theme::PaintStudio);
        assert_eq!(resolve_theme(Some("sunstone")), Theme::Sunstone);
        assert_eq!(resolve_theme(Some("blueprint")), Theme::Blueprint);
        assert_eq!(resolve_theme(Some("forge-sdf")), Theme::ForgeSdf);
    }

    #[test]
    fn unknown_falls_back_to_default() {
        assert_eq!(resolve_theme(Some("dracula")), Theme::ForgeSdf);
    }
}

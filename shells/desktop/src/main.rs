#![forbid(unsafe_code)]
//! Desktop shell — winit 0.30 + wgpu + ECS + sprite render + M6+M7+M12.
//!
//! Run with: `cargo run -p ph2d-host-desktop`
//!
//! Layered subsystems (each gated to keep the demo bootable even if
//! one fails — never crash the shell over an integration-demo issue):
//! - **M5** SpriteRenderer + 1000-sprite Vogel spiral with bouncing motion
//! - **M6** AssetDb loads 16 real PNGs from `assets/sprites/` (auto-
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
//! Out of scope here:
//! - **M8** ph2d-input gilrs adapter — already in PR #13 (cascade);
//!   cleanly merges once that lands and integration rebases.
//! - **M11** Vello text/widget overlay — needs surface-sharing pass.

mod integration;

use ph2d_asset::AssetDb;
use ph2d_core::{FixedStep, Vec2, install_panic_hook, panic};
use ph2d_ecs::{Component, PresentWorld, SimComponent, SimWorld};
use ph2d_editor::floating_panel::selection_demo_panel;
use ph2d_editor::{FloatingPanel, Toast, ToastQueue, ZenMode};
use ph2d_gpu::{AcquireError, GpuContext, SurfaceContext};
use ph2d_host::{
    CloseAction, HostHandler, KeyEvent, KeyKind, Lifecycle, Modifiers, PlatformHost, PointerEvent,
    PointerKind, PointerSource, WindowSize,
};
use ph2d_render::{Camera2d, RenderInstance, Sprite, SpriteRenderer, TextureAtlas};
use ph2d_script::ScriptHost;
use ph2d_tokens::Theme;
use std::cell::Cell;
use std::sync::Arc;
use std::time::Instant;
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
    /// M6 — true when the atlas was composed from real PNGs (vs the
    /// procedural dummy fallback). Surfaced in window title.
    atlas_is_real: bool,
    /// M7 — Luau VM with placeholder script loaded. Per-frame gc_step
    /// keeps the GC budget warm; set/get bindings ready for follow-up
    /// gameplay work.
    script: Option<ScriptHost>,
    /// M12 editor data layer (no Vello paint until M11 surface share).
    theme: Theme,
    zen: ZenMode,
    toasts: ToastQueue,
    selection_panel: FloatingPanel,
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
    title_dirty: bool,
}

impl App {
    fn new() -> Self {
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
            title_dirty: true,
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

    /// M12 demo control router. Tab toggles ZenMode (debounced 30
    /// frames per `ZenMode::try_toggle`), KeyM flips theme Dark↔Light,
    /// KeyT pushes an info toast. Each action also pushes a toast so
    /// the queue actually changes during the demo (visible via title).
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
                gfx.theme = gfx.theme.toggle();
                gfx.toasts
                    .push(Toast::info(format!("Theme → {:?}", gfx.theme)));
                self.title_dirty = true;
            }
            KeyCode::KeyT => {
                gfx.toasts.push(Toast::info("Toast key (T) pressed"));
                self.title_dirty = true;
            }
            KeyCode::KeyP => {
                gfx.selection_panel.toggle_collapsed();
                gfx.toasts.push(Toast::info(format!(
                    "Selection panel → {}",
                    if gfx.selection_panel.collapsed {
                        "collapsed"
                    } else {
                        "open"
                    }
                )));
                self.title_dirty = true;
            }
            _ => {}
        }
    }

    /// Compose the M6 atlas from real PNGs. Generates fixtures on first
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
            selection_panel,
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
        if let Some(host) = script {
            if let Err(e) = host.gc_step() {
                eprintln!("M7 gc_step error: {e}");
            }
        }

        // Apply coalesced resize once per frame.
        if let Some(size) = self.pending_resize.take() {
            surface.resize(size);
            self.handler.on_resize(size, host.scale_factor());
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
        match surface.acquire_frame() {
            Ok(frame) => {
                renderer.render(
                    &frame,
                    present,
                    camera,
                    window_size,
                    wgpu::Color { r, g, b, a: 1.0 },
                );
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

        // M12 visibility: window title carries editor data-layer state
        // until M11 ships Vello widget paint. Refresh only when state
        // actually changes — winit set_title triggers a platform call.
        if self.title_dirty {
            let panel_state = if selection_panel.collapsed {
                "collapsed"
            } else {
                "open"
            };
            let title = format!(
                "PH2D — M5+M6+M7+M12 demo | sprites={SPRITE_COUNT} | atlas={} ({} assets) \
                 | script={} | theme={:?} | zen={} | toasts={} | panel={}",
                if *atlas_is_real { "PNG" } else { "dummy" },
                asset_db.len_assets(),
                if script.is_some() { "ok" } else { "off" },
                theme,
                if zen.is_active() { "on" } else { "off" },
                toasts.len(),
                panel_state,
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

        // M6: try to compose the atlas from real PNGs on disk.
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

        // M12: editor data layer. ZenMode/ToastQueue/FloatingPanel
        // model state only — Vello widget paint requires sharing the
        // wgpu Surface with the sprite pipeline (M11 follow-up). For
        // now state is surfaced via the window title.
        let theme = Theme::Dark;
        let zen = ZenMode::new();
        let mut toasts = ToastQueue::new();
        toasts.push(Toast::success("Editor data layer wired (M12)"));
        let selection_panel = selection_demo_panel();

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
            selection_panel,
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
                self.handler.on_pointer(PointerEvent {
                    x: self.last_pointer.0,
                    y: self.last_pointer.1,
                    pressure: 1.0,
                    kind: PointerKind::Move,
                    source: PointerSource::Mouse,
                    timestamp_ns: Self::timestamp_ns(),
                });
            }

            WindowEvent::MouseInput { state, .. } => {
                let kind = match state {
                    ElementState::Pressed => PointerKind::Down,
                    ElementState::Released => PointerKind::Up,
                };
                self.handler.on_pointer(PointerEvent {
                    x: self.last_pointer.0,
                    y: self.last_pointer.1,
                    pressure: 1.0,
                    kind,
                    source: PointerSource::Mouse,
                    timestamp_ns: Self::timestamp_ns(),
                });
            }

            WindowEvent::KeyboardInput {
                event:
                    WinitKeyEvent {
                        physical_key,
                        state,
                        repeat,
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

                // M12 demo controls (only on key Down, no repeat).
                if matches!((state, repeat), (ElementState::Pressed, false)) {
                    if let PhysicalKey::Code(code) = physical_key {
                        self.handle_editor_key(code);
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                self.render_frame();
            }

            _ => {}
        }
    }
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

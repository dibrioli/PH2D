#![forbid(unsafe_code)]
//! Desktop shell — winit 0.30 + wgpu + ECS + sprite render (M5).
//!
//! Run with: `cargo run -p ph2d-host-desktop`
//!
//! M5 scope adds (over M3):
//! - SimWorld + PresentWorld instantiated; 1000 sprites spawned at
//!   startup with deterministic Vogel-spiral positions and pseudo-
//!   random velocities (no PRNG dep — index hashing).
//! - Sim tick: bouncing motion at world boundary `[-5, 5]²`.
//! - Extract phase via `ph2d_ecs::extract!`: rebuilds RenderInstance
//!   set in PresentWorld each frame from `(Position, Sprite)` in
//!   SimWorld.
//! - SpriteRenderer drives a single instanced draw call (4 verts ×
//!   N instances) with explicit pipeline+bind-group layouts.
//!
//! M5 still does NOT have: Luau scripting, asset loading, input → ECS
//! routing, MCP. Those land in M7+.
//!
//! M5 perf-tools add-on (this branch):
//! - Live FPS + frame-time in the window title (gated to 250 ms).
//! - `S` key toggles a drop-shadow pass — extract emits a second
//!   `RenderInstance` per sprite tinted black and offset, doubling
//!   the instance count. Lets the user A/B the cost of 2× draw load.

use ph2d_core::{FixedStep, Vec2, install_panic_hook, panic};
use ph2d_ecs::{Component, PresentWorld, SimComponent, SimWorld};
use ph2d_gpu::{AcquireError, GpuContext, SurfaceContext};
use ph2d_host::{
    CloseAction, HostHandler, KeyEvent, KeyKind, Lifecycle, Modifiers, PlatformHost, PointerEvent,
    PointerKind, PointerSource, WindowSize,
};
use ph2d_render::{Camera2d, RenderInstance, Sprite, SpriteRenderer, TextureAtlas};
use std::cell::Cell;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent as WinitKeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowId};

const SPRITE_COUNT: u32 = 10_000;
/// Half-extent of the bouncing world in meters. Camera default has
/// `height_world = 10`, so [-5, 5] in Y is exactly the visible region;
/// X depends on aspect (narrower than visible at 4:3+).
const WORLD_HALF: f32 = 5.0;
/// Drop-shadow offset in world units (meters). Right + down (Y-up world,
/// so screen-down is -Y). Roughly 1/3 of a sprite size for visibility.
const SHADOW_OFFSET: [f32; 2] = [0.06, -0.06];
/// Drop-shadow tint — black with 50 % alpha. Multiplied with the
/// sampled texel: `(1, R, G, B) * (0, 0, 0, 0.5) = (0, 0, 0, 0.5)`,
/// which the alpha-blend pipeline then composites under the sprite.
const SHADOW_TINT: [f32; 4] = [0.0, 0.0, 0.0, 0.5];

/// Rolling FPS averager. Stores the last `CAP` per-frame `dt` values
/// (in seconds) and computes `1 / mean(dt)`. Avoids per-frame title
/// updates by gating emission to ~250 ms intervals (winit windows
/// flicker on rapid `set_title`).
struct FpsCounter {
    samples: VecDeque<f64>,
    last_emit: Instant,
}

impl FpsCounter {
    const CAP: usize = 120;
    const EMIT_INTERVAL_MS: u128 = 250;

    fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(Self::CAP),
            last_emit: Instant::now(),
        }
    }

    fn record(&mut self, dt_secs: f64) {
        if self.samples.len() == Self::CAP {
            self.samples.pop_front();
        }
        self.samples.push_back(dt_secs);
    }

    fn fps(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.samples.iter().sum();
        let mean = sum / self.samples.len() as f64;
        if mean > 0.0 { (1.0 / mean) as f32 } else { 0.0 }
    }

    /// Mean per-frame time in milliseconds.
    fn mean_ms(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.samples.iter().sum();
        ((sum / self.samples.len() as f64) * 1000.0) as f32
    }

    /// True at most every `EMIT_INTERVAL_MS`; flips the gate when so.
    fn should_emit(&mut self) -> bool {
        if self.last_emit.elapsed().as_millis() >= Self::EMIT_INTERVAL_MS {
            self.last_emit = Instant::now();
            true
        } else {
            false
        }
    }
}

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
    /// Toggle with `S`. When `true`, every sprite emits a second
    /// `RenderInstance` offset by `SHADOW_OFFSET` and tinted black —
    /// doubling the instance count, which is the perf knob the user
    /// wanted to compare against.
    shadow_enabled: bool,
    fps: FpsCounter,
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
            shadow_enabled: false,
            fps: FpsCounter::new(),
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
        } = gfx;
        let Some(host) = self.host.as_ref() else {
            return;
        };

        // Apply coalesced resize once per frame.
        if let Some(size) = self.pending_resize.take() {
            surface.resize(size);
            self.handler.on_resize(size, host.scale_factor());
        }

        // Drive fixed-step accumulator.
        let now = Instant::now();
        let wall_dt = now.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = now;
        self.fps.record(wall_dt);
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
        //
        // Shadow draw order: insert the shadow `RenderInstance` BEFORE
        // the sprite. The renderer iterates the present world in
        // archetype-storage order, which equals insertion order since
        // we `clear_entities` and re-spawn from scratch every frame
        // (no removals → no gaps). Shadow lands earlier in the
        // instance buffer and the alpha-blend pipeline composites the
        // sprite on top of it.
        let atlas = renderer.atlas();
        let shadow_on = self.shadow_enabled;
        let mut sim_q = sim.world_mut().query::<(&Position, &Sprite)>();
        ph2d_ecs::extract!(*sim => *present, |sim_w, present_w| {
            present_w.clear_entities();
            for (pos, spr) in sim_q.iter(sim_w) {
                let uv = atlas.dummy_uv(spr.atlas_index);
                if shadow_on {
                    present_w.spawn(RenderInstance {
                        world_pos: [pos.0.x + SHADOW_OFFSET[0], pos.0.y + SHADOW_OFFSET[1]],
                        size: spr.size,
                        atlas_uv: uv,
                        tint: SHADOW_TINT,
                    });
                }
                present_w.spawn(RenderInstance {
                    world_pos: [pos.0.x, pos.0.y],
                    size: spr.size,
                    atlas_uv: uv,
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

        // Update window title with current FPS — gated to ~250 ms to
        // avoid Cocoa/Win32 title-bar redraw storms.
        if self.fps.should_emit()
            && let Some(window) = self.window.as_ref()
        {
            let title = format!(
                "PH2D — {fps:>3.0} FPS ({ms:.2} ms) — {n} sprites — shadow {s} (S to toggle)",
                fps = self.fps.fps(),
                ms = self.fps.mean_ms(),
                n = SPRITE_COUNT,
                s = if self.shadow_enabled { "ON " } else { "off" },
            );
            window.set_title(&title);
        }

        host.request_redraw();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("PH2D — starting…")
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

        let atlas = TextureAtlas::dummy(surface.gpu());
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

        self.window = Some(window);
        self.host = Some(host);
        self.gfx = Some(AppGfx {
            surface,
            renderer,
            sim,
            present,
            camera,
        });
        self.handler.on_lifecycle(Lifecycle::Foreground);
        self.handler.on_resize(size, scale);
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
                // Shell-level shortcut: S toggles drop shadow. Handled
                // here (not in HostHandler) because it mutates app
                // state; HostHandler is intentionally read-only-ish.
                if matches!(
                    physical_key,
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyS)
                ) && state == ElementState::Pressed
                    && !repeat
                {
                    self.shadow_enabled = !self.shadow_enabled;
                    println!(
                        "[{:>6}ms] shadow toggle → {}",
                        self.handler.elapsed_ms(),
                        if self.shadow_enabled { "ON" } else { "off" }
                    );
                }

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
    println!(
        "PH2D desktop shell starting ({SPRITE_COUNT} sprites; press S to toggle drop shadow; close window to exit)…"
    );
    event_loop.run_app(&mut app).expect("event loop crashed");
    println!("PH2D desktop shell exited cleanly.");
}

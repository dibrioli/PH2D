#![forbid(unsafe_code)]
//! Desktop shell — winit 0.30 ApplicationHandler + wgpu integration (M3).
//!
//! Run with: `cargo run -p ph2d-host-desktop`
//!
//! M3 scope adds (over M1):
//! - Real GPU init (wgpu Instance/Adapter/Device/Queue via ph2d-gpu).
//! - Surface lifecycle per ADR-0020 (acquire_frame protocol).
//! - Animated clear color (sin/cos of fixed-step tick count) — proves
//!   the render path works end-to-end without yet needing a real
//!   render graph (M5).
//! - Resize coalescer: latest size kept in `pending_resize`; applied
//!   once per frame in `redraw_requested`, not per OS event (per
//!   LLM1 audit + plan anti-pattern "resize não-coalescido").
//! - Lifecycle::Background → SurfaceContext::on_background() (proactive
//!   transient release per ADR-0020).
//!
//! M3 still does NOT have: real ECS, sprites, input routing. Those
//! land in M4/M5/M8.

use ph2d_core::{FixedStep, install_panic_hook, panic};
use ph2d_gpu::{AcquireError, GpuContext, SurfaceContext};
use ph2d_host::{
    CloseAction, HostHandler, KeyEvent, KeyKind, Lifecycle, Modifiers, PlatformHost, PointerEvent,
    PointerKind, PointerSource, WindowSize,
};
use std::cell::Cell;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent as WinitKeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowId};

/// Implementation of [`PlatformHost`] backed by a winit `Window`.
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

/// Reference [`HostHandler`] that logs events; render is wired
/// separately in `App` because it needs the SurfaceContext.
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

struct App {
    window: Option<Arc<Window>>,
    host: Option<WinitHost>,
    surface: Option<SurfaceContext>,
    handler: LoggingHandler,
    fixed_step: FixedStep,
    last_frame: Instant,
    /// Coalesced pending resize. None when no resize event since last
    /// applied. Applied at most once per frame in `redraw_requested`.
    pending_resize: Option<WindowSize>,
    modifiers: ModifiersState,
    last_pointer: (f32, f32),
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            host: None,
            surface: None,
            handler: LoggingHandler::new(),
            fixed_step: FixedStep::default(),
            last_frame: Instant::now(),
            pending_resize: None,
            modifiers: ModifiersState::default(),
            last_pointer: (0.0, 0.0),
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

    /// Apply the coalesced pending resize (if any) and render the frame.
    fn render_frame(&mut self) {
        let Some(surface) = self.surface.as_mut() else {
            return;
        };
        let Some(host) = self.host.as_ref() else {
            return;
        };

        // Apply coalesced resize once per frame (not per event).
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

        // Animated clear color: hue sweeps with the simulated tick clock.
        let t = self.fixed_step.tick_count() as f64 * self.fixed_step.fixed_dt();
        let r = (t.sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        let g = ((t + 2.094).sin() * 0.5 + 0.5).clamp(0.0, 1.0); // 120° phase
        let b = ((t + 4.188).sin() * 0.5 + 0.5).clamp(0.0, 1.0); // 240° phase

        match surface.acquire_frame() {
            Ok(frame) => {
                frame.clear(wgpu::Color { r, g, b, a: 1.0 });
                // FrameTarget presents on Drop.
            }
            Err(AcquireError::AwaitingReconfigure) => {
                // Lost cascade — reconfigure and try next frame.
                surface.reconfigure_after_lost();
            }
            Err(AcquireError::Occluded) => {
                // Window minimized; skip frame.
            }
            Err(AcquireError::Timeout) => {
                // Skip frame; protocol counter cascades to Lost at 3.
            }
            Err(AcquireError::Other(s)) => {
                eprintln!("acquire_frame other error: {s}");
            }
        }

        // Schedule the next frame.
        host.request_redraw();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("PH2D — desktop shell (M3 — animated clear)")
            .with_inner_size(winit::dpi::LogicalSize::new(1024, 768));
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("create_window must succeed"),
        );
        let host = WinitHost::new(window.clone());
        let size = host.window_size();
        let scale = host.scale_factor();

        // GPU init. The wgpu Surface borrows from `window`, so we hand
        // it an `Arc<Window>` whose lifetime matches the SurfaceContext.
        let instance = GpuContext::default_instance();
        let raw_surface = instance
            .create_surface(window.clone())
            .expect("create_surface");
        let gpu = GpuContext::new(instance, Some(&raw_surface)).expect("GpuContext::new");
        let surface = SurfaceContext::new(gpu, raw_surface, size).expect("SurfaceContext::new");

        self.window = Some(window);
        self.host = Some(host);
        self.surface = Some(surface);
        self.handler.on_lifecycle(Lifecycle::Foreground);
        self.handler.on_resize(size, scale);
        // Kick off the render loop.
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
                // Coalesce: keep latest, apply in render_frame.
                self.pending_resize = Some(WindowSize::new(size.width, size.height));
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(host) = &self.host {
                    host.scale.set(scale_factor as f32);
                    if let Some(surface) = self.surface.as_ref() {
                        // Re-resize at the same physical px (DPI changed).
                        self.pending_resize = Some(surface.size());
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
    // Poll so we drive frames continuously (animated clear).
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    println!("PH2D desktop shell starting (animated clear; close window to exit)…");
    event_loop.run_app(&mut app).expect("event loop crashed");
    println!("PH2D desktop shell exited cleanly.");
}

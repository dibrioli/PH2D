#![forbid(unsafe_code)]
//! Desktop shell — winit 0.30 ApplicationHandler skeleton (M1).
//!
//! Run with: `cargo run -p ph2d-host-desktop`
//!
//! Opens a window, forwards resize / pointer / key / lifecycle events
//! to a [`HostHandler`] implementor. The reference handler here just
//! logs events; real subsystems (renderer, input router, etc.) will
//! be plugged in later marcos.
//!
//! M1 scope: prove that the trait shape works end-to-end. No render
//! yet (M3). No input → ECS routing (M8). No gizmos / editor (M12).

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
    /// Cached scale factor — updated whenever the OS notifies us via
    /// `WindowEvent::ScaleFactorChanged`. Avoids querying the window
    /// on every `scale_factor()` call (cheap but allocs a syscall on
    /// some platforms).
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

/// Reference [`HostHandler`] that logs every event and exits on
/// close. Real subsystems will replace this with their own
/// implementations.
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
        // Pointer floods. Only log Down/Up (not Move) to keep stdout sane.
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

/// Glue: holds the winit window + the core handler. winit's
/// `ApplicationHandler` trait dispatches OS events here; we translate
/// to ph2d_host event types and forward to `HostHandler`.
struct App {
    window: Option<Arc<Window>>,
    host: Option<WinitHost>,
    handler: LoggingHandler,
    modifiers: ModifiersState,
    last_pointer: (f32, f32),
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            host: None,
            handler: LoggingHandler::new(),
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
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("PH2D — desktop shell (M1 skeleton)")
            .with_inner_size(winit::dpi::LogicalSize::new(1024, 768));
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("create_window must succeed"),
        );
        let host = WinitHost::new(window.clone());
        let size = host.window_size();
        let scale = host.scale_factor();
        self.window = Some(window);
        self.host = Some(host);
        // Synthetic Foreground on first resume.
        self.handler.on_lifecycle(Lifecycle::Foreground);
        // Synthetic initial resize so the core knows the surface size.
        self.handler.on_resize(size, scale);
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
                if let Some(host) = &self.host {
                    self.handler.on_resize(
                        WindowSize::new(size.width, size.height),
                        host.scale_factor(),
                    );
                }
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(host) = &self.host {
                    host.scale.set(scale_factor as f32);
                    self.handler
                        .on_resize(host.window_size(), scale_factor as f32);
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
                // No render yet (M3). Just request next frame so the
                // window stays "alive" (visible scheduling activity).
                if let Some(host) = &self.host {
                    host.request_redraw();
                }
            }

            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("create EventLoop");
    // Poll = always ask for a new frame; Wait = block until next event.
    // Wait is right for an event-driven shell with no render yet — saves CPU.
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    println!("PH2D desktop shell starting (close window to exit)…");
    event_loop.run_app(&mut app).expect("event loop crashed");
    println!("PH2D desktop shell exited cleanly.");
}

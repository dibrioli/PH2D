//! [`WinitHost`] + [`LoggingHandler`] — the winit-side adapters that
//! satisfy the editor's `PlatformHost` / `HostHandler` traits.
//!
//! Extracted from [`main`] (Track B6). Self-contained: depends only
//! on winit + ph2d-host. The application orchestrator lives in main
//! and threads one instance of each through its lifecycle.

use ph2d_host::{
    CloseAction, HostHandler, KeyEvent, Lifecycle, PlatformHost, PointerEvent, PointerKind,
    WindowSize,
};
use std::cell::Cell;
use std::sync::Arc;
use std::time::Instant;
use winit::window::Window;

pub struct WinitHost {
    window: Arc<Window>,
    scale: Cell<f32>,
}

impl WinitHost {
    pub fn new(window: Arc<Window>) -> Self {
        let scale = window.scale_factor() as f32;
        Self {
            window,
            scale: Cell::new(scale),
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn scale(&self) -> &Cell<f32> {
        &self.scale
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

pub struct LoggingHandler {
    started_at: Instant,
}

impl LoggingHandler {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
    pub fn elapsed_ms(&self) -> u128 {
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
        println!("[{:>6}ms] close requested · Close", self.elapsed_ms());
        CloseAction::Close
    }
}

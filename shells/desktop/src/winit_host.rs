//! [`WinitHost`] + [`LoggingHandler`]: the winit adapters of `PlatformHost` / `HostHandler`.
//!
//! Self-contained (winit + ph2d-host); main threads one of each through its lifecycle.

use ph2d_host::{
    CloseAction, HostHandler, KeyEvent, Lifecycle, PlatformHost, PointerEvent, PointerKind,
    WindowSize,
};
use std::cell::Cell;
use std::sync::Arc;
use std::time::Instant;
use winit::window::Window;

/// Whether the dev input log (pointer Down/Up + key lines) prints — `PH2D_INPUT_LOG=1`. Off by
/// default: the per-click spam drowned the useful console output (perf `[frame]` lines, warnings);
/// the log itself stays one env var away for pen-input debugging. Cached after the first read.
fn input_log_on() -> bool {
    use std::sync::OnceLock;
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_INPUT_LOG").is_ok_and(|v| v != "0"))
}

pub struct WinitHost {
    window: Arc<Window>,
    scale: Cell<f32>,
}

impl WinitHost {
    pub fn new(window: Arc<Window>) -> Self {
        let scale = window.scale_factor() as f32;
        let m = window.current_monitor(); // B21.1: o ritmo em Mailbox é o do monitor
        ph2d_host::ritmo::publica_refresco(m.and_then(|m| m.refresh_rate_millihertz()));
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
        // Gated (PH2D_INPUT_LOG=1): pointer Down/Up spam retired from the default console
        // (Enio 2026-07-07, pós-caça de perf) — the log stays available for pen-input debugging.
        if input_log_on() && matches!(event.kind, PointerKind::Down | PointerKind::Up) {
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
        // Gated with the pointer log (same env): keystroke spam out of the default console.
        if input_log_on() {
            println!(
                "[{:>6}ms] key {:?} keycode={} mods={:?}",
                self.elapsed_ms(),
                event.kind,
                event.keycode,
                event.modifiers
            );
        }
    }
    fn on_close_request(&mut self) -> CloseAction {
        println!("[{:>6}ms] close requested · Close", self.elapsed_ms());
        CloseAction::Close
    }
}

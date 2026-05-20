#![forbid(unsafe_code)]
//! ph2d-host — PlatformHost trait + shell↔core event types (HR-1, §13).
//!
//! The core knows nothing about the OS. Every interaction with the
//! platform — window, input, IME, file pickers, lifecycle — passes
//! through this crate.
//!
//! Two traits compose the boundary:
//!
//! - [`PlatformHost`]: the shell exposes services to the core
//!   (request a redraw, query window size, ask for the keyboard).
//! - [`HostHandler`]: the core exposes a sink to the shell
//!   (receive resize, pointer, lifecycle events).
//!
//! The shell calls `HostHandler` methods on each event arrival; the
//! core calls `PlatformHost` methods when it needs the OS to do
//! something on its behalf.

pub mod events;
pub mod filter;

pub use events::{
    CloseAction, KeyEvent, KeyKind, Lifecycle, Modifiers, PointerButton, PointerEvent, PointerKind,
    PointerSource, WheelEvent, WindowSize,
};
pub use filter::ImageFilterMode;

/// Services exposed by the shell to the core.
///
/// Each shell (desktop with winit, iPad with SwiftUI, Android with
/// SurfaceView, web with WebGPU canvas) implements this trait. The
/// core takes a `&dyn PlatformHost` (or generic `H: PlatformHost`)
/// and never sees the underlying OS types.
pub trait PlatformHost {
    /// Schedule a redraw on the next vsync. Idempotent within a frame.
    fn request_redraw(&self);

    /// Window dimensions in **physical pixels**.
    /// Use [`PlatformHost::scale_factor`] to convert to logical units.
    fn window_size(&self) -> WindowSize;

    /// Logical-to-physical scale factor (HiDPI).
    /// macOS Retina, iPadOS, Android, and Windows DPI awareness all
    /// surface here. The core multiplies logical lengths by this to
    /// get pixels.
    fn scale_factor(&self) -> f32;
}

/// Core-side handler for events emitted by the shell.
///
/// The shell calls these methods inline on the main thread as events
/// arrive from the OS event loop. Implementations must be cheap and
/// non-blocking; queue work for later instead of doing it here.
pub trait HostHandler {
    /// Surface resized. `size` is in physical pixels; `scale_factor`
    /// is the new HiDPI factor (may also have changed if the window
    /// moved between displays).
    fn on_resize(&mut self, size: WindowSize, scale_factor: f32);

    /// App lifecycle changed (foreground / background / low memory /
    /// will terminate). Mobile-heavy; desktop emits primarily
    /// `Foreground` on launch and `WillTerminate` on close.
    fn on_lifecycle(&mut self, kind: Lifecycle);

    /// Pointer event (mouse, touch, or Apple Pencil).
    fn on_pointer(&mut self, event: PointerEvent);

    /// Keyboard event (down, up, repeat). IME composing is separate
    /// (will be added when text editing widget lands — §11.3).
    fn on_key(&mut self, event: KeyEvent);

    /// User asked to close the window. Returning [`CloseAction::Cancel`]
    /// keeps the window open (e.g., to prompt "save changes?").
    fn on_close_request(&mut self) -> CloseAction;

    /// External file is being dragged OVER the window — paths the OS
    /// expects us to know about for hit-target highlighting. Called
    /// once per `HoveredFile` event from the platform (winit emits one
    /// per file when multiple are dragged together).
    ///
    /// Default: no-op. Shells that show drag-preview overlays implement
    /// this to push the path list to the editor UI.
    fn on_file_hover(&mut self, _paths: &[std::path::PathBuf]) {}

    /// The OS-level drag operation was cancelled (user dragged out of
    /// the window or hit ESC). Default: no-op.
    fn on_file_hover_cancel(&mut self) {}

    /// User dropped one or more files into the window. Implementations
    /// process the paths (decode + spawn for image imports, route by
    /// extension for other asset types, etc.).
    ///
    /// Default: no-op. The desktop shell wires this to the M14.4e
    /// drag-and-drop image import path.
    fn on_file_drop(&mut self, _paths: &[std::path::PathBuf]) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference no-op host — minimal example of how a shell might
    /// implement [`PlatformHost`].
    struct NullHost {
        size: WindowSize,
        scale: f32,
    }

    impl PlatformHost for NullHost {
        fn request_redraw(&self) {}
        fn window_size(&self) -> WindowSize {
            self.size
        }
        fn scale_factor(&self) -> f32 {
            self.scale
        }
    }

    /// Reference handler that records the events it receives.
    #[derive(Default)]
    struct RecordingHandler {
        resizes: Vec<(WindowSize, f32)>,
        lifecycles: Vec<Lifecycle>,
        pointers: Vec<PointerEvent>,
        keys: Vec<KeyEvent>,
        close_requests: u32,
    }

    impl HostHandler for RecordingHandler {
        fn on_resize(&mut self, size: WindowSize, scale_factor: f32) {
            self.resizes.push((size, scale_factor));
        }
        fn on_lifecycle(&mut self, kind: Lifecycle) {
            self.lifecycles.push(kind);
        }
        fn on_pointer(&mut self, event: PointerEvent) {
            self.pointers.push(event);
        }
        fn on_key(&mut self, event: KeyEvent) {
            self.keys.push(event);
        }
        fn on_close_request(&mut self) -> CloseAction {
            self.close_requests += 1;
            CloseAction::Close
        }
    }

    #[test]
    fn null_host_returns_configured_values() {
        let host = NullHost {
            size: WindowSize::new(1920, 1080),
            scale: 2.0,
        };
        assert_eq!(host.window_size(), WindowSize::new(1920, 1080));
        assert!((host.scale_factor() - 2.0).abs() < f32::EPSILON);
        host.request_redraw();
    }

    #[test]
    fn recording_handler_captures_events() {
        let mut handler = RecordingHandler::default();
        handler.on_resize(WindowSize::new(800, 600), 1.5);
        handler.on_lifecycle(Lifecycle::Foreground);
        let p = PointerEvent {
            x: 10.0,
            y: 20.0,
            pressure: 0.5,
            kind: PointerKind::Down,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 12345,
        };
        handler.on_pointer(p);
        let action = handler.on_close_request();
        assert_eq!(handler.resizes.len(), 1);
        assert_eq!(handler.resizes[0].0, WindowSize::new(800, 600));
        assert_eq!(handler.lifecycles.len(), 1);
        assert_eq!(handler.pointers.len(), 1);
        assert_eq!(handler.close_requests, 1);
        assert_eq!(action, CloseAction::Close);
    }
}

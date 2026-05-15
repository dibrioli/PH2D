//! Live cursor-position queries used by `DroppedFile`.
//!
//! Extracted from [`main`] (Track B3). winit pauses its event stream
//! between drag-enter and drop, so we have to ask the OS directly
//! for the cursor location at the moment of drop. macOS has a clean
//! API (`CGEventGetLocation`); other platforms return `None` and the
//! caller falls back to the last cursor seen by winit.

pub fn live_cursor_in_window(window: &winit::window::Window) -> Option<(f32, f32)> {
    #[cfg(target_os = "macos")]
    {
        let cursor_pts = macos_cursor::screen_position_points()?;
        let scale = window.scale_factor() as f32;
        // CGEventGetLocation returns screen-space points (logical
        // pixels, top-left origin). Window's `outer_position` is
        // physical pixels; bring cursor to the same space first.
        let cursor_phys_x = (cursor_pts.0 as f32) * scale;
        let cursor_phys_y = (cursor_pts.1 as f32) * scale;
        let outer = window.outer_position().ok()?;
        // `outer_position` is the window frame's upper-left in screen
        // coords. `inner_size` differs from outer because of the
        // titlebar — winit gives no `inner_position` accessor, so
        // approximate by subtracting (outer_size - inner_size) /
        // 2 vertically (titlebar height). This matches macOS's
        // standard chrome: equal margin on all four sides except the
        // top which carries the titlebar.
        let outer_size = window.outer_size();
        let inner_size = window.inner_size();
        let titlebar_h = outer_size.height.saturating_sub(inner_size.height) as f32;
        let rel_x = cursor_phys_x - outer.x as f32;
        let rel_y = cursor_phys_y - outer.y as f32 - titlebar_h;
        Some((rel_x, rel_y))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window;
        None
    }
}

#[cfg(target_os = "macos")]
mod macos_cursor {
    use core_graphics::event::CGEvent;
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    /// Current cursor position in global screen points (logical
    /// pixels, top-left origin). Always available — does not depend
    /// on any cocoa event being in flight, which is what makes it
    /// the right tool for `DroppedFile` (where winit's event stream
    /// is paused).
    pub fn screen_position_points() -> Option<(f64, f64)> {
        let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
        let event = CGEvent::new(source).ok()?;
        let p = event.location();
        Some((p.x, p.y))
    }
}

//! Panel state: the [`BrushSettings`] snapshot the shell publishes each frame
//! (the painter-layers `set_current_brush` pattern — the wet knob table, the
//! flags and the Paper-slot arm all ride inside it).

use ph2d_tool_painter::BrushSettings;
use std::cell::RefCell;

thread_local! {
    /// Live snapshot published by the shell before each `paint`. `None` until
    /// the first push — the panel then paints the defaults (which equal the
    /// engine's boot, gate-pinned in the tool).
    static CURRENT: RefCell<Option<BrushSettings>> = const { RefCell::new(None) };
}

/// Zero-size panel state (everything lives in the store + the snapshot).
#[derive(Default)]
pub struct WetTuningPanelState;

/// Publish this frame's brush snapshot (the shell's bridge calls this beside
/// the painter-layers publish — same source, same cadence).
pub fn set_current_brush(brush: Option<BrushSettings>) {
    CURRENT.with(|c| *c.borrow_mut() = brush);
}

/// The snapshot the paint/event halves read. With nothing published yet the
/// panel paints a default tool's settings — the engine boot, which is what
/// an untouched section authors anyway.
pub(crate) fn current() -> BrushSettings {
    CURRENT
        .with(|c| *c.borrow())
        .unwrap_or_else(|| ph2d_tool_painter::PainterTool::default().brush_settings())
}

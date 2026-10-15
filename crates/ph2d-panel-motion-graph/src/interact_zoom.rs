//! **Anchored wheel zoom** — keep the cursor's graph point fixed while scaling. Split from
//! `interact` for the panel LOC cap; `super` is `interact`.

use crate::state::MotionGraphPanelState;
use ph2d_editor_core::interaction::GraphZoom;
use ph2d_editor_core::zones::Rect;

// Wheel-zoom tuning (canvas-interaction constants, not chrome tokens).
const ZOOM_WHEEL_DIV: f32 = 240.0; // LITERAL-PX-OK: wheel-notch → zoom-factor sensitivity divisor
const ZOOM_MIN: f32 = 0.2; // LITERAL-PX-OK: min graph zoom
const ZOOM_MAX: f32 = 2.5; // LITERAL-PX-OK: max graph zoom

/// Anchored zoom: keep the cursor's graph point fixed while scaling.
pub(super) fn apply_zoom(state: &mut MotionGraphPanelState, rect: Rect, z: GraphZoom) {
    let old = state.view.zoom;
    let factor = (z.delta / ZOOM_WHEEL_DIV).exp();
    let new = (old * factor).clamp(ZOOM_MIN, ZOOM_MAX); // CLAMP-OK: const bounds, min<max, non-NaN
    let f = new / old;
    // screen = base + pan + graph*zoom ⇒ hold `anchor` ⇒ pan' = (anchor-base)(1-f) + pan*f.
    state.view.pan_x = (z.anchor_x - rect.x) * (1.0 - f) + state.view.pan_x * f;
    state.view.pan_y = (z.anchor_y - rect.y) * (1.0 - f) + state.view.pan_y * f;
    state.view.zoom = new;
}

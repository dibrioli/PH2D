//! Reproduce the SHELL render-loop drain for the Falloff handle menu (step 4 of
//! the chain) against the real `PainterTool` + the real panel selection
//! thread-local — the one hop with no test coverage anywhere, and the only place
//! left for the "Vector does nothing" defect to hide after the chrome path
//! (`falloff_handle_menu_e2e`) and the engine (`ph2d-tool-painter`) both prove
//! green.
//!
//! The shell drain (shells/desktop/src/render_loop/mod.rs) is:
//! ```ignore
//! if let Some(handle) = hero.pending_falloff_point_handle.take()
//!     && let Some(id) = ph2d_panel_painter_layers::selected_falloff_point()
//!     && let Some(painter) = downcast::<PainterTool>(active)
//! { painter.set_brush_falloff_point_handle(id, handle); }
//! ```
//! We replicate it verbatim (minus the downcast — we hold a concrete tool) and
//! prove the right-clicked point ends up Vector.

use ph2d_panel_painter_layers::{
    selected_falloff_point, set_current_dock_shows_layers, set_selected_falloff_point,
};
use ph2d_tool_painter::{Falloff, HandleType, PainterTool, brush_falloff_weight_at};

#[test]
fn shell_drain_applies_vector_to_the_right_clicked_point() {
    // ── Arrange the brush like the user does ──
    let mut painter = PainterTool::default();
    painter.set_brush_falloff(Falloff::Custom.to_u8());
    let mid = painter.add_brush_falloff_point_at(0.5, 0.5).expect("added");
    painter.set_brush_falloff_point(mid, 0.5, 0.9); // drag off the line

    // The falloff graph lives in the Brush-properties view; the panel publishes
    // this each frame. With it `true`, `selected_falloff_point` is still readable
    // (the dock flag only gates `falloff_hit_test`, not the stored selection),
    // but set it to the real value for fidelity.
    set_current_dock_shows_layers(false);

    // ── Right-click on the point selected it (painter_falloff_open_point_menu). ──
    set_selected_falloff_point(Some(mid));

    // ── The chrome handler parked the Vector wire u8 (proven in the e2e test). ──
    let mut pending: Option<u8> = Some(HandleType::Vector.to_u8());

    // ── The SHELL DRAIN (step 4), replicated verbatim. ──
    if let Some(handle) = pending.take()
        && let Some(id) = selected_falloff_point()
    {
        painter.set_brush_falloff_point_handle(id, handle);
    }

    // ── The point the user right-clicked is now Vector. ──
    let s = painter.brush_settings();
    let handle = s.falloff_points[..s.falloff_len as usize]
        .iter()
        .find(|p| p.id == mid)
        .map(|p| p.handle);
    assert_eq!(
        handle,
        Some(HandleType::Vector),
        "drain did not set Vector on the right-clicked point (id {mid})"
    );

    // ── And the published curve the panel plots shows the sharp corner. ──
    let l = (brush_falloff_weight_at(&s, 0.5) - brush_falloff_weight_at(&s, 0.49)) / 0.01;
    let r = (brush_falloff_weight_at(&s, 0.51) - brush_falloff_weight_at(&s, 0.5)) / 0.01;
    assert!(
        (l - r).abs() > 1.0,
        "no slope discontinuity after the drain — curve stayed smooth ({l} vs {r})"
    );
}

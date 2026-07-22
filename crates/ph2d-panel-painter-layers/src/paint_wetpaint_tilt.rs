//! The Wet Paint **TILT dial** (doc 22 — a faithful copy of the model's
//! polar pad): an 8-ring × 12-spoke grid, a knob that SNAPS to it, a line
//! from the centre while active, and a toggle that flips without losing the
//! direction. Dragging anywhere on the pad moves the knob (and turns the
//! tilt on — the model's law); the drag rides the foundational
//! `InteractiveState::CurvePoint` dispatch and lands in
//! `event.rs`, which converts it to (ring, spoke) through
//! [`drag_to_ring_spoke`] — paint and event share that ONE conversion.

use crate::PaintCtx;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_circle, resolve, stroke_polyline};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Spacing};
use ph2d_tool_painter::BrushSettings;

/// Dial geometry — the model's 128×128 canvas with a 6 px margin.
const DIAL_PX: f32 = 128.0; // LITERAL-PX-OK: the model's tilt dial canvas size
const DIAL_PAD_PX: f32 = 6.0; // LITERAL-PX-OK: the model's dial margin (R = c - 6)
const KNOB_R_PX: f32 = 6.0; // LITERAL-PX-OK: the model's knob dot radius
const RINGS: u8 = 8;
const SPOKES: u8 = 12;
const RING_SEGS: usize = 36; // LITERAL-PX-OK: polyline resolution of a grid ring

/// Convert a normalized CurvePoint drag (`x`, `y` in `[0,1]`, y INVERTED —
/// top = 1.0) into the dial's snapped (ring, spoke). ONE house for the
/// math: the event drain calls this, and the paint places the knob with its
/// inverse, so the knob lands exactly under the finger.
pub(crate) fn drag_to_ring_spoke(x: f32, y: f32) -> (u8, u8) {
    let dx = f64::from(x) - 0.5;
    let dy = 0.5 - f64::from(y); // screen-down displacement from the centre
    let r_n = f64::from(0.5 - DIAL_PAD_PX / DIAL_PX); // the grid's R, normalized
    let d = (dx * dx + dy * dy).sqrt();
    let ring = (d / r_n * f64::from(RINGS)).round().clamp(0.0, 8.0) as u8; // LITERAL-PX-OK: 8.0 == RINGS (the clamp gate wants literal bounds)
    let spoke_step = 360.0 / f64::from(SPOKES); // LITERAL-PX-OK: degrees in a circle (math constant)
    let deg = dy.atan2(dx).to_degrees();
    let spoke = ((deg / spoke_step).round().rem_euclid(f64::from(SPOKES))) as u8;
    (ring, spoke)
}

/// The dial card: a "Tilt" toggle row + the polar pad. Returns the next `y`.
pub(crate) fn paint_tilt_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let y = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_WETPAINT_TILT_TOGGLE,
        "Tilt",
        brush.wet_tilt_on,
    );
    let side = DIAL_PX.min(content_w);
    let canvas = Rect::new(x + (content_w - side) * 0.5, y, side, side);
    let (cx, cy) = (canvas.x + side * 0.5, canvas.y + side * 0.5);
    let r_max = side * 0.5 - DIAL_PAD_PX * (side / DIAL_PX);
    let grid = resolve(ColorToken::Border, theme);
    let accent = resolve(ColorToken::Accent, theme);
    let idle = resolve(ColorToken::TextDisabled, theme);
    // The polar grid: 8 concentric rings × 12 spokes of 30°.
    let mut pts: Vec<(f32, f32)> = Vec::with_capacity(RING_SEGS + 1);
    for k in 1..=RINGS {
        let r = r_max * f32::from(k) / f32::from(RINGS);
        pts.clear();
        for s in 0..=RING_SEGS {
            let a = std::f32::consts::TAU * s as f32 / RING_SEGS as f32;
            pts.push((cx + a.cos() * r, cy + a.sin() * r));
        }
        stroke_polyline(ctx.scene, &pts, 1.0, grid);
    }
    for s in 0..SPOKES {
        let a = std::f32::consts::TAU * f32::from(s) / f32::from(SPOKES);
        stroke_polyline(
            ctx.scene,
            &[(cx, cy), (cx + a.cos() * r_max, cy + a.sin() * r_max)],
            1.0,
            grid,
        );
    }
    // The knob at its snapped grid position (+ the active line from centre).
    let a = std::f32::consts::TAU * f32::from(brush.wet_tilt_spoke) / f32::from(SPOKES);
    let kr = r_max * f32::from(brush.wet_tilt_ring) / f32::from(RINGS);
    let (kx, ky) = (cx + a.cos() * kr, cy + a.sin() * kr);
    if brush.wet_tilt_on && brush.wet_tilt_ring > 0 {
        let lw = 2.0;
        stroke_polyline(ctx.scene, &[(cx, cy), (kx, ky)], lw, accent);
    }
    let knob = if brush.wet_tilt_on { accent } else { idle };
    fill_circle(ctx.scene, kx, ky, KNOB_R_PX * (side / DIAL_PX), knob);
    // The whole pad is the drag surface (the model drags from anywhere);
    // the CurvePoint registration is refreshed per frame so the carried
    // canvas tracks panel resizes, and `populate.rs` registers the same id
    // (focusability — the wiring-parity law).
    ctx.host.store_mut().register(
        core_ids::PAINTER_WETPAINT_TILT_PAD,
        InteractiveState::CurvePoint {
            parent: core_ids::PAINTER_WETPAINT_TILT_PAD,
            channel: 0,
            index: 0,
            canvas,
        },
    );
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_WETPAINT_TILT_PAD, canvas);
    canvas.y + side + Spacing::Xs.px()
}

/// The `event.rs` drain arm's body (kept HERE so the event router stays at
/// its LOC cap): drain the stashed 2-D drag, snap it, forward ring+spoke.
pub(crate) fn forward_tilt_pad_drag(host: &mut dyn ph2d_editor_core::panel::PanelHostInternal) {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::tool::PanelEvent;
    if let Some((_p, _ch, _idx, x, y)) = host.store_mut().take_curve_point_drag() {
        let (ring, spoke) = drag_to_ring_spoke(x, y);
        host.bus_mut()
            .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                core_ids::PAINTER_WETPAINT_TILT_RING,
                f64::from(ring),
            )));
        host.bus_mut()
            .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                core_ids::PAINTER_WETPAINT_TILT_SPOKE,
                f64::from(spoke),
            )));
    }
}

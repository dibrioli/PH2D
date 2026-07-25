//! The interactive **Curve** editor (A1) — the params-panel row for a `field.remap`
//! Curve contour (and, later, `value.curve` / `force.curve`).
//!
//! A curve is a `ph2d-curve` serialized in a text param; here it becomes a graph with
//! **draggable control points** — the artist never sees the string. The handles reuse
//! the foundational `InteractiveState::CurvePoint` 2-D drag dispatch
//! (`interaction/dispatch/curve.rs`, the Painter's falloff editor): each point registers
//! a `CurvePoint` carrying the plotting canvas; on a drag the dispatch normalizes the
//! pointer to `(x, y)` in `[0, 1]` and stashes it, and [`drain_drag`] folds it back into
//! the curve + emits a [`MotionParamIntent::SetTextParam`]. `+`/`−` add/remove a point and
//! the interp button cycles the selected point's segment (Linear → Smooth → Hold).
//!
//! **Points never cross in x** (each drag clamps between its neighbours), so their order
//! is stable, no re-sort is needed, and `eval` always sees an ascending curve — the same
//! reason the handle indices stay valid frame to frame.

use crate::snapshot::{
    CurveRow, MAX_CURVE_POINTS, param_curve_add_id, param_curve_editor_id, param_curve_interp_id,
    param_curve_point_id, param_curve_remove_id,
};
use ph2d_a11y::NodeId;
use ph2d_curve::{Curve, Interp, Point, parse, serialize};
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::math::safe_clamp;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text, paint_text_centered, resolve, stroke_polyline,
    stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;
use std::cell::Cell;

const CANVAS_H: f32 = 96.0; // LITERAL-PX-OK: curve-graph canvas height (falloff-editor parity)
const HANDLE_R: f32 = 4.0; // LITERAL-PX-OK: control-point dot radius
const GRAB_R: f32 = 9.0; // LITERAL-PX-OK: half-size of a handle's pointer grab box
const STROKE_W: f32 = 1.5; // LITERAL-PX-OK: plotted-curve stroke width
const GRID_W: f32 = 1.0; // LITERAL-PX-OK: grid / border stroke width
const RING_W: f32 = 1.5; // LITERAL-PX-OK: handle ring width
const BTN_W: f32 = 22.0; // LITERAL-PX-OK: +/− button width
const INTERP_W: f32 = 64.0; // LITERAL-PX-OK: interp cycle-button width
const GRID_DIVS: usize = 4; // quarter grid
const MIN_DX: f32 = 0.001; // LITERAL-PX-OK: vao minimo em x NORMALIZADO (0..1) — separacao, nao medida

thread_local! {
    /// `(slot, point index)` of the last-grabbed handle — the target of `−` / interp
    /// and the accent highlight. Panel-scoped (mirror of the falloff editor's selection).
    static SELECTED: Cell<Option<(usize, usize)>> = const { Cell::new(None) };
}

fn selected_for(slot: usize) -> Option<usize> {
    SELECTED
        .with(|s| s.get())
        .and_then(|(sl, p)| (sl == slot).then_some(p))
}

/// Parse the row's serialized curve, else the identity diagonal — a fresh Curve
/// contour opens on something draggable, never an empty box.
fn working(value: &str) -> Curve {
    match parse(value) {
        Some(c) if c.points.len() >= 2 => c,
        _ => Curve::identity(),
    }
}

/// One control point's store-registration data (the `CurvePoint` state the paint pass
/// hands back for the caller's mutable-store pass — `lib.rs` Phase C — to apply).
pub(crate) struct CurveWidgets {
    pub points: Vec<(NodeId, NodeId, u8, Rect)>, // (handle id, parent, index, canvas)
    pub buttons: Vec<NodeId>,
}

impl CurveWidgets {
    pub(crate) fn new() -> Self {
        Self {
            points: Vec::new(),
            buttons: Vec::new(),
        }
    }
}

/// Paint one Curve row: header (label + interp + `+`/`−`) then the graph with draggable
/// handles. Registers hit rects (it holds `hit_index`) + draws; the `CurvePoint`/`Button`
/// STORE states go into `out` for the caller's mutable-store pass. Returns the used height.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_curve_row(
    row: &CurveRow,
    slot: usize,
    x: f32,
    w: f32,
    y: f32,
    label_font: f32,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    out: &mut CurveWidgets,
) -> f32 {
    let gap = Spacing::Xs.px();
    let curve = working(&row.value);
    let n = curve.points.len().min(MAX_CURVE_POINTS);
    let sel = selected_for(slot).filter(|&p| p < n);

    // ── Header: label (left) + interp / + / − (right) ──
    paint_text(
        text_system,
        scene,
        &row.label,
        x,
        y + (ROW_H_PX - label_font) * 0.5,
        label_font,
        w - INTERP_W - BTN_W * 2.0 - gap * 3.0, // LITERAL-PX-OK: CONTAGEM (3 vaos entre os 4 elementos), nao medida
        resolve(ColorToken::Text2, theme),
    );
    let rem = Rect::new(x + w - BTN_W, y, BTN_W, ROW_H_PX);
    let add = Rect::new(rem.x - BTN_W - gap, y, BTN_W, ROW_H_PX);
    let interp = Rect::new(add.x - INTERP_W - gap, y, INTERP_W, ROW_H_PX);
    let interp_label = interp_name(
        curve
            .points
            .get(sel.unwrap_or(0))
            .map(|p| p.interp)
            .unwrap_or(Interp::Linear),
    );
    for (brect, label, id) in [
        (interp, interp_label, param_curve_interp_id(slot)),
        (add, "+", param_curve_add_id(slot)),
        (rem, "\u{2212}", param_curve_remove_id(slot)),
    ] {
        fill_rounded_rect(
            scene,
            brect,
            Radius::Sm.px(),
            resolve(ColorToken::Bg2, theme),
        );
        paint_text_centered(
            text_system,
            scene,
            label,
            brect,
            TypeToken::Base.px(),
            resolve(ColorToken::Text1, theme),
        );
        hit_index.register(id, brect);
        out.buttons.push(id);
    }
    let cy0 = y + ROW_H_PX + gap;

    // ── Canvas: bg + border + quarter grid ──
    let canvas = Rect::new(x, cy0, w.max(1.0), CANVAS_H);
    fill_rounded_rect(
        scene,
        canvas,
        Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );
    stroke_rounded_rect(
        scene,
        canvas,
        Radius::Sm.px(),
        GRID_W,
        resolve(ColorToken::TextDisabled, theme),
    );
    let grid = resolve(ColorToken::GridLine, theme);
    for i in 1..GRID_DIVS {
        let f = i as f32 / GRID_DIVS as f32;
        let gx = canvas.x + f * canvas.w;
        let gy = canvas.y + f * canvas.h;
        stroke_polyline(
            scene,
            &[(gx, canvas.y), (gx, canvas.y + canvas.h)],
            GRID_W,
            grid,
        );
        stroke_polyline(
            scene,
            &[(canvas.x, gy), (canvas.x + canvas.w, gy)],
            GRID_W,
            grid,
        );
    }

    // ── Plot eval(t) as a polyline ──
    let samples = (canvas.w * 0.5).clamp(48.0, 256.0) as usize; // LITERAL-PX-OK: sample clamp
    let mut poly: Vec<(f32, f32)> = Vec::with_capacity(samples + 1);
    for k in 0..=samples {
        let t = k as f32 / samples as f32;
        let v = curve.eval(t).clamp(0.0, 1.0);
        poly.push((canvas.x + t * canvas.w, canvas.y + (1.0 - v) * canvas.h));
    }
    stroke_polyline(scene, &poly, STROKE_W, resolve(ColorToken::Accent, theme));

    // ── Draggable handles ──
    let accent = resolve(ColorToken::Accent, theme);
    let ring = resolve(ColorToken::TextDisabled, theme);
    let fill = resolve(ColorToken::Text1, theme);
    for (i, p) in curve.points.iter().take(n).enumerate() {
        let id = param_curve_point_id(slot, i);
        let cx = canvas.x + p.x.clamp(0.0, 1.0) * canvas.w;
        let cy = canvas.y + (1.0 - p.y.clamp(0.0, 1.0)) * canvas.h;
        let grab = Rect::new(cx - GRAB_R, cy - GRAB_R, GRAB_R * 2.0, GRAB_R * 2.0);
        hit_index.register(id, grab);
        out.points
            .push((id, param_curve_editor_id(slot), i as u8, canvas));
        let ring_col = if sel == Some(i) { accent } else { ring };
        fill_circle(scene, cx, cy, HANDLE_R + RING_W, ring_col);
        fill_circle(scene, cx, cy, HANDLE_R, fill);
    }

    // Content height (header + gap + canvas); the caller adds the inter-row gap.
    (cy0 + CANVAS_H) - y
}

/// A dragged handle landed in the store's `curve_point_drag` slot — fold it into the
/// curve and emit the new serialized value. Returns `None` if the slot is empty (no
/// handle was dragged this frame). Sets the dragged point as SELECTED.
pub(crate) fn drain_drag(store: &mut WidgetStore, slot: usize, value: &str) -> Option<String> {
    let (parent, _channel, index, x, y) = store.take_curve_point_drag()?;
    if parent != param_curve_editor_id(slot) {
        // Not our editor — put it back is impossible (take), but the caller only reaches
        // here for the matching parent, so this is a defensive no-op.
        return None;
    }
    let mut curve = working(value);
    let i = index as usize;
    if i >= curve.points.len() {
        return None;
    }
    // Clamp x BETWEEN the neighbours so points never cross (order stays stable).
    let lo = if i > 0 {
        curve.points[i - 1].x + MIN_DX
    } else {
        0.0
    };
    let hi = if i + 1 < curve.points.len() {
        curve.points[i + 1].x - MIN_DX
    } else {
        1.0
    };
    // ⚠️ `safe_clamp`, não `f32::clamp`: os limites saem dos VIZINHOS (não são literais), então
    // uma curva degenerada pode entregá-los invertidos — e `f32::clamp` entra em pânico com
    // `min > max`. O `lo.min(hi), hi.max(lo)` que morava aqui já tolerava a troca, mas deixava
    // NaN passar; o `safe_clamp` é a porta única que trata os DOIS (ph2d_editor_core::math).
    curve.points[i].x = safe_clamp(x, lo, hi);
    curve.points[i].y = y.clamp(0.0, 1.0);
    SELECTED.with(|s| s.set(Some((slot, i))));
    Some(serialize(&curve))
}

/// Insert a point at the midpoint of the widest x-gap (its y on the current curve, so the
/// shape does not jump), capped at [`MAX_CURVE_POINTS`]. Returns the new serialized value.
pub(crate) fn add_point(value: &str) -> String {
    let mut curve = working(value);
    if curve.points.len() >= MAX_CURVE_POINTS {
        return serialize(&curve);
    }
    // Widest gap.
    let mut best = (0usize, 0.0f32);
    for i in 0..curve.points.len().saturating_sub(1) {
        let gap = curve.points[i + 1].x - curve.points[i].x;
        if gap > best.1 {
            best = (i, gap);
        }
    }
    let (at, _) = best;
    let mx = (curve.points[at].x + curve.points[at + 1].x) * 0.5;
    let my = curve.eval(mx).clamp(0.0, 1.0);
    let interp = curve.points[at].interp;
    curve.points.insert(
        at + 1,
        Point {
            x: mx,
            y: my,
            interp,
        },
    );
    serialize(&curve)
}

/// Remove the selected point (else the last), keeping at least two. Returns the new value.
pub(crate) fn remove_point(value: &str, slot: usize) -> String {
    let mut curve = working(value);
    if curve.points.len() <= 2 {
        return serialize(&curve);
    }
    let idx = selected_for(slot)
        .filter(|&p| p < curve.points.len())
        .unwrap_or(curve.points.len() - 1);
    curve.points.remove(idx);
    SELECTED.with(|s| s.set(None));
    serialize(&curve)
}

/// Cycle the selected point's segment interp (Linear → Smooth → Hold → Linear). Returns
/// the new value.
pub(crate) fn cycle_interp(value: &str, slot: usize) -> String {
    let mut curve = working(value);
    let idx = selected_for(slot)
        .filter(|&p| p < curve.points.len())
        .unwrap_or(0);
    if let Some(p) = curve.points.get_mut(idx) {
        p.interp = match p.interp {
            Interp::Linear => Interp::Smooth,
            Interp::Smooth => Interp::Hold,
            Interp::Hold => Interp::Linear,
        };
    }
    serialize(&curve)
}

/// The English caption for a segment interp (the interp button label).
fn interp_name(interp: Interp) -> &'static str {
    match interp {
        Interp::Linear => "Linear",
        Interp::Smooth => "Smooth",
        Interp::Hold => "Hold",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::interaction::WidgetStore;

    #[test]
    fn working_falls_back_to_identity() {
        // Empty / garbage / a 0-point curve all open on the draggable diagonal.
        assert_eq!(working("").points.len(), 2);
        assert_eq!(working("nonsense").points.len(), 2);
        assert_eq!(working("c1").points.len(), 2);
        assert_eq!(working("c1 0:0:L 0.5:1:S 1:0:L").points.len(), 3);
    }

    #[test]
    fn add_point_lands_in_the_widest_gap_on_the_curve() {
        // Identity: points at 0 and 1. Add → midpoint 0.5, y = eval(0.5) = 0.5 (no jump).
        let c = parse(&add_point("c1 0:0:L 1:1:L")).unwrap();
        assert_eq!(c.points.len(), 3);
        assert!((c.points[1].x - 0.5).abs() < 1e-6);
        assert!((c.points[1].y - 0.5).abs() < 1e-6);
    }

    #[test]
    fn add_point_stops_at_the_cap() {
        let mut v = "c1 0:0:L 1:1:L".to_string();
        for _ in 0..MAX_CURVE_POINTS + 4 {
            v = add_point(&v);
        }
        assert_eq!(parse(&v).unwrap().points.len(), MAX_CURVE_POINTS);
    }

    #[test]
    fn remove_keeps_at_least_two() {
        assert_eq!(
            parse(&remove_point("c1 0:0:L 1:1:L", 0))
                .unwrap()
                .points
                .len(),
            2
        );
        // With 3, the selected one goes.
        SELECTED.with(|s| s.set(Some((0, 1))));
        let c = parse(&remove_point("c1 0:0:L 0.5:1:L 1:0:L", 0)).unwrap();
        assert_eq!(c.points.len(), 2);
    }

    #[test]
    fn cycle_interp_advances_linear_smooth_hold() {
        SELECTED.with(|s| s.set(Some((0, 0))));
        let v = cycle_interp("c1 0:0:L 1:1:L", 0);
        assert_eq!(parse(&v).unwrap().points[0].interp, Interp::Smooth);
        let v = cycle_interp(&v, 0);
        assert_eq!(parse(&v).unwrap().points[0].interp, Interp::Hold);
        let v = cycle_interp(&v, 0);
        assert_eq!(parse(&v).unwrap().points[0].interp, Interp::Linear);
    }

    #[test]
    fn drain_drag_folds_the_point_and_never_lets_it_cross() {
        let slot = 0;
        let mut store = WidgetStore::with_capacity(2);
        // Drag the MIDDLE point (index 1) far right (x=2.0) + up (y=0.9). It must clamp
        // strictly below its right neighbour's x — points never cross.
        store.set_curve_point_drag(param_curve_editor_id(slot), 0, 1, 2.0, 0.9);
        let c = parse(&drain_drag(&mut store, slot, "c1 0:0:L 0.5:0.5:L 1:1:L").unwrap()).unwrap();
        assert!((c.points[1].y - 0.9).abs() < 1e-6, "y folded");
        assert!(
            c.points[0].x < c.points[1].x && c.points[1].x < c.points[2].x,
            "x order preserved: {:?}",
            c.points.iter().map(|p| p.x).collect::<Vec<_>>()
        );
        // The slot is drained (a second call sees nothing).
        assert!(store.take_curve_point_drag().is_none());
    }
}

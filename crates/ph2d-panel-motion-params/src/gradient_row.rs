//! The interactive **Gradient** editor (doc 85) — the params-panel row for a
//! `motion.color_ramp` Custom gradient (and, later, any multi-stop colour param).
//!
//! The colour sibling of [`crate::curve_row`]: a gradient is a `ph2d_color::ColorRamp`
//! serialized in a text param, drawn here as a **gradient bar** with **draggable position
//! markers** (the SAME foundational `InteractiveState::CurvePoint` x-drag the curve editor
//! uses — only the x is folded, the y ignored) + a **per-stop OKLCH swatch** (the canonical
//! colour UI, `register_picker_swatch` — the shell reads the pick back into the string, the
//! mirror of [`crate::snapshot::ColorRow`]). `+`/`−` add/remove a stop and the interp button
//! cycles the ramp's global interpolation. The artist never sees the string.
//!
//! **Markers never cross in position** (each drag clamps between its neighbours), so their
//! order is stable and the marker↔swatch indices stay valid frame to frame — exactly the
//! curve editor's reason for the same clamp.

use crate::snapshot::{
    GradientRow, MAX_GRADIENT_STOPS, param_grad_add_id, param_grad_editor_id, param_grad_interp_id,
    param_grad_preset_id, param_grad_remove_id, param_grad_stop_id, param_grad_swatch_id,
};
use ph2d_a11y::NodeId;
use ph2d_color::srgb::linear_to_srgb_byte;
use ph2d_color::{
    ColorRamp, GradientPreset, RampInterp, RampStop, parse_gradient, serialize_gradient,
};
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::math::safe_clamp;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text, paint_text_centered, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Color, VectorScene};
use std::cell::Cell;

const BAR_H: f32 = 24.0; // LITERAL-PX-OK: gradient-bar height
const SWATCH_H: f32 = 18.0; // LITERAL-PX-OK: per-stop swatch strip height
const PRESET_H: f32 = 14.0; // LITERAL-PX-OK: preset seed-chip strip height
const MARKER_R: f32 = 5.0; // LITERAL-PX-OK: position-marker dot radius
const GRAB_R: f32 = 9.0; // LITERAL-PX-OK: half-size of a marker's pointer grab box
const GRID_W: f32 = 1.0; // LITERAL-PX-OK: border / ring stroke width
const RING_W: f32 = 1.5; // LITERAL-PX-OK: selected-stop ring width
const BTN_W: f32 = 22.0; // LITERAL-PX-OK: +/− button width
const INTERP_W: f32 = 64.0; // LITERAL-PX-OK: interp cycle-button width
const MIN_DPOS: f32 = 0.001; // LITERAL-PX-OK: min position gap between stops (NORMALIZED 0..1)

thread_local! {
    /// `(slot, stop index)` of the last-grabbed marker — the target of `−` and the
    /// accent highlight. Panel-scoped (mirror of the curve editor's selection).
    static SELECTED: Cell<Option<(usize, usize)>> = const { Cell::new(None) };
}

fn selected_for(slot: usize) -> Option<usize> {
    SELECTED
        .with(|s| s.get())
        .and_then(|(sl, p)| (sl == slot).then_some(p))
}

/// Parse the row's serialized gradient, else the default black→white ramp — a fresh
/// Gradient opens on something draggable, never an empty box.
fn working(value: &str) -> ColorRamp {
    parse_gradient(value).unwrap_or_default()
}

/// The sRGB8 (straight) colour of a ramp stop — what the swatch paints + the picker
/// seeds with. RGB through the linear→sRGB transfer (the wire is linear), alpha 255
/// (stops are opaque, doc 85).
fn stop_srgb(stop: &RampStop) -> [u8; 4] {
    [
        linear_to_srgb_byte(stop.color[0]),
        linear_to_srgb_byte(stop.color[1]),
        linear_to_srgb_byte(stop.color[2]),
        255,
    ]
}

/// One Gradient row's store-registration data (the caller's Phase B/C mutable-store pass
/// applies these — swatches become picker swatches, markers become `CurvePoint` handles).
pub(crate) struct GradientWidgets {
    /// `(marker id, parent, index, canvas)` — registered as `CurvePoint` (position drag).
    pub markers: Vec<(NodeId, NodeId, u8, Rect)>,
    /// `(swatch id, srgb)` — registered as a picker swatch + seeded with the colour.
    pub swatches: Vec<(NodeId, [u8; 4])>,
    /// `+`/`−`/interp buttons + the preset-seed chips — all registered as plain buttons.
    pub buttons: Vec<NodeId>,
}

impl GradientWidgets {
    pub(crate) fn new() -> Self {
        Self {
            markers: Vec::new(),
            swatches: Vec::new(),
            buttons: Vec::new(),
        }
    }
}

/// Paint a gradient bar into `rect` as adjacent vertical strips of `ramp.eval(t)` (linear→sRGB).
/// Shared by the main editor bar and the small preset-seed chips — the colours are DATA (the
/// swatch precedent), not tokens.
fn paint_gradient_bar(scene: &mut VectorScene, rect: Rect, ramp: &ColorRamp) {
    let strips = (rect.w * 0.5).clamp(8.0, 128.0) as usize; // LITERAL-PX-OK: strip-count clamp
    for k in 0..strips {
        let t = (k as f32 + 0.5) / strips as f32;
        let [r, g, b, _a] = ramp.eval(t);
        // ⚠️ Uma linha só, e não é estilo: o gate casa o marcador **na linha da chamada**, e numa
        // chamada quebrada em cinco linhas ele acabava no `);` — ou seja, em lugar nenhum. É a
        // forma que o irmão `paint_ramp_widget.rs` já usa (integração 2026-07-30).
        let (rb, gb, bb) = (
            linear_to_srgb_byte(r),
            linear_to_srgb_byte(g),
            linear_to_srgb_byte(b),
        );
        let col = Color::from_rgba8(rb, gb, bb, 255); // LITERAL-COLOR-OK: the ramp's OWN colour is data (the swatch precedent), not a token
        let sx = rect.x + (k as f32 / strips as f32) * rect.w;
        let sw = rect.w / strips as f32 + 1.0; // +1: overlap so no seam gap between strips
        fill_rounded_rect(scene, Rect::new(sx, rect.y, sw, rect.h), 0.0, col);
    }
}

/// Paint one Gradient row: header (label + interp + `+`/`−`), then the gradient bar with
/// draggable position markers, then the per-stop swatch strip. Registers hit rects + draws;
/// the `CurvePoint`/`Button`/picker-swatch STORE states ride back in `out` for Phase B/C.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_gradient_row(
    row: &GradientRow,
    slot: usize,
    x: f32,
    w: f32,
    y: f32,
    label_font: f32,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    out: &mut GradientWidgets,
) -> f32 {
    let gap = Spacing::Xs.px();
    let ramp = working(&row.value);
    let n = ramp.len().min(MAX_GRADIENT_STOPS);
    let sel = selected_for(slot).filter(|&p| p < n);

    // ── Header: label (left) + interp / + / − (right) ──
    paint_text(
        text_system,
        scene,
        &row.label,
        x,
        y + (ROW_H_PX - label_font) * 0.5,
        label_font,
        w - INTERP_W - BTN_W * 2.0 - gap * 3.0, // LITERAL-PX-OK: CONTAGEM (3 vaos entre 4 elementos), nao medida
        resolve(ColorToken::Text2, theme),
    );
    let rem = Rect::new(x + w - BTN_W, y, BTN_W, ROW_H_PX);
    let add = Rect::new(rem.x - BTN_W - gap, y, BTN_W, ROW_H_PX);
    let interp = Rect::new(add.x - INTERP_W - gap, y, INTERP_W, ROW_H_PX);
    for (brect, label, id) in [
        (interp, interp_name(ramp.interp), param_grad_interp_id(slot)),
        (add, "+", param_grad_add_id(slot)),
        (rem, "\u{2212}", param_grad_remove_id(slot)),
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

    // ── Gradient bar: adjacent vertical strips filled with eval(t) ──
    let by0 = y + ROW_H_PX + gap;
    let bar = Rect::new(x, by0, w.max(1.0), BAR_H);
    paint_gradient_bar(scene, bar, &ramp);
    stroke_rounded_rect(
        scene,
        bar,
        Radius::Sm.px(),
        GRID_W,
        resolve(ColorToken::TextDisabled, theme),
    );

    // ── Position markers on the bar bottom edge (draggable, x-only) ──
    let accent = resolve(ColorToken::Accent, theme);
    let ring = resolve(ColorToken::TextDisabled, theme);
    for (i, stop) in ramp.stops().iter().take(n).enumerate() {
        let id = param_grad_stop_id(slot, i);
        let cx = bar.x + stop.pos.clamp(0.0, 1.0) * bar.w;
        let cy = bar.y + bar.h;
        let grab = Rect::new(cx - GRAB_R, cy - GRAB_R, GRAB_R * 2.0, GRAB_R * 2.0);
        hit_index.register(id, grab);
        // The drag canvas is the BAR: the dispatch normalizes the pointer's x to
        // `pos ∈ [0,1]` across it (the y is ignored — a stop has only a position).
        out.markers
            .push((id, param_grad_editor_id(slot), i as u8, bar));
        let ring_col = if sel == Some(i) { accent } else { ring };
        fill_circle(scene, cx, cy, MARKER_R + RING_W, ring_col);
        let sr = stop_srgb(stop);
        fill_circle(
            scene,
            cx,
            cy,
            MARKER_R,
            Color::from_rgba8(sr[0], sr[1], sr[2], 255), // LITERAL-COLOR-OK: the stop's own colour is data
        );
    }

    // ── Per-stop swatch strip: one swatch per stop, evenly spaced, opens the OKLCH picker ──
    let sy0 = bar.y + bar.h + gap;
    let cell = (w / n.max(1) as f32).max(1.0);
    for (i, stop) in ramp.stops().iter().take(n).enumerate() {
        let sid = param_grad_swatch_id(row.name, i);
        let srgb = stop_srgb(stop);
        let pad = 2.0; // LITERAL-PX-OK: swatch inner padding within its cell
        let srect = Rect::new(
            x + i as f32 * cell + pad,
            sy0,
            (cell - pad * 2.0).max(1.0),
            SWATCH_H,
        );
        fill_rounded_rect(
            scene,
            srect,
            Radius::Sm.px(),
            Color::from_rgba8(srgb[0], srgb[1], srgb[2], 255), // LITERAL-COLOR-OK: the stop's own colour is data
        );
        stroke_rounded_rect(
            scene,
            srect,
            Radius::Sm.px(),
            if sel == Some(i) { RING_W } else { GRID_W },
            if sel == Some(i) {
                accent
            } else {
                resolve(ColorToken::TextDisabled, theme)
            },
        );
        hit_index.register(sid, srect);
        out.swatches.push((sid, srgb));
    }

    // ── Preset seeds: a row of mini gradient chips (Rainbow / Heat / Ice / Grayscale). Clicking
    // one LOADS its stops into the editable ramp (doc 85) — the presets appear IN the editor and
    // become draggable/recolourable, not a separate immutable mode. The colours ARE the identity
    // (rainbow / warm / cool / grey), so no label is needed. ──
    let py0 = sy0 + SWATCH_H + gap;
    let pcell = (w / GradientPreset::ALL.len() as f32).max(1.0);
    for (i, preset) in GradientPreset::ALL.into_iter().enumerate() {
        let id = param_grad_preset_id(slot, i);
        let pad = 2.0; // LITERAL-PX-OK: chip inner padding within its cell
        let prect = Rect::new(
            x + i as f32 * pcell + pad,
            py0,
            (pcell - pad * 2.0).max(1.0),
            PRESET_H,
        );
        paint_gradient_bar(scene, prect, &preset.ramp());
        stroke_rounded_rect(
            scene,
            prect,
            Radius::Sm.px(),
            GRID_W,
            resolve(ColorToken::TextDisabled, theme),
        );
        hit_index.register(id, prect);
        out.buttons.push(id);
    }

    // Content height (header + gap + bar + gap + swatches + gap + preset chips); the caller adds
    // the inter-row gap.
    (py0 + PRESET_H) - y
}

/// A dragged marker landed in the store's `curve_point_drag` slot — fold its x into the
/// stop's position and emit the new serialized gradient. `None` if the slot is empty or
/// not our editor. Sets the dragged stop as SELECTED.
pub(crate) fn drain_drag(store: &mut WidgetStore, slot: usize, value: &str) -> Option<String> {
    // The stash is a GLOBAL channel: the ownership question is part of the call, so a drag that
    // belongs to another panel is LEFT for it (a `take` would be irreversible — see
    // `WidgetStore::take_curve_point_drag_if`).
    let (_parent, _channel, index, x, _y) =
        store.take_curve_point_drag_if(|p| p == param_grad_editor_id(slot))?;
    let mut ramp = working(value);
    let i = index as usize;
    if i >= ramp.len() {
        return None;
    }
    // Clamp the position BETWEEN the neighbours so stops never cross (order + indices
    // stay stable — the curve editor's reasoning, exactly).
    let lo = if i > 0 {
        ramp.stops()[i - 1].pos + MIN_DPOS
    } else {
        0.0
    };
    let hi = if i + 1 < ramp.len() {
        ramp.stops()[i + 1].pos - MIN_DPOS
    } else {
        1.0
    };
    // `safe_clamp` (not `f32::clamp`): the bounds come from the NEIGHBOURS, so a
    // degenerate ramp could hand them inverted — the curve editor's same guard.
    ramp.set_position(i, safe_clamp(x, lo, hi));
    SELECTED.with(|s| s.set(Some((slot, i))));
    Some(serialize_gradient(&ramp))
}

/// Insert a stop at the midpoint of the widest position gap (its colour sampled off the
/// current ramp, so the gradient does not jump), capped at [`MAX_GRADIENT_STOPS`].
pub(crate) fn add_stop(value: &str) -> String {
    let mut ramp = working(value);
    if ramp.len() >= MAX_GRADIENT_STOPS {
        return serialize_gradient(&ramp);
    }
    let mut best = (0usize, 0.0f32);
    for i in 0..ramp.len().saturating_sub(1) {
        let gap = ramp.stops()[i + 1].pos - ramp.stops()[i].pos;
        if gap > best.1 {
            best = (i, gap);
        }
    }
    let (at, _) = best;
    let mid = (ramp.stops()[at].pos + ramp.stops()[at + 1].pos) * 0.5;
    let col = ramp.eval(mid);
    ramp.add_stop(RampStop::new(mid, col));
    serialize_gradient(&ramp)
}

/// Remove the selected stop (else the last), keeping at least two.
pub(crate) fn remove_stop(value: &str, slot: usize) -> String {
    let mut ramp = working(value);
    if ramp.len() <= 2 {
        return serialize_gradient(&ramp);
    }
    let idx = selected_for(slot)
        .filter(|&p| p < ramp.len())
        .unwrap_or(ramp.len() - 1);
    ramp.remove_stop(idx);
    SELECTED.with(|s| s.set(None));
    serialize_gradient(&ramp)
}

/// How many preset seed chips the editor offers (Rainbow / Heat / Ice / Grayscale).
pub(crate) const PRESET_COUNT: usize = GradientPreset::ALL.len();

/// The `i`-th preset (doc 85) serialized — what clicking its chip LOADS into the ramp.
pub(crate) fn preset_gradient(i: usize) -> String {
    serialize_gradient(&GradientPreset::ALL[i.min(PRESET_COUNT - 1)].ramp())
}

/// Cycle the ramp's global interpolation (Linear → Ease → Constant → Cardinal → B-Spline).
pub(crate) fn cycle_interp(value: &str, _slot: usize) -> String {
    let mut ramp = working(value);
    ramp.interp = match ramp.interp {
        RampInterp::Linear => RampInterp::Ease,
        RampInterp::Ease => RampInterp::Constant,
        RampInterp::Constant => RampInterp::Cardinal,
        RampInterp::Cardinal => RampInterp::BSpline,
        RampInterp::BSpline => RampInterp::Linear,
    };
    serialize_gradient(&ramp)
}

/// The English caption for the ramp interp (the interp button label).
fn interp_name(interp: RampInterp) -> &'static str {
    interp.name()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::interaction::WidgetStore;

    #[test]
    fn working_falls_back_to_the_default_ramp() {
        // Empty / garbage open on the draggable black→white default (2 stops).
        assert_eq!(working("").len(), 2);
        assert_eq!(working("nonsense").len(), 2);
        let three = working("g1 2 0:1,0,0 0.5:0,1,0 1:0,0,1");
        assert_eq!(three.len(), 3);
    }

    #[test]
    fn add_stop_lands_in_the_widest_gap_without_a_jump() {
        // Default black→white: stops at 0 and 1. Add → midpoint 0.5, colour = eval(0.5).
        let r = parse_gradient(&add_stop("g1 2 0:0,0,0 1:1,1,1")).unwrap();
        assert_eq!(r.len(), 3);
        assert!((r.stops()[1].pos - 0.5).abs() < 1e-6);
        assert!((r.stops()[1].color[0] - 0.5).abs() < 1e-6, "no colour jump");
    }

    #[test]
    fn add_stop_stops_at_the_cap() {
        let mut v = "g1 2 0:0,0,0 1:1,1,1".to_string();
        for _ in 0..MAX_GRADIENT_STOPS + 4 {
            v = add_stop(&v);
        }
        assert_eq!(parse_gradient(&v).unwrap().len(), MAX_GRADIENT_STOPS);
    }

    #[test]
    fn remove_keeps_at_least_two() {
        assert_eq!(
            parse_gradient(&remove_stop("g1 2 0:0,0,0 1:1,1,1", 0))
                .unwrap()
                .len(),
            2
        );
        SELECTED.with(|s| s.set(Some((0, 1))));
        let r = parse_gradient(&remove_stop("g1 2 0:0,0,0 0.5:0,1,0 1:1,1,1", 0)).unwrap();
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn cycle_interp_advances_and_wraps() {
        let v = cycle_interp("g1 2 0:0,0,0 1:1,1,1", 0); // Linear -> Ease (u8 2 -> 0)
        assert_eq!(parse_gradient(&v).unwrap().interp, RampInterp::Ease);
    }

    #[test]
    fn drain_drag_folds_the_position_and_never_lets_it_cross() {
        let slot = 0;
        let mut store = WidgetStore::with_capacity(2);
        // Drag the MIDDLE stop (index 1) far right (x=2.0). It must clamp strictly below
        // its right neighbour's position — stops never cross.
        store.set_curve_point_drag(param_grad_editor_id(slot), 0, 1, 2.0, 0.5);
        let r = parse_gradient(
            &drain_drag(&mut store, slot, "g1 2 0:1,0,0 0.5:0,1,0 1:0,0,1").unwrap(),
        )
        .unwrap();
        assert!(
            r.stops()[0].pos < r.stops()[1].pos && r.stops()[1].pos < r.stops()[2].pos,
            "position order preserved: {:?}",
            r.stops().iter().map(|s| s.pos).collect::<Vec<_>>()
        );
        assert!(
            store
                .take_curve_point_drag_if(|p| p == param_grad_editor_id(slot))
                .is_none(),
            "slot drained"
        );
    }
}

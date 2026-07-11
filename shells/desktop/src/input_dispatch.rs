// ph2d-loc-cap: Onda 2C multi-select dispatch + hit_map routing +
// click-vs-drag + group-translate snapshot capture grew this file
// past the HR-18 600-LOC cap (currently ~900 LOC). The MouseInput
// Down/Up arms are the bulk; the natural decomposition is to move
// each Down sub-path (modifier override / pivot tool / gizmo handle /
// canvas pick) and the Up resolver into siblings under
// `input_dispatch/`, parallel to the existing eyedropper / gizmo_drag
// / keyboard / protect_brush splits. That refactor lands as a
// follow-up to Onda 2 once the gizmo polish is locked.
//! Window-event dispatch — one method per `WindowEvent` variant.
//!
//! PR 9b of `docs/Migracao/2026-05-convention-by-discovery.md`:
//! `window_event()` in `main.rs` used to inline ~700 LOC across 13
//! `WindowEvent` arms, with single arms (CursorMoved 166 LOC,
//! MouseInput 325 LOC, KeyboardInput 83 LOC) violating HR-18's 200-LOC
//! per-function cap. This module hosts each arm as a `pub(crate) fn
//! on_<arm>(&mut self, …)` method on `App` — bodies are verbatim
//! former arms (no behaviour change), so smoke parity is
//! byte-for-byte.
//!
//! `window_event()` in `main.rs` becomes a 13-line dispatch table.
//! Adding a new arm: one method here + one line in the table.
//!
//! Rust allows `impl App` to be split across files within the same
//! crate as long as both files are reachable via `mod`. `App` is
//! `private` to `main.rs`, but submodules see their parent's private
//! items — so this `impl` block compiles without exposing any field
//! visibility upstream.

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::event_loop::ActiveEventLoop;

use ph2d_editor::Toast;
use ph2d_host::{
    CloseAction, HostHandler, Lifecycle, PlatformHost, PointerEvent, PointerKind, PointerSource,
    WindowSize,
};

use crate::App;
use crate::Transform;
use crate::forwarding::{
    cursor_over_hero_panel, forward_text_to_hero, forward_to_hero, forward_wheel_to_hero,
    resolve_live_entry,
};

// `impl App` is split across sibling modules (see the eyedropper /
// keyboard handlers) to keep this file under the HR-18 LOC cap.
mod eyedropper;
pub(crate) mod fill_drag;
mod gizmo_drag;
mod keyboard;
pub(crate) mod painter_canvas_input;
pub(crate) mod painter_falloff_input;
pub(crate) mod protect_brush;
mod vec_snap;

/// Deslocamento diagonal de um paste/duplicate, em pixels de tela (o zoom converte
/// para world) — a cópia não nasce exatamente sob o original.
const PASTE_OFFSET_PX: f64 = 12.0;

/// The z-ordered (back → front) indices of the closed paths in the pen's OBJECT
/// selection. Boolean and Make Compound both need this: the back-most is the base
/// and the front-most donates the style (Illustrator's Pathfinder).
fn selected_closed_z(scene: &ph2d_vec_scene::VecScene, pen: &ph2d_vec_edit::PenTool) -> Vec<usize> {
    let mut zs: Vec<usize> = pen
        .selected_paths()
        .iter()
        .filter_map(|id| scene.paths().iter().position(|p| p.id == *id && p.closed))
        .collect();
    zs.sort_unstable();
    zs.dedup();
    zs
}

/// ADR-0108 Fase 1: apply a boolean `op` to the SELECTED closed regions of `scene`
/// (destructive — consumes the operands, inserts the result where the base sat,
/// records ONE undo step + selects the result). N-ary: `Subtract` keeps the
/// back-most and removes every path above it. A free fn so both the U/I/D hotkeys
/// ([`App::vec_boolean`]) and the panel's Boolean buttons (the render_loop drain,
/// where `AppGfx` is already destructured and the method isn't callable) can
/// invoke it with the decomposed refs. Logs + no-ops on < 2 selected closed
/// regions / empty result.
pub(crate) fn apply_vec_boolean(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &mut ph2d_vec_edit::PenTool,
    xforms: &ph2d_vec_scene::VecXforms,
    op: ph2d_vec_boolean::BoolOp,
) {
    let zs = selected_closed_z(scene, pen);
    if zs.len() < 2 {
        eprintln!("[ph2d-vec] boolean: selecione >= 2 regioes FECHADAS (Shift+clique)");
        return;
    }
    // ADR-0111: os operandos podem ter poses diferentes, e um resultado só vive num
    // frame. Assa cada um no MUNDO; o path novo nasce world-space e a entidade dele
    // nasce na identidade — a forma aparece exatamente onde as originais estavam.
    let operands: Vec<ph2d_vec_scene::VecPath> = zs
        .iter()
        .map(|&z| {
            let mut p = scene.paths()[z].clone();
            let x = ph2d_vec_scene::xform_of(xforms, p.id);
            ph2d_vec_scene::bake_xform(&mut p, &x);
            p
        })
        .collect();
    let refs: Vec<&ph2d_vec_scene::VecPath> = operands.iter().collect();
    let results = ph2d_vec_boolean::apply_many(&refs, op);
    if results.is_empty() {
        eprintln!("[ph2d-vec] boolean {op:?}: resultado vazio");
        return;
    }
    let pre = scene.clone(); // undo da booleana
    // A base é a de trás: o resultado ocupa a fatia de z dela (não salta pro topo).
    // Os operandos removidos estão todos em z >= `at`, então o índice segue válido.
    let at = zs[0];
    for p in &operands {
        scene.remove_path(p.id);
    }
    let new_ids: Vec<u64> = results
        .into_iter()
        .enumerate()
        .map(|(k, r)| scene.insert_path(at + k, r))
        .collect();
    history.push_undo(pre);
    pen.select_many(&new_ids);
    eprintln!("[ph2d-vec] boolean {op:?}: ok ({} path[s])", new_ids.len());
}

/// Make / Release Compound sobre a seleção. **Make** funde os paths fechados
/// selecionados num só (`EvenOdd` ⇒ um contorno dentro de outro vira buraco na
/// hora); **Release** devolve cada subpath do(s) selecionado(s) a path próprio.
/// Um passo de undo; re-seleciona o resultado. Free fn pelo mesmo motivo de
/// [`apply_vec_boolean`].
pub(crate) fn apply_vec_compound(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &mut ph2d_vec_edit::PenTool,
    make: bool,
) {
    let pre = scene.clone();
    if make {
        let ids: Vec<u64> = selected_closed_z(scene, pen)
            .iter()
            .map(|&z| scene.paths()[z].id)
            .collect();
        let Some(base) = scene.make_compound(&ids) else {
            eprintln!("[ph2d-vec] compound: selecione >= 2 regioes FECHADAS");
            return;
        };
        history.push_undo(pre);
        pen.select(Some(base));
        return;
    }
    let selected: Vec<u64> = pen.selected_paths().to_vec();
    let mut all: Vec<u64> = Vec::new();
    for id in &selected {
        let freed = scene.release_compound(*id);
        if !freed.is_empty() {
            all.push(*id);
            all.extend(freed);
        }
    }
    if all.is_empty() {
        eprintln!("[ph2d-vec] release: a selecao nao tem compound path");
        return;
    }
    history.push_undo(pre);
    pen.select_many(&all);
}

/// Set the selected path's fill rule (the panel's Non-Zero / Even-Odd segmented
/// row, shown only for compound paths). One undo step; no-op if unchanged.
pub(crate) fn apply_vec_fill_rule(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    even_odd: bool,
) {
    let rule = if even_odd {
        ph2d_vec_scene::FillRule::EvenOdd
    } else {
        ph2d_vec_scene::FillRule::NonZero
    };
    let pre = scene.clone();
    let Some(path) = pen.selected().and_then(|id| scene.path_mut(id)) else {
        return;
    };
    if path.fill_rule == rule {
        return;
    }
    path.fill_rule = rule;
    history.push_undo(pre);
}

/// Map a Vector-panel Boolean button `NodeId` to its op (`None` for any other
/// id). Pure — unit-tested; called from the render_loop drain to turn a
/// `ToolPanelEvent::Click` into a document boolean.
pub(crate) fn vec_bool_op_for_id(id: ph2d_editor::NodeId) -> Option<ph2d_vec_boolean::BoolOp> {
    use ph2d_vec_boolean::BoolOp;
    if id == ph2d_editor::ids::VECTOR_BOOL_UNION {
        Some(BoolOp::Union)
    } else if id == ph2d_editor::ids::VECTOR_BOOL_SUBTRACT {
        Some(BoolOp::Subtract)
    } else if id == ph2d_editor::ids::VECTOR_BOOL_INTERSECT {
        Some(BoolOp::Intersect)
    } else if id == ph2d_editor::ids::VECTOR_BOOL_EXCLUDE {
        Some(BoolOp::Exclude)
    } else {
        None
    }
}

/// Retype the Pen's SELECTED vertex (panel Vertex buttons), recording ONE undo
/// step iff it actually changed. Free fn (mirror of [`apply_vec_boolean`]) so the
/// render_loop drain can call it with the destructured shell refs.
pub(crate) fn apply_vec_vertex_kind(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &mut ph2d_vec_edit::PenTool,
    kind: ph2d_vec_scene::VertexKind,
) {
    let pre = scene.clone();
    if pen.set_selected_vertex_kind(scene, kind) {
        history.push_undo(pre);
    }
}

/// Delete the Pen's SELECTED vertex (panel "Delete Node" button / Delete key),
/// recording ONE undo step iff it removed anything. Free fn (mirror of
/// [`apply_vec_boolean`]) so the render_loop drain can call it with destructured
/// refs. Returns whether anything was deleted.
pub(crate) fn apply_vec_delete_vertex(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &mut ph2d_vec_edit::PenTool,
) -> bool {
    let pre = scene.clone();
    if pen.delete_selected_vertex(scene) {
        history.push_undo(pre);
        true
    } else {
        false
    }
}

/// Map a Vector-panel Vertex-type button `NodeId` to its `VertexKind` (`None` for
/// any other id). Pure — unit-tested; called from the render_loop drain.
pub(crate) fn vec_vertex_kind_for_id(
    id: ph2d_editor::NodeId,
) -> Option<ph2d_vec_scene::VertexKind> {
    use ph2d_vec_scene::VertexKind;
    if id == ph2d_editor::ids::VECTOR_VERT_CORNER {
        Some(VertexKind::Corner)
    } else if id == ph2d_editor::ids::VECTOR_VERT_SMOOTH {
        Some(VertexKind::Smooth)
    } else if id == ph2d_editor::ids::VECTOR_VERT_SYMMETRIC {
        Some(VertexKind::Symmetric)
    } else {
        None
    }
}

/// Duplicate the SELECTED path (panel "Duplicate" button), offsetting the clone
/// by `(dx, dy)` world-units so it's visible, and select the copy. Records ONE
/// undo step iff a path was cloned. Free fn (mirror of [`apply_vec_boolean`]) so
/// the render_loop drain can call it with the destructured shell refs.
pub(crate) fn apply_vec_duplicate(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &mut ph2d_vec_edit::PenTool,
    dx: f64,
    dy: f64,
) {
    let sel = pen.selected_paths().to_vec();
    if sel.is_empty() {
        eprintln!("[ph2d-vec] duplicate: nenhum path selecionado");
        return;
    }
    // Duplicar É copiar-e-colar: um caminho só, então a estrutura de grupo vem
    // junto e as duas rotas nunca divergem.
    let clip = scene.copy_paths(&sel);
    let pre = scene.clone();
    let new_ids = scene.paste_clip(&clip, dx, dy);
    if new_ids.is_empty() {
        return;
    }
    history.push_undo(pre);
    pen.select_many(&new_ids);
    eprintln!("[ph2d-vec] duplicate: {} path(s)", new_ids.len());
}

/// Restack the SELECTED path (panel Arrange z-order buttons), recording ONE undo
/// step iff the position changed. Free fn (mirror of [`apply_vec_boolean`]).
pub(crate) fn apply_vec_reorder(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    order: ph2d_vec_scene::ZOrder,
) {
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] arrange: nenhum path selecionado");
        return;
    };
    let pre = scene.clone();
    if scene.reorder_path(sel, order) {
        history.push_undo(pre);
    }
}

/// Map a Vector-panel Arrange z-order button `NodeId` to its [`ph2d_vec_scene::ZOrder`]
/// (`None` for any other id, incl. Duplicate). Pure — unit-tested; called from
/// the render_loop drain.
pub(crate) fn vec_reorder_for_id(id: ph2d_editor::NodeId) -> Option<ph2d_vec_scene::ZOrder> {
    use ph2d_vec_scene::ZOrder;
    if id == ph2d_editor::ids::VECTOR_ARRANGE_TO_BACK {
        Some(ZOrder::ToBack)
    } else if id == ph2d_editor::ids::VECTOR_ARRANGE_BACKWARD {
        Some(ZOrder::Lower)
    } else if id == ph2d_editor::ids::VECTOR_ARRANGE_FORWARD {
        Some(ZOrder::Raise)
    } else if id == ph2d_editor::ids::VECTOR_ARRANGE_TO_FRONT {
        Some(ZOrder::ToFront)
    } else {
        None
    }
}

/// Mirror the SELECTED path (panel Arrange Flip buttons), recording ONE undo step
/// iff it flipped. Free fn (mirror of [`apply_vec_boolean`]).
pub(crate) fn apply_vec_flip(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    axis: ph2d_vec_scene::FlipAxis,
) {
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] flip: nenhum path selecionado");
        return;
    };
    let pre = scene.clone();
    if scene.flip_path(sel, axis) {
        history.push_undo(pre);
    }
}

/// Map a Vector-panel Arrange Flip button `NodeId` to its [`ph2d_vec_scene::FlipAxis`]
/// (`None` for any other id). Pure — unit-tested; called from the render_loop drain.
pub(crate) fn vec_flip_for_id(id: ph2d_editor::NodeId) -> Option<ph2d_vec_scene::FlipAxis> {
    use ph2d_vec_scene::FlipAxis;
    if id == ph2d_editor::ids::VECTOR_ARRANGE_FLIP_H {
        Some(FlipAxis::Horizontal)
    } else if id == ph2d_editor::ids::VECTOR_ARRANGE_FLIP_V {
        Some(FlipAxis::Vertical)
    } else {
        None
    }
}

/// Rotate the SELECTED path 90° (panel Arrange Rotate buttons), recording ONE undo
/// step iff it rotated. Free fn (mirror of [`apply_vec_boolean`]).
pub(crate) fn apply_vec_rotate(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    dir: ph2d_vec_scene::Rotate90,
) {
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] rotate: nenhum path selecionado");
        return;
    };
    let pre = scene.clone();
    if scene.rotate_path(sel, dir) {
        history.push_undo(pre);
    }
}

/// Map a Vector-panel Arrange Rotate button `NodeId` to its [`ph2d_vec_scene::Rotate90`]
/// (`None` for any other id). Pure — unit-tested; called from the render_loop drain.
pub(crate) fn vec_rotate_for_id(id: ph2d_editor::NodeId) -> Option<ph2d_vec_scene::Rotate90> {
    use ph2d_vec_scene::Rotate90;
    if id == ph2d_editor::ids::VECTOR_ARRANGE_ROTATE_CW {
        Some(Rotate90::Cw)
    } else if id == ph2d_editor::ids::VECTOR_ARRANGE_ROTATE_CCW {
        Some(Rotate90::Ccw)
    } else {
        None
    }
}

/// Toggle the SELECTED path between closed (loop) and open (ribbon) — panel
/// Close/Open button — recording ONE undo step iff it flipped. Free fn (mirror of
/// [`apply_vec_flip`]).
pub(crate) fn apply_vec_toggle_closed(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
) {
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] close-toggle: nenhum path selecionado");
        return;
    };
    let Some(cur) = scene.paths().iter().find(|p| p.id == sel).map(|p| p.closed) else {
        return;
    };
    let pre = scene.clone();
    if scene.set_path_closed(sel, !cur) {
        // Closing a never-filled path seeds its fill from the current Style — so it
        // paints IMMEDIATELY, matching the pen's auto-close (click the start point).
        // An existing fill is preserved across open→close cycles.
        if !cur
            && let Some(path) = scene.path_mut(sel)
            && path.fill.is_none()
        {
            path.fill = Some(ph2d_vec_scene::Paint::solid(pen.style().fill));
        }
        history.push_undo(pre);
    }
}

/// Fill kind a Vector-panel Fill-type button targets on the selected path.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum VecFillKind {
    Solid,
    Linear,
    Radial,
    MultiPoint,
}

/// Map a Fill-type button `NodeId` to its [`VecFillKind`] (`None` otherwise).
pub(crate) fn vec_fill_kind_for_id(id: ph2d_editor::NodeId) -> Option<VecFillKind> {
    if id == ph2d_editor::ids::VECTOR_FILL_KIND_SOLID {
        Some(VecFillKind::Solid)
    } else if id == ph2d_editor::ids::VECTOR_FILL_KIND_LINEAR {
        Some(VecFillKind::Linear)
    } else if id == ph2d_editor::ids::VECTOR_FILL_KIND_RADIAL {
        Some(VecFillKind::Radial)
    } else if id == ph2d_editor::ids::VECTOR_FILL_KIND_MULTI {
        Some(VecFillKind::MultiPoint)
    } else {
        None
    }
}

/// Component-wise average of two colours (alpha too).
fn avg_color(a: ph2d_vec_scene::Rgba8, b: ph2d_vec_scene::Rgba8) -> ph2d_vec_scene::Rgba8 {
    let m = |x: u8, y: u8| ((u16::from(x) + u16::from(y)) / 2) as u8;
    ph2d_vec_scene::Rgba8::new(m(a.r, b.r), m(a.g, b.g), m(a.b, b.b), m(a.a, b.a))
}

/// Multi-point set to use when switching to a freeform fill: reuse existing points,
/// else seed 3 spread points `[fill, contrast, average]` across the bbox `(lo,hi)`.
fn gradient_points_from(
    fill: &Option<ph2d_vec_scene::Paint>,
    lo: [f64; 2],
    hi: [f64; 2],
) -> Vec<ph2d_vec_scene::GradientPoint> {
    use ph2d_vec_scene::{GradientPoint, Paint};
    if let Some(Paint::MultiPoint { points }) = fill
        && !points.is_empty()
    {
        return points.clone();
    }
    let base = fill
        .as_ref()
        .map_or(ph2d_vec_scene::Rgba8::new(255, 255, 255, 255), |p| {
            p.primary_color()
        });
    let contrast = contrast_color(base);
    let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
    let at = |fx: f64, fy: f64| [lo[0] + w * fx, lo[1] + h * fy];
    vec![
        GradientPoint::new(at(0.25, 0.25), base, 1.0),
        GradientPoint::new(at(0.75, 0.75), contrast, 1.0),
        GradientPoint::new(at(0.75, 0.25), avg_color(base, contrast), 1.0),
    ]
}

/// A luminance-opposite (black/white, alpha preserved) — the second stop seeded
/// when a solid fill first becomes a gradient, so the ramp is visibly a gradient.
fn contrast_color(c: ph2d_vec_scene::Rgba8) -> ph2d_vec_scene::Rgba8 {
    let lum = 0.2126 * f64::from(c.r) + 0.7152 * f64::from(c.g) + 0.0722 * f64::from(c.b);
    if lum > 128.0 {
        ph2d_vec_scene::Rgba8::new(0, 0, 0, c.a)
    } else {
        ph2d_vec_scene::Rgba8::new(255, 255, 255, c.a)
    }
}

/// Gradient stops to use when switching to a gradient: reuse the existing gradient's
/// stops (Linear↔Radial keep them), else seed a 2-stop ramp `[fill → contrast]`.
fn gradient_stops_from(fill: &Option<ph2d_vec_scene::Paint>) -> Vec<ph2d_vec_scene::GradientStop> {
    use ph2d_vec_scene::{GradientStop, Paint};
    match fill {
        Some(Paint::Linear { stops, .. }) | Some(Paint::Radial { stops, .. })
            if stops.len() >= 2 =>
        {
            stops.clone()
        }
        _ => {
            let base = fill
                .as_ref()
                .map_or(ph2d_vec_scene::Rgba8::new(255, 255, 255, 255), |p| {
                    p.primary_color()
                });
            vec![
                GradientStop::new(0.0, base),
                GradientStop::new(1.0, contrast_color(base)),
            ]
        }
    }
}

/// Linear ramp endpoints spanning the bbox `(lo,hi)` along `degrees` (0° = →),
/// centered on the bbox — the world-space geometry a linear gradient stores.
fn linear_span(lo: [f64; 2], hi: [f64; 2], degrees: f64) -> ([f64; 2], [f64; 2]) {
    let (cx, cy) = ((lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5);
    let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
    let r = degrees.to_radians();
    let (dx, dy) = (r.cos(), r.sin());
    let reach = 0.5 * ((w * dx).abs() + (h * dy).abs());
    (
        [cx - dx * reach, cy - dy * reach],
        [cx + dx * reach, cy + dy * reach],
    )
}

/// Switch the SELECTED path's fill kind (Solid/Linear/Radial), preserving colour(s)
/// and existing gradient geometry when the kind is unchanged; when entering a
/// gradient from Solid/other, the geometry is seeded to fit the path's bbox. One
/// undo step iff it changed.
pub(crate) fn apply_vec_set_fill_kind(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    kind: VecFillKind,
) {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] fill-kind: nenhum path selecionado");
        return;
    };
    let Some(cur) = scene
        .paths()
        .iter()
        .find(|p| p.id == sel)
        .map(|p| p.fill.clone())
    else {
        return;
    };
    let (lo, hi) = scene.path_bbox(sel).unwrap_or(([0.0, 0.0], [1.0, 1.0]));
    let (cx, cy) = ((lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5);
    let new_fill = match kind {
        VecFillKind::Solid => Paint::Solid(
            cur.as_ref()
                .map_or(ph2d_vec_scene::Rgba8::new(255, 255, 255, 255), |p| {
                    p.primary_color()
                }),
        ),
        // Already this kind → keep its geometry; else seed to fit the bbox.
        VecFillKind::Linear => match &cur {
            Some(p @ Paint::Linear { .. }) => p.clone(),
            _ => {
                let (start, end) = linear_span(lo, hi, 0.0);
                Paint::Linear {
                    stops: gradient_stops_from(&cur),
                    start,
                    end,
                }
            }
        },
        VecFillKind::Radial => match &cur {
            Some(p @ Paint::Radial { .. }) => p.clone(),
            _ => Paint::Radial {
                stops: gradient_stops_from(&cur),
                center: [cx, cy],
                radius: 0.5 * (hi[0] - lo[0]).hypot(hi[1] - lo[1]),
            },
        },
        VecFillKind::MultiPoint => match &cur {
            Some(p @ Paint::MultiPoint { .. }) => p.clone(),
            _ => Paint::MultiPoint {
                points: gradient_points_from(&cur, lo, hi),
            },
        },
    };
    if cur.as_ref() == Some(&new_fill) {
        return;
    }
    let pre = scene.clone();
    if let Some(path) = scene.path_mut(sel) {
        path.fill = Some(new_fill);
        history.push_undo(pre);
    }
}

/// Set the SELECTED path's Linear-gradient angle (degrees; from the Angle slider's
/// `track·360`) by re-fitting the ramp endpoints across the bbox at that angle.
/// No-op unless the fill is Linear. One undo step iff it changed.
pub(crate) fn apply_vec_set_grad_angle(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    degrees: f64,
) {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        return;
    };
    let is_linear = scene
        .paths()
        .iter()
        .find(|p| p.id == sel)
        .is_some_and(|p| matches!(p.fill, Some(Paint::Linear { .. })));
    if !is_linear {
        return;
    }
    let (lo, hi) = scene.path_bbox(sel).unwrap_or(([0.0, 0.0], [1.0, 1.0]));
    let (start, end) = linear_span(lo, hi, degrees);
    let pre = scene.clone();
    if let Some(Paint::Linear {
        start: s, end: e, ..
    }) = scene.path_mut(sel).and_then(|p| p.fill.as_mut())
    {
        if *s == start && *e == end {
            return;
        }
        *s = start;
        *e = end;
        history.push_undo(pre);
    }
}

/// Add a multi-point gradient point at the selected path's bbox center (colour =
/// the first existing point). No-op unless the fill is MultiPoint. One undo step.
pub(crate) fn apply_vec_grad_add_point(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
) {
    use ph2d_vec_scene::{GradientPoint, Paint};
    let Some(sel) = pen.selected() else {
        return;
    };
    let Some((lo, hi)) = scene.path_bbox(sel) else {
        return;
    };
    let center = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    let pre = scene.clone();
    if let Some(Paint::MultiPoint { points }) = scene.path_mut(sel).and_then(|p| p.fill.as_mut()) {
        let col = points
            .first()
            .map_or(ph2d_vec_scene::Rgba8::new(255, 255, 255, 255), |p| p.color);
        points.push(GradientPoint::new(center, col, 1.0));
        history.push_undo(pre);
    }
}

/// Remove a multi-point gradient point (`selected`, else the last), keeping at
/// least one. Returns the new selection (`None`). One undo step iff it removed.
pub(crate) fn apply_vec_grad_remove_point(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    selected: Option<usize>,
) -> Option<usize> {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        return selected;
    };
    let pre = scene.clone();
    if let Some(Paint::MultiPoint { points }) = scene.path_mut(sel).and_then(|p| p.fill.as_mut())
        && points.len() > 1
    {
        let idx = selected
            .filter(|&i| i < points.len())
            .unwrap_or(points.len() - 1);
        points.remove(idx);
        history.push_undo(pre);
        return None;
    }
    selected
}

/// Set the SELECTED multi-point gradient point's influence (`value` from the
/// Influence slider's `track·4`). No-op unless the fill is MultiPoint and `point`
/// is valid. One undo step iff it changed.
pub(crate) fn apply_vec_grad_influence(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    point: Option<usize>,
    value: f64,
) {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        return;
    };
    let Some(i) = point else {
        return;
    };
    let pre = scene.clone();
    if let Some(Paint::MultiPoint { points }) = scene.path_mut(sel).and_then(|p| p.fill.as_mut())
        && let Some(gp) = points.get_mut(i)
        && (gp.influence - value).abs() > 1e-9
    {
        gp.influence = value;
        history.push_undo(pre);
    }
}

/// Set the SELECTED multi-point gradient point's jitter (`value` 0..1, from the
/// Jitter slider's track). No-op unless the fill is MultiPoint and `point` is valid.
/// One undo step iff it changed.
pub(crate) fn apply_vec_grad_jitter(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    point: Option<usize>,
    value: f64,
) {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        return;
    };
    let Some(i) = point else {
        return;
    };
    let pre = scene.clone();
    if let Some(Paint::MultiPoint { points }) = scene.path_mut(sel).and_then(|p| p.fill.as_mut())
        && let Some(gp) = points.get_mut(i)
        && (gp.jitter - value).abs() > 1e-9
    {
        gp.jitter = value;
        history.push_undo(pre);
    }
}

/// Component-wise linear blend of two colours at `t ∈ [0,1]`.
fn lerp_color(a: ph2d_vec_scene::Rgba8, b: ph2d_vec_scene::Rgba8, t: f64) -> ph2d_vec_scene::Rgba8 {
    let m = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * t).round() as u8;
    ph2d_vec_scene::Rgba8::new(m(a.r, b.r), m(a.g, b.g), m(a.b, b.b), m(a.a, b.a))
}

/// The Linear/Radial gradient stops of the selected path (`None` for other fills).
fn selected_ramp_stops<'a>(
    scene: &'a ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
) -> Option<&'a [ph2d_vec_scene::GradientStop]> {
    use ph2d_vec_scene::Paint;
    let sel = pen.selected()?;
    match &scene.paths().iter().find(|p| p.id == sel)?.fill {
        Some(Paint::Linear { stops, .. }) | Some(Paint::Radial { stops, .. }) => Some(stops),
        _ => None,
    }
}

/// Add an interior ramp stop to the SELECTED Linear/Radial gradient, at the midpoint
/// of the widest gap (colour = the blend there). Returns the new stop's index (to
/// select), or `None` if the fill isn't a ramp. One undo step.
pub(crate) fn apply_vec_grad_add_stop(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
) -> Option<usize> {
    use ph2d_vec_scene::{GradientStop, Paint};
    let sel = pen.selected()?;
    // Interior stops may cross, so the Vec isn't sorted — find the widest gap on a
    // sorted (offset, colour) view; the new stop's colour is the blend across it.
    let stops = selected_ramp_stops(scene, pen)?;
    if stops.len() < 2 {
        return None;
    }
    let mut sorted: Vec<(f64, ph2d_vec_scene::Rgba8)> =
        stops.iter().map(|s| (s.offset, s.color)).collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut best = (0usize, f64::NEG_INFINITY);
    for k in 0..sorted.len() - 1 {
        let gap = sorted[k + 1].0 - sorted[k].0;
        if gap > best.1 {
            best = (k, gap);
        }
    }
    let k = best.0;
    let off = (sorted[k].0 + sorted[k + 1].0) * 0.5;
    let col = lerp_color(sorted[k].1, sorted[k + 1].1, 0.5);
    let pre = scene.clone();
    if let Some(Paint::Linear { stops, .. }) | Some(Paint::Radial { stops, .. }) =
        scene.path_mut(sel).and_then(|p| p.fill.as_mut())
    {
        // Insert as an INTERIOR stop (just before the last end stop) so the two ends
        // stay at index 0 / last; return its index to select it.
        let idx = stops.len() - 1;
        stops.insert(idx, GradientStop::new(off, col));
        history.push_undo(pre);
        return Some(idx);
    }
    None
}

/// Remove the SELECTED interior ramp stop (`selected` index) from the Linear/Radial
/// gradient, keeping the two end stops (≥2 total). Returns the new selection
/// (`None`). One undo step iff it removed.
pub(crate) fn apply_vec_grad_remove_stop(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    selected: Option<usize>,
) -> Option<usize> {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        return selected;
    };
    let pre = scene.clone();
    if let Some(Paint::Linear { stops, .. }) | Some(Paint::Radial { stops, .. }) =
        scene.path_mut(sel).and_then(|p| p.fill.as_mut())
        && let Some(i) = selected
        && i > 0
        && i + 1 < stops.len()
    {
        stops.remove(i);
        history.push_undo(pre);
        return None;
    }
    selected
}

/// An alignment edge/center the Align buttons snap the selected paths' bboxes to
/// (within the selection's union bbox).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum VecAlign {
    Left,
    HCenter,
    Right,
    Top,
    VCenter,
    Bottom,
}

/// Map an Align button `NodeId` to its [`VecAlign`] (`None` otherwise).
pub(crate) fn vec_align_for_id(id: ph2d_editor::NodeId) -> Option<VecAlign> {
    use ph2d_editor::ids as i;
    Some(match id {
        x if x == i::VECTOR_ALIGN_LEFT => VecAlign::Left,
        x if x == i::VECTOR_ALIGN_HCENTER => VecAlign::HCenter,
        x if x == i::VECTOR_ALIGN_RIGHT => VecAlign::Right,
        x if x == i::VECTOR_ALIGN_TOP => VecAlign::Top,
        x if x == i::VECTOR_ALIGN_VCENTER => VecAlign::VCenter,
        x if x == i::VECTOR_ALIGN_BOTTOM => VecAlign::Bottom,
        _ => return None,
    })
}

/// Distribute axis: even the selected paths' center spacing horizontally / vertically.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum VecDistribute {
    Horizontal,
    Vertical,
}

/// Map a Distribute button `NodeId` to its [`VecDistribute`] (`None` otherwise).
pub(crate) fn vec_distribute_for_id(id: ph2d_editor::NodeId) -> Option<VecDistribute> {
    if id == ph2d_editor::ids::VECTOR_DISTRIBUTE_H {
        Some(VecDistribute::Horizontal)
    } else if id == ph2d_editor::ids::VECTOR_DISTRIBUTE_V {
        Some(VecDistribute::Vertical)
    } else {
        None
    }
}

/// Align every selected path's bbox to the selection's union bbox per [`VecAlign`]
/// (needs ≥2 selected). One undo step iff anything moved.
pub(crate) fn apply_vec_align(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    xforms: &ph2d_vec_scene::VecXforms,
    kind: VecAlign,
) {
    // Bbox de CURVA (a caixa que o gizmo desenha), não a de âncoras: alinhar tem de
    // casar com o que o usuário vê — uma curva que abaula para fora das âncoras
    // encostaria errado.
    // Bboxes de MUNDO: alinhar compara formas ENTRE SI, e a bbox local de toda forma
    // assentada está centrada na origem (ADR-0112).
    let boxes: Vec<(u64, [f64; 2], [f64; 2])> = pen
        .selected_paths()
        .iter()
        .filter_map(|&id| {
            scene
                .path_world_curve_bbox(xforms, id)
                .map(|(lo, hi)| (id, lo, hi))
        })
        .collect();
    if boxes.len() < 2 {
        return;
    }
    // Union bbox of the selection.
    let mut ulo = [f64::INFINITY; 2];
    let mut uhi = [f64::NEG_INFINITY; 2];
    for &(_, lo, hi) in &boxes {
        ulo[0] = ulo[0].min(lo[0]);
        ulo[1] = ulo[1].min(lo[1]);
        uhi[0] = uhi[0].max(hi[0]);
        uhi[1] = uhi[1].max(hi[1]);
    }
    let pre = scene.clone();
    let mut moved = false;
    for (id, lo, hi) in boxes {
        let (mut dx, mut dy) = (0.0, 0.0);
        match kind {
            VecAlign::Left => dx = ulo[0] - lo[0],
            VecAlign::Right => dx = uhi[0] - hi[0],
            VecAlign::HCenter => dx = (ulo[0] + uhi[0]) * 0.5 - (lo[0] + hi[0]) * 0.5,
            // World Y is UP here, so "Top" = the selection's MAX y and "Bottom" =
            // its MIN y (matches what the user sees on-canvas).
            VecAlign::Top => dy = uhi[1] - hi[1],
            VecAlign::Bottom => dy = ulo[1] - lo[1],
            VecAlign::VCenter => dy = (ulo[1] + uhi[1]) * 0.5 - (lo[1] + hi[1]) * 0.5,
        }
        if dx.abs() > 1e-9 || dy.abs() > 1e-9 {
            moved |= scene.translate_path_world(xforms, id, dx, dy);
        }
    }
    if moved {
        history.push_undo(pre);
    }
}

/// Evenly space the selected paths' bbox CENTERS along `axis`, keeping the two
/// extremes fixed (needs ≥3 selected). One undo step iff anything moved.
pub(crate) fn apply_vec_distribute(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    xforms: &ph2d_vec_scene::VecXforms,
    axis: VecDistribute,
) {
    // (id, center-on-axis) for each selected path.
    let mut items: Vec<(u64, f64)> = pen
        .selected_paths()
        .iter()
        .filter_map(|&id| {
            scene.path_world_curve_bbox(xforms, id).map(|(lo, hi)| {
                let c = match axis {
                    VecDistribute::Horizontal => (lo[0] + hi[0]) * 0.5,
                    VecDistribute::Vertical => (lo[1] + hi[1]) * 0.5,
                };
                (id, c)
            })
        })
        .collect();
    if items.len() < 3 {
        return;
    }
    items.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let n = items.len();
    let (first, last) = (items[0].1, items[n - 1].1);
    let step = (last - first) / (n - 1) as f64;
    let pre = scene.clone();
    let mut moved = false;
    for (k, &(id, c)) in items.iter().enumerate().take(n - 1).skip(1) {
        let target = first + step * k as f64;
        let d = target - c;
        if d.abs() > 1e-9 {
            let (dx, dy) = match axis {
                VecDistribute::Horizontal => (d, 0.0),
                VecDistribute::Vertical => (0.0, d),
            };
            moved |= scene.translate_path(id, dx, dy);
        }
    }
    if moved {
        history.push_undo(pre);
    }
}

/// Rotate the SELECTED path by `degrees` (panel Transform Angle field — a
/// relative scrub) about its bbox center, recording ONE undo step iff it turned.
/// A zero delta is a no-op (no undo). Free fn (mirror of [`apply_vec_rotate`]).
pub(crate) fn apply_vec_rotate_by(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    degrees: f64,
) {
    if degrees.abs() < 1e-9 {
        return;
    }
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] rotate-by: nenhum path selecionado");
        return;
    };
    let Some((lo, hi)) = scene.path_bbox(sel) else {
        return;
    };
    let pivot = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    let pre = scene.clone();
    if scene.rotate_path_by(sel, degrees.to_radians(), pivot) {
        history.push_undo(pre);
    }
}

/// Whole-path reshape op (panel "Smooth" / "Sharpen" / "Simplify" buttons).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum VecPathShapeOp {
    Smooth,
    Sharpen,
    Simplify,
    Subdivide,
}

/// Smooth / sharpen / simplify ALL vertices of the SELECTED path (panel Path
/// buttons), recording ONE undo step iff it changed. Free fn (mirror of
/// [`apply_vec_flip`]).
pub(crate) fn apply_vec_path_shape(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    op: VecPathShapeOp,
) {
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] path-shape: nenhum path selecionado");
        return;
    };
    let pre = scene.clone();
    let changed = match op {
        VecPathShapeOp::Smooth => scene.smooth_path(sel),
        VecPathShapeOp::Sharpen => scene.sharpen_path(sel),
        VecPathShapeOp::Simplify => scene.simplify_path(sel),
        VecPathShapeOp::Subdivide => scene.subdivide_path(sel),
    };
    if changed {
        history.push_undo(pre);
    }
}

/// Map a Vector-panel Path button `NodeId` to its [`VecPathShapeOp`] (`None`
/// otherwise). Pure — unit-tested; called from the render_loop drain.
pub(crate) fn vec_path_shape_for_id(id: ph2d_editor::NodeId) -> Option<VecPathShapeOp> {
    if id == ph2d_editor::ids::VECTOR_PATH_SMOOTH {
        Some(VecPathShapeOp::Smooth)
    } else if id == ph2d_editor::ids::VECTOR_PATH_SHARPEN {
        Some(VecPathShapeOp::Sharpen)
    } else if id == ph2d_editor::ids::VECTOR_PATH_SIMPLIFY {
        Some(VecPathShapeOp::Simplify)
    } else if id == ph2d_editor::ids::VECTOR_PATH_SUBDIVIDE {
        Some(VecPathShapeOp::Subdivide)
    } else {
        None
    }
}

/// Which numeric-transform field a Vector-panel edit targets on the selected
/// path's anchor bbox (X/Y = top-left, W/H = size).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum VecTransformField {
    X,
    Y,
    W,
    H,
}

/// Map a Vector-panel Transform field `NodeId` to its [`VecTransformField`]
/// (`None` for any other id). Pure — unit-tested; called from the render_loop drain.
pub(crate) fn vec_transform_field_for_id(id: ph2d_editor::NodeId) -> Option<VecTransformField> {
    if id == ph2d_editor::ids::VECTOR_TRANSFORM_X {
        Some(VecTransformField::X)
    } else if id == ph2d_editor::ids::VECTOR_TRANSFORM_Y {
        Some(VecTransformField::Y)
    } else if id == ph2d_editor::ids::VECTOR_TRANSFORM_W {
        Some(VecTransformField::W)
    } else if id == ph2d_editor::ids::VECTOR_TRANSFORM_H {
        Some(VecTransformField::H)
    } else {
        None
    }
}

/// Apply a numeric transform edit to the SELECTED path: X/Y translate the anchor
/// bbox top-left to `target`; W/H scale (about the bbox min) so that dimension
/// becomes `target` (clamped > 0; a degenerate dimension can't be resized).
/// Records ONE undo step iff it changed. Free fn (mirror of [`apply_vec_boolean`]).
pub(crate) fn apply_vec_transform(
    scene: &mut ph2d_vec_scene::VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    xforms: &ph2d_vec_scene::VecXforms,
    field: VecTransformField,
    target: f64,
) {
    let Some(sel) = pen.selected() else {
        return;
    };
    // Os campos do painel são números de MUNDO (ADR-0112).
    let Some((lo, hi)) = scene.path_world_curve_bbox(xforms, sel) else {
        return;
    };
    // O escalonamento acontece na geometria local, em torno do canto local que
    // corresponde ao `lo` de mundo — a razão de escala é a mesma nos dois espaços.
    let local_lo = scene.path_curve_bbox(sel).map_or([0.0, 0.0], |(l, _)| l);
    let pre = scene.clone();
    let changed = match field {
        VecTransformField::X => {
            let dx = target - lo[0];
            dx.abs() > 1e-9 && scene.translate_path_world(xforms, sel, dx, 0.0)
        }
        VecTransformField::Y => {
            let dy = target - lo[1];
            dy.abs() > 1e-9 && scene.translate_path_world(xforms, sel, 0.0, dy)
        }
        VecTransformField::W => {
            let w = hi[0] - lo[0];
            if w <= 1e-6 {
                false
            } else {
                let sx = target.max(1e-4) / w;
                (sx - 1.0).abs() > 1e-9 && scene.scale_path(sel, sx, 1.0, local_lo)
            }
        }
        VecTransformField::H => {
            let h = hi[1] - lo[1];
            if h <= 1e-6 {
                false
            } else {
                let sy = target.max(1e-4) / h;
                (sy - 1.0).abs() > 1e-9 && scene.scale_path(sel, 1.0, sy, local_lo)
            }
        }
    };
    if changed {
        history.push_undo(pre);
    }
}

/// The shape kind a Vector draw-mode maps to (`None` = Pen, the non-shape
/// gesture). Lets the canvas dispatch route Down/Move/Up to the pen or the
/// shape tool.
fn shape_kind_for_mode(mode: ph2d_tool_vector::DrawMode) -> Option<ph2d_vec_edit::ShapeKind> {
    use ph2d_tool_vector::DrawMode;
    use ph2d_vec_edit::ShapeKind;
    match mode {
        // Select/Node não desenham (ADR-0112); Pen desenha à mão livre.
        DrawMode::Select | DrawMode::Node | DrawMode::Pen => None,
        DrawMode::Rectangle => Some(ShapeKind::Rectangle),
        DrawMode::Ellipse => Some(ShapeKind::Ellipse),
        DrawMode::Polygon => Some(ShapeKind::Polygon),
        DrawMode::Star => Some(ShapeKind::Star),
        DrawMode::RoundRect => Some(ShapeKind::RoundRect),
        DrawMode::Spiral => Some(ShapeKind::Spiral),
        DrawMode::Line => Some(ShapeKind::Line),
        DrawMode::Arc => Some(ShapeKind::Arc),
    }
}

/// The shape parameters (sides / star / radius / spiral) from the mirrored tool config.
fn shape_params(cfg: &ph2d_tool_vector::VectorDrawConfig) -> ph2d_vec_edit::ShapeParams {
    ph2d_vec_edit::ShapeParams {
        sides: cfg.polygon_sides,
        star_points: cfg.star_points,
        star_inner_ratio: cfg.star_inner_ratio,
        corner_radius_px: cfg.corner_radius_px,
        spiral_turns: cfg.spiral_turns,
        arc_degrees: cfg.arc_degrees,
    }
}

/// Whether a primary pointer-Up should be **consumed** by the shape tool (and
/// NOT fall through to the chrome dispatch). Critical: in a shape mode the Up is
/// consumed ONLY while a shape drag is actually live — otherwise releasing over
/// a panel button (mode switch / boolean / close) while in a shape mode would
/// silently swallow the click, leaving every button dead.
fn shape_up_consumes(mode: ph2d_tool_vector::DrawMode, shape_active: bool) -> bool {
    shape_kind_for_mode(mode).is_some() && shape_active
}

/// O `anchor` e o meio-tamanho **intrínsecos** de um objeto do canvas, na linguagem
/// do gizmo de sprite: do `Sprite`, se houver; da bbox local da curva, se for uma
/// forma vetorial (ADR-0111). `([0,0], [0,0])` para o que não é nem um nem outro —
/// um grupo, que não tem geometria própria.
fn gizmo_anchor_half(
    sim: &ph2d_ecs::SimWorld,
    vec_scene: &ph2d_vec_scene::VecScene,
    entity: ph2d_ecs::Entity,
) -> ([f32; 2], [f32; 2]) {
    if let Some(s) = sim.world().get::<ph2d_render::Sprite>(entity) {
        return (s.anchor, [s.size[0] * 0.5, s.size[1] * 0.5]);
    }
    // `sim`/`vec_scene` chegam separados (e não via `AppGfx`) porque `hero_screen`
    // está emprestado mutável no Down — campos irmãos, borrows disjuntos.
    crate::vec_gizmo_view::anchor_half(sim, vec_scene, entity).unwrap_or(([0.0, 0.0], [0.0, 0.0]))
}

impl App {
    pub(crate) fn on_close_request(&mut self, event_loop: &ActiveEventLoop) {
        match self.handler.on_close_request() {
            CloseAction::Close => {
                self.handler.on_lifecycle(Lifecycle::WillTerminate);
                // Tear the audio system down HERE, deterministically, while the
                // main thread is quiescent — instead of letting the `cpal::Stream`
                // (ALSA/PipeWire, `!Send`) drop LAST in the `App` field cascade at
                // the end of `main`, where `snd_pcm_close` on the pipewire-alsa
                // plugin segfaults on teardown (benign — fires after "exited
                // cleanly" — but returns 139, which pollutes exit-code checks).
                #[cfg(feature = "panel-audio-editor")]
                {
                    self.audio = None;
                }
                event_loop.exit();
            }
            CloseAction::Cancel => {}
        }
    }

    pub(crate) fn on_resized(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.pending_resize = Some(WindowSize::new(size.width, size.height));
    }

    /// M14.4e drag-and-drop. winit emits one HoveredFile per path when
    /// multiple files are dragged together. Buffer paths into
    /// `self.hovered_files` and push to the hero (for the overlay) on
    /// every HoveredFile event.
    pub(crate) fn on_hovered_file(&mut self, path: std::path::PathBuf) {
        self.hovered_files.push(path);
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.dragging_files = Some((self.hovered_files.clone(), self.last_cursor));
        }
        self.handler.on_file_hover(&self.hovered_files);
        if let Some(host) = self.host.as_ref() {
            host.request_redraw();
        }
    }

    pub(crate) fn on_hovered_file_cancelled(&mut self) {
        self.hovered_files.clear();
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.dragging_files = None;
        }
        self.handler.on_file_hover_cancel();
        if let Some(host) = self.host.as_ref() {
            host.request_redraw();
        }
    }

    pub(crate) fn on_dropped_file(&mut self, path: std::path::PathBuf) {
        // M14.7 polish (7.3 fix): winit fires `DroppedFile` once PER
        // FILE on macOS but the events arrive across multiple loop
        // iterations. Importing inline on each event was racy — some
        // imports silently dropped when an event came in mid-render.
        // Buffer the path here; `render_frame` drains `pending_drops`
        // atomically.
        self.pending_drops.push(path);
        if let Some(host) = self.host.as_ref() {
            host.request_redraw();
        }
    }

    pub(crate) fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        if let Some(host) = &self.host {
            host.scale().set(scale_factor as f32);
            if let Some(gfx) = self.gfx.as_ref() {
                self.pending_resize = Some(gfx.surface.size());
            }
        }
    }

    pub(crate) fn on_modifiers_changed(&mut self, mods: winit::event::Modifiers) {
        self.modifiers = mods.state();
        // M14.A: push the Shift state to the hero's WidgetStore so
        // `dispatch_pointer` Move can scale the NumberInput drag delta
        // correctly (Shift = fine adjustment). The ph2d-host
        // `PointerEvent` schema doesn't carry modifiers natively — the
        // store cache is the canonical bridge for now.
        // Fase 0c: also push the Cmd (macOS super) / Ctrl modifier
        // OR'd together — used by hierarchy + canvas multi-select to
        // map a click into `SelectModifier::Toggle`.
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            hero.store.set_shift_held(self.modifiers.shift_key());
            hero.store
                .set_cmd_held(self.modifiers.super_key() || self.modifiers.control_key());
            // Motion Nodes M0.T3 — Alt cache, folded into `GestureMods.alt` for
            // graph gestures (mirror of shift/cmd; pointer events carry no mods).
            hero.store.set_alt_held(self.modifiers.alt_key());
        }
    }

    /// IME composition commits — PT-BR / Spanish / French accent
    /// dead-key sequences arrive here on macOS, NOT in `KeyEvent::text`
    /// (the system text-input service swallows the dead-key keystroke
    /// and emits the composed char via `Ime::Commit`).
    pub(crate) fn on_ime_commit(&mut self, text: String) {
        for ch in text.chars() {
            if !ch.is_control() {
                forward_text_to_hero(self.gfx.as_mut(), ch);
            }
        }
        // `Preedit` (in-progress composition) is ignored for now — no
        // visible preedit caret yet. Future: render the preedit text
        // in italics at the caret.
    }

    /// Reflect the current hover context in the OS cursor. Called each
    /// CursorMoved (winit dedups the icon). Priority: an armed colour-picker
    /// eyedropper wins (a crosshair "target"), else the Motion graph's split
    /// divider shows a double-arrow resize cursor (`NsResize` ↕ for a horizontal
    /// divider, `EwResize` ↔ for a vertical one), else a timeline grab band
    /// (panel edge, label splitter, graph-height grip), else the default arrow.
    fn update_eyedropper_cursor(&self) {
        let Some(win) = self.window.as_ref() else {
            return;
        };
        use winit::window::CursorIcon;
        let cursor = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map(|h| {
                if h.store.eyedropper_pending().is_some() {
                    CursorIcon::Crosshair
                } else if self.over_motion_split_divider(h) {
                    if h.view.center_split.is_vertical() {
                        CursorIcon::EwResize
                    } else {
                        CursorIcon::NsResize
                    }
                } else if let Some(icon) = self.timeline_resize_cursor(h) {
                    icon
                } else {
                    CursorIcon::Default
                }
            })
            .unwrap_or(CursorIcon::Default);
        win.set_cursor(cursor);
    }

    /// The double-arrow cursor for the timeline grab band under the pointer, if
    /// any. Resolves the last pointer through the hit index to a `TimelineSurface`
    /// hit — the same channel the drag uses, so the cursor and the gesture always
    /// agree on where the band is (mirror of `over_motion_split_divider`).
    fn timeline_resize_cursor(
        &self,
        hero: &ph2d_editor::HeroScreen,
    ) -> Option<winit::window::CursorIcon> {
        use ph2d_editor::interaction::TimelineHitKind;
        use winit::window::CursorIcon;
        let (x, y) = self.last_pointer;
        let (_, kind) = hero
            .hit_index
            .hit(x, y)
            .and_then(|id| hero.store.timeline_surface_at_id(id))?;
        Some(match kind {
            // The names column widens sideways; the graph band grows downward.
            TimelineHitKind::LabelSplitter => CursorIcon::EwResize,
            TimelineHitKind::GraphResize => CursorIcon::NsResize,
            TimelineHitKind::ResizeEdge { edges } => resize_cursor_for_edges(edges),
            _ => return None,
        })
    }

    /// Is the cursor over the Motion graph's draggable split divider? Resolves
    /// the last-pointer position through the hit index to a `GraphSurface` hit
    /// and checks its kind — the same channel the divider drag uses, so the
    /// cursor and the gesture agree on the grab band.
    fn over_motion_split_divider(&self, hero: &ph2d_editor::HeroScreen) -> bool {
        let (x, y) = self.last_pointer;
        hero.hit_index
            .hit(x, y)
            .and_then(|id| hero.store.graph_surface_at_id(id))
            .is_some_and(|(_, kind)| {
                matches!(kind, ph2d_editor::interaction::GraphHitKind::SplitDivider)
            })
    }

    /// ADR-0108 Fase 1: booleana N-ária sobre as regiões fechadas SELECIONADAS
    /// (hotkeys U/I/D/X). Delega ao livre [`apply_vec_boolean`] com os refs
    /// decompostos — o mesmo caminho usado pelos botões Boolean do painel (drain
    /// do render_loop, onde `self.gfx` já está destruturado e o método não é
    /// chamável).
    fn vec_boolean(&mut self, op: ph2d_vec_boolean::BoolOp) {
        if let Some(gfx) = self.gfx.as_mut() {
            let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
            apply_vec_boolean(
                &mut gfx.vec_scene,
                &mut self.vec_history,
                &mut self.vec_pen,
                &xf,
                op,
            );
        }
    }

    /// Arrow-key nudge: move the selection by a SCREEN delta (px), converted to
    /// world (honours zoom + orientation). `record_undo` pushes ONE undo step —
    /// the caller passes `!repeat`, so a held arrow coalesces into a single step
    /// (auto-repeats move but don't each record). Returns whether anything moved.
    pub(crate) fn vec_nudge_selected(&mut self, dx_px: f64, dy_px: f64, record_undo: bool) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let base = gfx.camera.screen_to_world((0.0, 0.0), win);
        let moved = gfx
            .camera
            .screen_to_world((dx_px as f32, dy_px as f32), win);
        let (dx, dy) = ((moved[0] - base[0]) as f64, (moved[1] - base[1]) as f64);
        let pre = record_undo.then(|| gfx.vec_scene.clone());
        if self.vec_pen.nudge(&mut gfx.vec_scene, dx, dy) {
            if let Some(pre) = pre {
                self.vec_history.push_undo(pre);
            }
            true
        } else {
            false
        }
    }

    /// ADR-0108 Fase 1: Delete/Backspace no modo vetorial — prioriza apagar o
    /// VÉRTICE selecionado (edição de nó); sem vértice selecionado (ex.: resultado
    /// de booleana), apaga o PATH inteiro.
    pub(crate) fn vec_delete_selected_vertex_or_path(&mut self) -> bool {
        if self.vec_pen.selected_vert().is_some()
            && let Some(gfx) = self.gfx.as_mut()
            && apply_vec_delete_vertex(&mut gfx.vec_scene, &mut self.vec_history, &mut self.vec_pen)
        {
            eprintln!("[ph2d-vec] vértice apagado");
            return true;
        }
        self.vec_delete_selected()
    }

    /// ADR-0108 Fase 1: apaga o path selecionado (fallback do Delete sem vértice).
    /// World-space offset for a `px` screen-space diagonal shift (paste / dup
    /// placement), honouring the current zoom. `(0, 0)` when the gfx isn't ready.
    fn vec_screen_offset(&self, px: f64) -> (f64, f64) {
        let Some(gfx) = self.gfx.as_ref() else {
            return (0.0, 0.0);
        };
        let win = gfx.surface.size();
        let base = gfx.camera.screen_to_world((0.0, 0.0), win);
        let moved = gfx.camera.screen_to_world((px as f32, px as f32), win);
        ((moved[0] - base[0]) as f64, (moved[1] - base[1]) as f64)
    }

    /// True when a text / number / combobox widget holds keyboard focus — Vector
    /// Ctrl+C/X/V must defer to the OS text clipboard instead of the path one.
    fn vector_text_field_focused(&self) -> bool {
        let Some(h) = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref()) else {
            return false;
        };
        let Some(id) = h.store.focus_id() else {
            return false;
        };
        matches!(
            h.store.get(id),
            Some(
                ph2d_editor::InteractiveState::TextInput { .. }
                    | ph2d_editor::InteractiveState::NumberInput { .. }
                    | ph2d_editor::InteractiveState::Combobox { .. }
            )
        )
    }

    /// Vector Ctrl+C: copy the object selection into the in-app clipboard. O
    /// recorte leva os GRUPOS inteiramente selecionados junto (ver `VecScene::
    /// copy_paths`), então colar reconstrói a estrutura. No-op sem seleção.
    fn vec_copy(&mut self) {
        let sel = self.vec_pen.selected_paths().to_vec();
        if sel.is_empty() {
            return;
        }
        if let Some(gfx) = self.gfx.as_ref() {
            let clip = gfx.vec_scene.copy_paths(&sel);
            if !clip.is_empty() {
                self.vec_clipboard = Some(clip);
            }
        }
    }

    /// Vector Ctrl+X: copy the object selection, then delete it.
    fn vec_cut(&mut self) {
        self.vec_copy();
        self.vec_delete_selected();
    }

    /// Vector Ctrl+V: paste the clipboard, offset ~12 px (screen→world), e seleciona
    /// o resultado. Ctrl+Shift+V cola **no lugar** (sem deslocar). ONE undo step.
    fn vec_paste(&mut self, in_place: bool) {
        let Some(clip) = self.vec_clipboard.clone() else {
            return;
        };
        let (dx, dy) = if in_place {
            (0.0, 0.0)
        } else {
            self.vec_screen_offset(PASTE_OFFSET_PX)
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let pre = gfx.vec_scene.clone();
        let new_ids = gfx.vec_scene.paste_clip(&clip, dx, dy);
        if new_ids.is_empty() {
            return;
        }
        self.vec_history.push_undo(pre);
        self.vec_pen.select_many(&new_ids);
    }

    /// A seleção de objeto que tocar `path` produz — o grupo inteiro, se houver.
    /// A árvore é a Hierarquia (ADR-0110), então quem sabe disso é o ECS.
    pub(crate) fn vec_object_selection_for(&self, path: u64) -> Vec<u64> {
        let Some(gfx) = self.gfx.as_ref() else {
            return vec![path];
        };
        crate::vec_entities::object_selection_for(
            &gfx.sim,
            &gfx.vec_scene,
            &self.vec_entities,
            path,
        )
    }

    /// Ctrl+G / Ctrl+Shift+G: agrupa / desagrupa a seleção. O grupo é uma entidade
    /// comum, então ele aceita sprite e path vetorial no mesmo saco.
    fn vec_group(&mut self, group: bool) {
        let sel: Vec<u64> = self
            .vec_pen
            .selected_paths()
            .iter()
            .filter_map(|id| self.vec_entities.get(id).copied())
            .collect();
        if sel.is_empty() {
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else { return };
        let sim = &mut gfx.sim;
        if group {
            let name = format!("Group {}", sel.len());
            if crate::vec_entities::group_entities(sim, &sel, name).is_none() {
                eprintln!("[ph2d-vec] group: selecione >= 2 objetos distintos");
            }
        } else if crate::vec_entities::ungroup_entities(sim, &sel) == 0 {
            eprintln!("[ph2d-vec] ungroup: a selecao nao esta em nenhum grupo");
        }
    }

    /// Vector Ctrl+D: duplicate the object selection (offset ~12 px) — o irmão de
    /// teclado do botão Arrange "Duplicate". Preserva os grupos, como o paste.
    fn vec_duplicate_shortcut(&mut self) {
        let (dx, dy) = self.vec_screen_offset(PASTE_OFFSET_PX);
        if let Some(gfx) = self.gfx.as_mut() {
            apply_vec_duplicate(
                &mut gfx.vec_scene,
                &mut self.vec_history,
                &mut self.vec_pen,
                dx,
                dy,
            );
        }
    }

    /// Apaga TODA a seleção de objeto (um grupo some inteiro) e limpa os grupos
    /// que ficaram sem membro. ONE undo step.
    fn vec_delete_selected(&mut self) -> bool {
        let sel = self.vec_pen.selected_paths().to_vec();
        if sel.is_empty() {
            return false;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let pre = gfx.vec_scene.clone();
        let mut any = false;
        for id in &sel {
            any |= gfx.vec_scene.remove_path(*id);
        }
        if !any {
            return false;
        }
        self.vec_history.push_undo(pre);
        self.vec_pen.clear();
        eprintln!("[ph2d-vec] {} path(s) apagado(s)", sel.len());
        true
    }

    /// W4: write the timeline document to its sidecar (Ctrl+S while the timeline
    /// panel is the context). Tracks are named by their object's name so they
    /// reconnect on load (see `timeline_persist`).
    fn timeline_save_to_sidecar(&mut self) {
        let result = {
            let Some(gfx) = self.gfx.as_ref() else {
                return;
            };
            crate::timeline_persist::save(&mut self.timeline, gfx.sim.world())
        };
        let msg = match result {
            Ok(n) => format!("Timeline saved · {n} track(s)"),
            Err(e) => format!("Timeline save failed: {e}"),
        };
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts.push(Toast::info(msg));
        }
    }

    /// W4: load the timeline document from its sidecar (Ctrl+O), reconnecting
    /// tracks to the live objects by name. Toasts how many of N reconnected.
    fn timeline_load_from_sidecar(&mut self) {
        let result = {
            let Some(gfx) = self.gfx.as_mut() else {
                return;
            };
            crate::timeline_persist::load(&mut self.timeline, gfx.sim.world_mut())
        };
        let msg = match result {
            Ok((n, total)) => format!("Timeline loaded · {n}/{total} track(s) reconnected"),
            Err(e) => format!("Timeline load failed: {e}"),
        };
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts.push(Toast::info(msg));
        }
    }

    /// ADR-0108 cutover: is the Vector drawing tool the active tool? Gates the
    /// Pen input hooks (replaces the retired `PH2D_VEC_PEN` test flag).
    /// Põe a ORIGEM (o pivô) do path selecionado sob o cursor, sem mover a forma.
    /// `false` (e não consome o clique) se não há forma selecionada.
    fn vec_set_origin_to_cursor(&mut self, x: f32, y: f32) -> bool {
        let Some(sel) = self.vec_pen.selected() else {
            return false;
        };
        let Some(&bits) = self.vec_entities.get(&sel) else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let target = gfx.camera.screen_to_world((x, y), win);
        let moved = crate::vec_transform::move_origin_to(
            &mut gfx.sim,
            &mut gfx.vec_scene,
            ph2d_ecs::Entity::from_bits(bits),
            sel,
            target,
        );
        if moved {
            self.title_dirty = true;
        }
        moved
    }

    pub(crate) fn vector_tool_active(&self) -> bool {
        self.gfx.as_ref().is_some_and(|g| {
            g.tools
                .active()
                .is_some_and(|t| t.id() == ph2d_editor::ToolId::new("vector"))
        })
    }

    /// Motion Nodes M1: is the Motion Nodes tool the active tool? Gates the graph
    /// undo/redo chord (mirror of `vector_tool_active`).
    pub(crate) fn motion_tool_active(&self) -> bool {
        self.gfx.as_ref().is_some_and(|g| {
            g.tools
                .active()
                .is_some_and(|t| t.id() == ph2d_editor::ToolId::new("motion"))
        })
    }

    /// Motion Nodes M1 Phase 1b-3: undo the last graph edit (Ctrl/Cmd+Z). The
    /// `MotionHistory` stack is populated by the graph-edit intents (add / delete
    /// / connect / disconnect = one step each; a node drag is one bracketed step).
    /// Restoring the doc changes the cook, so re-cook via `mark_dirty`.
    fn motion_undo(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let m = &mut gfx.motion;
        if let Some(prev) = m.history.undo(&m.doc) {
            m.doc = prev;
            m.pump.mark_dirty();
        }
    }

    /// Motion Nodes M1 Phase 1b-3: redo (Ctrl/Cmd+Shift+Z / Ctrl+Y). Mirror of
    /// [`Self::motion_undo`].
    fn motion_redo(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let m = &mut gfx.motion;
        if let Some(next) = m.history.redo(&m.doc) {
            m.doc = next;
            m.pump.mark_dirty();
        }
    }

    /// ADR-0108: enquanto o Pen arrasta um handle, projeta o cursor pra world e
    /// puxa os handles Bézier do último vértice. No-op barato quando não há
    /// arrasto — chamado a cada CursorMoved.
    ///
    /// O snap é entregue como closure porque o Pen sabe o que é ÂNCORA (encaixa) e
    /// o que é handle (não encaixa); a shell só sabe a posição do cursor.
    fn vec_pen_drag_move(&mut self, x: f32, y: f32) -> bool {
        if !self.vector_tool_active() || !self.vec_pen.is_dragging() {
            return false;
        }
        let cfg = self.vec_snap_cfg(self.vec_px_to_world());
        // `take` evita emprestar `self` duas vezes: a closure fica com os alvos e as
        // guias, `self.vec_pen`/`self.gfx` seguem livres. Devolvidos logo abaixo.
        let targets = std::mem::take(&mut self.vec_snap_targets);
        let mut guides = Vec::new();
        let Some(gfx) = self.gfx.as_mut() else {
            self.vec_snap_targets = targets;
            return false;
        };
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world((x, y), win);
        // `hero_screen` e `vec_scene` são campos IRMÃOS de `AppGfx`: a grade pode ser
        // consultada enquanto o Pen muta a cena.
        let mut hero = gfx.hero_screen.as_mut();
        let mut snap = |p: [f64; 2]| {
            let mut grid = |q: [f64; 2]| {
                let h = hero.as_mut()?;
                crate::vec_snap::ask_grid(&mut h.grid.snap_state, q)
            };
            let r = ph2d_vec_edit::snap::snap(&[p], &targets, cfg, Some(&mut grid));
            guides = crate::vec_snap::guides_of(&r);
            r.apply(p)
        };
        let consumed =
            self.vec_pen
                .on_drag(&mut gfx.vec_scene, [w[0] as f64, w[1] as f64], &mut snap);
        self.vec_snap_targets = targets;
        self.vec_snap_guides = guides;
        consumed
    }

    /// Gradient group: hit-test the selected path's gradient handles (screen `pos`)
    /// → the handle within ~9 px (world-scaled): a multi-point point, or a
    /// linear/radial endpoint. `None` unless the Vector tool is active and the
    /// selected path has a gradient fill.
    fn vec_grad_hit(&self, pos: (f32, f32)) -> Option<ph2d_vec_render::GradHandle> {
        if !self.vector_tool_active() {
            return None;
        }
        let gfx = self.gfx.as_ref()?;
        let sel = self.vec_pen.selected()?;
        let path = gfx.vec_scene.paths().iter().find(|p| p.id == sel)?;
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world(pos, win);
        let (wx, wy) = (w[0] as f64, w[1] as f64);
        let w0 = gfx.camera.screen_to_world((0.0, 0.0), win);
        let w1 = gfx.camera.screen_to_world((1.0, 0.0), win);
        let px = (((w1[0] - w0[0]).powi(2) + (w1[1] - w0[1]).powi(2)).sqrt()) as f64;
        // ADR-0111: a geometria do gradiente é LOCAL, como a do path. O cursor desce
        // pelo afim, e o raio de captura com ele (a forma pode estar escalada).
        let x = crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(
            &gfx.sim,
            ph2d_ecs::Entity::from_bits(*self.vec_entities.get(&sel)?),
        ));
        let inv = x.inverse()?;
        let l = inv.apply([wx, wy]);
        ph2d_vec_render::hit_gradient_handle(path, l[0], l[1], 9.0 * px / x.mean_scale())
    }

    /// Gradient group: while a gradient handle is grabbed, move it to the cursor's
    /// world position (a radial edge sets the radius). No-op unless a grad drag is
    /// live. Reuses the pure `drag_gradient_handle` geometry helper.
    fn vec_grad_drag_move(&mut self, x: f32, y: f32) -> bool {
        let Some(handle) = self.vec_grad_drag else {
            return false;
        };
        let Some(sel) = self.vec_pen.selected() else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world((x, y), win);
        // O ponto do gradiente é guardado no espaço local do path (ADR-0111).
        let w = match self.vec_entities.get(&sel).and_then(|&b| {
            crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(
                &gfx.sim,
                ph2d_ecs::Entity::from_bits(b),
            ))
            .inverse()
        }) {
            Some(inv) => {
                let l = inv.apply([f64::from(w[0]), f64::from(w[1])]);
                [l[0] as f32, l[1] as f32]
            }
            None => w,
        };
        if let Some(path) = gfx.vec_scene.path_mut(sel) {
            return ph2d_vec_render::drag_gradient_handle(path, handle, w[0] as f64, w[1] as f64);
        }
        false
    }

    /// Motion Nodes M1: is the cursor over the docked graph panel? Drives the
    /// cursor-gated graph keyboard focus + middle-pan routing (Blender-style F
    /// acts on the hovered area, graph vs scene).
    pub(crate) fn cursor_over_motion_graph(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.store.panel_rect(ph2d_editor::ids::MOTION_GRAPH_PANEL))
            .is_some_and(|r| r.contains(self.last_pointer.0, self.last_pointer.1))
    }

    /// W2.E6: is the cursor over the general timeline dock? Mirrors
    /// [`Self::cursor_over_motion_graph`] — a middle-drag there pans the
    /// dope-sheet (via its `TimelineSurface` gesture), not the camera behind it.
    /// Blender-style: the hovered component owns the zoom/pan.
    pub(crate) fn cursor_over_timeline(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.store.panel_rect(ph2d_editor::ids::TIMELINE_PANEL))
            .is_some_and(|r| r.contains(self.last_pointer.0, self.last_pointer.1))
    }
    /// ADR-0108 Fase 1: while a shape drag is live, resize it to the cursor.
    /// No-op unless the Vector tool is active AND a shape gesture is in progress.
    /// A ferramenta de forma não faz hit-test, então o canto é encaixado direto.
    fn vec_shape_drag_move(&mut self, x: f32, y: f32) -> bool {
        if !self.vector_tool_active() || !self.vec_shape.is_active() {
            return false;
        }
        let Some(w) = self
            .gfx
            .as_ref()
            .map(|gfx| gfx.camera.screen_to_world((x, y), gfx.surface.size()))
        else {
            return false;
        };
        let cfg = self.vec_snap_cfg(self.vec_px_to_world());
        let p = self.vec_snap_point([f64::from(w[0]), f64::from(w[1])], cfg);
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        self.vec_shape.on_drag(&mut gfx.vec_scene, p)
    }

    /// The clip frame under `(x, y)` if it's inside the overlay waveform — for
    /// starting a selection drag. `None` if the overlay is hidden or the point is
    /// outside the waveform area.
    #[cfg(feature = "panel-audio-editor")]
    fn audio_wave_frame_at(&self, x: f32, y: f32) -> Option<u64> {
        let view = crate::audio::wave_view()?;
        self.audio.as_ref()?.editor_clip()?;
        let r = view.rect;
        (x >= r.x && x <= r.x + r.w && y >= r.y && y <= r.y + r.h)
            .then(|| crate::audio::frame_at_x(&view, x))
    }

    /// Extend the active waveform selection to the cursor `x`. Returns `true` if a
    /// selection drag is live (the caller early-returns so it doesn't also pan).
    #[cfg(feature = "panel-audio-editor")]
    fn audio_sel_drag_move(&mut self, x: f32) -> bool {
        let Some(anchor) = self.audio_sel_drag else {
            return false;
        };
        if let Some(view) = crate::audio::wave_view() {
            let cur = crate::audio::frame_at_x(&view, x);
            if let Some(a) = self.audio.as_mut() {
                a.editor_set_selection(anchor, cur);
            }
        }
        true
    }

    pub(crate) fn on_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        // Diagnostics: count every raw winit move (input rate), paired with `paint_stamps_this_frame`
        // in the HUD so the coalescing is visible (high events → 1 stamp).
        self.input_events_this_frame = self.input_events_this_frame.saturating_add(1);
        let prev = self.last_pointer;
        self.last_pointer = (position.x as f32, position.y as f32);
        // (Graph keyboard focus is set every frame by `motion_bridge` from the
        // cursor — reliable even when the cursor stops before the panel rect is
        // published; see its `over_graph` gate.)
        // M14.4e: cache the latest cursor for DroppedFile — winit's
        // DroppedFile carries no position, so we project the most-
        // recently-seen cursor to world.
        self.last_cursor = self.last_pointer;
        // Reflect the colour-picker eyedropper in the OS cursor (a crosshair "target" while armed).
        self.update_eyedropper_cursor();
        // BgRemoval eyedropper drag (SHELL-only): while the primary
        // button is held with the eyedropper armed, every motion
        // samples another colour. Early-return so the move does not
        // also drive a gizmo drag / panel slider.
        if self.eyedropper_dragging {
            self.try_eyedropper_sample(self.last_pointer.0, self.last_pointer.1);
            return;
        }
        // Audio Editor waveform selection drag (SHELL-only): while a selection is
        // being dragged over the overlay waveform, every motion extends it. Early-
        // return so it doesn't also pan / drive a gizmo.
        #[cfg(feature = "panel-audio-editor")]
        if self.audio_sel_drag_move(self.last_pointer.0) {
            return;
        }
        // Keep the brush-size ring gizmo following the cursor while the
        // protection brush is armed (published for the on-canvas overlay).
        self.update_protect_brush_cursor(self.last_pointer.0, self.last_pointer.1);
        // BgRemoval protection brush drag (SHELL-only): while a dab is in
        // progress, every motion paints/erases another disc into the keep
        // mask. Early-return so it doesn't also drive a gizmo drag / slider.
        if self.protect_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Painter Falloff add-drag (SHELL-only): while a freshly click-added
        // control point is grabbed, motion drags it. Early-return so it doesn't
        // pan / drive a gizmo. No-ops unless an add-drag is live.
        if self.painter_falloff_drag(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Painter brush stroke (SHELL-only): while a canvas stroke is open, every
        // motion feeds another `CanvasPointer` to the active PainterTool. Early-
        // return so it doesn't also drive a gizmo drag / pan / slider.
        if self.painter_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Fill (Bucket) ColorDrop drag (SHELL-only): while a colour is being dragged from the Fill rail
        // button onto the canvas, deliver it to the painter's Fill. Early-return so it doesn't pan.
        if self.fill_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Fill "Fill adjust" modal title-band drag (SHELL-only): while the card is grabbed, motion moves
        // it. Early-return so it doesn't pan / drive a gizmo. No-ops unless a modal drag is armed.
        if self.fill_modal_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // ADR-0108 Fase 1: node box-select marquee — while Shift+dragging, grow
        // the box. Early-return so it doesn't pan / draw. No-op unless active.
        if let Some(m) = self.vec_marquee.as_mut() {
            m.1 = self.last_pointer;
            return;
        }
        // ADR-0108 Fase 1.2: Pen NOVO — arrastar após a âncora puxa os handles
        // Bézier (simétricos). Early-return: não pan/gizmo. No-op sem drag ativo.
        if self.vec_pen_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // ADR-0111: não há gizmo vetorial próprio. O gizmo de sprite move o
        // `Transform` da entidade do path, pelo mesmo caminho de qualquer objeto.
        // Gradient group 3b: dragging a multi-point gradient handle. Same
        // early-return discipline; no-op unless a grad drag is live.
        if self.vec_grad_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // ADR-0108 Fase 1: shape drag-to-size (Rectangle/Ellipse/Polygon). Same
        // early-return discipline as the pen; no-op unless a shape drag is live.
        if self.vec_shape_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // M14.4b.bis: middle-drag camera pan. Applied BEFORE pointer
        // forwarding so widgets receive the move event but the camera
        // also follows.
        if let Some(anchor) = self.pan_anchor
            && let Some(gfx) = self.gfx.as_mut()
        {
            let dx = self.last_pointer.0 - anchor.0;
            let dy = self.last_pointer.1 - anchor.1;
            let size = gfx.surface.size();
            gfx.camera
                .pan_screen_delta(dx, dy, size.width as f32, size.height as f32);
            self.pan_anchor = Some(self.last_pointer);
            let _ = prev; // silence unused warning when feature shifts
        }
        // Fase 0f: extend the active rubber-band rect, if any.
        if let Some(rb) = self.rubber_band.as_mut() {
            rb.current_screen = self.last_pointer;
        }
        let evt = PointerEvent {
            x: self.last_pointer.0,
            y: self.last_pointer.1,
            pressure: 1.0,
            kind: PointerKind::Move,
            source: PointerSource::Mouse,
            // Motion Nodes M0.T1: carry the REAL held button (winit's Move has
            // none). A middle/right drag now reaches editor-core with its
            // identity intact — the graph channel needs it (pan/box-select).
            button: self
                .held_button
                .unwrap_or(ph2d_host::PointerButton::Primary),
            timestamp_ns: Self::timestamp_ns(),
        };
        self.handler.on_pointer(evt);
        // A reparent only fires on pointer-Up (handled in on_mouse_input);
        // Move never emits one.
        let _ = forward_to_hero(self.gfx.as_mut(), evt);
        // M14.7 C: advance an open gizmo drag against the latest cursor
        // (MovePivot / scale / rotate / translate). Extracted to the
        // `gizmo_drag` sibling to keep this dispatch hub readable.
        self.advance_gizmo_drag();
        // Enio 2026-07-10: snap vetorial em TEMPO REAL — depois de o advance seguir o
        // cursor, gruda a forma arrastada no vizinho mais próximo (ponta p/ aberta,
        // vértice p/ fechada). Roda todo Move, então a forma prende/solta ao vivo.
        self.snap_dragged_vec_during_drag();
        // Drag-in-progress: forward pointer to active tool panel
        // hit-test → updates slider value continuously.
        if self.dragging.is_some() {
            self.dispatch_panel_pointer(self.last_pointer.0, self.last_pointer.1, false);
        }
    }

    pub(crate) fn on_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let (dx, dy) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (x * 16.0, y * 16.0),
            MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
        };
        // M14.4b.bis: wheel over the canvas zooms the camera. Wheel
        // over a hero panel keeps the existing panel-scroll behavior
        // (forward to hero).
        let over_panel =
            cursor_over_hero_panel(self.gfx.as_ref(), self.last_pointer.0, self.last_pointer.1);
        if !over_panel && let Some(gfx) = self.gfx.as_mut() {
            // Wheel up (positive dy) zooms IN (smaller height_world).
            let factor = 0.9_f32.powf(dy / 16.0);
            gfx.camera.zoom(factor);
        } else {
            let evt = ph2d_host::WheelEvent {
                x: self.last_pointer.0,
                y: self.last_pointer.1,
                delta_x: dx,
                delta_y: dy,
                modifiers: Self::convert_modifiers(self.modifiers),
                timestamp_ns: Self::timestamp_ns(),
            };
            forward_wheel_to_hero(self.gfx.as_mut(), evt);
        }
    }

    pub(crate) fn on_mouse_input(&mut self, state: ElementState, button: MouseButton) {
        self.any_input_this_frame = true;
        let kind = match state {
            ElementState::Pressed => PointerKind::Down,
            ElementState::Released => PointerKind::Up,
        };
        let mapped_button = match button {
            MouseButton::Left => ph2d_host::PointerButton::Primary,
            MouseButton::Right => ph2d_host::PointerButton::Secondary,
            MouseButton::Middle => ph2d_host::PointerButton::Middle,
            _ => ph2d_host::PointerButton::Primary,
        };
        // Motion Nodes M0.T1: track the held button so `CursorMoved` can carry
        // its identity (winit Move events don't). Held between Down and Up.
        self.held_button = match kind {
            PointerKind::Down => Some(mapped_button),
            PointerKind::Up => None,
            PointerKind::Move => self.held_button,
        };
        let evt = PointerEvent {
            x: self.last_pointer.0,
            y: self.last_pointer.1,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: mapped_button,
            timestamp_ns: Self::timestamp_ns(),
        };
        self.handler.on_pointer(evt);
        // Audio Editor waveform selection (SHELL-only): a primary press INSIDE the
        // overlay waveform starts a selection (cleared to a point); release ends
        // it. Early-return so the press doesn't drive the canvas/gizmo underneath.
        // Presses on the overlay's title-bar / resize handles fall through (they're
        // outside the waveform rect) to the shared BlenderHit dispatch.
        #[cfg(feature = "panel-audio-editor")]
        match kind {
            PointerKind::Down
                if let Some(frame) =
                    self.audio_wave_frame_at(self.last_pointer.0, self.last_pointer.1) =>
            {
                self.audio_sel_drag = Some(frame);
                if let Some(a) = self.audio.as_mut() {
                    a.editor_clear_selection();
                }
                return;
            }
            PointerKind::Up if self.audio_sel_drag.take().is_some() => return,
            _ => {}
        }
        // Was a right-click context menu (or the Fill "Fill adjust" modal) open when this click
        // arrived? If so the click belongs to that overlay (its slider/buttons/items) — chrome dispatch
        // in `forward_to_hero` handles it, so the canvas-consume arms below (paint / gizmo / select /
        // pan) must NOT also fire on a click LANDING on the overlay (which sits over the canvas). The
        // Fill modal counts as a modal exactly like the new-image dialog — without this, clicking its
        // threshold slider started a fresh flood-fill on the canvas underneath (mirror of the
        // new-image-modal "leaked a dab" fix). Captured now because `forward_to_hero` may close it.
        let menu_open_before = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| {
                h.store.context_menu().is_some() || h.store.fill_modal_pos().is_some()
            });
        // Colour-picker eyedropper armed when this click arrived? `forward_to_hero` services the pick
        // (sampling the pixel) AND clears the pending flag, so by the time the consume arms below run
        // it reads as disarmed. Capture it now so the Painter brush does NOT also paint where the user
        // sampled — the eyedropper must inhibit the brush (the sampled click is consumed, not painted).
        let eyedropper_armed_before = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.store.eyedropper_pending().is_some());

        // ADR-0108 cutover: the Vector tool's Pen draws ONLY on empty canvas.
        // A press over ANY UI — a docked panel body, a topbar pill, an open
        // menu, or this tool's own Style panel controls — MUST fall through to
        // the chrome dispatch below, never the pen; otherwise the whole UI is
        // unclickable while drawing (can't even deactivate the tool). Guard
        // mirrors the sprite-pick path: no panel under the cursor AND no
        // interactive widget hit (`hit_index` covers pills / menus / panel
        // controls; `panel_at` covers panel bodies incl. the vector panel).
        let on_canvas = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map(|h| {
                h.store.panel_at(evt.x, evt.y).is_none() && h.hit_index.hit(evt.x, evt.y).is_none()
            })
            .unwrap_or(false);
        // "Set Center" armado (ADR-0112): a pressão põe a ORIGEM da forma selecionada
        // sob o cursor e desarma. Vale em QUALQUER modo — inclusive Select, onde o
        // pivô do gizmo é o que se está ajustando.
        if self.vec_pivot_edit
            && self.vector_tool_active()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && on_canvas
            && !menu_open_before
        {
            self.vec_pivot_edit = false;
            if self.vec_set_origin_to_cursor(evt.x, evt.y) {
                return;
            }
        }
        // ADR-0112: no modo **Select** a ferramenta não captura o canvas — o clique
        // cai no caminho de sempre (picking de sprite + gizmo), e é assim que uma
        // forma vetorial se transforma. Só Node e os modos de desenho entram aqui.
        if self.vector_tool_active()
            && self.vec_draw_config.mode != ph2d_tool_vector::DrawMode::Select
            && !menu_open_before
        {
            // A canvas press while a text field is focused must blur it (commit the
            // edit) — the pen/shape arms below consume the press and bypass the
            // chrome dispatch that normally does this. Route it explicitly (only a
            // primary press with a field actually focused, so normal draw clicks
            // don't churn dispatch and a right-click can't open a menu here).
            if mapped_button == ph2d_host::PointerButton::Primary
                && kind == PointerKind::Down
                && on_canvas
                && self.vector_text_field_focused()
            {
                let _ = forward_to_hero(self.gfx.as_mut(), evt);
            }
            // A canvas press dismisses an open colour picker (click-outside closes
            // it, mirroring the chrome light-dismiss). `on_canvas` already excludes
            // the picker rect, so any press reaching here is genuinely outside it —
            // done BEFORE the grad/pen/shape arms so the picker's colour is never
            // applied to the handle the press then selects (Enio 2026-07-08).
            if mapped_button == ph2d_host::PointerButton::Primary
                && kind == PointerKind::Down
                && on_canvas
                && let Some(gfx) = self.gfx.as_mut()
                && let Some(hero) = gfx.hero_screen.as_mut()
                && hero.store.picker_target().is_some()
            {
                hero.store.set_picker_target(None);
            }
            match (mapped_button, kind) {
                // Shift+Down on a PATH → toggle it in the object multi-selection
                // (Align/Distribute); Shift+Down on empty canvas → vertex marquee.
                // Tried first so Shift diverts the press from the pen/shape draw.
                (ph2d_host::PointerButton::Primary, PointerKind::Down)
                    if on_canvas && self.modifiers.shift_key() =>
                {
                    let hit = self.gfx.as_ref().and_then(|gfx| {
                        let win = gfx.surface.size();
                        let w = gfx.camera.screen_to_world(self.last_pointer, win);
                        let w0 = gfx.camera.screen_to_world((0.0, 0.0), win);
                        let w1 = gfx.camera.screen_to_world((1.0, 0.0), win);
                        let px =
                            (((w1[0] - w0[0]).powi(2) + (w1[1] - w0[1]).powi(2)).sqrt()) as f64;
                        self.vec_pen
                            .path_at(&gfx.vec_scene, [w[0] as f64, w[1] as f64], 10.0 * px)
                    });
                    if let Some(id) = hit {
                        // Um grupo entra e sai da seleção INTEIRO (a árvore é a
                        // Hierarquia — o ancestral de topo diz quem vem junto).
                        let members = self.vec_object_selection_for(id);
                        self.vec_pen.toggle_object_members(&members);
                        // Object selection changed → drop any gradient-handle selection.
                        self.vec_grad_selected = None;
                        self.vec_grad_drag = None;
                        return;
                    }
                    self.vec_marquee = Some((self.last_pointer, self.last_pointer));
                    return;
                }
                (ph2d_host::PointerButton::Primary, PointerKind::Down) if on_canvas => {
                    // Canvas press priority (most specific first):
                    //   1. "Set Center" armed mode (positions the gizmo pivot).
                    //   2. Gradient handles — tiny (~9 px) and only present when the
                    //      selected path has a gradient fill, so they must outrank the
                    //      gizmo, whose bbox interior otherwise swallows every dot.
                    //   3. Transform gizmo handles (scale / rotate / interior move).
                    //   4. Pen / shape drawing + vertex editing.
                    // Gradient group 3b: a Down on a gradient handle starts dragging it.
                    if let Some(i) = self.vec_grad_hit(self.last_pointer) {
                        self.vec_grad_selected = Some(i);
                        self.vec_grad_drag = Some(i);
                        if let Some(gfx) = self.gfx.as_ref() {
                            self.vec_history.begin(&gfx.vec_scene);
                        }
                        return;
                    }
                    let params = shape_params(&self.vec_draw_config);
                    let shape_kind = shape_kind_for_mode(self.vec_draw_config.mode);
                    // Alt held → the Pen breaks the tangent when grabbing a handle.
                    let alt = self.modifiers.alt_key();
                    // Snap targets for THIS gesture: the whole scene as it stands.
                    // Rebuilt right after the press, once we know what got grabbed.
                    self.vec_rebuild_snap_targets(&[], &[]);
                    let cfg = self.vec_snap_cfg(self.vec_px_to_world());
                    let targets = std::mem::take(&mut self.vec_snap_targets);
                    if let Some(gfx) = self.gfx.as_mut() {
                        let win = gfx.surface.size();
                        let w = gfx.camera.screen_to_world(self.last_pointer, win);
                        // world-units por pixel (delta de 1px) → limiar/traço em px.
                        let w0 = gfx.camera.screen_to_world((0.0, 0.0), win);
                        let w1 = gfx.camera.screen_to_world((1.0, 0.0), win);
                        let px_to_world =
                            (((w1[0] - w0[0]).powi(2) + (w1[1] - w0[1]).powi(2)).sqrt()) as f64;
                        // Fase 2: snapshot pré-interação (vira passo de undo no Up
                        // só se a cena mudar de fato).
                        self.vec_history.begin(&gfx.vec_scene);
                        // `hero_screen` e `vec_scene` são campos IRMÃOS de `AppGfx`: a
                        // grade pode ser consultada enquanto o Pen muta a cena.
                        let mut hero = gfx.hero_screen.as_mut();
                        let mut snap = |p: [f64; 2]| {
                            let mut grid = |q: [f64; 2]| {
                                let h = hero.as_mut()?;
                                crate::vec_snap::ask_grid(&mut h.grid.snap_state, q)
                            };
                            ph2d_vec_edit::snap::snap(&[p], &targets, cfg, Some(&mut grid)).apply(p)
                        };
                        let node_mode =
                            self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Node;
                        match shape_kind {
                            // Node edita nós e NUNCA cria (ADR-0112). Não encaixa
                            // tampouco: o snap serve a quem POSICIONA um ponto novo.
                            None if node_mode => {
                                self.vec_pen.on_press_node(
                                    &mut gfx.vec_scene,
                                    [w[0] as f64, w[1] as f64],
                                    px_to_world,
                                    alt,
                                );
                            }
                            None => {
                                self.vec_pen.on_press(
                                    &mut gfx.vec_scene,
                                    [w[0] as f64, w[1] as f64],
                                    px_to_world,
                                    alt,
                                    &mut snap,
                                );
                            }
                            Some(kind) => {
                                // A ferramenta de forma não faz hit-test: o canto pode
                                // ser encaixado antes de entrar.
                                let p = snap([w[0] as f64, w[1] as f64]);
                                self.vec_shape.on_press(
                                    &mut gfx.vec_scene,
                                    kind,
                                    params,
                                    p,
                                    px_to_world,
                                );
                            }
                        }
                        self.vec_snap_targets = targets;
                        // Tocar um filho seleciona o GRUPO (a árvore é a Hierarquia).
                        // Depois do press, porque só agora sabemos o que foi agarrado.
                        if let Some(primary) = self.vec_pen.selected() {
                            let members = self.vec_object_selection_for(primary);
                            self.vec_pen.set_object_selection(&members);
                        }
                        // Agora sabemos o que o press agarrou: o que se move sai dos
                        // alvos (uma âncora não pode encaixar em si mesma; a forma em
                        // desenho não é referência de nada).
                        match (self.vec_pen.dragging_anchors(), self.vec_shape.selected()) {
                            (Some((pid, verts)), _) => {
                                let moving: Vec<_> = verts.iter().map(|&v| (pid, v)).collect();
                                self.vec_rebuild_snap_targets(&[], &moving);
                            }
                            (None, Some(sid)) => self.vec_rebuild_snap_targets(&[sid], &[]),
                            (None, None) => {}
                        }
                        return;
                    }
                    self.vec_snap_targets = targets;
                }
                (ph2d_host::PointerButton::Primary, PointerKind::Up) => {
                    // Fim de gesto: as guias de snap não sobrevivem ao Up.
                    self.vec_clear_snap_guides();
                    // Gradient group 3b: end a gradient-handle drag (commit iff moved).
                    if self.vec_grad_drag.take().is_some() {
                        if let Some(gfx) = self.gfx.as_ref() {
                            self.vec_history.commit_if_changed(&gfx.vec_scene);
                        }
                        return;
                    }
                    // Marquee release → box-select the anchors inside the box.
                    if let Some((start, cur)) = self.vec_marquee.take() {
                        if let Some(gfx) = self.gfx.as_mut() {
                            let win = gfx.surface.size();
                            let a = gfx.camera.screen_to_world(start, win);
                            let b = gfx.camera.screen_to_world(cur, win);
                            self.vec_pen.box_select(
                                &gfx.vec_scene,
                                [a[0] as f64, a[1] as f64],
                                [b[0] as f64, b[1] as f64],
                            );
                        }
                        return;
                    }
                    if shape_kind_for_mode(self.vec_draw_config.mode).is_none() {
                        // Pen: the release ends a handle drag / grab.
                        let consumed = self.vec_pen.on_release();
                        if let Some(gfx) = self.gfx.as_mut() {
                            self.vec_history.commit_if_changed(&gfx.vec_scene);
                        }
                        if consumed {
                            return;
                        }
                    } else if shape_up_consumes(
                        self.vec_draw_config.mode,
                        self.vec_shape.is_active(),
                    ) {
                        // A shape drag is in progress → finalize it. Commit if the
                        // drag spanned a real size, else discard the stray click
                        // (cancel the pending undo so it doesn't record a spurious
                        // `next_id`-only step). ONLY consume the Up when a shape is
                        // actually being drawn — otherwise (e.g. releasing over a
                        // panel button while in a shape mode) the Up MUST fall
                        // through to the chrome dispatch, else every panel click
                        // (mode switch, boolean, close) is silently swallowed.
                        let committed = if let Some(gfx) = self.gfx.as_mut() {
                            let c = self.vec_shape.on_release(&mut gfx.vec_scene);
                            if c {
                                // Solda os endpoints da forma recém-criada com nós
                                // vizinhos: basta ficarem próximos para se fundirem, e
                                // várias linhas/arcos fecham numa forma (Enio
                                // 2026-07-09). A forma nova ainda não tem entidade
                                // (o sync roda depois), então está na identidade; a
                                // geometria PRÉ-existente nunca se mexe (só a nova
                                // snapa nela). Ao fechar num laço, recebe o fill do
                                // estilo atual — como uma região desenhada pela pen.
                                if let Some(new_id) = self.vec_shape.selected() {
                                    let fill = self.vec_pen.style().fill;
                                    let fill_on_close =
                                        (fill.a != 0).then(|| ph2d_vec_scene::Paint::solid(fill));
                                    let xforms =
                                        crate::vec_transform::build(&gfx.sim, &self.vec_entities);
                                    let win = gfx.surface.size();
                                    let tol =
                                        crate::vec_gizmo_view::stroke_hit_r(&gfx.camera, win) * 1.5;
                                    gfx.vec_scene.weld_new_shape(
                                        new_id,
                                        &xforms,
                                        tol,
                                        fill_on_close,
                                    );
                                }
                                self.vec_history.commit_if_changed(&gfx.vec_scene);
                            } else {
                                self.vec_history.cancel();
                            }
                            c
                        } else {
                            false
                        };
                        if committed {
                            // Seleciona a forma nova para edição imediata — a menos que
                            // o weld a tenha fundido noutro objeto (o id sumiu).
                            let sel = self.vec_shape.selected().filter(|id| {
                                self.gfx.as_ref().is_some_and(|g| {
                                    g.vec_scene.paths().iter().any(|p| p.id == *id)
                                })
                            });
                            self.vec_pen.select(sel);
                        }
                        return;
                    }
                    // Shape mode but no active drag → fall through to chrome so the
                    // panel buttons receive their Up.
                }
                (ph2d_host::PointerButton::Secondary, PointerKind::Down) if on_canvas => {
                    if shape_kind_for_mode(self.vec_draw_config.mode).is_none() {
                        self.vec_pen.finish();
                    } else {
                        if let Some(gfx) = self.gfx.as_mut() {
                            self.vec_shape.cancel(&mut gfx.vec_scene);
                        }
                        self.vec_history.cancel();
                    }
                    return;
                }
                _ => {}
            }
        }

        // Painter layers drag-reparent (W3 T3.8): the dispatch emits a
        // PainterLayerReparent on Up of an active layer-row drag; route it to
        // the active PainterTool, which reverses NodeId→LayerId and applies
        // move_into_group / reorder. The concrete-tool downcast lives in the
        // allowlisted painter bridge so central dispatch stays downcast-free
        // (architecture_no_downcast_to_concrete_tool_in_shell gate).
        if let Some((dragged, drop)) = forward_to_hero(self.gfx.as_mut(), evt)
            && let Some(gfx) = self.gfx.as_mut()
        {
            crate::render_loop::painter_bridge_queries::apply_layer_reparent(
                &mut gfx.tools,
                dragged,
                drop,
            );
        }

        // BgRemoval eyedropper (SHELL-only). A Secondary Down on an
        // extra-colour swatch deletes it; a Primary Down/drag over the
        // sprite samples colours. Both consume the event so the normal
        // canvas/gizmo/context-menu logic below does not run.
        // Fill (Bucket) ColorDrop: a Primary Down on the Fill rail button arms the drag-to-canvas gesture
        // + activates Fill. Self-gates on the hit id; the normal Up-click still selects the tool when the
        // press is released ON the button, and is suppressed when it drags off (release outside the rect).
        if matches!(mapped_button, ph2d_host::PointerButton::Primary)
            && matches!(kind, PointerKind::Down)
        {
            // A Down on the C&F button arms the ColorDrop drag AND consumes the event — otherwise it fell
            // through to `painter_canvas_down` below and the active shape tool dropped a stray point on the
            // canvas behind the button (Enio 2026-07-03). The rail button's own press/click already ran in
            // `forward_to_hero` above; the picker opens on release, Fill activates only if the drag reaches
            // the canvas.
            if self.arm_fill_drag_if_on_button(evt.x, evt.y) {
                return;
            }
            // A Primary Down on the Fill modal's title band starts a modal-move (the card follows the
            // cursor via CursorMoved) — consume it so it doesn't click through / start anything else.
            if self.arm_fill_modal_drag_if_on_handle(evt.x, evt.y) {
                return;
            }
        }
        match (mapped_button, kind) {
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.try_eyedropper_delete(evt.x, evt.y) =>
            {
                return;
            }
            // Protection brush ERASE: a Secondary Down with the brush armed
            // erases the first dab + starts an erase drag (continued in
            // CursorMoved). Consumes so it doesn't open a context menu.
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.try_protect_erase(evt.x, evt.y) =>
            {
                return;
            }
            // Painter Falloff curve: right-click a control point → open the
            // handle-type menu (Vector / Auto). No-op off a point.
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.painter_falloff_open_point_menu(evt.x, evt.y) =>
            {
                return;
            }
            // On-canvas Curve / Free Hand: right-click a control point → open the
            // handle-kind menu (Free / Aligned / Vector / Auto). No-op off a point.
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.painter_curve_open_point_menu(evt.x, evt.y) =>
            {
                return;
            }
            // On-canvas Line polyline: right-click ENDS point-creation (Blender/CAD
            // convention). No-op when no Line is being drawn (falls through to the
            // context menu).
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.painter_line_finish_points() =>
            {
                return;
            }
            (ph2d_host::PointerButton::Secondary, PointerKind::Up) => {
                // End any erase drag (no-op when not erasing).
                self.end_protect_paint();
            }
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if !menu_open_before && self.try_eyedropper_sample(evt.x, evt.y) =>
            {
                self.eyedropper_dragging = true;
                return;
            }
            // Painter Falloff curve: left-click the empty graph (Custom preset) →
            // add a control point where clicked. A press on a handle falls through
            // (the panel's drag dispatch grabs it); a click on an open context menu
            // is the menu's, not a canvas-add (`menu_open_before`).
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if !menu_open_before && self.painter_falloff_canvas_add(evt.x, evt.y) =>
            {
                return;
            }
            // Protection brush: a Primary Down with the brush armed paints
            // the first dab + starts the drag (drag continues in
            // CursorMoved). Consumes the event so it doesn't pick/move the
            // sprite.
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if !menu_open_before && self.try_protect_paint(evt.x, evt.y) =>
            {
                return;
            }
            // "Add area" automatic selector: a Primary Down with the
            // selector armed runs a single-click flood-fill from the
            // clicked source pixel into the force-remove mask
            // (Enio 2026-05-26). Mirror of the eyedropper sample dispatch.
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if !menu_open_before && self.try_add_area_click(evt.x, evt.y) =>
            {
                return;
            }
            // Colour-picker eyedropper sample: when the picker eyedropper was armed, `forward_to_hero`
            // already sampled the pixel — consume the click so the Painter brush does NOT paint there
            // and the sprite isn't picked/moved. Must precede the painter brush arm below.
            (ph2d_host::PointerButton::Primary, PointerKind::Down) if eyedropper_armed_before => {
                return;
            }
            // Painter brush: a Primary Down with the Painter active + a sprite
            // selected, inside the footprint, starts a stroke (the first dab) and
            // arms the drag (continues in CursorMoved). Consumes the event so it
            // doesn't pick / move the sprite. A click on an open modal / context
            // menu is the menu's (`menu_open_before`) — never a stroke on the
            // canvas below it (Enio 2026-06-24: new-image modal leaked a dab).
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if !menu_open_before && self.painter_canvas_down(evt.x, evt.y, evt.pressure) =>
            {
                return;
            }
            (ph2d_host::PointerButton::Primary, PointerKind::Up) => {
                self.eyedropper_dragging = false;
                self.end_protect_paint();
                // End a Falloff add-drag (no-op when not dragging).
                self.painter_falloff_release();
                // Close an open painter brush stroke (no-op when not painting).
                self.painter_canvas_up();
                // Finish a Fill ColorDrop drag (fill on the canvas, or open the picker for a plain click
                // on the Fill button). No-op when no fill drag is armed.
                self.fill_drag_up();
                // End a Fill "Fill adjust" modal title-band drag. No-op when not dragging the modal.
                self.fill_modal_drag_up();
            }
            _ => {}
        }

        // M14.7 C: gizmo drag begin/end. A Primary Down that lands on
        // a gizmo handle starts a drag (snapshot Transform + cursor
        // world pos); Up clears it. Move handling lives in CursorMoved
        // so every motion event gets the live cursor.
        if mapped_button == ph2d_host::PointerButton::Primary
            && let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            match kind {
                PointerKind::Down => {
                    // Onda 1 hotfix: Shift/Cmd in the canvas ALWAYS means
                    // selection-adjustment. Pre-empt the gizmo-handle /
                    // pivot-tool / canvas-pick cascade so a modifier
                    // click never accidentally opens a scale-handle drag
                    // (gizmo handles overlap the sprite bbox corners —
                    // bare Shift+click was landing on a handle and
                    // entering the `is_specific_handle` branch which
                    // bypasses the canvas pick where toggle lives).
                    let shift_held_early = self.modifiers.shift_key();
                    let cmd_held_early = self.modifiers.super_key() || self.modifiers.control_key();
                    if (shift_held_early || cmd_held_early)
                        && hero.store.panel_at(evt.x, evt.y).is_none()
                        && !menu_open_before
                    {
                        let window_size = gfx.surface.size();
                        let world_pos = gfx.camera.screen_to_world((evt.x, evt.y), window_size);
                        // ADR-0111: as formas vetoriais desenham POR CIMA dos sprites,
                        // então entram na frente da lista do clique-cíclico.
                        let vec_view =
                            crate::vec_entities::view_state(&gfx.sim, &self.vec_entities);
                        let mut hits = crate::vec_gizmo_view::pick_all_at_world(
                            &gfx.sim,
                            &gfx.vec_scene,
                            &vec_view,
                            &self.vec_entities,
                            world_pos,
                            crate::vec_gizmo_view::stroke_hit_r(&gfx.camera, window_size),
                        );
                        hits.extend(ph2d_render::pick_sprites_at_world(
                            gfx.present.world_mut(),
                            world_pos,
                        ));
                        if let Some(bits) = hits.first().copied() {
                            hero.gizmo.toggle_in_selection(bits);
                            let primary = hero.gizmo.selection;
                            if let Some(entry) = resolve_live_entry(gfx.hero_live.as_ref(), primary)
                            {
                                hero.selection = Some(ph2d_editor::HeroSelection {
                                    label: entry.name.clone(),
                                    kind: entry.badge.clone().unwrap_or_else(|| "ENT".to_string()),
                                    world_pos: (0.0, 0.0),
                                });
                            } else if primary.is_none() {
                                hero.selection = None;
                            }
                            self.title_dirty = true;
                            return;
                        }
                        // Modifier on empty canvas → fall through to
                        // existing cascade so a Shift-drag can still
                        // open an additive rubber-band.
                    }
                    let hit_id = hero.hit_index.hit(evt.x, evt.y);
                    let gizmo_kind = hit_id.and_then(ph2d_editor::gizmo_kind_for_id);
                    // Onda 2C: hit_map fills in for handles whose ids
                    // aren't canonical — extras + global. The primary
                    // keeps canonical IDs (matches the legacy
                    // `gizmo_kind_for_id` lookup above so the primary
                    // path runs unchanged when it's the only sprite
                    // selected).
                    let hit_map_entry: Option<ph2d_editor::GizmoHit> =
                        hit_id.and_then(|id| hero.gizmo.gizmo_hit_map.get(&id).copied());
                    let effective_target = hit_map_entry
                        .map(|h| h.target)
                        .unwrap_or(ph2d_editor::GizmoTarget::PrimaryIndividual);
                    let effective_kind = hit_map_entry.map(|h| h.kind).or(gizmo_kind);
                    let is_specific_handle = matches!(
                        effective_kind,
                        Some(ph2d_editor::GizmoDragKind::ScaleCorner { .. })
                            | Some(ph2d_editor::GizmoDragKind::ScaleEdge { .. })
                            | Some(ph2d_editor::GizmoDragKind::Rotate)
                    );
                    // Enio 2026-07-10: uma forma vetorial ABERTA (linha/arco/pen aberto)
                    // tem bbox FINA — o interior "Translate" do gizmo de sprite colapsa
                    // e os handles de scale/rotate cobrem o traço inteiro, roubando o
                    // clique (o hit-walk é back-to-front, handles vencem). Resultado:
                    // arrastar a linha a ESCALAVA em vez de mover, e o snap-ao-mover
                    // (que só dispara num Translate) nunca rodava. Se o cursor está
                    // sobre o TRAÇO de uma forma vetorial aberta, o arrasto é um
                    // Translate dela: pula o branch de handle e cai no canvas-pick.
                    // Handles de quina FORA do traço (arco/linha diagonal) seguem
                    // escalando — a checagem é só do traço.
                    let over_open_vec_stroke = {
                        let window_size = gfx.surface.size();
                        let world_pos = gfx.camera.screen_to_world((evt.x, evt.y), window_size);
                        let vec_view =
                            crate::vec_entities::view_state(&gfx.sim, &self.vec_entities);
                        let hits = crate::vec_gizmo_view::pick_all_at_world(
                            &gfx.sim,
                            &gfx.vec_scene,
                            &vec_view,
                            &self.vec_entities,
                            world_pos,
                            crate::vec_gizmo_view::stroke_hit_r(&gfx.camera, window_size),
                        );
                        hits.first().is_some_and(|&bits| {
                            gfx.sim
                                .world()
                                .get::<ph2d_ecs::VecPathRef>(ph2d_ecs::Entity::from_bits(bits))
                                .is_some_and(|vp| {
                                    gfx.vec_scene
                                        .paths()
                                        .iter()
                                        .any(|p| p.id == vp.0 && !p.closed)
                                })
                        })
                    };
                    // Also recognize Translate from a keyed bbox-interior
                    // hit — clicking the interior of an extra or the global
                    // gizmo should open a group translate via the
                    // `effective_target` route (the canvas-pick path below
                    // skips keyed ids since they aren't None / Translate /
                    // PIVOT canonical, so without this guard those clicks
                    // would fall through to nothing).
                    // Keyed Translate = click on the bbox interior of an
                    // extra or the global gizmo (whose interior IDs are
                    // hashed, so `gizmo_kind_for_id` doesn't recognise
                    // them). Treated as a multi-select translate
                    // through the canvas-pick branch below — that
                    // branch resolves the world position to a sprite
                    // via `pick_sprites_at_world` and opens a group
                    // translate drag.
                    let is_keyed_translate = hit_map_entry
                        .map(|h| matches!(h.kind, ph2d_editor::GizmoDragKind::Translate))
                        .unwrap_or(false);
                    // TOOL_PIVOT begin: when the Pivot transform tool is
                    // the active radio selection and the click lands on
                    // the selected sprite (or its pivot dot), open a
                    // MovePivot drag instead of the pick / scale path.
                    let pivot_tool_active = hero.store.button_state(ph2d_editor::ids::TOOL_PIVOT)
                        == Some(ph2d_editor::widget::ButtonState::Pressed);
                    let mut began_pivot = false;
                    if pivot_tool_active
                        && hero.store.panel_at(evt.x, evt.y).is_none()
                        && !menu_open_before
                        && let Some(entity_bits) = hero.gizmo.selection
                    {
                        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
                        let window_size = gfx.surface.size();
                        let world_pos = gfx.camera.screen_to_world((evt.x, evt.y), window_size);
                        let on_pivot_dot = hit_id == Some(ph2d_editor::gizmo::ids::GIZMO_PIVOT);
                        // ADR-0111: uma forma vetorial também é agarrável pelo interior.
                        let on_object =
                            ph2d_render::pick_sprite_at_world(gfx.present.world_mut(), world_pos)
                                == Some(entity_bits)
                                || crate::vec_gizmo_view::contains_world(
                                    &gfx.sim,
                                    &gfx.vec_scene,
                                    entity,
                                    world_pos,
                                    crate::vec_gizmo_view::stroke_hit_r(&gfx.camera, window_size),
                                );
                        if (on_pivot_dot || on_object)
                            && !ph2d_ecs::is_locked_for_edit(gfx.sim.world(), entity)
                            && let Some(t) = gfx.sim.world().get::<Transform>(entity)
                        {
                            let snap_t = ph2d_editor::TransformSnapshot {
                                translation: [t.translation.x, t.translation.y],
                                rotation: t.rotation,
                                scale: [t.scale.x, t.scale.y],
                            };
                            let pw = ph2d_ecs::parent_world_transform(gfx.sim.world(), entity);
                            let parent_world = ph2d_editor::TransformSnapshot {
                                translation: [pw.translation.x, pw.translation.y],
                                rotation: pw.rotation,
                                scale: [pw.scale.x, pw.scale.y],
                            };
                            let (anchor, half) =
                                gizmo_anchor_half(&gfx.sim, &gfx.vec_scene, entity);
                            // Invariant quad center = pivot + R·(anchor ⊙ scale).
                            let ax = anchor[0] * snap_t.scale[0];
                            let ay = anchor[1] * snap_t.scale[1];
                            // T1.3.5 cross-OS bit-identical.
                            let (sin_r, cos_r) = libm::sincosf(snap_t.rotation);
                            let quad_center = [
                                snap_t.translation[0] + ax * cos_r - ay * sin_r,
                                snap_t.translation[1] + ax * sin_r + ay * cos_r,
                            ];
                            hero.gizmo.drag = Some(ph2d_editor::GizmoDragState {
                                kind: ph2d_editor::GizmoDragKind::MovePivot,
                                entity_bits,
                                start_screen: (evt.x, evt.y),
                                cursor_screen: (evt.x, evt.y),
                                start_transform: snap_t,
                                pivot_world: quad_center,
                                start_cursor_world: world_pos,
                                sprite_half_intrinsic: half,
                                anchor_is_center: false,
                                target: ph2d_editor::GizmoTarget::PrimaryIndividual,
                                parent_world,
                            });
                            began_pivot = true;
                        }
                    }
                    if began_pivot {
                        // MovePivot drag opened; Move events drive it.
                    } else if is_specific_handle
                        && !over_open_vec_stroke
                        && let Some(gkind) = effective_kind
                        && let Some(entity_bits) = match effective_target {
                            ph2d_editor::GizmoTarget::ExtraIndividual(bits) => Some(bits),
                            _ => hero.gizmo.selection,
                        }
                    {
                        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
                        // 2026-05-26 — bloqueia drag se entidade tem
                        // `Locked` OU ancestral tem `GroupedChildren`.
                        if ph2d_ecs::is_locked_for_edit(gfx.sim.world(), entity) {
                            return;
                        }
                        let window_size = gfx.surface.size();
                        let start_world = gfx.camera.screen_to_world((evt.x, evt.y), window_size);
                        if let Some(t) = gfx.sim.world().get::<Transform>(entity) {
                            let snap = ph2d_editor::TransformSnapshot {
                                translation: [t.translation.x, t.translation.y],
                                rotation: t.rotation,
                                scale: [t.scale.x, t.scale.y],
                            };
                            // Enio 2026-05-26 fix: capture parent's world
                            // transform so compute_gizmo_transform can
                            // unrotate/unscale the delta before writing
                            // back to the entity's LOCAL Transform.
                            let pw = ph2d_ecs::parent_world_transform(gfx.sim.world(), entity);
                            let parent_world = ph2d_editor::TransformSnapshot {
                                translation: [pw.translation.x, pw.translation.y],
                                rotation: pw.rotation,
                                scale: [pw.scale.x, pw.scale.y],
                            };
                            let use_center_anchor =
                                self.modifiers.control_key() || self.modifiers.super_key();
                            let sprite_half_intrinsic =
                                gizmo_anchor_half(&gfx.sim, &gfx.vec_scene, entity).1;
                            // Onda 2C: pivot world depends on target.
                            // PrimaryIndividual / ExtraIndividual use the
                            // sprite's own anchor (transforms local to it).
                            // Global overrides pivot to the global bbox
                            // center so group transforms rotate/scale every
                            // sprite around a single shared point.
                            let pivot = if let ph2d_editor::GizmoTarget::Global = effective_target
                                && let Some(gv) = hero.gizmo.global_view.as_ref()
                            {
                                [
                                    (gv.bbox_min_world[0] + gv.bbox_max_world[0]) * 0.5,
                                    (gv.bbox_min_world[1] + gv.bbox_max_world[1]) * 0.5,
                                ]
                            } else {
                                // Composição parent×local pra que o
                                // pivot world seja correto mesmo com pai
                                // rotacionado/escalonado (Enio 2026-05-26
                                // fix: child de pai rotacionado tinha
                                // pivot calculado como root).
                                let world_snap = ph2d_editor::compose_snapshot(parent_world, snap);
                                ph2d_editor::anchor_pivot_world(
                                    gkind,
                                    sprite_half_intrinsic,
                                    world_snap,
                                    use_center_anchor,
                                )
                            };
                            // Onda 2 polish: capture the global view at
                            // drag start so snapshots::publish can keep
                            // the global gizmo's visual orientation /
                            // scale in lockstep with the live group
                            // transform (otherwise it would be the
                            // axis-aligned union of rotated sprites,
                            // which grows during rotation instead of
                            // rotating).
                            if matches!(effective_target, ph2d_editor::GizmoTarget::Global) {
                                hero.gizmo.global_view_start = hero.gizmo.global_view;
                            } else {
                                hero.gizmo.global_view_start = None;
                            }
                            hero.gizmo.drag = Some(ph2d_editor::GizmoDragState {
                                kind: gkind,
                                entity_bits,
                                start_screen: (evt.x, evt.y),
                                cursor_screen: (evt.x, evt.y),
                                start_transform: snap,
                                pivot_world: pivot,
                                start_cursor_world: start_world,
                                sprite_half_intrinsic,
                                anchor_is_center: use_center_anchor,
                                target: effective_target,
                                parent_world,
                            });
                            // Onda 1 + 2C.4: snapshot every OTHER selected
                            // sprite's full start_transform so
                            // advance_gizmo_drag can apply translate /
                            // local-scale / local-rotate / global-scale /
                            // global-rotate to the whole group. Captured
                            // for ANY drag kind that touches multi-select
                            // (Translate / Scale / Rotate) so the math
                            // branches can fire uniformly later.
                            self.group_drag_starts.clear();
                            if hero.gizmo.selected_len() > 1 {
                                for sel in hero.gizmo.iter_selected() {
                                    if sel == entity_bits {
                                        continue;
                                    }
                                    let e = ph2d_ecs::Entity::from_bits(sel);
                                    if let Some(t) = gfx.sim.world().get::<Transform>(e) {
                                        let epw =
                                            ph2d_ecs::parent_world_transform(gfx.sim.world(), e);
                                        self.group_drag_starts.push(
                                            crate::app_state::GroupDragSnapshot {
                                                entity_bits: sel,
                                                start_transform: ph2d_editor::TransformSnapshot {
                                                    translation: [t.translation.x, t.translation.y],
                                                    rotation: t.rotation,
                                                    scale: [t.scale.x, t.scale.y],
                                                },
                                                parent_world: ph2d_editor::TransformSnapshot {
                                                    translation: [
                                                        epw.translation.x,
                                                        epw.translation.y,
                                                    ],
                                                    rotation: epw.rotation,
                                                    scale: [epw.scale.x, epw.scale.y],
                                                },
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    } else if hero.store.panel_at(evt.x, evt.y).is_none()
                        && !menu_open_before
                        && (hit_id.is_none()
                            || matches!(gizmo_kind, Some(ph2d_editor::GizmoDragKind::Translate))
                            || hit_id == Some(ph2d_editor::gizmo::ids::GIZMO_PIVOT)
                            || is_keyed_translate
                            || over_open_vec_stroke)
                    {
                        // Canvas pick (M14.7 A) — see commit history
                        // for the four conditions enumerated.
                        let window_size = gfx.surface.size();
                        let world_pos = gfx.camera.screen_to_world((evt.x, evt.y), window_size);
                        // ADR-0111: as formas vetoriais desenham POR CIMA dos sprites,
                        // então entram na frente da lista do clique-cíclico.
                        let vec_view =
                            crate::vec_entities::view_state(&gfx.sim, &self.vec_entities);
                        let mut hits = crate::vec_gizmo_view::pick_all_at_world(
                            &gfx.sim,
                            &gfx.vec_scene,
                            &vec_view,
                            &self.vec_entities,
                            world_pos,
                            crate::vec_gizmo_view::stroke_hit_r(&gfx.camera, window_size),
                        );
                        hits.extend(ph2d_render::pick_sprites_at_world(
                            gfx.present.world_mut(),
                            world_pos,
                        ));
                        // Uma forma ABERTA (linha/arco) não é pega pelo interior — só
                        // pelo traço. Mas clicar no INTERIOR do gizmo dela (hit
                        // Translate) É o pedido de mover: a bbox inteira é área de
                        // arrasto, como num sprite. Sem nada sob o cursor, cai na
                        // seleção atual (Enio 2026-07-09).
                        if hits.is_empty()
                            && matches!(gizmo_kind, Some(ph2d_editor::GizmoDragKind::Translate))
                            && let Some(sel) = hero.gizmo.selection
                        {
                            hits.push(sel);
                        }
                        let same_list = !hits.is_empty() && hits == self.cycle_pick_hits;
                        if !same_list {
                            self.cycle_pick_world = Some(world_pos);
                            self.cycle_pick_hits = hits.clone();
                            self.cycle_pick_idx = 0;
                            self.cycle_pick_count = 1;
                        } else {
                            self.cycle_pick_count = self.cycle_pick_count.saturating_add(1);
                            if self.cycle_pick_count.is_multiple_of(2) {
                                // Even count → selection stays.
                            } else if !hits.is_empty() {
                                self.cycle_pick_idx = (self.cycle_pick_idx + 1) % hits.len();
                            }
                        }
                        let picked = if hits.is_empty() {
                            // No sprite under the cursor. (The old vector-scene
                            // object pick fell back here; retired with ADR-0108.)
                            None
                        } else {
                            hits.get(self.cycle_pick_idx).copied()
                        };
                        // Fase 0d: read modifier state at click time —
                        // Shift adds to the selection, Cmd/Ctrl toggles,
                        // bare click replaces (legacy default). Modifier
                        // clicks skip drag setup since the user is
                        // adjusting selection, not moving sprites.
                        let shift_held = self.modifiers.shift_key();
                        let cmd_held = self.modifiers.super_key() || self.modifiers.control_key();
                        // Smart-click preservation: bare click on a
                        // sprite that's already inside an active multi-
                        // selection KEEPS the whole set (user intends
                        // to interact with the group — e.g. drag the
                        // group or run a tool — not collapse to single).
                        let preserves_multi = picked.is_some_and(|bits| {
                            hero.gizmo.selected_len() > 1 && hero.gizmo.is_selected(bits)
                        });
                        // Drag-setup skip: modifier clicks adjust the
                        // selection but should not start a gizmo drag
                        // (the user is curating, not moving). Bare-
                        // click in a multi-selection DOES start a drag
                        // (group translate via the clicked sprite as
                        // pivot, Onda 1).
                        let is_modifier_click = picked.is_some() && (shift_held || cmd_held);
                        if let Some(bits) = picked {
                            if cmd_held || shift_held {
                                // Onda 1: unify Shift + Cmd as toggle on
                                // the canvas. Click on a sprite already
                                // in the selection → removes JUST that
                                // one. Click on a sprite outside → adds.
                                // The Hierarchy panel keeps Shift = range
                                // (list-style UX); the canvas has no
                                // natural linear order, so toggle is the
                                // sane semantic for both modifiers.
                                hero.gizmo.toggle_in_selection(bits);
                            } else if preserves_multi {
                                // Onda 2 hotfix: bare click on a sprite
                                // already in the multi-selection DEFERS
                                // the decision to PointerUp. If the user
                                // drags from here, the open Translate
                                // drag becomes a group translate (Onda 1
                                // semantics preserved). If they release
                                // without dragging, Up `replace_selection`
                                // collapses the multi to just this sprite
                                // (Enio: "se há multiplas sprites
                                // selecionas e eu clicar com botão
                                // esquerdo em uma delas, todas as outras
                                // devem ser desselecionadas").
                                self.pending_single_replace = Some((bits, (evt.x, evt.y)));
                            } else {
                                hero.gizmo.replace_selection(Some(bits));
                            }
                        } else {
                            // Empty click — Fase 0f: defer to PointerKind::Up
                            // so we can distinguish "bare click on empty"
                            // (= clear selection) from "start of a rubber-
                            // band box-select drag" (= keep selection
                            // until release, then resolve against the
                            // dragged rect). Cmd on empty stays a no-op
                            // (preserves built-up multi-selection). Shift
                            // on empty starts an additive rubber-band.
                            if !cmd_held {
                                self.rubber_band = Some(crate::app_state::RubberBandState {
                                    anchor_screen: (evt.x, evt.y),
                                    current_screen: (evt.x, evt.y),
                                    add_mode: shift_held,
                                });
                            }
                        }
                        if let Some(bits) = picked
                            && !is_modifier_click
                        {
                            let entity = ph2d_ecs::Entity::from_bits(bits);
                            if !ph2d_ecs::is_locked_for_edit(gfx.sim.world(), entity)
                                && let Some(t) = gfx.sim.world().get::<Transform>(entity)
                            {
                                let snap_t = ph2d_editor::TransformSnapshot {
                                    translation: [t.translation.x, t.translation.y],
                                    rotation: t.rotation,
                                    scale: [t.scale.x, t.scale.y],
                                };
                                let pw = ph2d_ecs::parent_world_transform(gfx.sim.world(), entity);
                                let parent_world = ph2d_editor::TransformSnapshot {
                                    translation: [pw.translation.x, pw.translation.y],
                                    rotation: pw.rotation,
                                    scale: [pw.scale.x, pw.scale.y],
                                };
                                let pivot = [t.translation.x, t.translation.y];
                                hero.gizmo.drag = Some(ph2d_editor::GizmoDragState {
                                    kind: ph2d_editor::GizmoDragKind::Translate,
                                    entity_bits: bits,
                                    start_screen: (evt.x, evt.y),
                                    cursor_screen: (evt.x, evt.y),
                                    start_transform: snap_t,
                                    pivot_world: pivot,
                                    start_cursor_world: world_pos,
                                    sprite_half_intrinsic: [0.0, 0.0],
                                    anchor_is_center: false,
                                    target: ph2d_editor::GizmoTarget::PrimaryIndividual,
                                    parent_world,
                                });
                                // Onda 1 + 2C.4: snapshot every OTHER
                                // selected sprite's full start_transform
                                // (skip the drag's own primary — its
                                // snapshot lives on GizmoDragState).
                                // Canvas pick always opens a Translate
                                // drag; the same snapshots also feed
                                // future scale/rotate handles on extras +
                                // global (advance_gizmo_drag dispatches
                                // by drag.kind + drag.target).
                                self.group_drag_starts.clear();
                                if hero.gizmo.selected_len() > 1 {
                                    for sel in hero.gizmo.iter_selected() {
                                        if sel == bits {
                                            continue;
                                        }
                                        let e = ph2d_ecs::Entity::from_bits(sel);
                                        if let Some(t) = gfx.sim.world().get::<Transform>(e) {
                                            let epw = ph2d_ecs::parent_world_transform(
                                                gfx.sim.world(),
                                                e,
                                            );
                                            self.group_drag_starts.push(
                                                crate::app_state::GroupDragSnapshot {
                                                    entity_bits: sel,
                                                    start_transform:
                                                        ph2d_editor::TransformSnapshot {
                                                            translation: [
                                                                t.translation.x,
                                                                t.translation.y,
                                                            ],
                                                            rotation: t.rotation,
                                                            scale: [t.scale.x, t.scale.y],
                                                        },
                                                    parent_world: ph2d_editor::TransformSnapshot {
                                                        translation: [
                                                            epw.translation.x,
                                                            epw.translation.y,
                                                        ],
                                                        rotation: epw.rotation,
                                                        scale: [epw.scale.x, epw.scale.y],
                                                    },
                                                },
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        // ADR-0029 Phase C.2: live entries owned by the
                        // Hierarchy panel crate; reach via the public
                        // thread-local snapshot. With multi-select the
                        // label mirrors the primary; the count is
                        // surfaced via hero.gizmo.selected_len() at
                        // paint time (Fase 0e polish).
                        let primary = hero.gizmo.selection;
                        if let Some(entry) = resolve_live_entry(gfx.hero_live.as_ref(), primary) {
                            hero.selection = Some(ph2d_editor::HeroSelection {
                                label: entry.name.clone(),
                                kind: entry.badge.clone().unwrap_or_else(|| "ENT".to_string()),
                                world_pos: (0.0, 0.0),
                            });
                        } else if primary.is_none() {
                            hero.selection = None;
                        }
                        self.title_dirty = true;
                    }
                }
                PointerKind::Up => {
                    // Fase 0f: resolve the rubber-band rect — pick every
                    // sprite whose world bbox intersects, then apply
                    // replace or add depending on `add_mode` (Shift held
                    // at Down). A click that didn't drift more than 4 px
                    // is treated as a bare click on empty: clear
                    // selection if !add_mode, else preserve.
                    if let Some(rb) = self.rubber_band.take() {
                        let dx = rb.current_screen.0 - rb.anchor_screen.0;
                        let dy = rb.current_screen.1 - rb.anchor_screen.1;
                        let moved = (dx * dx + dy * dy) > 16.0; // > 4 px
                        if moved {
                            let window_size = gfx.surface.size();
                            let world_a = gfx.camera.screen_to_world(rb.anchor_screen, window_size);
                            let world_b =
                                gfx.camera.screen_to_world(rb.current_screen, window_size);
                            let rmin = [world_a[0].min(world_b[0]), world_a[1].min(world_b[1])];
                            let rmax = [world_a[0].max(world_b[0]), world_a[1].max(world_b[1])];
                            let vec_view =
                                crate::vec_entities::view_state(&gfx.sim, &self.vec_entities);
                            let mut bits = crate::vec_gizmo_view::pick_in_world_rect(
                                &gfx.sim,
                                &gfx.vec_scene,
                                &vec_view,
                                &self.vec_entities,
                                rmin,
                                rmax,
                            );
                            bits.extend(ph2d_render::pick_sprites_in_world_rect(
                                gfx.present.world_mut(),
                                rmin,
                                rmax,
                            ));
                            if !rb.add_mode {
                                hero.gizmo.clear_all_selection();
                            }
                            for b in bits {
                                hero.gizmo.add_to_selection(b);
                            }
                            // Sync the panel header label to the new
                            // primary (Fase 0e parity).
                            let primary = hero.gizmo.selection;
                            if let Some(entry) = resolve_live_entry(gfx.hero_live.as_ref(), primary)
                            {
                                hero.selection = Some(ph2d_editor::HeroSelection {
                                    label: entry.name.clone(),
                                    kind: entry.badge.clone().unwrap_or_else(|| "ENT".to_string()),
                                    world_pos: (0.0, 0.0),
                                });
                            } else if primary.is_none() {
                                hero.selection = None;
                            }
                            self.title_dirty = true;
                        } else if !rb.add_mode {
                            // Bare click on empty = clear selection.
                            hero.gizmo.clear_all_selection();
                            hero.selection = None;
                            self.title_dirty = true;
                        }
                    }
                    // Onda 2 hotfix: resolve a pending click-vs-drag
                    // decision. `pending_single_replace` is Some when
                    // the user Down'd on a multi-selected sprite. If
                    // the cursor stayed within ~4 px of the Down point
                    // until now (a click, not a drag), collapse the
                    // multi-selection to just that sprite. If it moved
                    // past the threshold, the open Translate drag has
                    // already group-translated the selection; just
                    // clear the pending state.
                    if let Some((bits, (dx0, dy0))) = self.pending_single_replace.take() {
                        let dx = evt.x - dx0;
                        let dy = evt.y - dy0;
                        // 12 px tolerance — trackpads have micro
                        // tremor and acceleration that can move
                        // the cursor a few px even on what feels
                        // like a stationary click.
                        if (dx * dx + dy * dy) <= 144.0 {
                            hero.gizmo.replace_selection(Some(bits));
                            // Sync the panel header label to the new
                            // primary so the Hierarchy highlight
                            // matches the canvas immediately.
                            if let Some(entry) =
                                resolve_live_entry(gfx.hero_live.as_ref(), Some(bits))
                            {
                                hero.selection = Some(ph2d_editor::HeroSelection {
                                    label: entry.name.clone(),
                                    kind: entry.badge.clone().unwrap_or_else(|| "ENT".to_string()),
                                    world_pos: (0.0, 0.0),
                                });
                            }
                            self.title_dirty = true;
                        }
                    }
                    // Drop the drag — Transform is already committed
                    // up to the latest Move position.
                    let ended_drag = hero.gizmo.drag;
                    hero.gizmo.drag = None;
                    // Snap vetorial ao MOVER (Enio 2026-07-10): um Translate que
                    // acabou de reposicionar formas vetoriais ABERTAS solda as pontas
                    // que ficaram perto de nós vizinhos — o mesmo weld da criação. A
                    // forma movida cede o endpoint; a vizinha é sacrossanta. Formas
                    // fechadas não têm endpoints, então passam intactas.
                    // Qualquer manipulação da forma (mover / escalar / rotacionar) que
                    // aproxime as pontas deve soldar — só o MovePivot (que mexe no pivô,
                    // não na forma) fica de fora. O `rigid_snap_delta` só desliza para o
                    // encaixe, então serve de ajuste fino pós-scale/rotate também.
                    if ended_drag
                        .is_some_and(|d| !matches!(d.kind, ph2d_editor::GizmoDragKind::MovePivot))
                    {
                        let mut moved_bits = vec![ended_drag.unwrap().entity_bits];
                        moved_bits.extend(self.group_drag_starts.iter().map(|s| s.entity_bits));
                        let moved_ids: Vec<_> = moved_bits
                            .iter()
                            .filter_map(|&b| {
                                gfx.sim
                                    .world()
                                    .get::<ph2d_ecs::VecPathRef>(ph2d_ecs::Entity::from_bits(b))
                                    .map(|v| v.0)
                            })
                            .collect();
                        if !moved_ids.is_empty() {
                            let fill = self.vec_pen.style().fill;
                            let fill_on_close =
                                (fill.a != 0).then(|| ph2d_vec_scene::Paint::solid(fill));
                            let win = gfx.surface.size();
                            let tol = crate::vec_gizmo_view::stroke_hit_r(&gfx.camera, win) * 1.5;
                            let mut welded = false;
                            for id in moved_ids {
                                // Recomputa os xforms a cada solda (uma fusão muda o
                                // scene) e pula o que já foi consumido por outra.
                                if !gfx.vec_scene.paths().iter().any(|p| p.id == id) {
                                    continue;
                                }
                                let Some(bits) = self.vec_entities.get(&id).copied() else {
                                    continue;
                                };
                                let closed =
                                    gfx.vec_scene.paths().iter().any(|p| p.id == id && p.closed);
                                // Alinhamento final (X/Y independentes: bordas / centros /
                                // vértices) — vale p/ toda forma, o mesmo snap do arraste.
                                let xforms =
                                    crate::vec_transform::build(&gfx.sim, &self.vec_entities);
                                let align = gfx.vec_scene.align_snap_delta(id, &xforms, tol);
                                vec_snap::slide_entity_world(&mut gfx.sim, bits, align);
                                // Só a forma ABERTA funde: encaixa a PONTA num endpoint
                                // vizinho (RÍGIDO, sem distorcer) e solda — a ponta já
                                // coincide, uma segunda ponta (fecho) cede o mínimo.
                                if !closed {
                                    let xforms =
                                        crate::vec_transform::build(&gfx.sim, &self.vec_entities);
                                    if let Some(rd) =
                                        gfx.vec_scene.rigid_snap_delta(id, &xforms, tol)
                                    {
                                        vec_snap::slide_entity_world(&mut gfx.sim, bits, rd);
                                    }
                                    let xforms =
                                        crate::vec_transform::build(&gfx.sim, &self.vec_entities);
                                    if gfx.vec_scene.weld_new_shape(
                                        id,
                                        &xforms,
                                        tol,
                                        fill_on_close.clone(),
                                    ) > 0
                                    {
                                        welded = true;
                                    }
                                }
                            }
                            if welded {
                                self.vec_history.commit_if_changed(&gfx.vec_scene);
                            }
                        }
                    }
                    // Onda 1: release the group-translate snapshot so
                    // the next single-select drag doesn't accidentally
                    // pull stale extras along.
                    self.group_drag_starts.clear();
                    // Onda 2 polish: release the global drag-start view
                    // so snapshots::publish reverts to the live-union
                    // computation for the next frame.
                    hero.gizmo.global_view_start = None;
                }
                _ => {}
            }
        }
        // M14.4b.bis: middle button = camera pan anchor. Tracked here
        // so CursorMoved can drive the pan. Motion Nodes M1 / timeline W2.E6:
        // NOT over the graph or the timeline dock — there middle-drag pans that
        // editor (via its own surface gesture), not the camera underneath. The
        // hovered component owns the pan, Blender-style.
        let over_pan_editor = self.cursor_over_motion_graph() || self.cursor_over_timeline();
        if button == MouseButton::Middle && !(over_pan_editor && state == ElementState::Pressed) {
            match state {
                ElementState::Pressed => {
                    self.pan_anchor = Some(self.last_pointer);
                }
                ElementState::Released => {
                    self.pan_anchor = None;
                }
            }
        }
        match state {
            ElementState::Pressed => {
                // Mirror-sidebar chip takes precedence over the panel
                // hit-test (different zone, no overlap).
                let mut consumed = false;
                if let Some(gfx) = self.gfx.as_mut()
                    && !gfx.zen.is_active()
                    && let Some(btn) = gfx.layout.mirror_button_rect()
                    && btn.contains(self.last_pointer.0, self.last_pointer.1)
                {
                    gfx.layout.mirror_sidebar();
                    gfx.toasts.push(Toast::info(format!(
                        "Sidebar · {:?}",
                        gfx.layout.sidebar_side
                    )));
                    self.title_dirty = true;
                    consumed = true;
                }
                // Tool palette icon click — switch active tool.
                //
                // CRITICAL: only hit-test the palette where it is actually
                // PAINTED — the legacy no-hero (demo) path. In the editor
                // (`hero_screen` is `Some`) the palette is NOT painted (the
                // editor switches tools via the LeftRail + Image Tools
                // pills), yet this hit-test used to run unconditionally.
                // Zone::TopRight is the right HALF of the toolbar strip —
                // exactly where the TopBar paints its right clusters incl.
                // the Settings gear — so a click on "Config" also landed on
                // an INVISIBLE palette slot and silently switched tools
                // ("Tool · Move"/"Tool · Padding"). Gating on
                // `hero_screen.is_none()` (the paint condition) makes the
                // top-right belong solely to the TopBar in the editor.
                //
                // The visible-tools filter below still applies in the demo
                // path so its indices match the paint mapping (no drift).
                if !consumed
                    && let Some(gfx) = self.gfx.as_mut()
                    && !gfx.zen.is_active()
                    && gfx.hero_screen.is_none()
                {
                    let mode_on = gfx
                        .hero_screen
                        .as_ref()
                        .map(|h| h.image_edit.mode_on)
                        .unwrap_or(false);
                    let visible = crate::palette_visible_tool_indices(&gfx.tools, mode_on);
                    let palette = gfx.layout.tool_palette_rects(visible.len());
                    let hit_idx = palette
                        .iter()
                        .position(|r| r.contains(self.last_pointer.0, self.last_pointer.1));
                    if let Some(slot) = hit_idx {
                        let tool_idx = visible[slot];
                        let tool_id = gfx.tools.tools()[tool_idx].id();
                        let tool_label = gfx.tools.tools()[tool_idx].label().to_string();
                        if gfx.tools.set_active(&tool_id) {
                            gfx.toasts.push(Toast::info(format!("Tool · {tool_label}")));
                            self.title_dirty = true;
                        }
                        consumed = true;
                    }
                }
                if !consumed {
                    // Mouse down — start hit-test against active panel.
                    self.dispatch_panel_pointer(self.last_pointer.0, self.last_pointer.1, true);
                }
            }
            ElementState::Released => {
                // End any drag-in-progress.
                self.dragging = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        VecPathShapeOp, VecTransformField, apply_vec_path_shape, apply_vec_transform,
        shape_kind_for_mode, shape_up_consumes, vec_bool_op_for_id, vec_flip_for_id,
        vec_path_shape_for_id, vec_reorder_for_id, vec_rotate_for_id, vec_transform_field_for_id,
        vec_vertex_kind_for_id,
    };
    use ph2d_tool_vector::DrawMode;
    use ph2d_vec_boolean::BoolOp;
    use ph2d_vec_edit::ShapeKind;
    use ph2d_vec_scene::{FlipAxis, Rotate90, VertexKind, ZOrder};

    #[test]
    fn vertex_button_ids_map_to_their_kinds() {
        assert_eq!(
            vec_vertex_kind_for_id(ph2d_editor::ids::VECTOR_VERT_CORNER),
            Some(VertexKind::Corner)
        );
        assert_eq!(
            vec_vertex_kind_for_id(ph2d_editor::ids::VECTOR_VERT_SMOOTH),
            Some(VertexKind::Smooth)
        );
        assert_eq!(
            vec_vertex_kind_for_id(ph2d_editor::ids::VECTOR_VERT_SYMMETRIC),
            Some(VertexKind::Symmetric)
        );
        assert_eq!(
            vec_vertex_kind_for_id(ph2d_editor::ids::VECTOR_BOOL_UNION),
            None
        );
    }

    #[test]
    fn arrange_button_ids_map_to_their_zorder() {
        assert_eq!(
            vec_reorder_for_id(ph2d_editor::ids::VECTOR_ARRANGE_TO_BACK),
            Some(ZOrder::ToBack)
        );
        assert_eq!(
            vec_reorder_for_id(ph2d_editor::ids::VECTOR_ARRANGE_BACKWARD),
            Some(ZOrder::Lower)
        );
        assert_eq!(
            vec_reorder_for_id(ph2d_editor::ids::VECTOR_ARRANGE_FORWARD),
            Some(ZOrder::Raise)
        );
        assert_eq!(
            vec_reorder_for_id(ph2d_editor::ids::VECTOR_ARRANGE_TO_FRONT),
            Some(ZOrder::ToFront)
        );
        // Duplicate is NOT a reorder (handled separately), nor any non-Arrange id.
        assert_eq!(
            vec_reorder_for_id(ph2d_editor::ids::VECTOR_ARRANGE_DUPLICATE),
            None
        );
        assert_eq!(
            vec_reorder_for_id(ph2d_editor::ids::VECTOR_BOOL_UNION),
            None
        );
    }

    #[test]
    fn flip_button_ids_map_to_their_axis() {
        assert_eq!(
            vec_flip_for_id(ph2d_editor::ids::VECTOR_ARRANGE_FLIP_H),
            Some(FlipAxis::Horizontal)
        );
        assert_eq!(
            vec_flip_for_id(ph2d_editor::ids::VECTOR_ARRANGE_FLIP_V),
            Some(FlipAxis::Vertical)
        );
        // Flip is NOT a reorder and vice-versa.
        assert_eq!(
            vec_flip_for_id(ph2d_editor::ids::VECTOR_ARRANGE_TO_BACK),
            None
        );
        assert_eq!(
            vec_reorder_for_id(ph2d_editor::ids::VECTOR_ARRANGE_FLIP_H),
            None
        );
    }

    #[test]
    fn rotate_button_ids_map_to_their_direction() {
        assert_eq!(
            vec_rotate_for_id(ph2d_editor::ids::VECTOR_ARRANGE_ROTATE_CW),
            Some(Rotate90::Cw)
        );
        assert_eq!(
            vec_rotate_for_id(ph2d_editor::ids::VECTOR_ARRANGE_ROTATE_CCW),
            Some(Rotate90::Ccw)
        );
        assert_eq!(
            vec_rotate_for_id(ph2d_editor::ids::VECTOR_ARRANGE_FLIP_H),
            None
        );
        assert_eq!(
            vec_flip_for_id(ph2d_editor::ids::VECTOR_ARRANGE_ROTATE_CW),
            None
        );
    }

    #[test]
    fn transform_fields_map_and_apply_translates_and_scales() {
        use ph2d_vec_scene::{VecScene, rectangle};
        assert_eq!(
            vec_transform_field_for_id(ph2d_editor::ids::VECTOR_TRANSFORM_X),
            Some(VecTransformField::X)
        );
        assert_eq!(
            vec_transform_field_for_id(ph2d_editor::ids::VECTOR_TRANSFORM_H),
            Some(VecTransformField::H)
        );
        assert_eq!(
            vec_transform_field_for_id(ph2d_editor::ids::VECTOR_ARRANGE_FLIP_H),
            None
        );

        let mut scene = VecScene::new();
        let id = scene.push_path(rectangle([0.0, 0.0], [10.0, 4.0]));
        let mut hist = ph2d_vec_edit::History::new();
        let mut pen = ph2d_vec_edit::PenTool::new();
        pen.select(Some(id));

        // X → 5 moves the bbox min; W → 20 doubles the width.
        apply_vec_transform(
            &mut scene,
            &mut hist,
            &pen,
            &ph2d_vec_scene::VecXforms::new(),
            VecTransformField::X,
            5.0,
        );
        assert!((scene.path_bbox(id).unwrap().0[0] - 5.0).abs() < 1e-9);
        apply_vec_transform(
            &mut scene,
            &mut hist,
            &pen,
            &ph2d_vec_scene::VecXforms::new(),
            VecTransformField::W,
            20.0,
        );
        let (lo, hi) = scene.path_bbox(id).unwrap();
        assert!((hi[0] - lo[0] - 20.0).abs() < 1e-9, "W set to 20");
        assert!((lo[0] - 5.0).abs() < 1e-9, "min x pinned during scale");
    }

    #[test]
    fn path_shape_ids_map_and_apply_smooths_then_sharpens() {
        use ph2d_vec_scene::{VertexKind, regular_polygon};
        assert_eq!(
            vec_path_shape_for_id(ph2d_editor::ids::VECTOR_PATH_SMOOTH),
            Some(VecPathShapeOp::Smooth)
        );
        assert_eq!(
            vec_path_shape_for_id(ph2d_editor::ids::VECTOR_PATH_SHARPEN),
            Some(VecPathShapeOp::Sharpen)
        );
        assert_eq!(
            vec_path_shape_for_id(ph2d_editor::ids::VECTOR_PATH_SIMPLIFY),
            Some(VecPathShapeOp::Simplify)
        );
        assert_eq!(
            vec_path_shape_for_id(ph2d_editor::ids::VECTOR_PATH_SUBDIVIDE),
            Some(VecPathShapeOp::Subdivide)
        );
        assert_eq!(
            vec_path_shape_for_id(ph2d_editor::ids::VECTOR_ARRANGE_FLIP_H),
            None
        );

        let mut scene = ph2d_vec_scene::VecScene::new();
        let id = scene.push_path(regular_polygon([0.0, 0.0], 5.0, 5.0, 5));
        let mut hist = ph2d_vec_edit::History::new();
        let mut pen = ph2d_vec_edit::PenTool::new();
        pen.select(Some(id));

        apply_vec_path_shape(&mut scene, &mut hist, &pen, VecPathShapeOp::Smooth);
        assert!(
            scene.paths()[0]
                .verts
                .iter()
                .all(|v| v.kind == VertexKind::Smooth),
            "smooth button curves every vertex"
        );
        apply_vec_path_shape(&mut scene, &mut hist, &pen, VecPathShapeOp::Sharpen);
        assert!(
            scene.paths()[0]
                .verts
                .iter()
                .all(|v| v.kind == VertexKind::Corner && v.in_handle == v.anchor),
            "sharpen button flattens every vertex"
        );

        // Simplify: a closed square with a redundant midpoint on one edge drops it.
        let sq = scene.push_path(ph2d_vec_scene::VecPath {
            verts: vec![
                ph2d_vec_scene::VecVertex::corner([0.0, 0.0]),
                ph2d_vec_scene::VecVertex::corner([5.0, 0.0]), // redundant midpoint
                ph2d_vec_scene::VecVertex::corner([10.0, 0.0]),
                ph2d_vec_scene::VecVertex::corner([10.0, 10.0]),
                ph2d_vec_scene::VecVertex::corner([0.0, 10.0]),
            ],
            closed: true,
            ..ph2d_vec_scene::VecPath::default()
        });
        pen.select(Some(sq));
        let before = scene
            .paths()
            .iter()
            .find(|p| p.id == sq)
            .unwrap()
            .verts
            .len();
        apply_vec_path_shape(&mut scene, &mut hist, &pen, VecPathShapeOp::Simplify);
        let after = scene
            .paths()
            .iter()
            .find(|p| p.id == sq)
            .unwrap()
            .verts
            .len();
        assert_eq!(after, before - 1, "simplify drops the one redundant point");

        // Subdivide: one midpoint per segment (closed ⇒ doubles the vertex count).
        let n = scene
            .paths()
            .iter()
            .find(|p| p.id == sq)
            .unwrap()
            .verts
            .len();
        apply_vec_path_shape(&mut scene, &mut hist, &pen, VecPathShapeOp::Subdivide);
        let n2 = scene
            .paths()
            .iter()
            .find(|p| p.id == sq)
            .unwrap()
            .verts
            .len();
        assert_eq!(n2, n * 2, "subdivide doubles a closed path's vertices");

        // Close/Open toggle flips the selected path's `closed` flag each click.
        let was = scene.paths().iter().find(|p| p.id == sq).unwrap().closed;
        super::apply_vec_toggle_closed(&mut scene, &mut hist, &pen);
        assert_eq!(
            scene.paths().iter().find(|p| p.id == sq).unwrap().closed,
            !was,
            "toggle flips closed"
        );
        super::apply_vec_toggle_closed(&mut scene, &mut hist, &pen);
        assert_eq!(
            scene.paths().iter().find(|p| p.id == sq).unwrap().closed,
            was,
            "toggle flips back"
        );
        // Closing a never-filled path seeds a fill so it paints immediately.
        assert!(
            scene
                .paths()
                .iter()
                .find(|p| p.id == sq)
                .unwrap()
                .fill
                .is_some(),
            "closing seeds the Style fill (immediate paint)"
        );
    }

    #[test]
    fn shape_up_only_consumed_while_a_drag_is_live() {
        // Pen mode never consumes via the shape path (the pen path handles it).
        assert!(!shape_up_consumes(DrawMode::Pen, false));
        assert!(!shape_up_consumes(DrawMode::Pen, true));
        // In a shape mode, a live drag consumes the Up (finalize the shape)...
        assert!(shape_up_consumes(DrawMode::Rectangle, true));
        assert!(shape_up_consumes(DrawMode::Polygon, true));
        // ...but with NO active drag the Up must fall through so a panel-button
        // click (mode switch / boolean / close) is not swallowed. This is the
        // exact regression that made every button dead after entering Rect mode.
        assert!(!shape_up_consumes(DrawMode::Rectangle, false));
        assert!(!shape_up_consumes(DrawMode::Ellipse, false));
        assert!(!shape_up_consumes(DrawMode::Polygon, false));
    }

    #[test]
    fn boolean_button_ids_map_to_their_ops() {
        assert_eq!(
            vec_bool_op_for_id(ph2d_editor::ids::VECTOR_BOOL_UNION),
            Some(BoolOp::Union)
        );
        assert_eq!(
            vec_bool_op_for_id(ph2d_editor::ids::VECTOR_BOOL_SUBTRACT),
            Some(BoolOp::Subtract)
        );
        assert_eq!(
            vec_bool_op_for_id(ph2d_editor::ids::VECTOR_BOOL_INTERSECT),
            Some(BoolOp::Intersect)
        );
        assert_eq!(
            vec_bool_op_for_id(ph2d_editor::ids::VECTOR_BOOL_EXCLUDE),
            Some(BoolOp::Exclude)
        );
        // A non-boolean id (a mode button) is not a boolean op.
        assert_eq!(vec_bool_op_for_id(ph2d_editor::ids::VECTOR_MODE_PEN), None);
    }

    #[test]
    fn draw_mode_maps_to_shape_kind_pen_is_none() {
        assert_eq!(shape_kind_for_mode(DrawMode::Pen), None);
        assert_eq!(
            shape_kind_for_mode(DrawMode::Rectangle),
            Some(ShapeKind::Rectangle)
        );
        assert_eq!(
            shape_kind_for_mode(DrawMode::Ellipse),
            Some(ShapeKind::Ellipse)
        );
        assert_eq!(
            shape_kind_for_mode(DrawMode::Polygon),
            Some(ShapeKind::Polygon)
        );
        assert_eq!(shape_kind_for_mode(DrawMode::Star), Some(ShapeKind::Star));
        assert_eq!(
            shape_kind_for_mode(DrawMode::RoundRect),
            Some(ShapeKind::RoundRect)
        );
    }

    // ─── boolean/compound: a costura shell ↔ documento ────────────────────────

    /// Cena com um quadrado externo e outro DENTRO dele, ambos selecionados
    /// (z: externo atrás, interno na frente). Devolve `(scene, pen, ids)`.
    fn nested_selection() -> (ph2d_vec_scene::VecScene, ph2d_vec_edit::PenTool, [u64; 2]) {
        let mut scene = ph2d_vec_scene::VecScene::new();
        let outer = scene.push_path(ph2d_vec_scene::rectangle([0.0, 0.0], [10.0, 10.0]));
        let inner = scene.push_path(ph2d_vec_scene::rectangle([3.0, 3.0], [7.0, 7.0]));
        let mut pen = ph2d_vec_edit::PenTool::default();
        pen.select_many(&[outer, inner]);
        (scene, pen, [outer, inner])
    }

    /// A regressão que motivou o bloco: Subtract agia nas duas últimas regiões
    /// fechadas do DOCUMENTO, ignorando a seleção — e devolvia dois discos
    /// sólidos em vez de uma rosquinha.
    #[test]
    fn boolean_subtract_uses_the_selection_and_makes_a_real_hole() {
        let (mut scene, mut pen, _) = nested_selection();
        // Um terceiro path, NÃO selecionado, bem longe: a booleana antiga o teria
        // agarrado (é uma das duas últimas fechadas); a nova tem de ignorá-lo.
        let bystander = scene.push_path(ph2d_vec_scene::rectangle([90.0, 90.0], [95.0, 95.0]));
        let mut history = ph2d_vec_edit::History::default();

        super::apply_vec_boolean(
            &mut scene,
            &mut history,
            &mut pen,
            &ph2d_vec_scene::VecXforms::new(),
            ph2d_vec_boolean::BoolOp::Subtract,
        );

        assert_eq!(scene.paths().len(), 2, "resultado + o bystander intacto");
        assert!(scene.paths().iter().any(|p| p.id == bystander));
        let donut = scene.paths().iter().find(|p| p.id != bystander).unwrap();
        assert!(donut.is_compound(), "o furo vive num subpath");
        let id = donut.id;
        assert!(scene.path_contains_point(id, [1.0, 5.0]), "o anel é sólido");
        assert!(
            !scene.path_contains_point(id, [5.0, 5.0]),
            "o centro é vazado"
        );
        // O resultado entra na fatia de z da BASE (não salta pro topo).
        assert_eq!(scene.paths()[0].id, id);
        assert_eq!(pen.selected(), Some(id), "a booleana seleciona o resultado");
    }

    #[test]
    fn boolean_needs_two_selected_closed_regions() {
        let (mut scene, mut pen, ids) = nested_selection();
        pen.select(Some(ids[0])); // só um selecionado
        let mut history = ph2d_vec_edit::History::default();
        super::apply_vec_boolean(
            &mut scene,
            &mut history,
            &mut pen,
            &ph2d_vec_scene::VecXforms::new(),
            ph2d_vec_boolean::BoolOp::Union,
        );
        assert_eq!(scene.paths().len(), 2, "no-op");
    }

    /// Make Compound é como o usuário desenha um buraco à mão; Release desfaz.
    #[test]
    fn make_and_release_compound_from_the_selection() {
        let (mut scene, mut pen, ids) = nested_selection();
        let mut history = ph2d_vec_edit::History::default();

        super::apply_vec_compound(&mut scene, &mut history, &mut pen, true);
        assert_eq!(scene.paths().len(), 1);
        assert!(
            !scene.path_contains_point(ids[0], [5.0, 5.0]),
            "virou buraco"
        );
        assert_eq!(pen.selected(), Some(ids[0]));

        super::apply_vec_compound(&mut scene, &mut history, &mut pen, false);
        assert_eq!(scene.paths().len(), 2);
        assert!(
            scene.path_contains_point(ids[0], [5.0, 5.0]),
            "sólido de novo"
        );
        assert_eq!(pen.selected_paths().len(), 2, "base + liberado");
    }

    /// A regra de preenchimento troca o buraco por região sólida, sem tocar a geometria.
    #[test]
    fn fill_rule_toggle_vacates_or_fills_the_hole() {
        let (mut scene, mut pen, ids) = nested_selection();
        let mut history = ph2d_vec_edit::History::default();
        super::apply_vec_compound(&mut scene, &mut history, &mut pen, true);
        assert!(!scene.path_contains_point(ids[0], [5.0, 5.0]));

        super::apply_vec_fill_rule(&mut scene, &mut history, &pen, false); // Non-Zero
        assert!(
            scene.path_contains_point(ids[0], [5.0, 5.0]),
            "NonZero preenche"
        );
        super::apply_vec_fill_rule(&mut scene, &mut history, &pen, true); // Even-Odd
        assert!(
            !scene.path_contains_point(ids[0], [5.0, 5.0]),
            "EvenOdd vaza"
        );
    }
}

/// The double-arrow cursor for a panel-border grip, given its edge bitmask
/// (`TIMELINE_EDGE_*`; a corner sets two bits). Corners point along their own
/// diagonal: the top-left / bottom-right pair is `Nwse` (↖↘), the other `Nesw`.
fn resize_cursor_for_edges(edges: u8) -> winit::window::CursorIcon {
    use ph2d_editor::interaction::{
        TIMELINE_EDGE_B, TIMELINE_EDGE_L, TIMELINE_EDGE_R, TIMELINE_EDGE_T,
    };
    use winit::window::CursorIcon;
    let (l, r) = (edges & TIMELINE_EDGE_L != 0, edges & TIMELINE_EDGE_R != 0);
    let (t, b) = (edges & TIMELINE_EDGE_T != 0, edges & TIMELINE_EDGE_B != 0);
    match (l, r, t, b) {
        (true, _, true, _) | (_, true, _, true) => CursorIcon::NwseResize,
        (_, true, true, _) | (true, _, _, true) => CursorIcon::NeswResize,
        (_, _, true, _) | (_, _, _, true) => CursorIcon::NsResize,
        _ => CursorIcon::EwResize,
    }
}

#[cfg(test)]
mod cursor_tests {
    use super::resize_cursor_for_edges;
    use ph2d_editor::interaction::{
        TIMELINE_EDGE_B, TIMELINE_EDGE_L, TIMELINE_EDGE_R, TIMELINE_EDGE_T,
    };
    use winit::window::CursorIcon;

    #[test]
    fn each_edge_points_across_the_side_it_moves() {
        assert_eq!(
            resize_cursor_for_edges(TIMELINE_EDGE_L),
            CursorIcon::EwResize
        );
        assert_eq!(
            resize_cursor_for_edges(TIMELINE_EDGE_R),
            CursorIcon::EwResize
        );
        assert_eq!(
            resize_cursor_for_edges(TIMELINE_EDGE_T),
            CursorIcon::NsResize
        );
        assert_eq!(
            resize_cursor_for_edges(TIMELINE_EDGE_B),
            CursorIcon::NsResize
        );
    }

    #[test]
    fn each_corner_points_along_its_own_diagonal() {
        let tl = TIMELINE_EDGE_T | TIMELINE_EDGE_L;
        let br = TIMELINE_EDGE_B | TIMELINE_EDGE_R;
        let tr = TIMELINE_EDGE_T | TIMELINE_EDGE_R;
        let bl = TIMELINE_EDGE_B | TIMELINE_EDGE_L;
        assert_eq!(resize_cursor_for_edges(tl), CursorIcon::NwseResize);
        assert_eq!(resize_cursor_for_edges(br), CursorIcon::NwseResize);
        assert_eq!(resize_cursor_for_edges(tr), CursorIcon::NeswResize);
        assert_eq!(resize_cursor_for_edges(bl), CursorIcon::NeswResize);
    }

    #[test]
    fn an_empty_mask_never_shows_a_vertical_arrow() {
        // Defensive: a mask with no bits is a horizontal edge by fallback, not a
        // panic and not a misleading up-down arrow on a left/right grip.
        assert_eq!(resize_cursor_for_edges(0), CursorIcon::EwResize);
    }
}

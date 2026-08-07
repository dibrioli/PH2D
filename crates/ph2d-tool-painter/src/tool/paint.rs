//! Canvas painting — drives the clean-room Blender brush engine (`ph2d-painter-brush`) from pointer
//! samples, writing dabs into the active raster layer's `canvas_rgba`.
//!
//! The brush *behaviour* (falloff, spacing, pressure, blend) is the engine's; this module is only the
//! glue between the editor's [`CanvasPaintTool`] contract (ADR-0040 Amendment 3) and the painter's
//! layer buffers + preview/dirty plumbing.

use super::*;

use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};
use ph2d_painter_brush::{
    AIRBRUSH_RATE_MAX_S, AIRBRUSH_RATE_MIN_S, BrushSpec, Dab, Dynamics, MAX_INPUT_SAMPLES, Stroke,
    StrokeMethod, StrokePoint,
};

mod brush_core_settings; // Falloff/size/spacing/jitter/dash setters; split from `brush_settings` (LOC cap)
mod brush_image; // The imported brush-texture image type (`BrushTextureImage`); split from `brush_settings`
mod brush_settings; // Brush + Stroke-section snapshot & setters (shares `PaintState`'s private brush access)
mod brush_texture_settings; // Grain-texture / Stencil / Dab setters; split from `brush_settings` (LOC cap)
/// The Curve stroke method's on-canvas point editor (submodule, as `brush_settings`).
mod curve;
mod curve_commit; // Apply / Apply & Keep commit verbs for the Curve editor; split from `curve`
mod curve_geom; // flatten (incl. closed seam) + point hit-test/nearest/insert; split from `curve`
mod curve_gizmo; // whole-curve transform gizmo (move/scale/rotate the entire curve); split from `curve`
mod curve_handle; // per-anchor handle kinds (Free/Aligned/Vector/Auto) + derived geometry; split from `curve`
mod curve_join; // corner joins for the offset: smooth-merge / convex-miter / concave-split; split from `curve_offset`
mod curve_model; // shared editing core (points/handles/kinds/selected + ops) — stroke + selection both own one
mod curve_offset; // perpendicular offset (parallel curve) + CAD-grade reconstruction; split from `curve_geom`
mod curve_refit; // Simplify/Merge quality funnel: corner-split + piecewise Schneider least-squares refit
mod curve_tangent; // Bézier tangent-handle hit-test, aligned mirror, overlay snapshot; split from `curve`
mod curve_trim; // self-intersection trim of the offset spine (open + closed); split from `curve_offset`
mod stroke_boolean; // multi-shape Add/Remove boolean composite (rasterise → union/subtract → trace contours)
mod stroke_boolean_raster; // com que PIXELS uma forma entra no composite (sub-janela + rasterizadores)
mod stroke_multi; // multi-shape: parked (inactive-but-editable) stroke shapes + their Operation; pixels are a derived recompose
mod stroke_outline; // o CONTORNO de uma figura -- produtor unico do que o gizmo desenha E do que o clique alcanca
mod stroke_router; // o que um Down SIGNIFICA com varias figuras: editar / reativar / comecar outra
pub use self::stroke_multi::StrokeOpBadge;
pub use self::wetpaint_settings::{WetKnobs, WetTool};
mod impasto; // Impasto: the height channel (paint thickness) — the dab pipeline's SECOND output
mod impasto_ceiling; // Impasto: the glass ceiling — how the paint TOPS OUT (a compression, not a clamp)
mod impasto_fill; // o Fill no Impasto: cor + CORPO, borda no perfil do Falloff (Enio 2026-08-07)
pub mod impasto_gpu; // Impasto: the composed relief, materialised for the GPU light pass
mod impasto_light; // Impasto: the light pass — normal from the height field + Lambert/Blinn-Phong
mod impasto_live; // Impasto: the stroke's commit + the Body card's live re-derivation (LOC split)
mod impasto_material; // Impasto: the paint's MATERIAL on the canvas (deposit + the live re-bake)
mod impasto_settings; // Impasto: section setters + the panel-event route (mirrors watercolor_settings)
mod impasto_settle; // Impasto: the deposit settling under its own weight + the material constants
mod impasto_shade; // Impasto: the RIG + how one pixel is shaded (the optics; its sibling is the plumbing)
mod impasto_tool; // Impasto: the TEN tools that act on the body, as one list (Deposit · Knife · 8 verbs)
mod jitter_settings; // per-dab randomize setters (Jitter Scale / Rotate / Randomize Color); split from `brush_settings`
pub(crate) mod media; // the paint's MEDIUM (Digital / Watercolor / Impasto / Wet Paint) + the door that switches it
mod paint_mode; // the canvas pointer's operation (Paint / Smear / … ) — the MEDIUM's sibling question
mod sculpt; // Sculpt: the MODEL — the five verbs, the two knobs, the routing — `docs/Painter/18…`
mod sculpt_blur; // Sculpt: the kernel (one expression, eight verbs) + the per-tile memo the family reads
mod sculpt_close; // Sculpt: Inflate's footprint is a morphological CLOSING (fill concave, keep convex) — EDT
mod sculpt_filter; // Sculpt: the WHOLE-LAYER filter (W5b) — the same kernel, driven by a uniform `amount`
mod sculpt_inflate; // Sculpt: Inflate's render — the BLOB (the one verb that moves matter), split from sculpt_blur
pub use sculpt_filter::FilterScope; // the two scopes the card's Filter buttons ask for
mod sculpt_offset; // Sculpt: Inflate's kernel — the relief offset by a BALL (dilation / erosion)
mod sculpt_panel; // Sculpt: the seam with the card — the accessors it paints from + the event router
mod sculpt_session; // Sculpt: the per-stroke session — birth, the dab walk, snapshot, cancel, re-stamp
mod shape_draft; // o meio caro renderiza em REPOUSO: gesto em voo re-carimba um rascunho plano
/// Multi-layer Shape (z-ordered layers + per-layer-colour state); split from `paint.rs` (LOC cap).
mod shape_layers;
/// Imported-image slots (Grain + Shape) + Shape geometry + Grain Depth setters; split from `brush_settings`.
mod shape_settings;
mod shape_snapshot; // unified shape+paint undo: each create/edit/bake = one ModelSnapshot on the timeline
mod stamp_color_cache; // the cached multi-layer coloured stamp (bake the composite once, blit per dab)
mod stamp_color_dynamic;
/// Stroke begin/extend/tick/end lifecycle; split from `paint.rs` (LOC cap).
mod stroke_lifecycle;
/// Watercolor stroke buffers: per-stroke coverage + deposited-colour accumulation (+ dirty tracking).
mod watercolor_accum;
/// Watercolor real GROUND (backdrop under the active layer + document paper colour) + water soak.
mod watercolor_backdrop;
/// Watercolor SECAGEM: o decaimento por-quadro do mapa de umidade; irmão do backdrop (LOC + assunto).
mod watercolor_dry;
/// Watercolor field math (noise, blur, samplers, per-stroke styles); split from `watercolor_render`.
pub(crate) mod watercolor_field;
/// Watercolor optical LUTs (`s2l`/`ln`/`exp`) + pigment-body helpers; split for the LOC cap (HR-5).
mod watercolor_lut;
/// Watercolor Wet Mix mixer-brush state (Charge/Dilution/Pull) — per-dab colour pickup + carry.
mod watercolor_mixer;
/// Watercolor canvas-anchored value noise + [`NoiseTile`] sprite-wrap (seamless tiling, doc 13 #2).
mod watercolor_noise;
/// Watercolor edge darkening (#1): per-stroke coverage + the pen-up blur-difference "fringe" pass.
mod watercolor_render;
/// Watercolor per-pixel rewet terms (lift/dissolve/pool/backrun); split from `watercolor_render`.
mod watercolor_rewet_px;
/// Watercolor section setters + router (edge darkening / granulation / pigment); no fluid sim.
mod watercolor_settings;
/// Watercolor Wet Mix reservoir (Smudge/Pickup): lift the pre-stroke paint, mix into the dab colour.
mod watercolor_smudge;
pub(crate) use paint_mode::{PAINT_MODE_COUNT, PaintMode};
mod lifecycle; // transient-edit reset run at each document (re)bind — abandons pending Fill/stroke/etc.
/// Drawing symmetry (mirror / radial) — engine glue, canvas-centre resolution + on-canvas pick modes.
mod symmetry;
mod tool_link; // "Sync with other tools": per-mode brush-settings swap + the link toggle; LOC-cap split
pub(crate) use symmetry::SymmetryPick;
/// Seamless Tiling (wrap-around painting) — dab replication across sprite edges + the toggles.
mod tiling;
mod wet_editable;
mod wetpaint; // Wet Paint (PaintMode::WetPaint): the fluid-engine session — ADR-0134; see its module doc
mod wetpaint_commit;
mod wetpaint_settings; // Wet Paint authored state (checkbox + W3 knobs + routing) — LOC-cap sibling // Wet Paint deposit-at-commit door (doc 21) — LOC-cap sibling
pub use self::{curve_gizmo::TransformGizmo, curve_tangent::TangentHandles};
pub use curve::CurveOverlay;
/// The Ellipse stroke method's on-canvas ellipse editor (same submodule rationale as `curve`).
mod ellipse;
pub use ellipse::EllipseOverlay;
/// The Line stroke method's on-canvas polyline editor (plain corner points, no Bézier handles).
mod line;
pub use line::LineOverlay;
/// Per-corner Fillet / Chamfer geometry + gizmos for the Line editor (split from `line` for the LOC cap).
mod line_corner;
pub use line_corner::LineCornerGizmo;
/// Live dx/dy + corner-angle dimensions for the active Line segment (split from `line` for the LOC cap).
mod line_dim;
pub use line_dim::LineDimensions;
/// Line editor commit / cancel / finish paths (split from `line` for the LOC cap).
mod line_commit;
/// Perpendicular Offset (parallel-polyline) geometry for the Line editor (split from `line` for the cap).
mod line_offset;
/// Drag-time snapping for the Line editor (Shift 15° + auto point-to-point align); split from `line` (cap).
mod line_snap;
/// The Polygon stroke method's on-canvas regular-N-gon editor (same submodule rationale).
mod polygon;
pub use polygon::PolygonOverlay;
/// The Stencil texture mapping's on-canvas handle editor (move/resize the image-space rect).
mod stencil;
/// Stroke-method control (set / non-shape memory / restore) — the Brush-panel + rail Shapes seam.
mod stroke_ctl;
pub use stencil::{StencilOverlay, StencilPreview};
mod blur_route;
/// The `impl CanvasPaintTool` pointer entry (`on_canvas_pointer`); split from `paint.rs` (LOC cap).
mod canvas_pointer;
mod composite;
pub(crate) use composite::{CompositeLayer, CompositeOp};
mod clone;
mod eyedropper;
mod fill; // Fill (Bucket) — Procreate ColorDrop flood fill + live threshold adjust; split for LOC cap
mod inpaint; // content-aware heal brush (mark defect + reconstruct on pen-up); split for LOC cap
/// The **Mask** tool's extras — sub-brush (Paint/Erase/Blur/Smear), whole-canvas ops, overlay tint. [LOC split].
mod mask;
mod ramp;
mod ramp_lut; // ramp LUT baking (colour owner + colour/tone LUTs); split from `stamp_cache` (LOC cap)
/// Pixel-region save/restore helpers for the drag preview (`dab_bbox`/`save_region`/`restore_region`).
pub(crate) mod region;
/// The **Selection** tool (ADR-0103) — the document-wide selection mask, undo integration + paint gate. [LOC split].
mod selection;
/// Selection **actions** (Wave 5): Select layer contents / Color Fill / Copy-Paste / Save-Load slots. [LOC split].
mod selection_actions;
mod selection_curve_gizmo; // converted-curve point editor: Convert/Simplify → editable anchors + handles
/// Selection **Edit** mode (ADR-0103 Am.2): Convert-to-Curve + Simplify (list ops). [LOC split].
mod selection_edit;
/// Selection **isolated gizmos** (ADR-0103 Am.2 v2): per-shape gizmos decoupled from the stroke editors.
mod selection_gizmo;
/// **Deform** (Liquify) — the single inverse-warp kernel + per-mode displacement fields + Reconstruct/Amount.
mod warp;
pub use self::{selection_gizmo::SelectionGizmoView, warp::DeformGizmoView};
#[cfg(test)]
mod impasto_fill_tests; // o Fill no Impasto: corpo + a borda com o perfil do Falloff
/// Selection **shape list** model (ADR-0103 Am.2): the `Vec<SelectionShape>` source of truth + compositing. [LOC split].
pub mod impasto_rig;
/// The stamp route dispatcher (Shape + Grain → which of the 4 stamp paths); split for the LOC cap.
mod plane_fork;
pub(super) mod relief_state;
/// Selection creation input: mode/op/threshold setters + on-canvas pointer gestures (marquee/lasso/flood). [LOC split].
mod selection_input;
/// Selection **Offset** (ADR-0103 Am.3): signed-distance grow/shrink + concentric alternating protected /
/// paint bands driven by Apply & Keep. [LOC split].
mod selection_offset;
mod selection_offset_geom; // sharp-corner offset: trace(+holes) → refit → CAD offset per level (no SDF rounding)
/// Selection on-canvas overlay (marching ants + hatching) + panel event routing. [LOC split].
mod selection_overlay;
mod selection_pen; // a CANETA: o Pen do vetor autorando uma regiao de selecao (Enio 2026-08-07)
#[cfg(test)]
mod selection_pen_tests;
/// Selection rasterization: shape → coverage buffers, boolean combine, Feather (box-blur). [LOC split].
mod selection_raster;
#[cfg(test)]
#[path = "paint/selection_verbs_tests.rs"]
mod selection_verbs_tests; // Cut / Select All / Intersect (report do Enio, 2026-08-07)

/// O **Paste FLUTUANTE**: a peça colada transformável antes de pousar (Enio, 2026-08-07).
mod paste_patch;

pub(super) mod selection_shapes; // SelectionEntry is re-exported at `crate::tool` for the undo snapshot
/// Selection **Edit** mode contour tracing (mask → editable boundary polyline); split for the LOC cap.
mod selection_trace;
mod shape_ramp;
mod smear_warp;
mod snapshot;
/// The Blender-style cached brush stamp (render falloff×texture once, scale-blit per dab).
pub mod stamp_banded;
mod stamp_cache;
pub mod stamp_device; // o lote publicado para um dispositivo (doc 33 S3); a ponte e' do shell
mod stamp_preview; // interactive drag-preview stamping (restore+re-stamp, dirty-rect); split for the LOC cap
mod stamp_route;
/// `PaintState::default` body — split out for the workspace file-LOC cap (struct stays in `paint.rs`).
mod state_default;

mod brush_ranges; // Stroke-slider UI range consts (BRUSH_*_MAX / airbrush rate) + shape grab tol; split for the LOC cap
pub use brush_ranges::*;
pub use impasto_rig::{ImpastoLight, LightRig, MAX_IMPASTO_LIGHTS, MIN_ELEV_DEG};

// The panel-facing snapshot [`BrushSettings`] + the falloff preview helper `brush_falloff_weight_at`
// live in the `brush_settings` submodule (their single clamp source); re-exported for the `paint::` path.
pub use brush_settings::{
    BrushSettings, DEFORM_TEMPERAMENT_NONE, DEFORM_TEMPERAMENT_RESHAPE,
    DEFORM_TEMPERAMENT_TRANSFORM, PANEL_RAMP_STOPS,
};
pub use shape_layers::MAX_SHAPE_LAYERS;
pub use snapshot::brush_falloff_weight_at;

use stamp_preview::DragPreview;

/// **O que uma sessão de pintura segura** — irmão pelo teto de LOC; o corte é por responsabilidade:
/// aqui fica o MANIFESTO do subsistema (módulos, re-exports, gates), lá o estado que ele carrega.
mod state;
pub(crate) use state::PaintState;

// (LOC cap) `set_line_constrain`/`set_shape_grab_tol_px` live beside the `impl CanvasPaintTool`
// pointer entry in `canvas_pointer` (it drives the private stroke-lifecycle methods); the open-shape
// verbs in `curve_commit`; drag-preview stamping in `stamp_preview`; `union_region` in `region`.
use region::union_region;

#[cfg(test)]
mod tests;
// Each gate family below gets its own file rather than the end of the 21k-line `tests` — a wave's
// worth of gates appended there is a wave's worth of gates nobody can find again.
#[cfg(test)]
mod impasto_aa_tests; // screen-space AA of the impasto film silhouette (BUGS #16, impasto half)
#[cfg(test)]
mod impasto_fingerprint_tests; // the deposit's byte-for-byte pin — the net any kernel rewrite needs
#[cfg(test)]
mod sculpt_tests;
#[cfg(test)]
mod watercolor_aa_tests; // screen-space AA of the thin-stroke watercolor silhouette
#[cfg(test)]
mod watercolor_dry_tests; // o decaimento da umidade: janela deslizante + rect que encolhe

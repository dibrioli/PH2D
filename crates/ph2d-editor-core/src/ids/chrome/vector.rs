//! Vector module chrome NodeIds (VGRAPH_* geometry-graph + VECTOR_INSPECTOR_*).
use super::{NodeId, hash_node_id};

/// Vector Geometry-Graph panel (W3 T3.1) — docked panel that places the
/// `vector.source` node + drives its 8 params (sliders) and renders the cooked
/// `VectorNetwork` live. Outer-rect id for `z_order`; the 8 param sliders below.
pub const VGRAPH_PANEL: NodeId = hash_node_id("vgraph.panel");
pub const VGRAPH_KIND: NodeId = hash_node_id("vgraph.kind");
pub const VGRAPH_WIDTH: NodeId = hash_node_id("vgraph.width");
pub const VGRAPH_HEIGHT: NodeId = hash_node_id("vgraph.height");
pub const VGRAPH_SIDES: NodeId = hash_node_id("vgraph.sides");
pub const VGRAPH_INNER_RATIO: NodeId = hash_node_id("vgraph.inner_ratio");
pub const VGRAPH_TURNS: NodeId = hash_node_id("vgraph.turns");
pub const VGRAPH_SAMPLES_PER_TURN: NodeId = hash_node_id("vgraph.samples_per_turn");
pub const VGRAPH_ROTATION: NodeId = hash_node_id("vgraph.rotation");
/// Vector Inspector panel (W2 T2.4) — minimal right-docked panel hosting the
/// fill swatch (+ future vertex/node params). Outer-rect id for `z_order`.
pub const VECTOR_INSPECTOR_PANEL: NodeId = hash_node_id("vector_inspector.panel");
/// Vector Inspector close (X) button.
pub const VECTOR_INSPECTOR_CLOSE: NodeId = hash_node_id("vector_inspector.close");
/// Vector Inspector fill-color swatch — a picker swatch (opens the Blender
/// picker on Down via `is_picker_swatch`); the shell read-back applies the
/// picked color to the selected regions.
pub const VECTOR_INSPECTOR_FILL_SWATCH: NodeId = hash_node_id("vector_inspector.fill_swatch");
/// Vector Inspector Shape-kind picker (W2 §4.2) — the on-screen replacement for
/// the interim hotkeys 1-5. A vertical 5-option segmented control shown only
/// while the `vector_shape` tool is active. `_KIND` is the group/label id; the
/// five `_SHAPE_*` ids are the per-option Button hit-targets (the generic
/// RadioGroup dispatch is not wired yet, so each option is a Button — its
/// `Click` routes through the shell bridge to `VectorShapeTool::set_kind`).
pub const VECTOR_INSPECTOR_SHAPE_KIND: NodeId = hash_node_id("vector_inspector.shape_kind");
pub const VECTOR_INSPECTOR_SHAPE_RECT: NodeId = hash_node_id("vector_inspector.shape.rect");
pub const VECTOR_INSPECTOR_SHAPE_ELLIPSE: NodeId = hash_node_id("vector_inspector.shape.ellipse");
pub const VECTOR_INSPECTOR_SHAPE_POLYGON: NodeId = hash_node_id("vector_inspector.shape.polygon");
pub const VECTOR_INSPECTOR_SHAPE_STAR: NodeId = hash_node_id("vector_inspector.shape.star");
pub const VECTOR_INSPECTOR_SHAPE_SPIRAL: NodeId = hash_node_id("vector_inspector.shape.spiral");

// ── Vector tool Style panel (ADR-0108 cutover — docked `ph2d-panel-vector`) ──
// The `vector` tool's Style controls live in a right-docked `Panel<State>` (the
// tool `FloatingPanel` is unpainted). Width slider (1..20 px) + Stroke / Fill
// colour swatches (each opens the shared OKLCH picker via `is_picker_swatch`) +
// a Fill "None" affordance. Distinct slug family from the retired
// `vector_inspector.*` ids above.
/// Vector Style panel outer rect id (for `z_order` + hit-barrier).
pub const VECTOR_PANEL: NodeId = hash_node_id("vector.panel");
/// Vector Style panel close (X) button.
pub const VECTOR_CLOSE: NodeId = hash_node_id("vector.close");
/// Stroke-width slider (bipolar-less: track `0..1` → `1..20` px).
pub const VECTOR_WIDTH: NodeId = hash_node_id("vector.width");
/// Px-valued chip linked to [`VECTOR_WIDTH`].
pub const VECTOR_WIDTH_NUM: NodeId = hash_node_id("vector.width_num");
/// Stroke-colour swatch — a picker swatch (opens the Blender picker on Down);
/// the shell read-back applies the picked colour to the new + selected path.
pub const VECTOR_STROKE_SWATCH: NodeId = hash_node_id("vector.stroke_swatch");
/// Fill-colour swatch — a picker swatch (alpha 0 ⇒ no fill).
pub const VECTOR_FILL_SWATCH: NodeId = hash_node_id("vector.fill_swatch");
/// Fill "None" button — clears the fill (alpha 0) on the selected closed path.
pub const VECTOR_FILL_NONE: NodeId = hash_node_id("vector.fill_none");

// ── Draw-mode selector (ADR-0108 Fase 1 — Pen / shape tools) ─────────────────
// A segmented row that switches the canvas gesture: Pen (draw + edit anchors)
// vs a drag-to-size shape (Rectangle / Ellipse / Polygon). The tool owns the
// mode (`VectorTool::mode`); each button's `Click` routes through the seam to
// `handle_panel_event`, mirror of the retired `vector_inspector.shape.*` ids.
pub const VECTOR_MODE_PEN: NodeId = hash_node_id("vector.mode.pen");
pub const VECTOR_MODE_RECT: NodeId = hash_node_id("vector.mode.rect");
pub const VECTOR_MODE_ELLIPSE: NodeId = hash_node_id("vector.mode.ellipse");
pub const VECTOR_MODE_POLYGON: NodeId = hash_node_id("vector.mode.polygon");
pub const VECTOR_MODE_STAR: NodeId = hash_node_id("vector.mode.star");
pub const VECTOR_MODE_RRECT: NodeId = hash_node_id("vector.mode.rrect");
/// Polygon "Sides" slider (3..12) — shown only in Polygon mode; drives
/// `VectorTool::polygon_sides`.
pub const VECTOR_SIDES: NodeId = hash_node_id("vector.sides");
/// Integer chip paired with [`VECTOR_SIDES`].
pub const VECTOR_SIDES_NUM: NodeId = hash_node_id("vector.sides_num");
/// Star "Points" slider (3..12) + "Inner" ratio slider (0.1..0.9) — shown only
/// in Star mode. Rounded-rect "Radius" slider (0..40 px) — shown only in
/// RoundRect mode. Each drives the matching `VectorTool` field.
pub const VECTOR_STAR_POINTS: NodeId = hash_node_id("vector.star_points");
pub const VECTOR_STAR_POINTS_NUM: NodeId = hash_node_id("vector.star_points_num");
pub const VECTOR_STAR_INNER: NodeId = hash_node_id("vector.star_inner");
pub const VECTOR_STAR_INNER_NUM: NodeId = hash_node_id("vector.star_inner_num");
pub const VECTOR_RRECT_RADIUS: NodeId = hash_node_id("vector.rrect_radius");
pub const VECTOR_RRECT_RADIUS_NUM: NodeId = hash_node_id("vector.rrect_radius_num");

// ── Boolean ops (ADR-0108 Fase 1 — edit-time union/subtract/intersect) ───────
// Act on the DOCUMENT (shell-owned `vec_scene`), NOT the tool's Style: the
// panel forwards a `Click` over `ToolPanelEvent` and the shell drain applies
// the op to the two last closed regions (mirror of the U/I/D hotkeys).
pub const VECTOR_BOOL_UNION: NodeId = hash_node_id("vector.bool.union");
pub const VECTOR_BOOL_SUBTRACT: NodeId = hash_node_id("vector.bool.subtract");
pub const VECTOR_BOOL_INTERSECT: NodeId = hash_node_id("vector.bool.intersect");

// ── Vertex type (ADR-0108 Fase 1 — rich handle editing) ──────────────────────
// Retype the SELECTED vertex (Corner cusp / Smooth colinear / Symmetric mirror).
// A document edit (mutates the path via the shell-side PenTool), shown only when
// a vertex is selected; each `Click` routes through the shell drain.
pub const VECTOR_VERT_CORNER: NodeId = hash_node_id("vector.vert.corner");
pub const VECTOR_VERT_SMOOTH: NodeId = hash_node_id("vector.vert.smooth");
pub const VECTOR_VERT_SYMMETRIC: NodeId = hash_node_id("vector.vert.symmetric");
/// "Delete Node" button — removes the selected vertex (re-stitching neighbors);
/// a document edit routed through the shell drain (mirror of the vertex-type
/// buttons). Insert is a canvas gesture (click a segment) — no button.
pub const VECTOR_VERT_DELETE: NodeId = hash_node_id("vector.vert.delete");

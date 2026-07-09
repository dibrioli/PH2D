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
/// the shell read-back applies the picked colour (RGB; alpha kept) to the path.
pub const VECTOR_STROKE_SWATCH: NodeId = hash_node_id("vector.stroke_swatch");
/// Fill-colour swatch — a picker swatch (RGB; the Opacity slider owns alpha).
pub const VECTOR_FILL_SWATCH: NodeId = hash_node_id("vector.fill_swatch");
/// Stroke / Fill **Opacity** sliders (0..100 %) — the single source of the
/// stroke/fill alpha. `0 %` = invisible (no fill). Each drives the matching
/// `VectorTool` colour's alpha channel.
// ── Fill type + gradient (ADR-0108 gradient group) ───────────────────────────
// Segmented selector switching the SELECTED path's fill between Solid / Linear /
// Radial (a document command via the shell drain). The Angle slider drives a
// Linear gradient's direction (shown only in Linear).
pub const VECTOR_FILL_KIND_SOLID: NodeId = hash_node_id("vector.fill_kind.solid");
pub const VECTOR_FILL_KIND_LINEAR: NodeId = hash_node_id("vector.fill_kind.linear");
pub const VECTOR_FILL_KIND_RADIAL: NodeId = hash_node_id("vector.fill_kind.radial");
/// Multi-point (Cavalry freeform IDW) fill.
pub const VECTOR_FILL_KIND_MULTI: NodeId = hash_node_id("vector.fill_kind.multi");
pub const VECTOR_GRAD_ANGLE: NodeId = hash_node_id("vector.grad.angle");
pub const VECTOR_GRAD_ANGLE_NUM: NodeId = hash_node_id("vector.grad.angle_num");
/// Multi-point gradient: add a point (bbox center) / remove the selected point.
pub const VECTOR_GRAD_ADD_POINT: NodeId = hash_node_id("vector.grad.add_point");
pub const VECTOR_GRAD_REMOVE_POINT: NodeId = hash_node_id("vector.grad.remove_point");
/// Influence (strength / reach) of the selected multi-point gradient point.
pub const VECTOR_GRAD_INFLUENCE: NodeId = hash_node_id("vector.grad.influence");
pub const VECTOR_GRAD_INFLUENCE_NUM: NodeId = hash_node_id("vector.grad.influence_num");
/// Jitter (per-texel grain, 0..1) of the selected multi-point gradient point.
pub const VECTOR_GRAD_JITTER: NodeId = hash_node_id("vector.grad.jitter");
pub const VECTOR_GRAD_JITTER_NUM: NodeId = hash_node_id("vector.grad.jitter_num");
/// Linear/Radial gradient: add an interior ramp stop / remove the selected one.
pub const VECTOR_GRAD_ADD_STOP: NodeId = hash_node_id("vector.grad.add_stop");
pub const VECTOR_GRAD_REMOVE_STOP: NodeId = hash_node_id("vector.grad.remove_stop");

// ── Align + Distribute (multi-path object selection) ─────────────────────────
// Shown when ≥2 paths are selected (Align) / ≥3 (Distribute). Align snaps each
// selected path's bbox edge/center to the selection's bbox; Distribute evenly
// spaces the middle paths' centers between the two extremes.
pub const VECTOR_ALIGN_LEFT: NodeId = hash_node_id("vector.align.left");
pub const VECTOR_ALIGN_HCENTER: NodeId = hash_node_id("vector.align.hcenter");
pub const VECTOR_ALIGN_RIGHT: NodeId = hash_node_id("vector.align.right");
pub const VECTOR_ALIGN_TOP: NodeId = hash_node_id("vector.align.top");
pub const VECTOR_ALIGN_VCENTER: NodeId = hash_node_id("vector.align.vcenter");
pub const VECTOR_ALIGN_BOTTOM: NodeId = hash_node_id("vector.align.bottom");
pub const VECTOR_DISTRIBUTE_H: NodeId = hash_node_id("vector.distribute.h");
pub const VECTOR_DISTRIBUTE_V: NodeId = hash_node_id("vector.distribute.v");
/// Arm the transform gizmo's "Set Center" mode (redefine the rotation/scale pivot).
pub const VECTOR_PIVOT_EDIT: NodeId = hash_node_id("vector.pivot.edit");

pub const VECTOR_STROKE_OPACITY: NodeId = hash_node_id("vector.stroke_opacity");
pub const VECTOR_STROKE_OPACITY_NUM: NodeId = hash_node_id("vector.stroke_opacity_num");
pub const VECTOR_FILL_OPACITY: NodeId = hash_node_id("vector.fill_opacity");
pub const VECTOR_FILL_OPACITY_NUM: NodeId = hash_node_id("vector.fill_opacity_num");

// ── Stroke details (ADR-0108 Fase 1 — cap / join / dash + gap) ───────────────
// Line cap (Butt/Round/Square) + join (Miter/Round/Bevel) segmented rows + a
// Dash length slider (0 = solid) + a Gap length slider (space between dashes).
// Both are multiples of the stroke width. Drive the matching `VectorTool`
// fields; the bridge applies them to new + selected paths (like colour/width).
pub const VECTOR_CAP_BUTT: NodeId = hash_node_id("vector.cap.butt");
pub const VECTOR_CAP_ROUND: NodeId = hash_node_id("vector.cap.round");
pub const VECTOR_CAP_SQUARE: NodeId = hash_node_id("vector.cap.square");
pub const VECTOR_JOIN_MITER: NodeId = hash_node_id("vector.join.miter");
pub const VECTOR_JOIN_ROUND: NodeId = hash_node_id("vector.join.round");
pub const VECTOR_JOIN_BEVEL: NodeId = hash_node_id("vector.join.bevel");
pub const VECTOR_DASH: NodeId = hash_node_id("vector.dash");
pub const VECTOR_DASH_NUM: NodeId = hash_node_id("vector.dash_num");
pub const VECTOR_GAP: NodeId = hash_node_id("vector.gap");
pub const VECTOR_GAP_NUM: NodeId = hash_node_id("vector.gap_num");

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
pub const VECTOR_MODE_SPIRAL: NodeId = hash_node_id("vector.mode.spiral");
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
/// Spiral "Turns" slider (1..8) — shown only in Spiral mode; drives
/// `VectorTool::spiral_turns`.
pub const VECTOR_SPIRAL_TURNS: NodeId = hash_node_id("vector.spiral_turns");
pub const VECTOR_SPIRAL_TURNS_NUM: NodeId = hash_node_id("vector.spiral_turns_num");

// ── Boolean ops (ADR-0108 Fase 1 — edit-time union/subtract/intersect) ───────
// Act on the DOCUMENT (shell-owned `vec_scene`), NOT the tool's Style: the
// panel forwards a `Click` over `ToolPanelEvent` and the shell drain applies
// the op to the two last closed regions (mirror of the U/I/D hotkeys).
pub const VECTOR_BOOL_UNION: NodeId = hash_node_id("vector.bool.union");
pub const VECTOR_BOOL_SUBTRACT: NodeId = hash_node_id("vector.bool.subtract");
pub const VECTOR_BOOL_INTERSECT: NodeId = hash_node_id("vector.bool.intersect");
pub const VECTOR_BOOL_EXCLUDE: NodeId = hash_node_id("vector.bool.exclude");

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

// ── Arrange (ADR-0108 — path ops: duplicate + z-order) ───────────────────────
// Act on the SELECTED path (shell-side PenTool selection); document commands
// routed through the shell drain (mirror of Boolean/Vertex). Duplicate clones
// the path with a small offset; the four z-order buttons restack it (render
// order = paths-vec order, index 0 = back).
pub const VECTOR_ARRANGE_DUPLICATE: NodeId = hash_node_id("vector.arrange.duplicate");
pub const VECTOR_ARRANGE_TO_BACK: NodeId = hash_node_id("vector.arrange.to_back");
pub const VECTOR_ARRANGE_BACKWARD: NodeId = hash_node_id("vector.arrange.backward");
pub const VECTOR_ARRANGE_FORWARD: NodeId = hash_node_id("vector.arrange.forward");
pub const VECTOR_ARRANGE_TO_FRONT: NodeId = hash_node_id("vector.arrange.to_front");
// Mirror the selected path (H = left↔right, V = up↔down) around its bbox center.
pub const VECTOR_ARRANGE_FLIP_H: NodeId = hash_node_id("vector.arrange.flip_h");
pub const VECTOR_ARRANGE_FLIP_V: NodeId = hash_node_id("vector.arrange.flip_v");
// Rotate the selected path 90° (CW / CCW) around its bbox center.
pub const VECTOR_ARRANGE_ROTATE_CW: NodeId = hash_node_id("vector.arrange.rotate_cw");
pub const VECTOR_ARRANGE_ROTATE_CCW: NodeId = hash_node_id("vector.arrange.rotate_ccw");

// ── Transform (ADR-0108 — precise numeric position + size) ───────────────────
// Standalone NumberInputs (NOT slider-linked) showing the selected path's anchor
// bbox: X/Y = top-left (world), W/H = size. Seeded each frame from the published
// bbox (unless focused); editing routes a document command through the shell
// drain (X/Y → translate, W/H → scale about the bbox min).
pub const VECTOR_TRANSFORM_X: NodeId = hash_node_id("vector.transform.x");
pub const VECTOR_TRANSFORM_Y: NodeId = hash_node_id("vector.transform.y");
pub const VECTOR_TRANSFORM_W: NodeId = hash_node_id("vector.transform.w");
pub const VECTOR_TRANSFORM_H: NodeId = hash_node_id("vector.transform.h");
/// Rotation (degrees) — a RELATIVE scrub field (not a bbox readout): each change
/// rotates the selected path by the delta about its bbox center. Seeded to 0 while
/// unfocused; the panel owns the per-gesture accumulator.
pub const VECTOR_TRANSFORM_R: NodeId = hash_node_id("vector.transform.r");

// ── Path shape (ADR-0108 — whole-path handle ops) ────────────────────────────
// One-shot buttons acting on ALL vertices of the SELECTED path (document commands
// via the shell drain, mirror of Arrange). Smooth = auto-colinear handles from
// neighbors (Inkscape 1/3, curve-ifies a polygon/hand path); Sharpen = collapse
// handles onto the anchor (straight-segment corners).
pub const VECTOR_PATH_SMOOTH: NodeId = hash_node_id("vector.path.smooth");
pub const VECTOR_PATH_SHARPEN: NodeId = hash_node_id("vector.path.sharpen");
/// Simplify = drop redundant/near-colinear anchors (RDP-style vertex reduction).
pub const VECTOR_PATH_SIMPLIFY: NodeId = hash_node_id("vector.path.simplify");
/// Subdivide = insert a midpoint on every segment (exact de Casteljau split).
pub const VECTOR_PATH_SUBDIVIDE: NodeId = hash_node_id("vector.path.subdivide");
/// Close/Open toggle — flips the selected path between a closed loop and an open
/// ribbon (label driven by the published `closed` flag).
pub const VECTOR_PATH_CLOSE: NodeId = hash_node_id("vector.path.close");

// ── Compound paths (ADR-0108 — subpaths + fill rule) ─────────────────────────
/// Merge the selected closed paths into ONE compound path (a contour inside
/// another becomes a hole, via `EvenOdd`). Inverse: [`VECTOR_COMPOUND_RELEASE`].
pub const VECTOR_COMPOUND_MAKE: NodeId = hash_node_id("vector.compound.make");
/// Split the selected compound path's subpaths back into standalone paths.
pub const VECTOR_COMPOUND_RELEASE: NodeId = hash_node_id("vector.compound.release");
/// Fill rule of the selected COMPOUND path — the two agree on a single contour,
/// so the row only shows when the path actually has subpaths.
pub const VECTOR_FILL_RULE_NONZERO: NodeId = hash_node_id("vector.fill.rule.nonzero");
pub const VECTOR_FILL_RULE_EVENODD: NodeId = hash_node_id("vector.fill.rule.evenodd");

// ── Snap (ADR-0108 — smart guides) ───────────────────────────────────────────
/// Snap to the other shapes' anchors / bbox key points. Held Alt bypasses it.
/// The GRID toggle is NOT here: the editor's universal Grid Snap panel owns it
/// (`grid_snap::ids`), and the Vector module just asks `GridSnapState::snap_world`.
pub const VECTOR_SNAP_OFF: NodeId = hash_node_id("vector.snap.off");
pub const VECTOR_SNAP_ON: NodeId = hash_node_id("vector.snap.on");

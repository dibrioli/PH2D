//! Sculpt-panel NodeIds (`docs/Painter/18_plano_sculpt_relevo.md`, Wave 1). The Painter's **Sculpt**
//! mode reshapes the impasto relief itself — it reads the layer's committed height and rewrites it,
//! and it lays no pigment (§5).
//!
//! Unlike Deform, the Sculpt section is **additive, not mode-exclusive**: the sculpt rides the SAME dab
//! list the colour does (§10.1), so the brush's own Size / Spacing / Falloff / Shape / Grain / Symmetry /
//! Tiling / stroke method ARE the sculpt's controls and must stay on screen. What this card adds is the
//! part the brush does not already say: which verb, how hard, and over what scale.
//!
//! Fixed-id, tool-global widgets — registered in the painter-layers `populate` and forwarded to the tool
//! over the frozen `PanelEvent` channel (`Click` / `SetValue`).
use super::{NodeId, hash_node_id};

// ── Sub-mode picker (segmented): Smooth · Sharpen · Flatten · Scrape · Fill ─────────────────────────
/// The sub-mode segmented group (a11y RadioGroup).
pub const PAINTER_SCULPT_MODE: NodeId = hash_node_id("painter_sculpt.mode");
/// **Smooth** — pull the relief toward its own local average. The whole reason this wave exists.
pub const PAINTER_SCULPT_MODE_SMOOTH: NodeId = hash_node_id("painter_sculpt.mode_smooth");
/// **Sharpen** — the same kernel with the sign flipped (`h + k·(h − blur h)`, unsharp mask on a height
/// field). It is not a second engine; it is the first one run backwards.
pub const PAINTER_SCULPT_MODE_SHARPEN: NodeId = hash_node_id("painter_sculpt.mode_sharpen");
/// **Flatten** — pull the relief toward the **tilted** plane fitted to the footprint (§7), both ways.
pub const PAINTER_SCULPT_MODE_FLATTEN: NodeId = hash_node_id("painter_sculpt.mode_flatten");
/// **Scrape** — the same plane, DOWN only: the spatula taking the high ground off.
pub const PAINTER_SCULPT_MODE_SCRAPE: NodeId = hash_node_id("painter_sculpt.mode_scrape");
/// **Fill** — the same plane, UP only: paint pushed into the valleys.
pub const PAINTER_SCULPT_MODE_FILL: NodeId = hash_node_id("painter_sculpt.mode_fill");
/// **Chisel** — the knife tipped onto its edge: the plane plus a V about the stroke's axis, scraped down to.
/// At Angle 0 it IS Scrape, to the byte.
pub const PAINTER_SCULPT_MODE_CHISEL: NodeId = hash_node_id("painter_sculpt.mode_chisel");
/// **Layer** — bounded build-up: the relief rises toward `pre + Depth` and stops there, however long you
/// dwell. The one thing neither the deposit (which accumulates) nor the plane verbs (which level) can do.
pub const PAINTER_SCULPT_MODE_LAYER: NodeId = hash_node_id("painter_sculpt.mode_layer");
/// **Inflate** — raise along the surface normal, so the flats rise and the walls do not: crests get rounded
/// off rather than translated. Not the 3D Inflate, and the docs say so.
pub const PAINTER_SCULPT_MODE_INFLATE: NodeId = hash_node_id("painter_sculpt.mode_inflate");
/// Sub-mode segments in `SculptMode` discriminant order (`0` Smooth · `1` Sharpen · `2` Flatten ·
/// `3` Scrape · `4` Fill · `5` Chisel · `6` Layer · `7` Inflate) — **the array grows, the order never
/// shifts**: the discriminant is the segmented control's index and a reorder would silently re-bind every
/// artist's muscle memory.
///
/// Blender's Clay, Clay Strips and Draw Sharp are deliberately absent, and each absence is a finding rather
/// than a gap — see `SculptMode` in `ph2d-tool-painter` (Clay is Flatten with a positive Offset; a square
/// dab belongs to the brush; and our per-stroke engine makes every additive verb "sharp" by construction).
pub const PAINTER_SCULPT_MODE_IDS: [NodeId; 8] = [
    PAINTER_SCULPT_MODE_SMOOTH,
    PAINTER_SCULPT_MODE_SHARPEN,
    PAINTER_SCULPT_MODE_FLATTEN,
    PAINTER_SCULPT_MODE_SCRAPE,
    PAINTER_SCULPT_MODE_FILL,
    PAINTER_SCULPT_MODE_CHISEL,
    PAINTER_SCULPT_MODE_LAYER,
    PAINTER_SCULPT_MODE_INFLATE,
];

// ── Knobs (0..1 track; the tool maps it to its range — the setter is the single clamp source) ──
//
// There is **no Sculpt Strength**, and its absence is a decision, not an omission. The brush already has
// a Strength, and the sculpt rides the brush's dab list — so the brush's Strength IS the spatula's
// pressure. A second one beside it would be two knobs fighting over one number, which is a design bug
// wearing the costume of a feature. (`Dab::coverage` already carries it; see `ph2d_painter_brush::sculpt`
// for the fold, and for the reason that fold is one `× strength` shorter than the deposit's.)
/// **Radius** — the kernel's own scale in px, NOT the brush's. A big brush can polish fine grain, and a
/// small one can knock down a broad swell; conflating the two would remove a control the artist wants.
/// Capped small on purpose: smoothing at a LARGE scale is Flatten, and Flatten is a different kernel
/// (the plane fit, §7) — not a bigger blur.
pub const PAINTER_SCULPT_RADIUS_SLIDER: NodeId = hash_node_id("painter_sculpt.radius_slider");
pub const PAINTER_SCULPT_RADIUS_CHIP: NodeId = hash_node_id("painter_sculpt.radius_chip");

/// **Offset** — where the plane sits, in paint-loads above (`+`) or below (`−`) the surface it was fitted
/// to. It is what gives Scrape and Fill their **bite**: at `0` the spatula only takes off what stands above
/// the local slope, and a negative Offset lets it dig in below.
///
/// Plane family only (Flatten / Scrape / Fill). The Radius row above is the Smooth family's, and the card
/// shows **one or the other** — a knob that does nothing to the active verb is a knob that lies.
pub const PAINTER_SCULPT_OFFSET_SLIDER: NodeId = hash_node_id("painter_sculpt.offset_slider");
pub const PAINTER_SCULPT_OFFSET_CHIP: NodeId = hash_node_id("painter_sculpt.offset_chip");

/// **Depth** — the Height family's knob (Layer / Inflate), in paint-loads. How thick a coat Layer lays, how
/// hard Inflate puffs; signed, so the lower half carves and deflates.
pub const PAINTER_SCULPT_DEPTH_SLIDER: NodeId = hash_node_id("painter_sculpt.depth_slider");
pub const PAINTER_SCULPT_DEPTH_CHIP: NodeId = hash_node_id("painter_sculpt.depth_chip");

/// **Angle** — how far the **Chisel**'s knife is tipped onto its edge (degrees). At `0` the Chisel is Scrape,
/// to the byte, so the slider's bottom end is not a dead zone: it is the flat blade.
///
/// The Chisel is the one verb that shows TWO knobs (Offset *and* Angle) — the plane still has to be placed
/// before the V is folded about it.
pub const PAINTER_SCULPT_ANGLE_SLIDER: NodeId = hash_node_id("painter_sculpt.angle_slider");
pub const PAINTER_SCULPT_ANGLE_CHIP: NodeId = hash_node_id("painter_sculpt.angle_chip");

/// Inflate's **Smoothness** (Radius) slider — softens the ball dilation's hard edge (Enio 2026-07-14).
///
/// The paired chip shows a texel radius (`0..16`). It is Inflate's second knob, next to Depth, the way the
/// Chisel's Angle sits next to its Offset: `0` is the raw ball, and turning it up rounds the edge off exactly
/// the way Smooth's own Radius sets its kernel — which is the parallel Enio drew.
pub const PAINTER_SCULPT_SMOOTH_SLIDER: NodeId = hash_node_id("painter_sculpt.smooth_slider");
pub const PAINTER_SCULPT_SMOOTH_CHIP: NodeId = hash_node_id("painter_sculpt.smooth_chip");

/// The Sculpt card's a11y group id (a visual surface; not hit-indexed).
pub const PAINTER_SCULPT_CARD: NodeId = hash_node_id("painter_sculpt.card");

/// The Chisel's **Rake** toggle — *does the V follow the direction of the stroke?* `Click` →
/// `toggle_sculpt_rake`.
///
/// It lives on the SCULPT card and not on the brush's Shape card, and the difference is not cosmetic. The
/// Shape's own Rake checkbox rotates a silhouette **image**, so the panel only paints it once a Shape image
/// exists (`kind != None`) — with the default round falloff there is no silhouette to turn, and the box is
/// not drawn at all. Enio asked for a Rake and could not find one: it was there, behind a texture he had not
/// loaded, governing something else. The chisel's V has an axis whether or not the brush has a picture.
pub const PAINTER_SCULPT_RAKE: NodeId = hash_node_id("painter_sculpt.rake");

/// **Filter Layer** (W5b, plan §8) — apply the selected verb to the WHOLE layer at once, with no stroke
/// (Blender's *Mesh Filter*). A button, not a mode: the verb chips and the verb's own knob already say
/// *what* and *how much*; this says *everywhere*. Offered only for the verbs whose target is a function of
/// the relief itself — the plane family is fitted to the brush's footprint, and a layer has none
/// (`sculpt_filter::filters_layer`, the one door the panel and the tool both ask).
pub const PAINTER_SCULPT_FILTER: NodeId = hash_node_id("painter_sculpt.filter");

/// **Filter Stroke** (W5b, Enio 2026-07-16) — the same verb, scoped to the LAST stroke instead of the
/// whole layer. Weighted by that stroke's own paint envelope (`relief.live_paint`), so it feathers out
/// exactly where the paint did. Offered only when the verb reshapes AND a last stroke exists on this layer
/// (`PainterTool::can_filter_last_stroke`) — a button with nothing to act on would be a button that refuses.
pub const PAINTER_SCULPT_FILTER_STROKE: NodeId = hash_node_id("painter_sculpt.filter_stroke");

/// Every Sculpt widget a pointer can CLICK — the sweep list for the seam gate (`tests/seam_sculpt.rs`).
///
/// It exists because a widget that paints, registers a hit rect and is forwarded by `event.rs` is STILL
/// dead if `populate` never gave it an `InteractiveState`: `is_focusable` answers `None => false`, the
/// Down never activates it, and the `Click` never happens. That is how the Impasto light rig shipped
/// inert. Keep this list exhaustive and the sweep cannot go stale.
pub const PAINTER_SCULPT_CLICKS: [NodeId; 11] = [
    PAINTER_SCULPT_MODE_IDS[0],
    PAINTER_SCULPT_MODE_IDS[1],
    PAINTER_SCULPT_MODE_IDS[2],
    PAINTER_SCULPT_MODE_IDS[3],
    PAINTER_SCULPT_MODE_IDS[4],
    PAINTER_SCULPT_MODE_IDS[5],
    PAINTER_SCULPT_MODE_IDS[6],
    PAINTER_SCULPT_MODE_IDS[7],
    PAINTER_SCULPT_RAKE,
    PAINTER_SCULPT_FILTER,
    PAINTER_SCULPT_FILTER_STROKE,
];

/// Every Sculpt slider (the `SetValue` half of the same sweep). All four are here even though the card only
/// ever shows the ones the active verb USES — the sweep gates the wiring, and wiring that is reachable in
/// only one mode is still wiring that can be dead.
pub const PAINTER_SCULPT_FIELDS: [NodeId; 5] = [
    PAINTER_SCULPT_RADIUS_SLIDER,
    PAINTER_SCULPT_OFFSET_SLIDER,
    PAINTER_SCULPT_DEPTH_SLIDER,
    PAINTER_SCULPT_ANGLE_SLIDER,
    PAINTER_SCULPT_SMOOTH_SLIDER,
];

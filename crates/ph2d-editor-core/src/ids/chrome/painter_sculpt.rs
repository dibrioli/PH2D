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
/// Sub-mode segments in `SculptMode` discriminant order (`0` Smooth · `1` Sharpen · `2` Flatten ·
/// `3` Scrape · `4` Fill). Wave 3 appends (Clay / Layer / Draw Sharp / Inflate …) — **the array grows, the
/// order never shifts**: the discriminant is the segmented control's index and a reorder would silently
/// re-bind every artist's muscle memory.
pub const PAINTER_SCULPT_MODE_IDS: [NodeId; 5] = [
    PAINTER_SCULPT_MODE_SMOOTH,
    PAINTER_SCULPT_MODE_SHARPEN,
    PAINTER_SCULPT_MODE_FLATTEN,
    PAINTER_SCULPT_MODE_SCRAPE,
    PAINTER_SCULPT_MODE_FILL,
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

/// The Sculpt card's a11y group id (a visual surface; not hit-indexed).
pub const PAINTER_SCULPT_CARD: NodeId = hash_node_id("painter_sculpt.card");

/// Every Sculpt widget a pointer can CLICK — the sweep list for the seam gate (`tests/seam_sculpt.rs`).
///
/// It exists because a widget that paints, registers a hit rect and is forwarded by `event.rs` is STILL
/// dead if `populate` never gave it an `InteractiveState`: `is_focusable` answers `None => false`, the
/// Down never activates it, and the `Click` never happens. That is how the Impasto light rig shipped
/// inert. Keep this list exhaustive and the sweep cannot go stale.
pub const PAINTER_SCULPT_CLICKS: [NodeId; 5] = PAINTER_SCULPT_MODE_IDS;

/// Every Sculpt slider (the `SetValue` half of the same sweep). Both rows are here even though the card
/// shows only ONE of them at a time (by family) — the sweep gates the wiring, and wiring that is only
/// reachable in one mode is still wiring that can be dead.
pub const PAINTER_SCULPT_FIELDS: [NodeId; 2] =
    [PAINTER_SCULPT_RADIUS_SLIDER, PAINTER_SCULPT_OFFSET_SLIDER];

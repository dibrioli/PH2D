//! Flip module chrome NodeIds (ADR-0114 W2 — docked `ph2d-panel-flip`).
//!
//! The `flip` tool's Brush / Color / Layers controls live in a right-docked
//! `Panel<State>` (the tool `FloatingPanel` is unpainted, mirror of the Vector
//! Style panel). Fixed chrome ids below (`FLIP_*`); the per-layer row widgets
//! use a runtime-hashed id family ([`flip_layer_widget_id`], mirror of the
//! Painter layers panel) since the layer count is only known at runtime.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/flip.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ── Canvas mode (Select / Draw / Erase — ADR-0112 arbitration) ───────────────
/// Select: the sprite gizmo moves the object (no drawing).
pub const FLIP_MODE_SELECT: NodeId = hash_node_id("flip.mode.select");

/// Draw: each canvas drag creates a stroke on the active drawing.
pub const FLIP_MODE_DRAW: NodeId = hash_node_id("flip.mode.draw");

/// Erase: removes coverage / strokes (see the Erase sub-mode row).
pub const FLIP_MODE_ERASE: NodeId = hash_node_id("flip.mode.erase");

/// Fill: one click floods the region bounded by the line-art (W4).
pub const FLIP_MODE_FILL: NodeId = hash_node_id("flip.mode.fill");

/// Shape row (Draw mode): does the stroke carry its OWN fill?
///
/// This is the Grease Pencil material with `show_stroke` + `show_fill` on the SAME
/// curve (that is how the Suzanne artwork is made): the fill is the triangulation of
/// the stroke's own points, so line and colour are ONE geometry — sculpt the line and
/// the colour follows, exactly, in the same frame.
pub const FLIP_SHAPE_LINE: NodeId = hash_node_id("flip.shape.line");

pub const FLIP_SHAPE_FILLED: NodeId = hash_node_id("flip.shape.filled");

/// Tip (Draw mode, 03 §8): the brush POINT along the stroke — a full line, or beads spaced
/// by arc-length (round Dots / Squares). `FLIP_DOT_SPACING` is the gap between beads (WORLD).
pub const FLIP_TIP_LINE: NodeId = hash_node_id("flip.tip.line");

pub const FLIP_TIP_DOTS: NodeId = hash_node_id("flip.tip.dots");

pub const FLIP_TIP_SQUARES: NodeId = hash_node_id("flip.tip.squares");

pub const FLIP_DOT_SPACING: NodeId = hash_node_id("flip.tip.spacing");

/// Cap (Draw mode): the stroke's TIP SHAPE — the three of the drawing standard. `Round` ends in
/// the brush disc (the GP default), `Flat` cuts straight where the hand stopped (the SVG `butt`),
/// `Square` extends half a thickness past the end and cuts there.
///
/// ⚠️ **The engine has honoured `FlipStroke::cap` end-to-end since the walk engine landed** — the
/// flag bits, the half-plane `max` in the silhouette, the `flip.wgsl` branch, all gated and
/// parity-proven. What did not exist was a way for the artist to ASK: `build_stroke` never wrote
/// `s.cap`, so every stroke got the default and `Cap::Flat` was reachable only from a test. These
/// three ids are that door.
pub const FLIP_CAP_ROUND: NodeId = hash_node_id("flip.cap.round");

pub const FLIP_CAP_FLAT: NodeId = hash_node_id("flip.cap.flat");

pub const FLIP_CAP_SQUARE: NodeId = hash_node_id("flip.cap.square");

/// Self Overlap (Draw mode, 03 §8): a toggle — when on, a stroke that crosses itself ACCUMULATES
/// (darkens at the crossing, like a marker) instead of the flat union. The GP `GP_STROKE_OVERLAP`.
pub const FLIP_SELF_OVERLAP: NodeId = hash_node_id("flip.self_overlap");

/// Airbrush (Draw mode, 03 §8): a toggle — when on, the brush edge falloff becomes the physical
/// Beer-Lambert transmittance of a spherical dab (a wide soft dome) instead of pow+smoothstep.
pub const FLIP_AIRBRUSH: NodeId = hash_node_id("flip.airbrush");

/// Pressure dynamics (Draw): the MINIMUM width (fraction of Size at zero pen pressure) + its chip.
pub const FLIP_PRESSURE_MIN: NodeId = hash_node_id("flip.pressure.min");

/// Pressure dynamics (Draw): the RESPONSE curve (soft↔hard, 0.5 = linear) + its chip.
pub const FLIP_PRESSURE_RESPONSE: NodeId = hash_node_id("flip.pressure.response");

/// Reshape: sculpts the strokes already drawn (W5 — see the Reshape section).
pub const FLIP_MODE_RESHAPE: NodeId = hash_node_id("flip.mode.reshape");

/// Edit: selects STROKES (W6 — the Grease Pencil Edit Mode).
///
/// A mode of its own, NOT an overload of Select: in Select the sprite **gizmo** owns the
/// click (it moves the whole Flip object, ADR-0112). Same split as the Grease Pencil's
/// Object Mode vs Edit Mode.
pub const FLIP_MODE_EDIT: NodeId = hash_node_id("flip.mode.edit");

/// Colorize: scribble colours over the line-art; the LazyBrush cut turns the scribbles
/// into filled regions in ONE solve (COLORIZE C2 — `docs/Flip/09_colorize.md`).
pub const FLIP_MODE_COLORIZE: NodeId = hash_node_id("flip.mode.colorize");

/// Trace (Shift & Trace, `docs/Flip/04 §4`): canvas drags SHIFT the ghost under the
/// cursor (Ctrl rotates) — display only, the digital sliding paper of the lightbox.
pub const FLIP_MODE_TRACE: NodeId = hash_node_id("flip.mode.trace");

/// Edit section (W8): the selection DOMAIN — whole strokes (the GP Curve domain).
pub const FLIP_EDIT_DOM_STROKE: NodeId = hash_node_id("flip.edit.dom.stroke");

/// Edit section (W8): the selection DOMAIN — individual points (the GP Point domain).
pub const FLIP_EDIT_DOM_POINT: NodeId = hash_node_id("flip.edit.dom.point");

/// Edit section (ADR-0114 §4.B): the selection DOMAIN — the stretch of a stroke between
/// two crossings (the GP Segment mode; Point domain plus a pick policy).
pub const FLIP_EDIT_DOM_SEGMENT: NodeId = hash_node_id("flip.edit.dom.segment");

// ── Reshape section (shown only in Reshape mode, ADR-0114 W5) ───────────────
/// The eight sculpt brushes, in panel order (two rows of four). They share the
/// Brush section's Size (radius) and Strength — a Reshape with its own pair of
/// sliders for the same two quantities would be duplicate state, and the user
/// would have to re-tune the brush on every mode switch.
pub const FLIP_RS_SMOOTH: NodeId = hash_node_id("flip.reshape.smooth");

pub const FLIP_RS_PUSH: NodeId = hash_node_id("flip.reshape.push");

pub const FLIP_RS_GRAB: NodeId = hash_node_id("flip.reshape.grab");

pub const FLIP_RS_PINCH: NodeId = hash_node_id("flip.reshape.pinch");

pub const FLIP_RS_TWIST: NodeId = hash_node_id("flip.reshape.twist");

pub const FLIP_RS_THICKNESS: NodeId = hash_node_id("flip.reshape.thickness");

pub const FLIP_RS_STRENGTH: NodeId = hash_node_id("flip.reshape.strength");

pub const FLIP_RS_RANDOMIZE: NodeId = hash_node_id("flip.reshape.randomize");

/// The eight ids in the SAME order as `ReshapeKind::ALL` — the table that the
/// panel paints and the event router decodes. One list, one order: adding a
/// brush means adding it here and in `ReshapeKind::ALL`, and the seam test that
/// drives every id proves the two never drift apart.
pub const FLIP_RESHAPE_KIND_IDS: [NodeId; 8] = [
    FLIP_RS_SMOOTH,
    FLIP_RS_PUSH,
    FLIP_RS_GRAB,
    FLIP_RS_PINCH,
    FLIP_RS_TWIST,
    FLIP_RS_THICKNESS,
    FLIP_RS_STRENGTH,
    FLIP_RS_RANDOMIZE,
];

/// Paint (the usual bucket) / Paint Behind (colour under what is already painted) /
/// Unpaint (remove the fill under the click) — the animation-bucket semantics.
pub const FLIP_FILL_PAINT: NodeId = hash_node_id("flip.fill.paint");

pub const FLIP_FILL_BEHIND: NodeId = hash_node_id("flip.fill.behind");

pub const FLIP_FILL_UNPAINT: NodeId = hash_node_id("flip.fill.unpaint");

/// Gap Closure reach (screen px; `0` = off) + its chip.
pub const FLIP_GAP: NodeId = hash_node_id("flip.fill.gap");

/// Grow / Shrink (buffer px; positive tucks the colour under the line) + its chip.
pub const FLIP_GROW: NodeId = hash_node_id("flip.fill.grow");

/// Precision (bucket buffer resolution) + its chip.
pub const FLIP_PRECISION: NodeId = hash_node_id("flip.fill.precision");

/// Trap — the trapped-ball radius (screen px; `0` = off) + its chip. A ball of radius
/// `r` cannot cross a gap narrower than `2r`, so the bucket stops leaking through an
/// unclosed outline without the artist hunting for the gap (COLORIZE C1).
pub const FLIP_TRAP: NodeId = hash_node_id("flip.fill.trap");

/// **Bleed** slider (6º smoke) + its chip: how deep a colour reaches through an OPEN gap in
/// a divider (the lens). `0` = hugs the line, `1` = deep bulge — the CONTINUOUS,
/// zoom-immune leak control (a distance-to-ink metric), where the Trap is the BINARY seal.
/// The Colorize section reuses `FLIP_TRAP` for the seal (it drives the same `style.trap`).
pub const FLIP_COLORIZE_BLEED: NodeId = hash_node_id("flip.colorize.bleed");

// ── Brush section (size / hardness / opacity / smoothing) ────────────────────
/// Stroke width slider (track `0..1` → `1..64` px) + its px chip.
pub const FLIP_SIZE: NodeId = hash_node_id("flip.size");

/// Edge hardness slider (`0..1`) + its chip.
pub const FLIP_HARDNESS: NodeId = hash_node_id("flip.hardness");

/// Stroke opacity slider (`0..1` → `0..100 %`) + its chip.
pub const FLIP_OPACITY: NodeId = hash_node_id("flip.opacity");

/// Active-smoothing slider (`0..1`, the "settle") + its chip.
pub const FLIP_SMOOTHING: NodeId = hash_node_id("flip.smoothing");

// ── Eraser's OWN size / strength + the LINK toggles (ADR-0114 §4.C) ──────────
//
// Blender's *Unified Paint Settings*: a property is either shared between the paint
// brush and the eraser or owned by each, and a small link toggle ON THE PROPERTY ROW
// says which. **Linked is the default**, so the eraser keeps using the brush's Size /
// Strength (`FLIP_SIZE` / `FLIP_OPACITY`) exactly as it always has — unlinking is
// opt-in, and only then do the ids below get painted and drive the eraser.
//
// Two widgets per property (never one re-seeded): a slider slot holds ONE value, and
// a single slot could not remember the brush's number and the eraser's at once.
/// Link toggle on the Size row (paint brush ↔ eraser). ON = the eraser uses the
/// brush's Size; OFF = it uses [`FLIP_ERASE_SIZE`].
pub const FLIP_LINK_SIZE: NodeId = hash_node_id("flip.link.size");

/// Link toggle on the Strength row. ON = the eraser uses the brush's Strength
/// (`FLIP_OPACITY`); OFF = it uses [`FLIP_ERASE_STRENGTH`].
pub const FLIP_LINK_STRENGTH: NodeId = hash_node_id("flip.link.strength");

/// The eraser's OWN radius slider + px chip — painted only in Erase mode with the
/// Size link OFF.
pub const FLIP_ERASE_SIZE: NodeId = hash_node_id("flip.erase.size");

/// The eraser's OWN strength slider + % chip — painted only in Erase mode with the
/// Strength link OFF.
pub const FLIP_ERASE_STRENGTH: NodeId = hash_node_id("flip.erase.strength");

// ── Erase sub-mode (shown only in Erase mode) ────────────────────────────────
/// Soft (reduce opacity — default, most paint-like), Hard (cut), Stroke (erase
/// whole touched stroke). Mirror of the Grease Pencil eraser.
pub const FLIP_ERASE_SOFT: NodeId = hash_node_id("flip.erase.soft");

pub const FLIP_ERASE_HARD: NodeId = hash_node_id("flip.erase.hard");

pub const FLIP_ERASE_STROKE: NodeId = hash_node_id("flip.erase.stroke");

// ── Layers toolbar (fixed) ───────────────────────────────────────────────────
/// Add a new layer on top (of the active object).
pub const FLIP_LAYER_ADD: NodeId = hash_node_id("flip.layer.add");

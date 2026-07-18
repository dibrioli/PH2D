//! Timeline context menus — the five tables the timeline right-clicks paint.
//!
//! Split out of `menus.rs` (which crossed the file LOC cap) because they are one
//! subject: the segment presets and their easing submenu, the track row, the stack
//! lane, and the clip strip. Every one of them is a **table**, and every table has
//! the same three consumers — the overlay paints it, `pre_populate` registers it,
//! and someone downstream resolves each row. A row added to a table and forgotten
//! by the resolver is a menu item that silently does nothing; that shape is why
//! they are tables and not hand-listed consts, and a gate walks each one.

use super::*;

// ── Timeline segment (graph editor) preset menu — W3.E4 ────────────────────
// Right-click a key (its dope-sheet diamond or its graph anchor) to retune the
// interpolation LEAVING that key. `Hold`/`Linear`/`Custom` land directly; the
// three cascade rows replace the menu with the easing-family submenu for that
// mode (`ContextMenuKind::TimelineSegmentEase`, same `\u{25b6}` cascade shape as
// Settings). editor-core never names an easing: the clicked id crosses to the
// shell, which owns the `ph2d-anim` vocabulary.
/// Step: hold the value, then jump at the next key.
pub const CTX_MENU_TL_HOLD: NodeId = hash_node_id("ctx_menu_tl_hold");
/// Straight line to the next key.
pub const CTX_MENU_TL_LINEAR: NodeId = hash_node_id("ctx_menu_tl_linear");
/// Freeze the drawn handles into an editable bézier (`Interp::to_bezier`).
pub const CTX_MENU_TL_CUSTOM: NodeId = hash_node_id("ctx_menu_tl_custom");
/// Toggle roving (AE "rove across time"): the key's time stops being authored
/// and is derived so the value travels at constant speed between its pinned
/// neighbours.
pub const CTX_MENU_TL_ROVE: NodeId = hash_node_id("ctx_menu_tl_rove");
/// Cascade row: easing families, accelerating from rest.
pub const CTX_MENU_TL_EASE_IN: NodeId = hash_node_id("ctx_menu_tl_ease_in");
/// Cascade row: easing families, decelerating to rest.
pub const CTX_MENU_TL_EASE_OUT: NodeId = hash_node_id("ctx_menu_tl_ease_out");
/// Cascade row: easing families, symmetric.
pub const CTX_MENU_TL_EASE_INOUT: NodeId = hash_node_id("ctx_menu_tl_ease_inout");

/// Sinusoidal easing family.
pub const CTX_MENU_TL_FAM_SINE: NodeId = hash_node_id("ctx_menu_tl_fam_sine");
/// Quadratic easing family.
pub const CTX_MENU_TL_FAM_QUAD: NodeId = hash_node_id("ctx_menu_tl_fam_quad");
/// Cubic easing family.
pub const CTX_MENU_TL_FAM_CUBIC: NodeId = hash_node_id("ctx_menu_tl_fam_cubic");
/// Quartic easing family.
pub const CTX_MENU_TL_FAM_QUART: NodeId = hash_node_id("ctx_menu_tl_fam_quart");
/// Quintic easing family.
pub const CTX_MENU_TL_FAM_QUINT: NodeId = hash_node_id("ctx_menu_tl_fam_quint");
/// Exponential easing family.
pub const CTX_MENU_TL_FAM_EXPO: NodeId = hash_node_id("ctx_menu_tl_fam_expo");
/// Circular easing family.
pub const CTX_MENU_TL_FAM_CIRC: NodeId = hash_node_id("ctx_menu_tl_fam_circ");
/// Anticipation/overshoot easing family.
pub const CTX_MENU_TL_FAM_BACK: NodeId = hash_node_id("ctx_menu_tl_fam_back");
/// Elastic-spring easing family.
pub const CTX_MENU_TL_FAM_ELASTIC: NodeId = hash_node_id("ctx_menu_tl_fam_elastic");
/// Damped-bounce easing family.
pub const CTX_MENU_TL_FAM_BOUNCE: NodeId = hash_node_id("ctx_menu_tl_fam_bounce");

/// Wire encoding of an easing mode, carried in
/// `ContextMenuKind::TimelineSegmentEase` from the cascade row that opened the
/// submenu. Opaque to editor-core; the shell decodes it into an `EasingMode`.
pub const TL_EASE_MODE_IN: u8 = 0;
/// Decelerate to rest — see [`TL_EASE_MODE_IN`].
pub const TL_EASE_MODE_OUT: u8 = 1;
/// Accelerate then decelerate — see [`TL_EASE_MODE_IN`].
pub const TL_EASE_MODE_INOUT: u8 = 2;

/// The top-level segment menu, in paint order: the two shapes that need no
/// tuning, the three easing cascades, then the escape hatch to free handles.
///
/// One table, three consumers — the overlay paints it, `pre_populate` registers
/// it, and the shell's gate proves every row resolves to something. A row added
/// here and forgotten in the shell is a compile-green menu item that does
/// nothing; that is the bug this shape exists to make impossible.
pub const TIMELINE_SEGMENT_MENU: [(NodeId, &str, Option<[u8; 4]>); 7] = [
    (CTX_MENU_TL_HOLD, "Hold", None),
    (CTX_MENU_TL_LINEAR, "Linear", None),
    (CTX_MENU_TL_EASE_IN, "Ease In \u{25b6}", None),
    (CTX_MENU_TL_EASE_OUT, "Ease Out \u{25b6}", None),
    (CTX_MENU_TL_EASE_INOUT, "Ease In-Out \u{25b6}", None),
    (CTX_MENU_TL_CUSTOM, "Custom (B\u{e9}zier)", None),
    (CTX_MENU_TL_ROVE, "Rove Across Time", None),
];

// ── Timeline track-row menu ─────────────────────────────────────────────────
// Right-click a track row's LABEL (the left name column) for actions on the
// whole track. Same one-table-three-consumers shape as the segment menu.
/// Delete the track: unbind the row's `(entity, prop)` — binding + keys go in
/// one undo step (`TimelineIntent::Unbind`).
pub const CTX_MENU_TL_DELETE_TRACK: NodeId = hash_node_id("ctx_menu_tl_delete_track");

/// The track-row menu, in paint order. One table, three consumers — the
/// overlay paints it, `pre_populate` registers it, and the timeline panel's
/// `apply_event` resolves every row (a row added here and unhandled there is a
/// menu item that silently does nothing — the bug this table shape prevents).
pub const TIMELINE_TRACK_MENU: [(NodeId, &str, Option<[u8; 4]>); 1] =
    [(CTX_MENU_TL_DELETE_TRACK, "Delete Track", None)];

// ── Timeline lane menu (ADR-0115 B5) ────────────────────────────────────────
// Right-click a lane's LABEL. The lane's WEIGHT is a field on the row (a number
// you nudge continuously belongs under the pointer, not two clicks deep); its
// MODE is set once and belongs here. And Delete Lane lives here because a lane
// row has no room for a third button — not because deleting is rare.
/// Mix toward this lane's value (`lerp`) — it replaces what is under it.
pub const CTX_MENU_TL_LANE_OVERRIDE: NodeId = hash_node_id("ctx_menu_tl_lane_override");
/// Add this lane's DELTA against the first frame of its own clip.
pub const CTX_MENU_TL_LANE_ADDITIVE: NodeId = hash_node_id("ctx_menu_tl_lane_additive");
/// Delete the lane and every strip on it.
pub const CTX_MENU_TL_LANE_DELETE: NodeId = hash_node_id("ctx_menu_tl_lane_delete");

/// The lane menu, in paint order. One table, three consumers — same shape, same
/// gate as the strip menu below.
pub const TIMELINE_LANE_MENU: [(NodeId, &str, Option<[u8; 4]>); 3] = [
    (CTX_MENU_TL_LANE_OVERRIDE, "Override", None),
    (CTX_MENU_TL_LANE_ADDITIVE, "Additive", None),
    (CTX_MENU_TL_LANE_DELETE, "Delete Lane", None),
];

// ── Timeline clip-strip menu (ADR-0115 B6) ──────────────────────────────────
// Right-click a strip in a stack lane. What is NOT here is as deliberate as what
// is: no blend-in/blend-out numbers (the overlap IS the blend — ADR-0115 R1) and
// no speed field (a rate is felt, not typed: Cmd-drag an edge to stretch). The
// menu carries only what a pointer cannot say.
/// Copy the strip and lay the copy down right after it.
pub const CTX_MENU_TL_STRIP_ENTER: NodeId = hash_node_id("ctx.tl.strip.enter");
pub const CTX_MENU_TL_STRIP_DUPLICATE: NodeId = hash_node_id("ctx_menu_tl_strip_duplicate");
/// Remove the strip from its lane.
pub const CTX_MENU_TL_STRIP_DELETE: NodeId = hash_node_id("ctx_menu_tl_strip_delete");
/// Source plays once, then holds its last value.
pub const CTX_MENU_TL_STRIP_ONCE: NodeId = hash_node_id("ctx_menu_tl_strip_once");
/// Source wraps to the start of its slice.
pub const CTX_MENU_TL_STRIP_LOOP: NodeId = hash_node_id("ctx_menu_tl_strip_loop");
/// Source reflects: forward, backward, forward.
pub const CTX_MENU_TL_STRIP_PINGPONG: NodeId = hash_node_id("ctx_menu_tl_strip_pingpong");
/// Back to real time — the span re-lengthens to fit the slice at rate 1.
pub const CTX_MENU_TL_STRIP_RESET_SPEED: NodeId = hash_node_id("ctx_menu_tl_strip_reset_speed");

/// The strip menu, in paint order: the two edits to the strip itself, then the
/// three things its source can do when it runs out, then the retime escape.
///
/// One table, three consumers — the overlay paints it, `pre_populate` registers
/// it, and the timeline panel's `apply_event` resolves every row. A row added
/// here and unhandled there is a menu item that silently does nothing; the seam
/// test walks this table and fails on exactly that.
pub const TIMELINE_STRIP_MENU: [(NodeId, &str, Option<[u8; 4]>); 7] = [
    // First, because it is the only row that CHANGES WHERE YOU ARE rather than editing the
    // strip under the cursor — and because on a container strip it is the thing you came for.
    // On a clip strip it raises nothing (`enter_target` returns `None`): a row that acted on
    // the wrong kind of strip would be worse than one that is simply inert here.
    (CTX_MENU_TL_STRIP_ENTER, "Enter Container", None),
    (CTX_MENU_TL_STRIP_DUPLICATE, "Duplicate Strip", None),
    (CTX_MENU_TL_STRIP_DELETE, "Delete Strip", None),
    (CTX_MENU_TL_STRIP_ONCE, "Play Once", None),
    (CTX_MENU_TL_STRIP_LOOP, "Loop", None),
    (CTX_MENU_TL_STRIP_PINGPONG, "Ping-Pong", None),
    (CTX_MENU_TL_STRIP_RESET_SPEED, "Reset Speed", None),
];

/// The easing-family submenu, shared by all three modes (the mode rides in the
/// `ContextMenuKind`). Ordered gentlest-first, with the two non-monotone
/// families (overshoot, bounce) last — the order animators scan.
pub const TIMELINE_EASE_MENU: [(NodeId, &str, Option<[u8; 4]>); 10] = [
    (CTX_MENU_TL_FAM_SINE, "Sine", None),
    (CTX_MENU_TL_FAM_QUAD, "Quad", None),
    (CTX_MENU_TL_FAM_CUBIC, "Cubic", None),
    (CTX_MENU_TL_FAM_QUART, "Quart", None),
    (CTX_MENU_TL_FAM_QUINT, "Quint", None),
    (CTX_MENU_TL_FAM_EXPO, "Expo", None),
    (CTX_MENU_TL_FAM_CIRC, "Circ", None),
    (CTX_MENU_TL_FAM_BACK, "Back", None),
    (CTX_MENU_TL_FAM_ELASTIC, "Elastic", None),
    (CTX_MENU_TL_FAM_BOUNCE, "Bounce", None),
];

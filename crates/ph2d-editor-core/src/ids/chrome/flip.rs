//! Flip module chrome NodeIds (ADR-0114 W2 — docked `ph2d-panel-flip`).
//!
//! The `flip` tool's Brush / Color / Layers controls live in a right-docked
//! `Panel<State>` (the tool `FloatingPanel` is unpainted, mirror of the Vector
//! Style panel). Fixed chrome ids below (`FLIP_*`); the per-layer row widgets
//! use a runtime-hashed id family ([`flip_layer_widget_id`], mirror of the
//! Painter layers panel) since the layer count is only known at runtime.
use super::{NodeId, hash_node_id};

// ── Flip Style panel (docked `ph2d-panel-flip`) ──────────────────────────────
/// Flip panel outer rect id (for `z_order` + hit-barrier).
pub const FLIP_PANEL: NodeId = hash_node_id("flip.panel");
/// Flip panel close (X) button.
pub const FLIP_CLOSE: NodeId = hash_node_id("flip.close");

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
/// Edit section: deletes the selected strokes.
pub const FLIP_EDIT_DELETE: NodeId = hash_node_id("flip.edit.delete");
/// Edit section (W8): the selection DOMAIN — whole strokes (the GP Curve domain).
pub const FLIP_EDIT_DOM_STROKE: NodeId = hash_node_id("flip.edit.dom.stroke");
/// Edit section (W8): the selection DOMAIN — individual points (the GP Point domain).
pub const FLIP_EDIT_DOM_POINT: NodeId = hash_node_id("flip.edit.dom.point");
/// Edit section (ADR-0114 §4.B): the selection DOMAIN — the stretch of a stroke between
/// two crossings (the GP Segment mode; Point domain plus a pick policy).
pub const FLIP_EDIT_DOM_SEGMENT: NodeId = hash_node_id("flip.edit.dom.segment");
/// Edit section: clears the selection (deselect all).
pub const FLIP_EDIT_DESELECT: NodeId = hash_node_id("flip.edit.deselect");
/// Edit section: selects every stroke of the active drawing.
pub const FLIP_EDIT_SELECT_ALL: NodeId = hash_node_id("flip.edit.select_all");

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

// ── Fill section (shown only in Fill mode, ADR-0114 W4) ─────────────────────
/// Fill-colour swatch — its OWN colour (colouring uses a different palette than
/// drawing; forcing the stroke colour to double as the fill colour would be hostile).
pub const FLIP_FILL_SWATCH: NodeId = hash_node_id("flip.fill.swatch");
/// Paint (the usual bucket) / Paint Behind (colour under what is already painted) /
/// Unpaint (remove the fill under the click) — the animation-bucket semantics.
pub const FLIP_FILL_PAINT: NodeId = hash_node_id("flip.fill.paint");
pub const FLIP_FILL_BEHIND: NodeId = hash_node_id("flip.fill.behind");
pub const FLIP_FILL_UNPAINT: NodeId = hash_node_id("flip.fill.unpaint");
/// Gap Closure reach (screen px; `0` = off) + its chip.
pub const FLIP_GAP: NodeId = hash_node_id("flip.fill.gap");
pub const FLIP_GAP_NUM: NodeId = hash_node_id("flip.fill.gap_num");
/// Grow / Shrink (buffer px; positive tucks the colour under the line) + its chip.
pub const FLIP_GROW: NodeId = hash_node_id("flip.fill.grow");
pub const FLIP_GROW_NUM: NodeId = hash_node_id("flip.fill.grow_num");
/// Precision (bucket buffer resolution) + its chip.
pub const FLIP_PRECISION: NodeId = hash_node_id("flip.fill.precision");
pub const FLIP_PRECISION_NUM: NodeId = hash_node_id("flip.fill.precision_num");
/// Trap — the trapped-ball radius (screen px; `0` = off) + its chip. A ball of radius
/// `r` cannot cross a gap narrower than `2r`, so the bucket stops leaking through an
/// unclosed outline without the artist hunting for the gap (COLORIZE C1).
pub const FLIP_TRAP: NodeId = hash_node_id("flip.fill.trap");
pub const FLIP_TRAP_NUM: NodeId = hash_node_id("flip.fill.trap_num");

// ── Colorize section (shown only in Colorize mode, C2 — `docs/Flip/09`) ──────
/// Colorize-colour swatch — the colour the NEXT scribble seeds (its own palette).
pub const FLIP_COLORIZE_SWATCH: NodeId = hash_node_id("flip.colorize.swatch");
/// Apply: run the LazyBrush cut over the accumulated scribbles + the line-art, commit
/// each region as a filled stroke, and clear the scribble buffer.
pub const FLIP_COLORIZE_APPLY: NodeId = hash_node_id("flip.colorize.apply");
/// Clear: drop the accumulated scribbles without colouring.
pub const FLIP_COLORIZE_CLEAR: NodeId = hash_node_id("flip.colorize.clear");
/// **Bleed** slider (6º smoke) + its chip: how deep a colour reaches through an OPEN gap in
/// a divider (the lens). `0` = hugs the line, `1` = deep bulge — the CONTINUOUS,
/// zoom-immune leak control (a distance-to-ink metric), where the Trap is the BINARY seal.
/// The Colorize section reuses `FLIP_TRAP` for the seal (it drives the same `style.trap`).
pub const FLIP_COLORIZE_BLEED: NodeId = hash_node_id("flip.colorize.bleed");
pub const FLIP_COLORIZE_BLEED_NUM: NodeId = hash_node_id("flip.colorize.bleed_num");

// ── Brush section (size / hardness / opacity / smoothing) ────────────────────
/// Stroke width slider (track `0..1` → `1..64` px) + its px chip.
pub const FLIP_SIZE: NodeId = hash_node_id("flip.size");
pub const FLIP_SIZE_NUM: NodeId = hash_node_id("flip.size_num");
/// Edge hardness slider (`0..1`) + its chip.
pub const FLIP_HARDNESS: NodeId = hash_node_id("flip.hardness");
pub const FLIP_HARDNESS_NUM: NodeId = hash_node_id("flip.hardness_num");
/// Stroke opacity slider (`0..1` → `0..100 %`) + its chip.
pub const FLIP_OPACITY: NodeId = hash_node_id("flip.opacity");
pub const FLIP_OPACITY_NUM: NodeId = hash_node_id("flip.opacity_num");
/// Active-smoothing slider (`0..1`, the "settle") + its chip.
pub const FLIP_SMOOTHING: NodeId = hash_node_id("flip.smoothing");
pub const FLIP_SMOOTHING_NUM: NodeId = hash_node_id("flip.smoothing_num");

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
pub const FLIP_ERASE_SIZE_NUM: NodeId = hash_node_id("flip.erase.size_num");
/// The eraser's OWN strength slider + % chip — painted only in Erase mode with the
/// Strength link OFF.
pub const FLIP_ERASE_STRENGTH: NodeId = hash_node_id("flip.erase.strength");
pub const FLIP_ERASE_STRENGTH_NUM: NodeId = hash_node_id("flip.erase.strength_num");

// ── Color section ────────────────────────────────────────────────────────────
/// Stroke-colour swatch — a picker swatch (opens the shared OKLCH picker on
/// Down); the shell `flip_bridge` reads the pick back into the tool.
pub const FLIP_STROKE_SWATCH: NodeId = hash_node_id("flip.stroke_swatch");

// ── Erase sub-mode (shown only in Erase mode) ────────────────────────────────
/// Soft (reduce opacity — default, most paint-like), Hard (cut), Stroke (erase
/// whole touched stroke). Mirror of GP `erase.cc`.
pub const FLIP_ERASE_SOFT: NodeId = hash_node_id("flip.erase.soft");
pub const FLIP_ERASE_HARD: NodeId = hash_node_id("flip.erase.hard");
pub const FLIP_ERASE_STROKE: NodeId = hash_node_id("flip.erase.stroke");

// ── Layers toolbar (fixed) ───────────────────────────────────────────────────
/// Add a new layer on top (of the active object).
pub const FLIP_LAYER_ADD: NodeId = hash_node_id("flip.layer.add");
/// Duplicate the active layer (an independent copy, above the original — ADR-0114 §4.C).
pub const FLIP_LAYER_DUPLICATE: NodeId = hash_node_id("flip.layer.duplicate");
/// Delete the active layer.
pub const FLIP_LAYER_DELETE: NodeId = hash_node_id("flip.layer.delete");
/// Inline layer-rename field (ADR-0114 §4.C): double-clicking a layer's name opens
/// a single-line `TextInput` over the name strip, seeded with the current name.
/// ONE field, reused across rows (only one rename is open at a time) — mirror of the
/// timeline's `TIMELINE_MARKER_RENAME_INPUT`.
pub const FLIP_LAYER_RENAME_INPUT: NodeId = hash_node_id("flip.layer.rename_input");

/// Runtime FNV-1a 64-bit over `s`, byte-identical to the `const fn`
/// [`hash_node_id`] (which only accepts `&'static str`). Needed because the
/// per-layer row ids are derived from a runtime `format!` (the `LayerId` is only
/// known at runtime). Kept flip-local (isolation, ADR-0114) — its agreement with
/// `hash_node_id` is pinned by the test at the bottom of this module.
fn flip_fnv_node_id(s: &str) -> NodeId {
    const FNV_OFFSET_BASIS_64: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME_64: u64 = 0x0000_0100_0000_01b3;
    let mut hash: u64 = FNV_OFFSET_BASIS_64;
    for &b in s.as_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME_64);
    }
    if hash == 0 {
        hash = 1; // reserve NodeId(0) = a11y root, mirror of hash_node_id
    }
    NodeId(hash)
}

/// Which control on a Flip layers-panel row a runtime id addresses. The layer id
/// is only known at runtime, so per-row widgets hash `(layer_u64, kind)` into a
/// [`NodeId`] via [`flip_layer_widget_id`] (mirror of `PainterLayerWidget`). A
/// new control kind needs no new fixed const — add a variant here.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FlipLayerWidget {
    /// The row body — click selects (activates) the layer.
    Row,
    /// The eye toggle — click flips the layer's visibility.
    Visibility,
    /// The padlock toggle — click flips the layer's lock.
    Lock,
    /// The opacity slider (stores `0..1`).
    Opacity,
    /// The blend-mode dropdown chip (opens the blend popover).
    Blend,
    /// The move-up (↑) reorder button — moves the layer toward the top.
    MoveUp,
    /// The move-down (↓) reorder button — moves the layer toward the back.
    MoveDown,
}

impl FlipLayerWidget {
    /// Stable tag woven into the hashed id string. Changing a tag changes every
    /// derived id for that kind — keep stable.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::Visibility => "vis",
            Self::Lock => "lock",
            Self::Opacity => "opacity",
            Self::Blend => "blend",
            Self::MoveUp => "move_up",
            Self::MoveDown => "move_down",
        }
    }

    /// All kinds, in a fixed order — the decoder iterates this.
    pub const ALL: [FlipLayerWidget; 7] = [
        Self::Row,
        Self::Visibility,
        Self::Lock,
        Self::Opacity,
        Self::Blend,
        Self::MoveUp,
        Self::MoveDown,
    ];
}

/// Derive the stable [`NodeId`] for the `kind` control on the Flip layers-panel
/// row whose layer has runtime id `layer_id`. FNV-hashed from
/// `"flip_layer.<kind>.<layer_id>"`. Runtime `format!` is acceptable here: the
/// layers panel is not a hot path (≤ a handful of layers, repainted per frame
/// like the sidebar formats "NN px"). See [`FlipLayerWidget`].
#[must_use]
pub fn flip_layer_widget_id(layer_id: u64, kind: FlipLayerWidget) -> NodeId {
    flip_fnv_node_id(&format!("flip_layer.{}.{}", kind.tag(), layer_id))
}

/// Derive the stable [`NodeId`] for blend-mode option `mode` (the `BlendMode`
/// wire discriminant, `0..MAX_BLEND_MODES`) in the open blend dropdown popover
/// of the row whose layer has runtime id `layer_id`. Only the single open
/// popover's options are ever hit-registered, so the `format!` cost is bounded.
#[must_use]
pub fn flip_layer_blend_option_id(layer_id: u64, mode: u8) -> NodeId {
    flip_fnv_node_id(&format!("flip_layer.blendopt.{layer_id}.{mode}"))
}

// ── Frame strip (bottom-docked `ph2d-panel-flip-frames`, ADR-0114 W3) ────────
// The animator's inner loop: the cells of the active layer, the transport, Ghost
// Frames, autokey and the tween. Bottom dock (its own band — the global timeline
// is a separate, deferred integration, W6).

/// Frame-strip panel outer rect id (z-order + hit-barrier).
pub const FLIP_STRIP_PANEL: NodeId = hash_node_id("flip.strip.panel");
/// Frame-strip close (X) button.
pub const FLIP_STRIP_CLOSE: NodeId = hash_node_id("flip.strip.close");
/// **Scrub lane** (W7.3): the draggable ruler at the top of the strip that moves the
/// playhead WITHOUT touching the multiframe selection — the standard split of every
/// animation tool (the ruler scrubs, the cells select). A horizontal `Slider` drives
/// the drag; the panel maps its `0..1` value to a frame and the shell seeks there.
pub const FLIP_SCRUB: NodeId = hash_node_id("flip.strip.scrub");

// ── Transport ────────────────────────────────────────────────────────────────
/// Play / pause toggle.
pub const FLIP_PLAY: NodeId = hash_node_id("flip.strip.play");
/// Previous DRAWING (skips holds — the animator's flip, not a frame step).
pub const FLIP_PREV_DRAWING: NodeId = hash_node_id("flip.strip.prev");
/// Next DRAWING.
pub const FLIP_NEXT_DRAWING: NodeId = hash_node_id("flip.strip.next");
/// Frames-per-second chip (the object's own FPS).
pub const FLIP_FPS_NUM: NodeId = hash_node_id("flip.strip.fps_num");

// ── Ghost Frames ─────────────────────────────────────────────────────────────
/// Ghost Frames on/off (per object).
pub const FLIP_GHOST: NodeId = hash_node_id("flip.strip.ghost");
/// How many drawings BEFORE to ghost.
pub const FLIP_GHOST_BEFORE_NUM: NodeId = hash_node_id("flip.strip.ghost_before_num");
/// How many drawings AFTER to ghost.
pub const FLIP_GHOST_AFTER_NUM: NodeId = hash_node_id("flip.strip.ghost_after_num");

// ── Autokey (per-tool semantics — see `ph2d_flip::AutokeyPolicy`) ────────────
/// Autokey on/off: drawing past a key's hold creates a new key.
pub const FLIP_AUTOKEY: NodeId = hash_node_id("flip.strip.autokey");
/// Multiframe **Falloff** (W7): os quadros vizinhos recebem menos influência que o
/// active one. Brushes only — the bucket is a discrete op and always uses 1.0.
pub const FLIP_FALLOFF: NodeId = hash_node_id("flip.strip.falloff");
/// Additive: a new key born of DRAWING starts as a copy (not blank).
pub const FLIP_ADDITIVE: NodeId = hash_node_id("flip.strip.additive");

// ── Key ops ──────────────────────────────────────────────────────────────────
/// Add a blank key after the current one.
pub const FLIP_KEY_ADD: NodeId = hash_node_id("flip.strip.key_add");
/// Duplicate the current key (deep copy).
pub const FLIP_KEY_DUP: NodeId = hash_node_id("flip.strip.key_dup");
/// Duplicate the current key **as an instance**: the new key points at the SAME
/// drawing (`FlipDrawing::users += 1`), so editing one edits both — how a cycle
/// reuses art. Blender calls it a *linked duplicate*; the strip marks such cells
/// with a dot.
pub const FLIP_KEY_INSTANCE: NodeId = hash_node_id("flip.strip.key_instance");
/// **Quebra o vínculo** da chave atual com a arte compartilhada (o *make single user*):
/// ela passa a ter um desenho só dela. A saída de emergência da instância — sem ela,
/// instanciar seria irreversível.
pub const FLIP_KEY_UNLINK: NodeId = hash_node_id("flip.strip.key_unlink");
/// Delete the current key.
pub const FLIP_KEY_DELETE: NodeId = hash_node_id("flip.strip.key_del");
/// Exposure (hold) of the selected key, in frames.
pub const FLIP_HOLD_NUM: NodeId = hash_node_id("flip.strip.hold_num");
/// Move the selected key one frame earlier / later.
pub const FLIP_KEY_LEFT: NodeId = hash_node_id("flip.strip.key_left");
pub const FLIP_KEY_RIGHT: NodeId = hash_node_id("flip.strip.key_right");

// ── Tween ────────────────────────────────────────────────────────────────────
/// How many inbetweens to generate.
pub const FLIP_TWEEN_NUM: NodeId = hash_node_id("flip.strip.tween_num");
/// Generate the inbetweens between the two selected keys (or the current key and
/// the next one).
pub const FLIP_TWEEN_ADD: NodeId = hash_node_id("flip.strip.tween_add");

/// The easing preset of the generated inbetweens (`Linear / Ease In / Out / In-Out`).
///
/// The MOTOR always supported it (`TweenOptions::easing`); only the toolbar did not
/// offer it — the "carry-over of UI, not of engine" that plan T3.7 declared.
pub const FLIP_TWEEN_EASE_DD: NodeId = hash_node_id("flip.strip.tween_ease_dd");

/// Fade the strokes that exist in only ONE of the two keys, instead of copying them
/// statically (`TweenOptions::fade_orphans`).
pub const FLIP_TWEEN_FADE: NodeId = hash_node_id("flip.strip.tween_fade");

/// Toggle the **pair-correction** overlay (the CACAni lesson: the matcher errs, the
/// artist corrects). While on, the canvas shows which stroke of A becomes which of B,
/// and a click re-pairs; the Add button then commits with the corrected plan.
pub const FLIP_TWEEN_PAIRS: NodeId = hash_node_id("flip.strip.tween_pairs");

/// Derive the id of easing option `preset` in the open easing dropdown popover.
#[must_use]
pub fn flip_tween_ease_option_id(preset: u8) -> NodeId {
    flip_fnv_node_id(&format!("flip.strip.easeopt.{preset}"))
}

// ── Cycle (post behavior of the active layer) ────────────────────────────────
/// The cycle dropdown chip (None / Hold / Loop / Ping-Pong).
pub const FLIP_CYCLE_DD: NodeId = hash_node_id("flip.strip.cycle_dd");

/// Derive the id of cycle option `mode` (`CycleMode as u8`) in the open cycle
/// dropdown popover.
#[must_use]
pub fn flip_cycle_option_id(mode: u8) -> NodeId {
    flip_fnv_node_id(&format!("flip.strip.cycleopt.{mode}"))
}

/// Derive the id of the strip cell at `index` (position in the active layer's
/// cell list, NOT the frame number — the index is bounded by what is painted).
#[must_use]
pub fn flip_cell_id(index: usize) -> NodeId {
    flip_fnv_node_id(&format!("flip.strip.cell.{index}"))
}

/// Derive the id of the **hold edge** of the strip cell at `index` — the grip on the
/// cell's right boundary that stretches its exposure.
///
/// A separate id from [`flip_cell_id`] because it is a separate target: the body of the
/// cell moves the key in time, its edge changes how long the key is held, and a single
/// widget cannot answer both. Same index space as the cell it belongs to.
#[must_use]
pub fn flip_hold_edge_id(index: usize) -> NodeId {
    flip_fnv_node_id(&format!("flip.strip.holdedge.{index}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_hasher_matches_const_hash_node_id() {
        // The runtime twin must agree with the const hasher for a static string,
        // so runtime-derived ids live in the same space as the fixed consts.
        assert_eq!(flip_fnv_node_id("flip.panel"), FLIP_PANEL);
        assert_eq!(flip_fnv_node_id("flip.mode.draw"), FLIP_MODE_DRAW);
    }

    #[test]
    fn per_layer_ids_are_distinct_by_layer_and_kind() {
        let a = flip_layer_widget_id(0, FlipLayerWidget::Visibility);
        let b = flip_layer_widget_id(1, FlipLayerWidget::Visibility);
        let c = flip_layer_widget_id(0, FlipLayerWidget::Lock);
        assert_ne!(a, b, "different layer -> different id");
        assert_ne!(a, c, "different kind -> different id");
    }
}

//! Deform-panel NodeIds (Deform Wave 1). The Painter's Deform mode owns a **mode-exclusive** panel
//! section (like Selection / Inpaint): a sub-mode picker (segmented), the Brush knobs (Size / Pressure /
//! Distortion / Momentum / Strength sliders), a Freeze toggle + Invert action, and the Reset / Apply /
//! Apply & Keep session buttons. Fixed-id, tool-global widgets — registered in the painter-layers
//! `populate` and forwarded to the tool over the frozen `PanelEvent` channel (`Click` / `SetValue`).
use super::{NodeId, hash_node_id};

// ── Sub-mode picker (segmented): Push · Twist · Pinch · Wrinkle · Fold · Reconstruct ─────────────────
pub const PAINTER_DEFORM_MODE: NodeId = hash_node_id("painter_deform.mode"); // group (a11y RadioGroup)
pub const PAINTER_DEFORM_MODE_PUSH: NodeId = hash_node_id("painter_deform.mode_push");
pub const PAINTER_DEFORM_MODE_TWIST: NodeId = hash_node_id("painter_deform.mode_twist");
pub const PAINTER_DEFORM_MODE_PINCH: NodeId = hash_node_id("painter_deform.mode_pinch");
pub const PAINTER_DEFORM_MODE_WRINKLE: NodeId = hash_node_id("painter_deform.mode_wrinkle");
pub const PAINTER_DEFORM_MODE_FOLD: NodeId = hash_node_id("painter_deform.mode_fold");
pub const PAINTER_DEFORM_MODE_RECONSTRUCT: NodeId = hash_node_id("painter_deform.mode_reconstruct");
/// Mode segments in `DeformState::mode` discriminant order (`0` Push … `5` Reconstruct).
pub const PAINTER_DEFORM_MODE_IDS: [NodeId; 6] = [
    PAINTER_DEFORM_MODE_PUSH,
    PAINTER_DEFORM_MODE_TWIST,
    PAINTER_DEFORM_MODE_PINCH,
    PAINTER_DEFORM_MODE_WRINKLE,
    PAINTER_DEFORM_MODE_FOLD,
    PAINTER_DEFORM_MODE_RECONSTRUCT,
];

// ── Brush sliders (0..1 track; the tool maps to its range) ───────────────────────────────────────────
pub const PAINTER_DEFORM_SIZE_SLIDER: NodeId = hash_node_id("painter_deform.size_slider");
pub const PAINTER_DEFORM_SIZE_CHIP: NodeId = hash_node_id("painter_deform.size_chip");
pub const PAINTER_DEFORM_PRESSURE_SLIDER: NodeId = hash_node_id("painter_deform.pressure_slider");
pub const PAINTER_DEFORM_PRESSURE_CHIP: NodeId = hash_node_id("painter_deform.pressure_chip");
pub const PAINTER_DEFORM_DISTORTION_SLIDER: NodeId =
    hash_node_id("painter_deform.distortion_slider");
pub const PAINTER_DEFORM_DISTORTION_CHIP: NodeId = hash_node_id("painter_deform.distortion_chip");
pub const PAINTER_DEFORM_MOMENTUM_SLIDER: NodeId = hash_node_id("painter_deform.momentum_slider");
pub const PAINTER_DEFORM_MOMENTUM_CHIP: NodeId = hash_node_id("painter_deform.momentum_chip");
/// **Strength** — bipolar (`0.5` = neutral; Pinch −suck / +bulge, Twist CW/CCW).
pub const PAINTER_DEFORM_STRENGTH_SLIDER: NodeId = hash_node_id("painter_deform.strength_slider");
pub const PAINTER_DEFORM_STRENGTH_CHIP: NodeId = hash_node_id("painter_deform.strength_chip");

// ── Freeze (Card C) ──────────────────────────────────────────────────────────────────────────────────
/// **Freeze selected area** — protect the selection from the warp (a toggle; disabled without a selection).
pub const PAINTER_DEFORM_FREEZE: NodeId = hash_node_id("painter_deform.freeze");
/// **Invert freeze** — protect the complement of the selection (a toggle action).
pub const PAINTER_DEFORM_FREEZE_INVERT: NodeId = hash_node_id("painter_deform.freeze_invert");

// ── Session actions (Card D) ─────────────────────────────────────────────────────────────────────────
/// **Reset** — discard the whole session's deformation (restore the pre-deform pixels).
pub const PAINTER_DEFORM_RESET: NodeId = hash_node_id("painter_deform.reset");
/// **Apply** — finalize the session (keep pixels, drop the reconstruct baseline).
pub const PAINTER_DEFORM_APPLY: NodeId = hash_node_id("painter_deform.apply");
/// **Apply & Keep** — bank the current pixels as the new baseline and keep deforming.
pub const PAINTER_DEFORM_APPLY_KEEP: NodeId = hash_node_id("painter_deform.apply_keep");
/// The three session-action buttons as a group.
pub const PAINTER_DEFORM_ACTION_IDS: [NodeId; 3] = [
    PAINTER_DEFORM_RESET,
    PAINTER_DEFORM_APPLY,
    PAINTER_DEFORM_APPLY_KEEP,
];

/// The Mode card's a11y group id (a visual surface; not hit-indexed). (Wave 1 paints the section flat with
/// separators — collapsible section headers are a follow-up; the mode card is the one framed surface.)
pub const PAINTER_DEFORM_MODE_CARD: NodeId = hash_node_id("painter_deform.mode_card");

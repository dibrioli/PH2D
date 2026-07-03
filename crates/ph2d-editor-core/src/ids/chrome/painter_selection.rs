//! Selection-panel NodeIds (ADR-0103). The Painter's Selection mode owns a **mode-exclusive** panel
//! section (like Inpaint): a mode picker + boolean-op picker (both segmented Toggle groups), a Feather
//! slider, an Automatic threshold slider, an "Edit Selection" toggle, and Invert / Clear actions. These
//! are fixed-id, tool-global widgets — registered in the painter-layers `populate` and forwarded to the
//! tool over the frozen `PanelEvent` channel (`Click` / `SetValue`).
use super::{NodeId, hash_node_id};

// ── Mode picker (segmented): Automatic · Freehand · Rectangle · Ellipse ──────────────────────────────
pub const PAINTER_SEL_MODE: NodeId = hash_node_id("painter_sel.mode"); // group (a11y RadioGroup)
pub const PAINTER_SEL_MODE_AUTO: NodeId = hash_node_id("painter_sel.mode_auto");
pub const PAINTER_SEL_MODE_FREEHAND: NodeId = hash_node_id("painter_sel.mode_freehand");
pub const PAINTER_SEL_MODE_RECT: NodeId = hash_node_id("painter_sel.mode_rect");
pub const PAINTER_SEL_MODE_ELLIPSE: NodeId = hash_node_id("painter_sel.mode_ellipse");
/// Mode segments in `selection_mode` discriminant order (`0` Automatic … `3` Ellipse).
pub const PAINTER_SEL_MODE_IDS: [NodeId; 4] = [
    PAINTER_SEL_MODE_AUTO,
    PAINTER_SEL_MODE_FREEHAND,
    PAINTER_SEL_MODE_RECT,
    PAINTER_SEL_MODE_ELLIPSE,
];

// ── Boolean-op picker (segmented): New · Add · Remove ────────────────────────────────────────────────
pub const PAINTER_SEL_OP: NodeId = hash_node_id("painter_sel.op"); // group
pub const PAINTER_SEL_OP_NEW: NodeId = hash_node_id("painter_sel.op_new");
pub const PAINTER_SEL_OP_ADD: NodeId = hash_node_id("painter_sel.op_add");
pub const PAINTER_SEL_OP_REMOVE: NodeId = hash_node_id("painter_sel.op_remove");
/// Op segments in `selection_bool_op` discriminant order (`0` New / `1` Add / `2` Remove).
pub const PAINTER_SEL_OP_IDS: [NodeId; 3] = [
    PAINTER_SEL_OP_NEW,
    PAINTER_SEL_OP_ADD,
    PAINTER_SEL_OP_REMOVE,
];

// ── Sliders (0..1 track; the tool maps to its range) ─────────────────────────────────────────────────
pub const PAINTER_SEL_FEATHER_SLIDER: NodeId = hash_node_id("painter_sel.feather_slider");
pub const PAINTER_SEL_FEATHER_CHIP: NodeId = hash_node_id("painter_sel.feather_chip");
pub const PAINTER_SEL_THRESHOLD_SLIDER: NodeId = hash_node_id("painter_sel.threshold_slider");
pub const PAINTER_SEL_THRESHOLD_CHIP: NodeId = hash_node_id("painter_sel.threshold_chip");
/// **Overlay opacity** — how strongly the deselected-area hatching reads (a view preference).
pub const PAINTER_SEL_OPACITY_SLIDER: NodeId = hash_node_id("painter_sel.opacity_slider");
pub const PAINTER_SEL_OPACITY_CHIP: NodeId = hash_node_id("painter_sel.opacity_chip");
/// **Offset** (grow/shrink) — expands/contracts the edited boundary (`0.5` = no change). Edit-mode only.
pub const PAINTER_SEL_OFFSET_SLIDER: NodeId = hash_node_id("painter_sel.offset_slider");
pub const PAINTER_SEL_OFFSET_CHIP: NodeId = hash_node_id("painter_sel.offset_chip");
/// The Operation card's a11y group id (not hit-indexed; the card is a visual surface).
pub const PAINTER_SEL_OP_CARD: NodeId = hash_node_id("painter_sel.op_card");

// ── Toggle + action buttons ──────────────────────────────────────────────────────────────────────────
/// **Edit Selection** — enter/leave the editable-Shape boundary mode (Wave EDIT). A toggle checkbox.
pub const PAINTER_SEL_EDIT: NodeId = hash_node_id("painter_sel.edit");
/// **Convert to Curve** — flatten every gizmo/shape into ONE editable Bézier curve (ADR-0103 Am.2 §3).
pub const PAINTER_SEL_CONVERT: NodeId = hash_node_id("painter_sel.convert");
/// **Invert** the selection (one undo entry).
pub const PAINTER_SEL_INVERT: NodeId = hash_node_id("painter_sel.invert");
/// **Clear** (deselect) the selection (one undo entry).
pub const PAINTER_SEL_CLEAR: NodeId = hash_node_id("painter_sel.clear");
/// **Select layer contents** — set the mask from the active layer's opaque texels (Wave 5).
pub const PAINTER_SEL_LAYER_CONTENTS: NodeId = hash_node_id("painter_sel.layer_contents");
/// **Color Fill** — flood the selected region with the brush colour (Wave 5).
pub const PAINTER_SEL_FILL: NodeId = hash_node_id("painter_sel.fill");
/// **Copy** the selected pixels to the in-memory clipboard (Wave 5).
pub const PAINTER_SEL_COPY: NodeId = hash_node_id("painter_sel.copy");
/// **Paste** the clipboard back over the active layer (Wave 5).
pub const PAINTER_SEL_PASTE: NodeId = hash_node_id("painter_sel.paste");
/// Momentary action segments (Invert / Clear) rendered as a button group.
pub const PAINTER_SEL_ACTION_IDS: [NodeId; 2] = [PAINTER_SEL_INVERT, PAINTER_SEL_CLEAR];
/// Wave-5 action buttons (Select layer contents / Color Fill / Copy / Paste).
pub const PAINTER_SEL_WAVE5_IDS: [NodeId; 4] = [
    PAINTER_SEL_LAYER_CONTENTS,
    PAINTER_SEL_FILL,
    PAINTER_SEL_COPY,
    PAINTER_SEL_PASTE,
];

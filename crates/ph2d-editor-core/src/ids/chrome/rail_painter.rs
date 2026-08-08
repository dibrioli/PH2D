//! Left-rail **Painter mode** tool NodeIds.
//!
//! When the active image-edit tool is the Painter
//! ([`ImageEditState::active_tool_id`](crate::screens::hero::state::ImageEditState)
//! `== Some("painter")`), the rail swaps its object-mode transform block
//! (Translate / Rotate / Scale / Pivot) for these paint tools. They form an
//! exclusive radio group (`ButtonState::Pressed`), mirroring the transform
//! tools' dispatch in `chrome/rail_tools.rs` — clicking one activates it and
//! clears the rest. The **Shapes** and **Mask** buttons are radio members that
//! also own a flyout of sub-tools to their right (Shapes → the 5 stroke shapes;
//! Mask → Mask + Selection), adopting the active sub-tool's icon.
//!
//! Selecting an image-edit tool BEHAVIOUR (wiring each tool to the Painter
//! engine) is a later step; this layer is the rail UI + selection state only.
use super::{NodeId, hash_node_id};

// ── Paint tools (rail order: Brush · Eyedropper · Eraser · Clone · Smear ·
//    Blur · Mask · Inpaint · Shapes) ─────────────────────────────────────────
/// Brush — the current paint mode (default-selected paint tool).
pub const PAINTER_RAIL_BRUSH: NodeId = hash_node_id("painter_rail.brush");
/// Fill (Bucket) — the flood-fill tool. Its rail chip is a COLOUR SWATCH (shows the current paint
/// colour) rather than an icon, so it doubles as the panel's colour selector. Sits below Brush, above
/// Eyedropper. Behaviour (fill + colour picker) wired in a follow-up.
pub const PAINTER_RAIL_FILL: NodeId = hash_node_id("painter_rail.fill");
/// Eyedropper — sample colour from the canvas (rich colour picker built in).
pub const PAINTER_RAIL_EYEDROPPER: NodeId = hash_node_id("painter_rail.eyedropper");
/// Eraser — erase to transparency (Erase-Alpha brush).
pub const PAINTER_RAIL_ERASER: NodeId = hash_node_id("painter_rail.eraser");
/// Clone — clone-stamp from a source point.
pub const PAINTER_RAIL_CLONE: NodeId = hash_node_id("painter_rail.clone");
/// Smear — smudge / blend / smear paint along the drag.
pub const PAINTER_RAIL_SMEAR: NodeId = hash_node_id("painter_rail.smear");
/// Blur — soften pixels under the brush.
pub const PAINTER_RAIL_BLUR: NodeId = hash_node_id("painter_rail.blur");
/// **Liquify** — the brush-driven warp (Push / Twist / Pinch / Wrinkle / Fold / Reconstruct), directly
/// under Blur because it is the last of the brush-family tools. Forwards paint mode `"liquify"`, which
/// lands the artist IN the Reshape temperament: this chip is the tool, not a lobby.
///
/// ⚠️ **It took this slot from `Sculpt`, and the trade was measured** (`measure_rail_chips`, a drag
/// through `on_canvas_pointer` in the medium the Painter opens in — Digital):
///
/// | chip | pixels moved | relief texels moved |
/// |---|---:|---:|
/// | Sculpt | **0** | **0** |
/// | Liquify | **26 964** | — |
///
/// Sculpt is not broken — it reshapes RELIEF, and Digital has none (the control in the same probe:
/// with impasto on the canvas the same gesture moves 1 676 relief texels). But the rail is the
/// UNIVERSAL bar, identical in all four media, and a slot there that does nothing in the medium the
/// app opens in is a slot most artists never see work. Sculpt keeps the home it already had — the
/// Impasto TOOL list, which only exists where its medium is armed — exactly like the **Knife**, which
/// has deliberately had no rail chip since 2026-07-19.
pub const PAINTER_RAIL_LIQUIFY: NodeId = hash_node_id("painter_rail.liquify");
/// **Transform** — the gizmo half of the warp: lift a patch and move / scale / rotate it freely.
/// Forwards paint mode `"transform"`, which lands the artist IN the Transform temperament.
///
/// ⚠️ **This is the ex-`Deform` chip, and it was an ANTECHAMBER** — the same probe measured a drag on
/// it moving **0** pixels, because entering Deform opens the temperament UNSELECTED and the canvas
/// router consumes the event without acting (`_ => true`). One extra click in the panel and the same
/// chip moved 26 964. Splitting the lobby into its two doors is what removes the dead click; the chip
/// keeps [`IconId::Transform`](crate::icons::IconId::Transform), which it was already wearing.
pub const PAINTER_RAIL_TRANSFORM: NodeId = hash_node_id("painter_rail.transform");
/// **Mask group** — the shared rail button (mirrors [`PAINTER_RAIL_SHAPES`]): pressing it reveals a
/// flyout of its two sub-tools ([`PAINTER_RAIL_MASK_SUB_IDS`]) — **Mask** (paint a layer mask) and
/// **Selection** (Procreate-style marquee) — to its right. A member of the tool radio group; the button
/// adopts the ACTIVE sub-tool's icon (Photoshop tool-group style).
pub const PAINTER_RAIL_MASK_GROUP: NodeId = hash_node_id("painter_rail.mask_group");
/// Mask — paint a layer mask. A Mask-group sub-tool (shown in the group's flyout, not directly on the
/// rail). Forwards paint mode `"mask"`.
pub const PAINTER_RAIL_MASK: NodeId = hash_node_id("painter_rail.mask");
/// Selection — Procreate-style selection (Automatic / Freehand / Rectangle / Ellipse). A Mask-group
/// sub-tool; forwards paint mode `"selection"`. The selection engine (marching-ants mask + Add/Remove/
/// Invert/Feather) is wired in a follow-up — for now selecting it just enters the mode.
pub const PAINTER_RAIL_SELECTION: NodeId = hash_node_id("painter_rail.selection");
/// Inpaint — content-aware fill / heal.
pub const PAINTER_RAIL_INPAINT: NodeId = hash_node_id("painter_rail.inpaint");
/// Shapes — multi-shape button; press-and-hold reveals the shape flyout
/// ([`PAINTER_RAIL_SHAPE_IDS`]) to the right. A member of the tool radio group:
/// activating it puts the rail in "draw a shape" mode using the current shape.
pub const PAINTER_RAIL_SHAPES: NodeId = hash_node_id("painter_rail.shapes");

// ── Shape options (the Shapes flyout: Freehand · Line · Curve · Ellipse ·
//    Polygon) — a sub-radio; the Pressed one is the current shape ───────────
/// Free Hand stroke shape.
pub const PAINTER_RAIL_SHAPE_FREEHAND: NodeId = hash_node_id("painter_rail.shape_freehand");
/// Straight Line stroke shape.
pub const PAINTER_RAIL_SHAPE_LINE: NodeId = hash_node_id("painter_rail.shape_line");
/// Bézier Curve stroke shape.
pub const PAINTER_RAIL_SHAPE_CURVE: NodeId = hash_node_id("painter_rail.shape_curve");
/// Ellipse stroke shape.
pub const PAINTER_RAIL_SHAPE_ELLIPSE: NodeId = hash_node_id("painter_rail.shape_ellipse");
/// Polygon stroke shape.
pub const PAINTER_RAIL_SHAPE_POLYGON: NodeId = hash_node_id("painter_rail.shape_polygon");

/// Routing id (not a painted widget): the rail forwards the selected paint tool's mode to the active
/// Painter via `PanelEvent::SelectOption(PAINTER_PAINT_MODE, "brush" | "eraser" | "smear" | "blur" |
/// "liquify" | "transform" | …)`,
/// drained by the shell into `PainterTool::handle_panel_event` → `set_paint_tool_mode`.
/// Dependency-legal (a frozen-channel message, no dep on the concrete painter crate).
pub const PAINTER_PAINT_MODE: NodeId = hash_node_id("painter_rail.paint_mode");

/// The Painter-mode tool radio group, in rail (paint) order. Exclusive
/// `ButtonState::Pressed` selection, like the transform tools. The Shapes
/// button is the last member (it also owns the shape flyout).
pub const PAINTER_RAIL_TOOL_IDS: [NodeId; 12] = [
    PAINTER_RAIL_BRUSH,
    PAINTER_RAIL_FILL,
    PAINTER_RAIL_EYEDROPPER,
    PAINTER_RAIL_ERASER,
    PAINTER_RAIL_CLONE,
    PAINTER_RAIL_SMEAR,
    PAINTER_RAIL_BLUR,
    PAINTER_RAIL_LIQUIFY,
    PAINTER_RAIL_TRANSFORM,
    PAINTER_RAIL_MASK_GROUP,
    PAINTER_RAIL_INPAINT,
    PAINTER_RAIL_SHAPES,
];

/// The Shapes-flyout sub-radio, in flyout order. The Pressed one is the
/// current shape (used when the Shapes tool is active).
pub const PAINTER_RAIL_SHAPE_IDS: [NodeId; 5] = [
    PAINTER_RAIL_SHAPE_FREEHAND,
    PAINTER_RAIL_SHAPE_LINE,
    PAINTER_RAIL_SHAPE_CURVE,
    PAINTER_RAIL_SHAPE_ELLIPSE,
    PAINTER_RAIL_SHAPE_POLYGON,
];

/// The Mask-group flyout sub-radio, in flyout order. The Pressed one is the active sub-tool (Mask by
/// default); its icon is adopted by the [`PAINTER_RAIL_MASK_GROUP`] rail button.
pub const PAINTER_RAIL_MASK_SUB_IDS: [NodeId; 2] = [PAINTER_RAIL_MASK, PAINTER_RAIL_SELECTION];

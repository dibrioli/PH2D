//! Painter sidebar + layers chrome NodeIds, per-row id helpers, and the
//! private FNV runtime-hash twin used by them (PAINTER_SIDEBAR_*, PAINTER_LAYERS_*,
//! PAINTER_CURVE_*, PAINTER_GRADIENT_*, PAINTER_MIXER_*, PAINTER_SELCOLOR_*).
use super::{NodeId, hash_node_id};

/// Painter sidebar panel container (`ph2d-panel-painter-sidebar` outer rect; right-docked, painter-only).
pub const PAINTER_SIDEBAR_PANEL: NodeId = hash_node_id("painter_sidebar_panel");
/// Size slider (size_px) no Painter sidebar (normalizado 0..1).
pub const PAINTER_SIDEBAR_SIZE_SLIDER: NodeId = hash_node_id("painter_sidebar.size_slider");
/// Chip numeric do size_slider (link via store.link_slider_number).
pub const PAINTER_SIDEBAR_SIZE_CHIP: NodeId = hash_node_id("painter_sidebar.size_chip");
/// Opacity slider no Painter sidebar (normalizado 0..1).
pub const PAINTER_SIDEBAR_OPACITY_SLIDER: NodeId = hash_node_id("painter_sidebar.opacity_slider");
/// Chip numeric do opacity_slider.
pub const PAINTER_SIDEBAR_OPACITY_CHIP: NodeId = hash_node_id("painter_sidebar.opacity_chip");
/// Botão Undo (sidebar). 2-finger gesture também dispara.
pub const PAINTER_SIDEBAR_UNDO_BUTTON: NodeId = hash_node_id("painter_sidebar.undo_button");
/// Botão Redo (sidebar). 3-finger gesture também dispara.
pub const PAINTER_SIDEBAR_REDO_BUTTON: NodeId = hash_node_id("painter_sidebar.redo_button");
/// Modifier square (centro da sidebar) — W2 default eyedropper-while-held.
pub const PAINTER_SIDEBAR_MODIFIER_SQUARE: NodeId = hash_node_id("painter_sidebar.modifier_square");
/// Pigment toggle (W5) — flips the active brush Linear ↔ Subtractive (pigment).
pub const PAINTER_SIDEBAR_PIGMENT_TOGGLE: NodeId = hash_node_id("painter_sidebar.pigment_toggle");
/// Accumulate toggle (W5) — flips the active brush wash ↔ build-up (orthogonal
/// to pigment).
pub const PAINTER_SIDEBAR_ACCUMULATE_TOGGLE: NodeId =
    hash_node_id("painter_sidebar.accumulate_toggle");
/// Grain toggle (W5) — flips the active brush GrainSource None ↔ Procedural
/// (Simplex paper grain).
pub const PAINTER_SIDEBAR_GRAIN_TOGGLE: NodeId = hash_node_id("painter_sidebar.grain_toggle");
/// Grain depth slider + chip (W5) — procedural grain intensity (0..100%). Shown
/// only while grain is active.
pub const PAINTER_SIDEBAR_GRAIN_DEPTH_SLIDER: NodeId =
    hash_node_id("painter_sidebar.grain_depth_slider");
pub const PAINTER_SIDEBAR_GRAIN_DEPTH_CHIP: NodeId =
    hash_node_id("painter_sidebar.grain_depth_chip");
/// Close (X) button do Painter sidebar — routes pra `CancelActiveTool`
/// (canon BgRemoval/Padding). Deactivates Painter tool quando clicado.
pub const PAINTER_SIDEBAR_CLOSE: NodeId = hash_node_id("painter_sidebar.close");
/// Floating color thumb painted in the canvas top-right while the
/// Painter tool is active (W2.T2.3). Clicking it opens the shared
/// `INSP_BLENDER_PICKER` seeded with the Painter's active color; the
/// chosen color is applied back via `PainterUiEdit::SetColorSrgb`.
pub const PAINTER_COLOR_THUMB: NodeId = hash_node_id("painter.color_thumb");

/// Painter layers panel container — the typed `ph2d-panel-painter-layers`
/// outer rect. Right-docked (same geometry slot as the Inspector, mirror
/// do `PAINTER_SIDEBAR_PANEL`) and only visible while the `painter` tool
/// is active. W3.T3.4 plan §6 / design 02_layers.md §2.3.
pub const PAINTER_LAYERS_PANEL: NodeId = hash_node_id("painter_layers_panel");
/// Brush Studio panel (W5) — the brush parameter editor. Shares the right-dock
/// geometry with the sidebar (occupies the same slot when opened).
pub const PAINTER_BRUSH_STUDIO_PANEL: NodeId = hash_node_id("painter_brush_studio_panel");
/// Close (X) button do Painter layers panel — routes pra `CancelActiveTool`
/// (canon BgRemoval/Painter sidebar). Deactivates Painter tool quando clicado.
pub const PAINTER_LAYERS_CLOSE: NodeId = hash_node_id("painter_layers.close");
/// "+ Layer" button no rodapé do body do Painter layers panel. Click →
/// `PainterTool::add_raster_layer` (cria + ativa uma raster transparente no
/// topo). W3.T3.4 UI-plumbing.
pub const PAINTER_LAYERS_ADD: NodeId = hash_node_id("painter_layers.add");
/// Header "Duplicate" icon-button — Click → `PainterTool::duplicate_layer`
/// (clones the active raster + its pixels above itself). W3.T3.5 header batch.
pub const PAINTER_LAYERS_DUPLICATE: NodeId = hash_node_id("painter_layers.duplicate");
/// Header "Delete" icon-button — Click → `PainterTool::delete_layer` (removes
/// the active layer + its subtree/mask; the base sprite is not removable).
pub const PAINTER_LAYERS_DELETE: NodeId = hash_node_id("painter_layers.delete");
/// Header "Group" icon-button — Click → `PainterTool::add_group` (empty group
/// at the top; the user nests layers into it). W3.T3.7.
pub const PAINTER_LAYERS_GROUP: NodeId = hash_node_id("painter_layers.group");
/// Modifier toolbar "Mask" — Click → `PainterTool::add_mask_to_active` (creates
/// a grayscale mask on the active raster + makes it the edit target). W3.T3.5.
pub const PAINTER_LAYERS_MASK: NodeId = hash_node_id("painter_layers.mask");
/// Modifier toolbar "Clip" toggle — flips the active layer's clipping-mask
/// modifier (§2.8). W3.T3.6.
pub const PAINTER_LAYERS_CLIP: NodeId = hash_node_id("painter_layers.clip");
/// Modifier toolbar "Lock" toggle — flips the active layer's alpha-lock
/// modifier (§2.10). W3.T3.7.
pub const PAINTER_LAYERS_ALPHA_LOCK: NodeId = hash_node_id("painter_layers.alpha_lock");
/// Modifier toolbar "Ref" toggle — flips the active layer's reference modifier
/// (§2.9, exclusive). W3.T3.7.
pub const PAINTER_LAYERS_REFERENCE: NodeId = hash_node_id("painter_layers.reference");
/// Action toolbar "+ Adj" — creates a non-destructive adjustment layer (W4
/// T4.3; HSB for the Day-4 smoke, full 24-kind menu lands with T4.15).
pub const PAINTER_LAYERS_ADD_ADJUSTMENT: NodeId = hash_node_id("painter_layers.add_adjustment");
/// Action toolbar "+ Texture" — creates a Texture layer (a procedural brush-texture fill recoloured by
/// a Color Ramp, covering the sprite). Click → `PainterTool::add_texture_layer`. Sits next to "+ Adj".
pub const PAINTER_LAYERS_ADD_TEXTURE: NodeId = hash_node_id("painter_layers.add_texture");
/// Dock-mode toggle no header do Painter **layers** panel — alterna o slot
/// docado de volta pra brush-settings (mostra "Brush"). Enio escolheu o modo
/// C = toggle (um slot, dois painéis). Estado vive no `PainterTool`
/// (`dock_shows_layers`); o bridge lê e computa a visibilidade.
pub const PAINTER_LAYERS_TOGGLE_DOCK: NodeId = hash_node_id("painter_layers.toggle_dock");
/// Dock-mode toggle no header do Painter **sidebar** (brush) panel — alterna o
/// slot docado pra layers (mostra "Layers"). Mirror simétrico de
/// [`PAINTER_LAYERS_TOGGLE_DOCK`].
pub const PAINTER_SIDEBAR_TOGGLE_DOCK: NodeId = hash_node_id("painter_sidebar.toggle_dock");
/// "Apply" CTA (shared by both painter panels) — commits the live layer
/// composite into the active sprite. Routes `PanelEvent::Click` →
/// `PainterTool::request_commit` (the bridge bakes via `run_full` next frame).
/// Without it the only way to commit was the invisible Cmd/Ctrl+Enter shortcut.
pub const PAINTER_APPLY: NodeId = hash_node_id("painter.apply");

// ── Brush settings (the layers-panel "Brush" section) ──────────────────────
// Fixed ids (the brush is tool-global, not per-layer), so they live as consts
// alongside the chrome buttons and are pre-registered in `populate`. The tool
// owns the brush state; the panel reads a `BrushSettings` snapshot to position
// these and forwards edits over the frozen `PanelEvent` channel.

/// Brush "Size" slider (stores the size slider's `0..1` track; the tool maps it
/// to a pixel radius). `SetValue` → `PainterTool::set_brush_size_norm`.
pub const PAINTER_BRUSH_SIZE_SLIDER: NodeId = hash_node_id("painter_brush.size_slider");
/// Brush "Strength" slider (`0..1`, overall opacity). `SetValue` → `set_brush_strength`.
pub const PAINTER_BRUSH_STRENGTH_SLIDER: NodeId = hash_node_id("painter_brush.strength_slider");
/// Brush "Falloff" dropdown chip — the dab distance-falloff preset (Blender's
/// "Falloff Curve Preset"). Mirror of the per-row blend chip but with a fixed id.
pub const PAINTER_BRUSH_FALLOFF: NodeId = hash_node_id("painter_brush.falloff");
/// Brush "Eraser" mode toggle — overrides the blend to Erase Alpha while on.
/// `Click` → `PainterTool::toggle_brush_eraser`.
pub const PAINTER_BRUSH_ERASER: NodeId = hash_node_id("painter_brush.eraser");
/// Brush colour Red channel slider (`0..1`). `SetValue` → channel 0.
pub const PAINTER_BRUSH_COLOR_R: NodeId = hash_node_id("painter_brush.color_r");
/// Brush colour Green channel slider (`0..1`). `SetValue` → channel 1.
pub const PAINTER_BRUSH_COLOR_G: NodeId = hash_node_id("painter_brush.color_g");
/// Brush colour Blue channel slider (`0..1`). `SetValue` → channel 2.
pub const PAINTER_BRUSH_COLOR_B: NodeId = hash_node_id("painter_brush.color_b");
/// "Randomize Color" subsection enable toggle (Blender Color → Randomize). `Click` →
/// `toggle_brush_color_jitter_enabled`.
pub const PAINTER_BRUSH_COLOR_JITTER_ENABLE: NodeId =
    hash_node_id("painter_brush.color_jitter_enable");
/// Randomize-Color **Hue** amount slider (`0..1`). `SetValue` → `set_brush_color_jitter(0, _)`.
pub const PAINTER_BRUSH_COLOR_JITTER_HUE: NodeId = hash_node_id("painter_brush.color_jitter_hue");
/// Randomize-Color **Saturation** amount slider (`0..1`). `SetValue` → `set_brush_color_jitter(1, _)`.
pub const PAINTER_BRUSH_COLOR_JITTER_SAT: NodeId = hash_node_id("painter_brush.color_jitter_sat");
/// Randomize-Color **Value** amount slider (`0..1`). `SetValue` → `set_brush_color_jitter(2, _)`.
pub const PAINTER_BRUSH_COLOR_JITTER_VAL: NodeId = hash_node_id("painter_brush.color_jitter_val");
/// Brush blend-mode dropdown chip (opens the brush-blend popover; mirror of the
/// per-row blend chip but with a fixed id + the 24 `BrushBlend` modes).
pub const PAINTER_BRUSH_BLEND: NodeId = hash_node_id("painter_brush.blend");

/// Derive the stable [`NodeId`] for brush blend-mode option `mode` (the
/// `BrushBlend` wire discriminant, `0..MAX_BRUSH_BLEND_MODES`) in the open brush
/// blend dropdown popover. Mirror of [`painter_layer_blend_option_id`] but fixed
/// (the brush is tool-global). Only the single open popover's options are
/// hit-registered, so the `format!` is bounded.
#[must_use]
pub fn painter_brush_blend_option_id(mode: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.blendopt.{mode}"))
}

/// Derive the stable [`NodeId`] for falloff preset option `preset` in the open
/// brush Falloff dropdown popover. Mirror of [`painter_brush_blend_option_id`].
#[must_use]
pub fn painter_brush_falloff_option_id(preset: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.falloffopt.{preset}"))
}

// ── Stroke section (the layers-panel "Stroke" sub-section; Blender Stroke panel) ──
// Clean-room port of the Blender 2D-paint "Stroke" controls — all forward over the frozen `PanelEvent`.

/// Brush "Stroke Method" dropdown chip. `SelectOption` → `set_brush_stroke_method`.
pub const PAINTER_BRUSH_STROKE_METHOD: NodeId = hash_node_id("painter_brush.stroke_method");
/// "Apply": bake the stroke + DISCARD the curve. `Click` → `commit_open_shape`.
pub const PAINTER_BRUSH_STROKE_APPLY: NodeId = hash_node_id("painter_brush.stroke_apply");
/// "Apply & Keep": bake but KEEP the editable curve. `Click` → `commit_open_shape_keep`.
pub const PAINTER_BRUSH_STROKE_APPLY_KEEP: NodeId = hash_node_id("painter_brush.stroke_apply_keep");
/// "Delete" (✕): drop the open shape WITHOUT baking. `Click` → `cancel_open_shape`.
pub const PAINTER_BRUSH_STROKE_DELETE: NodeId = hash_node_id("painter_brush.stroke_delete");
/// "Edit" (E): convert the open Circle/Polygon to an editable curve. `Click` → `convert_open_shape_to_curve`.
pub const PAINTER_BRUSH_STROKE_EDIT: NodeId = hash_node_id("painter_brush.stroke_edit");
/// Brush "Rate" slider — airbrush timer period (`0..1` → `[0.01,1.0]`s, Airbrush only). `SetValue`.
pub const PAINTER_BRUSH_RATE: NodeId = hash_node_id("painter_brush.rate");
/// "Edge to Edge" toggle (Blender `BRUSH_EDGE_TO_EDGE`) — Anchored only. `Click` → `set_brush_edge_to_edge`.
pub const PAINTER_BRUSH_EDGE_TO_EDGE: NodeId = hash_node_id("painter_brush.edge_to_edge");
/// Brush "Spacing" slider (`0..1` track = fraction of diameter, shown as %). `SetValue` → `set_brush_spacing`.
pub const PAINTER_BRUSH_SPACING: NodeId = hash_node_id("painter_brush.spacing");
/// "Adjust Strength for Spacing" toggle (Blender `BRUSH_SPACE_ATTEN`). `Click` → `set_brush_space_attenuation`.
pub const PAINTER_BRUSH_SPACE_ATTEN: NodeId = hash_node_id("painter_brush.space_atten");
/// "Accumulate" toggle (Blender `BRUSH_ACCUMULATE`): off caps a stroke at Strength. `Click` → toggle.
pub const PAINTER_BRUSH_ACCUMULATE: NodeId = hash_node_id("painter_brush.accumulate");
/// Brush "Jitter" slider (`0..1`; relative-to-diameter under Brush unit, px under View). `SetValue` → `set_brush_jitter_norm`.
pub const PAINTER_BRUSH_JITTER: NodeId = hash_node_id("painter_brush.jitter");
/// Brush "Jitter Unit" dropdown chip (Brush = relative / View = absolute px). `SelectOption` → `set_brush_jitter_unit`.
pub const PAINTER_BRUSH_JITTER_UNIT: NodeId = hash_node_id("painter_brush.jitter_unit");
/// Brush "Jitter Scale" slider (`0..1`; per-dab radius scatter, PH2D extra). `SetValue` → `set_brush_jitter_scale`.
pub const PAINTER_BRUSH_JITTER_SCALE: NodeId = hash_node_id("painter_brush.jitter_scale");
/// Brush "Jitter Rotate" slider (`0..1`; per-dab texture-rotation scatter; texture only). `SetValue` → `set_brush_jitter_rotate`.
pub const PAINTER_BRUSH_JITTER_ROTATE: NodeId = hash_node_id("painter_brush.jitter_rotate");
/// Brush "Jitter Spacing" slider (`0..1`; per-gap dab-spacing scatter, PH2D extra). `SetValue` → `set_brush_jitter_spacing`.
pub const PAINTER_BRUSH_JITTER_SPACING: NodeId = hash_node_id("painter_brush.jitter_spacing");

/// The per-dab randomize **slider** ids (Randomize-Color Hue/Sat/Value + Jitter Scale/Rotate/Spacing).
/// Lets the panel dispatch forward them all with one `.contains` check (mirror of
/// `PAINTER_BRUSH_TEXTURE_PARAMS`); the enable toggle is a separate `Click`.
pub const PAINTER_BRUSH_RANDOMIZE_SLIDERS: [NodeId; 6] = [
    PAINTER_BRUSH_COLOR_JITTER_HUE,
    PAINTER_BRUSH_COLOR_JITTER_SAT,
    PAINTER_BRUSH_COLOR_JITTER_VAL,
    PAINTER_BRUSH_JITTER_SCALE,
    PAINTER_BRUSH_JITTER_ROTATE,
    PAINTER_BRUSH_JITTER_SPACING,
];
/// Brush "Dash Ratio" slider (`0..1` on-fraction of the dash period). `SetValue` → `set_brush_dash_ratio`.
pub const PAINTER_BRUSH_DASH_RATIO: NodeId = hash_node_id("painter_brush.dash_ratio");
/// Brush "Dash Length" slider (`0..1` track → 1..64 dab-slots). `SetValue` → `set_brush_dash_length_norm`.
pub const PAINTER_BRUSH_DASH_LENGTH: NodeId = hash_node_id("painter_brush.dash_length");
/// Brush "Input Samples" slider (`0..1` track → 1..64 averaged samples).
/// `SetValue` → `set_brush_input_samples_norm`.
pub const PAINTER_BRUSH_INPUT_SAMPLES: NodeId = hash_node_id("painter_brush.input_samples");
/// "Stabilize Stroke" (smooth-stroke) toggle. `Click` → `set_brush_smooth_stroke` (toggles).
pub const PAINTER_BRUSH_STABILIZE: NodeId = hash_node_id("painter_brush.stabilize");
/// Stabilizer "Radius" slider (`0..1` track → 0..200 px dead-zone). `SetValue` → `set_brush_smooth_radius_norm`.
pub const PAINTER_BRUSH_STABILIZE_RADIUS: NodeId = hash_node_id("painter_brush.stabilize_radius");
/// Stabilizer "Factor" slider (`0..1` lag). `SetValue` → `set_brush_smooth_factor`.
pub const PAINTER_BRUSH_STABILIZE_FACTOR: NodeId = hash_node_id("painter_brush.stabilize_factor");

/// Derive the stable [`NodeId`] for stroke-method option `m` (the [`StrokeMethod`] wire
/// discriminant) in the open Stroke Method dropdown popover. Mirror of the blend option factory.
#[must_use]
pub fn painter_brush_stroke_method_option_id(m: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.strokeopt.{m}"))
}

/// Derive the stable [`NodeId`] for jitter-unit option `u` (`0` = Brush, `1` = View) in the open
/// Jitter Unit dropdown popover.
#[must_use]
pub fn painter_brush_jitter_unit_option_id(u: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.jituopt.{u}"))
}

// ── Brush Falloff curve editor (the always-on shape preview that becomes an
// editable curve when the `Custom` preset is selected — Blender's Falloff
// `CurveMapping` graph). The brush is tool-global, so these are fixed ids. The
// 2-D drag reuses the same `CurvePoint` dispatch as the adjustment Curves
// editor; the panel drains the result and forwards `PAINTER_BRUSH_FALLOFF_EDIT`. ─

/// Fixed `parent`/routing id for the brush Falloff curve editor. Each draggable
/// control point registers an
/// [`InteractiveState::CurvePoint`](crate::interaction::InteractiveState) with
/// THIS parent; the panel drains the drag on `ValueChanged(PAINTER_BRUSH_FALLOFF_EDIT)`
/// and forwards `SelectOption(PAINTER_BRUSH_FALLOFF_EDIT, "index:x:y")`, which the
/// tool parses into a `set_brush_falloff_point` call. Tool-global (not per-layer),
/// so the payload carries only the point index.
pub const PAINTER_BRUSH_FALLOFF_EDIT: NodeId = hash_node_id("painter_brush.falloff_edit");
/// Fixed routing id — panel → tool "add a Falloff control point". Click; no payload.
pub const PAINTER_BRUSH_FALLOFF_ADD: NodeId = hash_node_id("painter_brush.falloff_add");
/// Fixed routing id — panel → tool "remove a Falloff control point". Payload
/// `"index"`; the tool calls `remove_brush_falloff_point`.
pub const PAINTER_BRUSH_FALLOFF_REMOVE: NodeId = hash_node_id("painter_brush.falloff_remove");

/// Derive the stable [`NodeId`] for control point `index` of the brush Falloff
/// curve editor. Registered as an
/// [`InteractiveState::CurvePoint`](crate::interaction::InteractiveState) with a
/// small grab rect; the 2-D drag normalizes against the editor's canvas.
#[must_use]
pub fn painter_brush_falloff_point_id(index: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.falloff_pt.{index}"))
}

/// Per-layer-row interactive widget kind, used to derive a stable, collision-
/// safe [`NodeId`] for each control painted on a Painter layers-panel row via
/// [`painter_layer_widget_id`]. The id is hash-derived (FNV) from the layer's
/// runtime id + the kind tag, so the panel (paint/event) and the tool
/// (`handle_panel_event`) agree on the id without sharing any per-row id table
/// — the decoder simply iterates `layers × kinds`, recomputes the id, and
/// matches. Mirror in spirit of the `hier_*_companion` per-row ids, but
/// hash-based (no companion-bit dispatcher allowlist needed, since the panel
/// registers these in the `WidgetStore` itself during `paint`).
///
/// `layer_id` is passed as a raw `u64` (the `LayerId`/`RtLayerId` newtype's
/// inner value) so this stays in `ph2d-editor-core` without a dependency edge
/// to `ph2d-tool-painter` (which would be a cycle).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PainterLayerWidget {
    /// The row body — click selects (activates) the layer.
    Row,
    /// The eye toggle — click flips the layer's visibility.
    Visibility,
    /// The opacity slider (stores `0..1`).
    Opacity,
    /// The opacity numeric chip paired with [`Self::Opacity`].
    OpacityChip,
    /// The blend-mode dropdown chip (opens the blend popover).
    Blend,
    /// The move-up (↑) reorder button — moves the layer toward the front/top.
    MoveUp,
    /// The move-down (↓) reorder button — moves the layer toward the back.
    MoveDown,
    /// (Mask rows only) the Invert toggle — flips the mask's `inverted` flag
    /// (§2.7); the compositor already honors it (`1 - value`).
    MaskInvert,
    /// (Mask rows only) the Apply button — destructively bakes the mask into
    /// the parent layer's alpha and removes the mask (§2.7).
    MaskApply,
    /// (Mask rows only) the grayscale-VIEW eye — flips the canvas between the mask's grayscale (open) and the effect (closed, default).
    MaskView,
    /// (Adjustment rows only) generic slider for the adjustment's Nth slider param (W4 T4.3+); the kind
    /// decides each slot (`adjustments::adjustment_slider_params`). Indexed, so a new kind needs no new id.
    AdjParam0,
    AdjParam1,
    AdjParam2,
    AdjParam3,
    AdjParam4,
    AdjParam5,
    /// Slots 6/7 cover slider-heavier kinds (Black & White: 6 hue weights + the
    /// Tint Hue/amount sliders).
    AdjParam6,
    AdjParam7,
    /// (Adjustment rows only) generic boolean toggle for the adjustment's Nth
    /// toggle param (W4 BATCH-1, e.g. Photo Filter's "Preserve Luminosity"). The
    /// kind decides what each slot means
    /// (`ph2d_painter_brush::adjustments::adjustment_toggle_params`); a click
    /// flips it tool-side (like [`Self::MaskInvert`]), source of truth = the
    /// params. 2 slots cover the toggle-bearing kinds.
    AdjToggle0,
    AdjToggle1,
    /// (Adjustment rows only) the Nth segment of the adjustment's single
    /// segmented (1-of-N, N ≤ 3) param (W4 BATCH-1, e.g. Color Balance's tonal
    /// range Shadows/Midtones/Highlights). A click selects that option tool-side
    /// (the Curves channel-tab pattern); source of truth = the params.
    AdjSegment0,
    AdjSegment1,
    AdjSegment2,
}

impl PainterLayerWidget {
    /// Stable tag woven into the hashed id string. Changing a tag changes
    /// every derived id for that kind — keep stable.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::Visibility => "vis",
            Self::Opacity => "opacity",
            Self::OpacityChip => "opacity_chip",
            Self::Blend => "blend",
            Self::MoveUp => "move_up",
            Self::MoveDown => "move_down",
            Self::MaskInvert => "mask_invert",
            Self::MaskApply => "mask_apply",
            Self::MaskView => "mask_view",
            Self::AdjParam0 => "adj_param0",
            Self::AdjParam1 => "adj_param1",
            Self::AdjParam2 => "adj_param2",
            Self::AdjParam3 => "adj_param3",
            Self::AdjParam4 => "adj_param4",
            Self::AdjParam5 => "adj_param5",
            Self::AdjParam6 => "adj_param6",
            Self::AdjParam7 => "adj_param7",
            Self::AdjToggle0 => "adj_toggle0",
            Self::AdjToggle1 => "adj_toggle1",
            Self::AdjSegment0 => "adj_segment0",
            Self::AdjSegment1 => "adj_segment1",
            Self::AdjSegment2 => "adj_segment2",
        }
    }

    /// All kinds, in a fixed order — the decoder iterates this.
    pub const ALL: [PainterLayerWidget; 23] = [
        Self::Row,
        Self::Visibility,
        Self::Opacity,
        Self::OpacityChip,
        Self::Blend,
        Self::MoveUp,
        Self::MoveDown,
        Self::MaskInvert,
        Self::MaskApply,
        Self::MaskView,
        Self::AdjParam0,
        Self::AdjParam1,
        Self::AdjParam2,
        Self::AdjParam3,
        Self::AdjParam4,
        Self::AdjParam5,
        Self::AdjParam6,
        Self::AdjParam7,
        Self::AdjToggle0,
        Self::AdjToggle1,
        Self::AdjSegment0,
        Self::AdjSegment1,
        Self::AdjSegment2,
    ];
}

/// Runtime FNV-1a 64-bit over `s`, byte-identical to the `const fn`
/// [`ph2d_tool_registry::hash_node_id`] (which only accepts `&'static str`).
/// Needed because the per-row layer ids below are derived from a runtime
/// `format!` (the layer id is only known at runtime). Kept here, private to
/// the additive per-row helpers, so the hashing stays consistent with the rest
/// of the id space (same offset basis / prime / `NodeId(0)` bump). `pub(super)`
/// so the sibling `painter_texture` module's option-id factories share the one
/// twin (its agreement with `hash_node_id` is pinned by the test below).
pub(super) fn fnv_node_id_runtime(s: &str) -> NodeId {
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

/// Derive the stable [`NodeId`] for the `kind` control on the Painter
/// layers-panel row whose layer has runtime id `layer_id`. FNV-hashed from
/// `"painter_layer.<kind>.<layer_id>"`. Runtime `format!` is acceptable here:
/// the layers panel is not a hot path (≤8 layers, repainted per frame like the
/// sidebar formats "NN px"). See [`PainterLayerWidget`].
#[must_use]
pub fn painter_layer_widget_id(layer_id: u64, kind: PainterLayerWidget) -> NodeId {
    fnv_node_id_runtime(&format!("painter_layer.{}.{}", kind.tag(), layer_id))
}

/// Derive the stable [`NodeId`] for blend-mode option `mode` (the
/// [`BlendMode`](ph2d_painter_brush) wire discriminant, `0..MAX_BLEND_MODES`)
/// in the open blend dropdown popover of the row whose layer has runtime id
/// `layer_id`. Only the single open popover's options are ever hit-registered,
/// so the `format!` cost is bounded.
#[must_use]
pub fn painter_layer_blend_option_id(layer_id: u64, mode: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_layer.blendopt.{layer_id}.{mode}"))
}

/// Derive the stable [`NodeId`] for the `index`-th kind option in the open
/// "+ Adjustment" kind-picker popover (W4 T4.15). The index is the position in
/// `AdjustmentKind::ALL` (the wire value the panel forwards back to the tool's
/// `add_adjustment_layer`). Fixed (not per-layer) like the toolbar buttons, but
/// derived so the panel paint/event and the popover stay in sync without a table.
/// Only the open popover's options are hit-registered, so the `format!` is bounded.
#[must_use]
pub fn painter_adjustment_kind_option_id(index: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_layers.adjkind.{index}"))
}

/// Derive the stable [`NodeId`] of the bespoke Curves editor (the `parent` of its
/// draggable control points, W4 §3). The 2-D drag dispatch stashes the result
/// keyed by this parent; the panel drains it on `ValueChanged(<this id>)` and
/// forwards to `PainterTool::set_curve_point`. Per Curves adjustment layer.
#[must_use]
pub fn painter_curve_editor_id(layer_id: u64) -> NodeId {
    fnv_node_id_runtime(&format!("painter_curve.editor.{layer_id}"))
}

/// Derive the stable [`NodeId`] for control point `index` of `channel`
/// (0 = master, 1 = R, 2 = G, 3 = B) in the Curves editor of `layer_id` (W4 §3).
/// Registered as an [`InteractiveState::CurvePoint`](crate::interaction::InteractiveState)
/// with a small grab rect; the 2-D drag normalizes against the editor's canvas.
#[must_use]
pub fn painter_curve_point_id(layer_id: u64, channel: u8, index: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_curve.pt.{layer_id}.{channel}.{index}"))
}

/// Fixed routing id for a Curves control-point edit forwarded from the panel to
/// the tool (W4 §3). The panel drains the 2-D drag from the store and emits
/// `SelectOption(PAINTER_CURVE_EDIT, "layer:channel:index:x:y")`; the tool's
/// `handle_panel_event` parses it into a `set_curve_point` call. A fixed id (not
/// per-layer) avoids a reverse hash — the payload carries the layer.
pub const PAINTER_CURVE_EDIT: NodeId = hash_node_id("painter_curve_edit");

/// Derive the [`NodeId`] of the `channel` tab (0 = RGB/master, 1 = R, 2 = G,
/// 3 = B) of `layer_id`'s Curves editor (W4 §3). A click switches the canvas to
/// that channel — pure panel view state, not forwarded to the tool.
#[must_use]
pub fn painter_curve_tab_id(layer_id: u64, channel: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_curve.tab.{layer_id}.{channel}"))
}

/// Derive the [`NodeId`] of the "+ point" / "− point" button of `layer_id`'s
/// Curves editor (W4 §3). Click → the panel forwards an add/remove on the active
/// channel via [`PAINTER_CURVE_ADD`] / [`PAINTER_CURVE_REMOVE`].
#[must_use]
pub fn painter_curve_add_id(layer_id: u64) -> NodeId {
    fnv_node_id_runtime(&format!("painter_curve.add.{layer_id}"))
}

/// See [`painter_curve_add_id`].
#[must_use]
pub fn painter_curve_remove_id(layer_id: u64) -> NodeId {
    fnv_node_id_runtime(&format!("painter_curve.remove.{layer_id}"))
}

/// Fixed routing id — panel → tool "add a control point" (W4 §3). Payload
/// `"layer:channel"`; the tool calls `add_curve_point`.
pub const PAINTER_CURVE_ADD: NodeId = hash_node_id("painter_curve_add");
/// Fixed routing id — panel → tool "remove a control point" (W4 §3). Payload
/// `"layer:channel:index"`; the tool calls `remove_curve_point`.
pub const PAINTER_CURVE_REMOVE: NodeId = hash_node_id("painter_curve_remove");

/// Derive the [`NodeId`] of the `channel` output tab (0 = Red/Gray, 1 = Green,
/// 2 = Blue) of `layer_id`'s bespoke Channel-Mixer editor (W4 BATCH-1). A click
/// switches which output row the 4 weight sliders edit — pure panel view state,
/// not forwarded to the tool.
#[must_use]
pub fn painter_mixer_tab_id(layer_id: u64, channel: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_mixer.tab.{layer_id}.{channel}"))
}

/// Fixed routing id for a Channel-Mixer weight edit forwarded from the panel to
/// the tool (W4 BATCH-1). The bespoke editor's 4 sliders emit
/// `SelectOption(PAINTER_MIXER_EDIT, "layer:output:slot:value")` (the active
/// output tab carries the channel the frozen `SetValue` can not); the tool's
/// `handle_panel_event` parses it into a `set_channel_mixer_weight` call. A fixed
/// id (not per-layer) — the payload carries the layer.
pub const PAINTER_MIXER_EDIT: NodeId = hash_node_id("painter_mixer_edit");

/// Derive the [`NodeId`] of the `bucket` color-group tab (0..9: Reds … Blacks) of
/// `layer_id`'s bespoke Selective-Color editor (W4 BATCH-2). A click switches
/// which group the 4 CMYK sliders edit — pure panel view state, not forwarded.
#[must_use]
pub fn painter_selcolor_bucket_id(layer_id: u64, bucket: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_selcolor.bucket.{layer_id}.{bucket}"))
}

/// Fixed routing id for a Selective-Color CMYK edit forwarded from the panel to
/// the tool (W4 BATCH-2). The bespoke editor's 4 sliders emit
/// `SelectOption(PAINTER_SELCOLOR_EDIT, "layer:bucket:slot:value")` (the active
/// bucket carries the group the frozen `SetValue` can not); the tool parses it
/// into a `set_selective_color_value` call.
pub const PAINTER_SELCOLOR_EDIT: NodeId = hash_node_id("painter_selcolor_edit");

/// Derive the [`NodeId`] of the bespoke Gradient-Map editor (the `parent` of its
/// draggable stop handles, W4 BATCH-2). The 1-D drag dispatch (a `CurvePoint`
/// whose `x` is the stop offset) stashes the result keyed by this parent.
#[must_use]
pub fn painter_gradient_editor_id(layer_id: u64) -> NodeId {
    fnv_node_id_runtime(&format!("painter_gradient.editor.{layer_id}"))
}

/// Derive the [`NodeId`] of gradient stop `index`'s draggable handle on
/// `layer_id`'s Gradient-Map editor (W4 BATCH-2). Registered as an
/// [`InteractiveState::CurvePoint`](crate::interaction::InteractiveState) over the
/// preview bar; the drag's `x` becomes the stop offset.
#[must_use]
pub fn painter_gradient_stop_id(layer_id: u64, index: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_gradient.stop.{layer_id}.{index}"))
}

/// Derive the [`NodeId`] of the "+ stop" / "− stop" button of `layer_id`'s
/// Gradient-Map editor (W4 BATCH-2). See [`PAINTER_GRADIENT_ADD`] / [`PAINTER_GRADIENT_REMOVE`].
#[must_use]
pub fn painter_gradient_add_id(layer_id: u64) -> NodeId {
    fnv_node_id_runtime(&format!("painter_gradient.add.{layer_id}"))
}

/// See [`painter_gradient_add_id`].
#[must_use]
pub fn painter_gradient_remove_id(layer_id: u64) -> NodeId {
    fnv_node_id_runtime(&format!("painter_gradient.remove.{layer_id}"))
}

/// Fixed routing id — panel → tool Gradient-Map stop drag (W4 BATCH-2). Payload
/// `"layer:index:offset"`; the tool calls `set_gradient_stop_offset`.
pub const PAINTER_GRADIENT_EDIT: NodeId = hash_node_id("painter_gradient_edit");
/// Fixed routing id — panel → tool "add a gradient stop". Payload `"layer"`.
pub const PAINTER_GRADIENT_ADD: NodeId = hash_node_id("painter_gradient_add");
/// Fixed routing id — panel → tool "remove a gradient stop". Payload `"layer:index"`.
pub const PAINTER_GRADIENT_REMOVE: NodeId = hash_node_id("painter_gradient_remove");
/// Fixed routing id — panel → tool selected-stop RGB edit (W4 BATCH-2). Payload
/// `"layer:stop:slot:value"`; the tool calls `set_gradient_stop_color`.
pub const PAINTER_GRADIENT_COLOR: NodeId = hash_node_id("painter_gradient_color");
#[cfg(test)]
mod tests {
    use super::*;

    /// W3 audit-2 B.2: [`fnv_node_id_runtime`] is a hand-copied twin of
    /// [`ph2d_tool_registry::hash_node_id`] (same offset basis / prime /
    /// `NodeId(0)` bump) used to derive the per-row painter widget ids. Editing
    /// one twin's basis or prime would silently diverge the runtime ids from the
    /// const id space, misrouting clicks. Pin the twin agreement + the
    /// empty-string FNV-1a-64 offset basis (the value `hash_node_id("")` also
    /// returns; `!= 0`, so it never shadows `NodeId(0)` = a11y root).
    #[test]
    fn fnv_node_id_runtime_agrees_with_hash_node_id() {
        assert_eq!(
            fnv_node_id_runtime("").0,
            0xcbf2_9ce4_8422_2325,
            "empty-string hash must equal the FNV-1a-64 offset basis",
        );
        assert_eq!(fnv_node_id_runtime("").0, hash_node_id("").0);

        for s in [
            "a",
            "painter_layer.row.0",
            "painter_layer.blend.42",
            "painter_layer.blendopt.7.3",
            "painter_layers_panel",
            "the quick brown fox",
            "x",
        ] {
            assert_eq!(
                fnv_node_id_runtime(s).0,
                hash_node_id(s).0,
                "fnv_node_id_runtime / hash_node_id diverged on {s:?}",
            );
        }
    }
}

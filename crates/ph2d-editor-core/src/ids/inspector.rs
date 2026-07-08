use super::*;

// ── Inspector Transform editor (M14.A) ──────────────────────────────────────
// Live binding for `ph2d_ecs::Transform` on the selected entity. The
// section paints when `HeroScreen::inspector_transform` is `Some` and
// commits via `pending_transform_edit` → `EditorCommandQueue` at the
// shell boundary (first real consumer of the editor command pipeline).
// Z is intentionally hidden — `Transform` is 2D by design (SKILL §3,
// ADR-0025); X/Y NumberInputs only, with R/G axis-color labels.
/// Collapsible section header for the Transform editor.
pub const INSP_TRANSFORM_SECTION: NodeId = hash_node_id("insp_transform_section");
/// Position X NumberInput (meters, R-tinted label).
pub const INSP_TRANSFORM_POS_X: NodeId = hash_node_id("insp_transform_pos_x");
/// Position Y NumberInput (meters, G-tinted label).
pub const INSP_TRANSFORM_POS_Y: NodeId = hash_node_id("insp_transform_pos_y");
/// Rotation NumberInput (displayed in degrees; stored in radians).
pub const INSP_TRANSFORM_ROT: NodeId = hash_node_id("insp_transform_rot");
/// Scale X NumberInput (unitless, R-tinted label).
pub const INSP_TRANSFORM_SCALE_X: NodeId = hash_node_id("insp_transform_scale_x");
/// Scale Y NumberInput (unitless, G-tinted label).
pub const INSP_TRANSFORM_SCALE_Y: NodeId = hash_node_id("insp_transform_scale_y");
/// Skew X NumberInput (degrees in UI, R-tinted label; ADR-0025-amendment-1).
pub const INSP_TRANSFORM_SKEW_X: NodeId = hash_node_id("insp_transform_skew_x");
/// Skew Y NumberInput (degrees in UI, G-tinted label).
pub const INSP_TRANSFORM_SKEW_Y: NodeId = hash_node_id("insp_transform_skew_y");
/// Reset-to-Identity button in the Transform section header.
pub const INSP_TRANSFORM_RESET: NodeId = hash_node_id("insp_transform_reset");

// ── Inspector Visibility checkbox (M14.D) ───────────────────────────────────
// Mirrors the Hierarchy eye toggle (M14.6A). Painted as a single row
// above the Transform section. Click commits via
// `pending_visibility_edit` → `EditorCommand::SetComponent` for the
// `ph2d_ecs::Visibility` component (same pipeline as Transform).
/// Visibility checkbox in the Inspector header strip.
pub const INSP_VISIBILITY_CHECK: NodeId = hash_node_id("insp_visibility_check");

// ── Inspector Render Source — Strategy switcher (M14.C) ─────────────────────
// Three segmented buttons in the Render Source section that let the
// user switch the sprite's source-storage strategy. Pressed = current
// strategy (driven from the host snapshot every frame). Click on a
// non-pressed button raises `pending_sprite_source_change`; the shell
// does the renderer-side swap. Atlas ↔ Individual is wired in v1;
// HandPacked transitions surface a toast (asset-picker arrives in
// M14.C+).
pub const INSP_RENDER_STRATEGY_ATLAS: NodeId = hash_node_id("insp_render_strategy_atlas");
pub const INSP_RENDER_STRATEGY_INDIVIDUAL: NodeId = hash_node_id("insp_render_strategy_individual");
pub const INSP_RENDER_STRATEGY_HANDPACKED: NodeId = hash_node_id("insp_render_strategy_handpacked");

/// M14.E: editable entity-name TextInput at the top of the Inspector
/// body. Replaces the read-only name display that previously lived in
/// the Inspector header subtitle and again as a "Name" row inside the
/// Render Source section. Edits commit live via `TextChanged` →
/// `EditorCommand::SetComponent` for `ph2d_ecs::Name`.
pub const INSP_ENTITY_NAME: NodeId = hash_node_id("insp_entity_name");

// ── Inspector live section headers (Wave 4.1 restore) ─────────────────
// Right-click on these header areas opens the SectionOutline context
// menu — same affordance the Widget Gallery (showcase) has for its 10
// `INSP_SECTION_*` headers. The live Inspector originally had Transform
// + Render Source + Visibility + Name "sections" without right-click
// hit areas; restoring the outline feature here means each editable
// block now registers a header rect under one of these ids and reads
// `store.section_outline_color(...)` to paint the colored frame.
pub const INSP_LIVE_NAME_SECTION: NodeId = hash_node_id("insp_live_name_section");
pub const INSP_LIVE_VISIBILITY_SECTION: NodeId = hash_node_id("insp_live_visibility_section");
pub const INSP_LIVE_TRANSFORM_SECTION: NodeId = hash_node_id("insp_live_transform_section");
pub const INSP_LIVE_RENDER_SECTION: NodeId = hash_node_id("insp_live_render_section");
/// W2 Sprite Inspector v2 — Color & Tint live section header.
pub const INSP_LIVE_COLOR_SECTION: NodeId = hash_node_id("insp_live_color_section");
/// W2 Sprite Inspector v2 — Sprite Sheet live section header.
pub const INSP_LIVE_SHEET_SECTION: NodeId = hash_node_id("insp_live_sheet_section");
/// W3 Sprite Inspector v2 §7 — Ordering / Sorting live section header.
pub const INSP_LIVE_ORDERING_SECTION: NodeId = hash_node_id("insp_live_ordering_section");
/// W3 Sprite Inspector v2 §9 — Sampling live section header.
pub const INSP_LIVE_SAMPLING_SECTION: NodeId = hash_node_id("insp_live_sampling_section");
/// Sprite Inspector v2 §10 — Material & Blend live section header.
pub const INSP_LIVE_BLEND_SECTION: NodeId = hash_node_id("insp_live_blend_section");
/// Color-circle hit NodeIds — one per Inspector live section, parallel
/// to [`LIVE_SECTION_IDS`]. Clicking the circle opens the canonical
/// BlenderPicker pointing at this id; the picker writes the chosen
/// rgba back via `set_widget_color(<color_id>, rgba)`, and the next
/// `paint_section_header` call paints the dot in that color.
pub const INSP_LIVE_NAME_COLOR: NodeId = hash_node_id("insp_live_name_color");
pub const INSP_LIVE_VISIBILITY_COLOR: NodeId = hash_node_id("insp_live_visibility_color");
pub const INSP_LIVE_TRANSFORM_COLOR: NodeId = hash_node_id("insp_live_transform_color");
pub const INSP_LIVE_RENDER_COLOR: NodeId = hash_node_id("insp_live_render_color");
/// Color-circle hit id for the Color & Tint section header.
pub const INSP_LIVE_COLOR_COLOR: NodeId = hash_node_id("insp_live_color_color");
/// Color-circle hit id for the Sprite Sheet section header.
pub const INSP_LIVE_SHEET_COLOR: NodeId = hash_node_id("insp_live_sheet_color");
/// Color-circle hit id for the Ordering / Sorting section header.
pub const INSP_LIVE_ORDERING_COLOR: NodeId = hash_node_id("insp_live_ordering_color");

// W3 Sprite Inspector v2 §7 — Ordering / Sorting control ids.
// (INSP_ORDER_Z_OVERRIDE retired 2026-05-31 — the Z Index field IS the
// override: a non-zero value attaches `ZIndexOverride`, 0 detaches it.)
/// Z Index value (NumberInput); 0 = no override (default / DFS order).
pub const INSP_ORDER_Z_INDEX: NodeId = hash_node_id("insp_order_z_index");
/// "Z as Relative" toggle (shown when override on).
pub const INSP_ORDER_Z_RELATIVE: NodeId = hash_node_id("insp_order_z_relative");
/// "Show Behind Parent" marker toggle.
pub const INSP_ORDER_SHOW_BEHIND: NodeId = hash_node_id("insp_order_show_behind");
/// "Order in Layer" value (NumberInput).
pub const INSP_ORDER_ORDER_IN_LAYER: NodeId = hash_node_id("insp_order_order_in_layer");
/// "Y-Sort" enabled toggle.
pub const INSP_ORDER_YSORT_ENABLED: NodeId = hash_node_id("insp_order_ysort_enabled");
/// "Sorting Group" toggle.
pub const INSP_ORDER_SORTING_GROUP: NodeId = hash_node_id("insp_order_sorting_group");
/// "Sort At Root" toggle (shown when Sorting Group on).
pub const INSP_ORDER_SORT_AT_ROOT: NodeId = hash_node_id("insp_order_sort_at_root");
/// "Top Level" marker toggle.
pub const INSP_ORDER_TOP_LEVEL: NodeId = hash_node_id("insp_order_top_level");
/// Sorting Layer dropdown chip.
pub const INSP_ORDER_SORTING_LAYER: NodeId = hash_node_id("insp_order_sorting_layer");
/// Sorting Layer dropdown option ids (one per canonical project layer,
/// index = LayerId). Spec §5.2 default set: Background / Midground /
/// Default / Foreground / UI.
pub const INSP_ORDER_LAYER_OPT: [NodeId; 5] = [
    hash_node_id("insp_order_layer_opt_0"),
    hash_node_id("insp_order_layer_opt_1"),
    hash_node_id("insp_order_layer_opt_2"),
    hash_node_id("insp_order_layer_opt_3"),
    hash_node_id("insp_order_layer_opt_4"),
];
/// Y-Sort Sort Point segmented control items (Center / Pivot / Custom).
pub const INSP_ORDER_SP_CENTER: NodeId = hash_node_id("insp_order_sp_center");
pub const INSP_ORDER_SP_PIVOT: NodeId = hash_node_id("insp_order_sp_pivot");
pub const INSP_ORDER_SP_CUSTOM: NodeId = hash_node_id("insp_order_sp_custom");
/// Y-Sort Custom Axis NumberInputs (shown only when Sort Point = Custom).
pub const INSP_ORDER_AXIS_X: NodeId = hash_node_id("insp_order_axis_x");
pub const INSP_ORDER_AXIS_Y: NodeId = hash_node_id("insp_order_axis_y");

/// W3 §9 Sampling — section accent color dot.
pub const INSP_LIVE_SAMPLING_COLOR: NodeId = hash_node_id("insp_live_sampling_color");
/// Texture Filter segmented items (Inherit / Nearest / Linear → tags 0/1/2).
pub const INSP_SAMPLE_FILTER: [NodeId; 3] = [
    hash_node_id("insp_sample_filter_inherit"),
    hash_node_id("insp_sample_filter_nearest"),
    hash_node_id("insp_sample_filter_linear"),
];
/// Texture Repeat segmented items (Inherit / Disabled / Enabled / Mirror
/// → tags 0/1/2/3).
pub const INSP_SAMPLE_REPEAT: [NodeId; 4] = [
    hash_node_id("insp_sample_repeat_inherit"),
    hash_node_id("insp_sample_repeat_disabled"),
    hash_node_id("insp_sample_repeat_enabled"),
    hash_node_id("insp_sample_repeat_mirror"),
];
/// UV tiling/scroll NumberInputs (W3 UvTransform): scale X/Y, offset X/Y.
pub const INSP_SAMPLE_UV_SCALE_X: NodeId = hash_node_id("insp_sample_uv_scale_x");
pub const INSP_SAMPLE_UV_SCALE_Y: NodeId = hash_node_id("insp_sample_uv_scale_y");
pub const INSP_SAMPLE_UV_OFFSET_X: NodeId = hash_node_id("insp_sample_uv_offset_x");
pub const INSP_SAMPLE_UV_OFFSET_Y: NodeId = hash_node_id("insp_sample_uv_offset_y");

/// §10 Material & Blend — section accent color dot.
pub const INSP_LIVE_BLEND_COLOR: NodeId = hash_node_id("insp_live_blend_color");
/// §10 Blend Mode segmented items, indexed by `BlendMode::tag()` (0..5):
/// Mix / Add / Subtract / Multiply / Screen / Premult. Tag 0 (Mix) =
/// detach the optional `BlendMode` component (default).
pub const INSP_SAMPLE_BLEND: [NodeId; 6] = [
    hash_node_id("insp_sample_blend_mix"),
    hash_node_id("insp_sample_blend_add"),
    hash_node_id("insp_sample_blend_subtract"),
    hash_node_id("insp_sample_blend_multiply"),
    hash_node_id("insp_sample_blend_screen"),
    hash_node_id("insp_sample_blend_premult"),
];

// ─── W3 §8 Visibility-section controls (ClipChildren / Mask / Layer) ───
/// Clip Children segmented: Disabled / ClipOnly / ClipAndDraw (tags 0/1/2).
pub const INSP_VIS_CLIP: [NodeId; 3] = [
    hash_node_id("insp_vis_clip_disabled"),
    hash_node_id("insp_vis_clip_clip_only"),
    hash_node_id("insp_vis_clip_clip_and_draw"),
];
/// Mask Interaction segmented: None / VisibleInside / VisibleOutside (0/1/2).
pub const INSP_VIS_MASK: [NodeId; 3] = [
    hash_node_id("insp_vis_mask_none"),
    hash_node_id("insp_vis_mask_inside"),
    hash_node_id("insp_vis_mask_outside"),
];
/// Mask alpha-cutoff NumberInput (shown when Mask != None).
pub const INSP_VIS_ALPHA_CUTOFF: NodeId = hash_node_id("insp_vis_alpha_cutoff");
/// "Mask Source (Mask2D)" toggle — makes this sprite a mask source.
pub const INSP_VIS_MASK_SOURCE: NodeId = hash_node_id("insp_vis_mask_source");
/// Collapsible sub-header for the Visibility Layer 4×8 bitmask grid — a
/// `mark_collapsible_section` id so clicking the row folds the grid.
pub const INSP_VIS_LAYER_HEADER: NodeId = hash_node_id("insp_vis_layer_header");
/// On-Screen Enabler toggle + its Rect2 editor (x/y/w/h, shown when on).
pub const INSP_VIS_ON_SCREEN: NodeId = hash_node_id("insp_vis_on_screen");
pub const INSP_VIS_RECT_X: NodeId = hash_node_id("insp_vis_rect_x");
pub const INSP_VIS_RECT_Y: NodeId = hash_node_id("insp_vis_rect_y");
pub const INSP_VIS_RECT_W: NodeId = hash_node_id("insp_vis_rect_w");
pub const INSP_VIS_RECT_H: NodeId = hash_node_id("insp_vis_rect_h");
/// VisibilityLayer bitmask — 32 checkboxes (4 cols × 8 rows), bit `n`.
pub const INSP_VIS_LAYER_BIT: [NodeId; 32] = [
    hash_node_id("insp_vis_layer_bit_0"),
    hash_node_id("insp_vis_layer_bit_1"),
    hash_node_id("insp_vis_layer_bit_2"),
    hash_node_id("insp_vis_layer_bit_3"),
    hash_node_id("insp_vis_layer_bit_4"),
    hash_node_id("insp_vis_layer_bit_5"),
    hash_node_id("insp_vis_layer_bit_6"),
    hash_node_id("insp_vis_layer_bit_7"),
    hash_node_id("insp_vis_layer_bit_8"),
    hash_node_id("insp_vis_layer_bit_9"),
    hash_node_id("insp_vis_layer_bit_10"),
    hash_node_id("insp_vis_layer_bit_11"),
    hash_node_id("insp_vis_layer_bit_12"),
    hash_node_id("insp_vis_layer_bit_13"),
    hash_node_id("insp_vis_layer_bit_14"),
    hash_node_id("insp_vis_layer_bit_15"),
    hash_node_id("insp_vis_layer_bit_16"),
    hash_node_id("insp_vis_layer_bit_17"),
    hash_node_id("insp_vis_layer_bit_18"),
    hash_node_id("insp_vis_layer_bit_19"),
    hash_node_id("insp_vis_layer_bit_20"),
    hash_node_id("insp_vis_layer_bit_21"),
    hash_node_id("insp_vis_layer_bit_22"),
    hash_node_id("insp_vis_layer_bit_23"),
    hash_node_id("insp_vis_layer_bit_24"),
    hash_node_id("insp_vis_layer_bit_25"),
    hash_node_id("insp_vis_layer_bit_26"),
    hash_node_id("insp_vis_layer_bit_27"),
    hash_node_id("insp_vis_layer_bit_28"),
    hash_node_id("insp_vis_layer_bit_29"),
    hash_node_id("insp_vis_layer_bit_30"),
    hash_node_id("insp_vis_layer_bit_31"),
];

/// W2 Sprite Inspector v2 — Color & Tint section controls.
/// Final opacity Slider (`0..1` storage; renders today via
/// `RenderInstance.opacity`). Paired with [`INSP_SPRITE_OPACITY_CHIP`].
pub const INSP_SPRITE_OPACITY: NodeId = hash_node_id("insp_sprite_opacity");
/// Numeric chip linked to the Opacity slider, displaying `0..100`
/// (percent) via an integer-mapped projection.
pub const INSP_SPRITE_OPACITY_CHIP: NodeId = hash_node_id("insp_sprite_opacity_chip");
/// Tint Fill (silhouette) checkbox; renders today via `flip_uv` bit 2.
pub const INSP_SPRITE_TINT_FILL: NodeId = hash_node_id("insp_sprite_tint_fill");
/// Inherited modulate color swatch (`Sprite::tint`, cascades to
/// children). Clicking opens the shared `INSP_BLENDER_PICKER`
/// targeting this id; the chosen color round-trips through
/// `widget_color(id)` and is dispatched as `SpriteFieldEdit::Tint`.
/// Renders today via `RenderInstance.tint`.
pub const INSP_SPRITE_TINT_SWATCH: NodeId = hash_node_id("insp_sprite_tint_swatch");
/// Local modulate color swatch (`Sprite::self_tint`, does NOT cascade).
/// Same picker mechanism as [`INSP_SPRITE_TINT_SWATCH`]; dispatched as
/// `SpriteFieldEdit::SelfTint`.
pub const INSP_SPRITE_SELF_TINT_SWATCH: NodeId = hash_node_id("insp_sprite_self_tint_swatch");
/// Per-corner tint swatches `[TL, TR, BL, BR]` — a 4-stop bilinear
/// gradient (Phaser-style). Each opens the shared picker; the chosen
/// color replaces one corner of the `[[f32;4];4]` array and dispatches
/// `SpriteFieldEdit::PerCornerTint`. Renders via the shader's
/// `@location(9..12)` per-corner attributes.
pub const INSP_SPRITE_CORNER_TL: NodeId = hash_node_id("insp_sprite_corner_tl");
/// Per-corner tint swatch — top-right. See [`INSP_SPRITE_CORNER_TL`].
pub const INSP_SPRITE_CORNER_TR: NodeId = hash_node_id("insp_sprite_corner_tr");
/// Per-corner tint swatch — bottom-left. See [`INSP_SPRITE_CORNER_TL`].
pub const INSP_SPRITE_CORNER_BL: NodeId = hash_node_id("insp_sprite_corner_bl");
/// Per-corner tint swatch — bottom-right. See [`INSP_SPRITE_CORNER_TL`].
pub const INSP_SPRITE_CORNER_BR: NodeId = hash_node_id("insp_sprite_corner_br");
/// "Equalize corners" button — copies the top-left corner color to the
/// other three (spec §3.6 hotkey); dispatches `SpriteFieldEdit::PerCornerTint`.
pub const INSP_SPRITE_CORNER_EQUALIZE: NodeId = hash_node_id("insp_sprite_corner_equalize");
// (The Color & Tint sub-tab ids `INSP_COLOR_TAB_*` were retired
// 2026-05-31 — the section now stacks every control visible at once.)

/// W2 Sprite Inspector v2 — Region sampling controls (Render Source
/// section, spec §3.3). Toggle + 4 px NumberInputs (x/y/w/h) + filter
/// clip. Renders via the extract `region_subrect` sub-UV (W2.T2.4).
pub const INSP_REGION_ENABLED: NodeId = hash_node_id("insp_region_enabled");
/// Region rect X NumberInput (source pixels). See [`INSP_REGION_ENABLED`].
pub const INSP_REGION_X: NodeId = hash_node_id("insp_region_x");
/// Region rect Y NumberInput (source pixels).
pub const INSP_REGION_Y: NodeId = hash_node_id("insp_region_y");
/// Region rect W NumberInput (source pixels, `>= 0`).
pub const INSP_REGION_W: NodeId = hash_node_id("insp_region_w");
/// Region rect H NumberInput (source pixels, `>= 0`).
pub const INSP_REGION_H: NodeId = hash_node_id("insp_region_h");
/// Region filter-clip toggle (anti atlas-bleed). See [`INSP_REGION_ENABLED`].
pub const INSP_REGION_FILTER_CLIP: NodeId = hash_node_id("insp_region_filter_clip");

/// W2 Sprite Inspector v2 — origin controls (Sprite Sheet section, spec
/// §3.4). Centered toggle + Offset X/Y px NumberInputs. Renders via
/// `Sprite::resolve_anchor` (W2.T2.6).
pub const INSP_SPRITE_CENTERED: NodeId = hash_node_id("insp_sprite_centered");
/// Intrinsic offset X NumberInput (px). See [`INSP_SPRITE_CENTERED`].
pub const INSP_SPRITE_OFFSET_X: NodeId = hash_node_id("insp_sprite_offset_x");
/// Intrinsic offset Y NumberInput (px). See [`INSP_SPRITE_CENTERED`].
pub const INSP_SPRITE_OFFSET_Y: NodeId = hash_node_id("insp_sprite_offset_y");

/// W2 Sprite Inspector v2 — Sprite Sheet grid controls (render today via
/// the extract atlas-rect sub-division). HFrames / VFrames / Frame.
pub const INSP_SPRITE_HFRAMES: NodeId = hash_node_id("insp_sprite_hframes");
/// Sprite-sheet rows NumberInput.
pub const INSP_SPRITE_VFRAMES: NodeId = hash_node_id("insp_sprite_vframes");
/// Active sheet frame index NumberInput.
pub const INSP_SPRITE_FRAME: NodeId = hash_node_id("insp_sprite_frame");

/// Title-bar color dot for the Grid Snap panel. Kept (Grid Snap is a
/// settings panel, not an image tool). The original broadcast added
/// PAD/BGR/CEQ/UPS/EQS dots too, but those were removed 2026-05-24
/// per user feedback: image-tool panels are transient operation
/// surfaces, not annotation surfaces.
pub const GS_TITLE_COLOR: NodeId = hash_node_id("gs_title_color");
/// Widget Gallery floating panel — root id. The gallery is a Procreate-
/// style floating reference panel that hosts the canonical widget
/// showcase. Toggle visibility via [`TOPBAR_WIDGET_GALLERY`].
pub const GAL_PANEL: NodeId = hash_node_id("gal_panel");
/// Audio Mixer floating panel — root id. Must match
/// `ph2d_panel_audio_mixer::AMIX_PANEL` (same `hash_node_id` string) so the
/// z-order paint walk in `screens::hero::paint` resolves + paints it. Toggle
/// visibility via [`crate::ids::TOPBAR_AUDIO_MIXER`].
pub const AUDIO_MIXER_PANEL: NodeId = hash_node_id("audio_mixer_panel");
/// Audio Editor docked panel — root id. Must match
/// `ph2d_panel_audio_editor::AEDIT_PANEL` (same `hash_node_id` string) so the
/// z-order paint walk in `screens::hero::paint` resolves + paints it. Toggle
/// visibility via [`crate::ids::TOPBAR_AUDIO_EDITOR`]. The docked panel holds the
/// transport + load/export controls; the big waveform + timeline live in the
/// separate floating [`AUDIO_OVERLAY_PANEL`] on the canvas.
pub const AUDIO_EDITOR_PANEL: NodeId = hash_node_id("audio_editor_panel");
/// Audio Editor **floating overlay** — root id for the resizable waveform +
/// timeline window that floats over the canvas in the gap between the Hierarchy
/// and Inspector docks. Drag/resize reuse the panel-agnostic
/// `blender_picker_offset` + `panel_resize_delta` store, keyed by this id
/// (mirror of the Inspector dock).
pub const AUDIO_OVERLAY_PANEL: NodeId = hash_node_id("audio_overlay_panel");
/// Drag handle pill at the top of the Widget Gallery panel.
pub const GAL_DRAG_HANDLE: NodeId = hash_node_id("gal_drag_handle");
/// Resize gripper at the Widget Gallery's bottom-right corner.
pub const GAL_RESIZE_HANDLE: NodeId = hash_node_id("gal_resize_handle");
/// Resize gripper at the Widget Gallery's bottom-LEFT corner. Mirror
/// of [`GAL_RESIZE_HANDLE`].
pub const GAL_RESIZE_HANDLE_BL: NodeId = hash_node_id("gal_resize_handle_bl");
/// Close (X) button at the top-right of the Widget Gallery — alternate
/// way to dismiss the panel beyond clicking the TopBar palette pill.
pub const GAL_CLOSE: NodeId = hash_node_id("gal_close");
/// Drag handle at the top of the Hierarchy.
pub const HIER_DRAG_HANDLE: NodeId = hash_node_id("hier_drag_handle");
/// Resize gripper at the Hierarchy's bottom-right corner.
pub const HIER_RESIZE_HANDLE: NodeId = hash_node_id("hier_resize_handle");
/// Resize gripper at the Hierarchy's bottom-LEFT corner. Mirror of
/// [`HIER_RESIZE_HANDLE`].
pub const HIER_RESIZE_HANDLE_BL: NodeId = hash_node_id("hier_resize_handle_bl");

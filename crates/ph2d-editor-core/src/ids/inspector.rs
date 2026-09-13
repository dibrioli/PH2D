use super::*;

// ── Inspector Render Source — Strategy switcher (M14.C) ─────────────────────
// Three segmented buttons in the Render Source section that let the
// user switch the sprite's source-storage strategy. Pressed = current
// strategy (driven from the host snapshot every frame). Click on a
// non-pressed button raises `pending_sprite_source_change`; the shell
// does the renderer-side swap. Atlas ↔ Individual is wired in v1;
// HandPacked transitions surface a toast (asset-picker arrives in
// M14.C+).
pub const INSP_RENDER_STRATEGY_ATLAS: NodeId = hash_node_id("insp_render_strategy_atlas");

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

/// §11 Physics Body — collapsible section header.
pub const INSP_LIVE_PHYSICS_SECTION: NodeId = hash_node_id("insp_live_physics_section");
/// §11 Physics Body — section accent color dot.
pub const INSP_LIVE_PHYSICS_COLOR: NodeId = hash_node_id("insp_live_physics_color");

/// Collapsible sub-header for the Visibility Layer 4×8 bitmask grid — a
/// `mark_collapsible_section` id so clicking the row folds the grid.
pub const INSP_VIS_LAYER_HEADER: NodeId = hash_node_id("insp_vis_layer_header");

// (The Color & Tint sub-tab ids `INSP_COLOR_TAB_*` were retired
// 2026-05-31 — the section now stacks every control visible at once.)

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

// ---------------------------------------------------------------------------
// §12 Physics Joint (W3). A joint is an ENTITY, so this section describes the
// selected joint object — kind, the two bodies it names, and the parameters
// the chosen kind actually uses.
// ---------------------------------------------------------------------------

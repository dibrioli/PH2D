//! Stable [`NodeId`] constants for the hero screen's interactive
//! widgets + helper mappings between fixture entity names and ids.
//!
//! Pre-populated in [`crate::interaction::WidgetStore`] at
//! construction time so the dispatcher always finds an entry on
//! hit-test.
//!
//! ## NodeId derivation (Wave 2 PR 11.3 — convention-by-discovery)
//!
//! Chrome ids are derived from stable string slugs via
//! [`ph2d_tool_registry::hash_node_id`] (FNV-1a 64-bit, `const fn`).
//! Adding a new chrome widget no longer requires hunting for a free
//! integer in some hand-allocated range — pick a unique slug
//! (`"topbar.save"`-style by convention) and the hash is deterministic
//! cross-platform. Collisions are caught by the
//! `tests/architecture/node_id_collisions.rs` regression test, which
//! enumerates every chrome const and asserts pairwise uniqueness.
//!
//! Pre-PR-11.3 the file allocated ids by hand in numeric buckets
//! (100..199 TopBar, 200..299 Rail, 300..399 Inspector, 400..499
//! Hierarchy, 600..699 BlenderColorPicker, 800..899 Notes, 900..999
//! Context menus, 950..999 Widget Gallery). Six collisions had already
//! slipped in (e.g. 380, 381, 382 + 853, 854, 855) — exactly the class
//! of bug the M14.4d audit comment below warned about, and exactly the
//! class of bug hash-based ids eliminate.
//!
//! ## Hierarchy fixture rows kept numeric
//!
//! [`HIER_PLAYER`]..[`HIER_MAIN_CAMERA`] (12 fixture entity row ids in
//! the 400..411 range) are deliberately NOT hashed. They participate
//! in the [`EYE_TOGGLE_BIT`] / [`EXPAND_TOGGLE_BIT`] companion-id math
//! at the bottom of this file (`row.0 | bit`), which assumes the high
//! bits 61+62 are free. FNV-1a output is uniformly distributed over 64
//! bits, so hashing rows would silently break the companion-detection
//! invariant on ~25% of slugs. Real (non-fixture) rows allocated by
//! the host bridge sit at `BASE_NODE_ID = 100_000` upward and also
//! have those bits clear. See `hero_bridge.rs` for the runtime path.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

pub const TOPBAR_THEME: NodeId = hash_node_id("topbar_theme");
pub const TOPBAR_SAVE: NodeId = hash_node_id("topbar_save");
pub const TOPBAR_PROJECT: NodeId = hash_node_id("topbar_project");
pub const TOPBAR_PLAY_TOGGLE: NodeId = hash_node_id("topbar_play_toggle");
pub const TOPBAR_PLAY_BUTTON: NodeId = hash_node_id("topbar_play_button");
pub const TOPBAR_RIGHT_LAYERS: NodeId = hash_node_id("topbar_right_layers");
pub const TOPBAR_RIGHT_ASSETS: NodeId = hash_node_id("topbar_right_assets");
pub const TOPBAR_RIGHT_SCRIPT: NodeId = hash_node_id("topbar_right_script");
pub const TOPBAR_PAUSE: NodeId = hash_node_id("topbar_pause");
pub const TOPBAR_RESET: NodeId = hash_node_id("topbar_reset");
pub const TOPBAR_SAVE_AS: NodeId = hash_node_id("topbar_save_as");
pub const TOPBAR_OPEN: NodeId = hash_node_id("topbar_open");
/// Settings cluster (gear icon) — opens the SettingsMenu context menu
/// with project-level toggles (pixels-per-meter presets, future
/// global config). Added M14.4d retrofit.
pub const TOPBAR_SETTINGS: NodeId = hash_node_id("topbar_settings");
/// Image Tools cluster — toggle entry-point for the image-editing
/// action row (Trim Transparency in V1; BG Removal / Equalize / etc.
/// to follow). Click flips the TopBar between Edit mode and
/// ImageTools mode; the state lives on
/// [`crate::screens::HeroScreen::image_tools_mode`].
pub const TOPBAR_IMAGE_TOOLS: NodeId = hash_node_id("topbar_image_tools");
/// Widget Gallery cluster — toggles the floating reference panel
/// that showcases every canonical widget (Inputs / Slider /
/// Switches / Lists / Vector / Status / Color / Actions / Identity /
/// Card). Peripheral agents open this from the live app as the
/// single in-app source of truth for UI decoration. State lives on
/// [`crate::screens::HeroScreen::widget_gallery_visible`].
pub const TOPBAR_WIDGET_GALLERY: NodeId = hash_node_id("topbar_widget_gallery");
/// Grid Settings cluster — opens the floating Grid Settings panel
/// (grid-snap subsystem). Toggles `HeroScreen::grid_snap_state.panel_visible`.
/// Panel internal widgets are hash-derived in `grid_snap::ids`.
pub const TOPBAR_GRID_SETTINGS: NodeId = hash_node_id("topbar_grid_settings");

/// Image Tools action — Trim Transparency pill. Lives in the action
/// row that replaces the right-side TopBar clusters when
/// `image_tools_mode` is on. Click is no-op for now — wiring to the
/// `ph2d_editor::trim_transparency()` algorithm on a selected sprite
/// requires the live asset model (out of scope for this PR).
// Wave 2 PR 11.4: the three Image Tools action pills are now derived
// from the `image_tools` cluster in the runtime registry. To make
// hand-written click dispatch (`id == ids::IMAGE_ACTION_*`) work
// against registry-derived pills, each chrome const hashes the SAME
// slug as the matching tool manifest's `id` field. The
// `chrome_manifest_coverage` integration test pins this contract.
pub const IMAGE_ACTION_TRIM: NodeId = hash_node_id("trim_transparency");

/// Image Tools action — Make Square pill. Sibling of `IMAGE_ACTION_TRIM`,
/// pads the selected sprite with transparent pixels on the shorter axis
/// so width == height. Click raises `pending_make_square` on `HeroScreen`;
/// host drains, runs the algorithm, replaces sprite pixels + reprojects
/// pivot. Algorithm in `tools/make_square/algorithm.rs`.
pub const IMAGE_ACTION_MAKE_SQUARE: NodeId = hash_node_id("make_square");

/// Image Tools action — Background Removal pill. Unlike `IMAGE_ACTION_TRIM`
/// and `IMAGE_ACTION_MAKE_SQUARE` (one-shot algorithms), this one
/// ACTIVATES the stateful `BgRemovalTool` so its floating panel opens
/// at the BottomCenter with a live 160×160 preview. Click raises
/// `pending_activate_bgremoval` on `HeroScreen`; host drains via
/// `tools.set_active(ToolId::new("bgremoval"))` and force-refreshes
/// the snapshot push.
pub const IMAGE_ACTION_BGREMOVAL: NodeId = hash_node_id("bgremoval");

pub const HIERARCHY_ADD: NodeId = hash_node_id("hierarchy_add");

pub const TOOL_TRANSLATE: NodeId = hash_node_id("tool_translate");
pub const TOOL_ROTATE: NodeId = hash_node_id("tool_rotate");
pub const TOOL_SCALE: NodeId = hash_node_id("tool_scale");
pub const TOOL_PIVOT: NodeId = hash_node_id("tool_pivot");
pub const TOOL_SPACE: NodeId = hash_node_id("tool_space");
pub const TOOL_PROJECTION: NodeId = hash_node_id("tool_projection");
pub const TOOL_HOME: NodeId = hash_node_id("tool_home");
pub const TOOL_UNDO: NodeId = hash_node_id("tool_undo");
pub const TOOL_REDO: NodeId = hash_node_id("tool_redo");
/// Show/Hide toggles for the side panels — top of the left rail.
/// `Pressed` state == panel currently visible.
pub const RAIL_SHOW_INSPECTOR: NodeId = hash_node_id("rail_show_inspector");
pub const RAIL_SHOW_HIERARCHY: NodeId = hash_node_id("rail_show_hierarchy");

/// Inspector panel container — used as the wheel-scroll key.
pub const INSP_PANEL: NodeId = hash_node_id("insp_panel");
/// Drag handle at the top of the Inspector — click+drag moves the
/// panel. Registered as `BlenderHit { parent: INSP_PANEL, kind:
/// DragHandle }` so the existing picker-drag dispatch infra
/// (panel-agnostic on parent NodeId) drives it.
pub const INSP_DRAG_HANDLE: NodeId = hash_node_id("insp_drag_handle");
/// Resize gripper at the Inspector's bottom-right corner. Registered
/// as `BlenderHit { parent: INSP_PANEL, kind: ResizeHandle }`.
pub const INSP_RESIZE_HANDLE: NodeId = hash_node_id("insp_resize_handle");

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
/// Widget Gallery floating panel — root id. The gallery is a Procreate-
/// style floating reference panel that hosts the canonical widget
/// showcase. Toggle visibility via [`TOPBAR_WIDGET_GALLERY`].
pub const GAL_PANEL: NodeId = hash_node_id("gal_panel");
/// Drag handle pill at the top of the Widget Gallery panel.
pub const GAL_DRAG_HANDLE: NodeId = hash_node_id("gal_drag_handle");
/// Resize gripper at the Widget Gallery's bottom-right corner.
pub const GAL_RESIZE_HANDLE: NodeId = hash_node_id("gal_resize_handle");
/// Close (X) button at the top-right of the Widget Gallery — alternate
/// way to dismiss the panel beyond clicking the TopBar palette pill.
pub const GAL_CLOSE: NodeId = hash_node_id("gal_close");
/// Drag handle at the top of the Hierarchy.
pub const HIER_DRAG_HANDLE: NodeId = hash_node_id("hier_drag_handle");
/// Resize gripper at the Hierarchy's bottom-right corner.
pub const HIER_RESIZE_HANDLE: NodeId = hash_node_id("hier_resize_handle");

// ── Inspector widget samples ───────────────────────────────────────────────
// One of each canonical widget, parented to the Inspector panel.
// These are *demonstration* widgets; their state lives on the store
// but is not wired to any simulation. The placeholder fixture-driven
// rows that used to live in 300..370 were removed pre-samples.
pub const INSP_SAMPLE_TEXT: NodeId = hash_node_id("insp_sample_text");
pub const INSP_SAMPLE_TEXTAREA: NodeId = hash_node_id("insp_sample_textarea");
pub const INSP_SAMPLE_COMBO: NodeId = hash_node_id("insp_sample_combo");
pub const INSP_SAMPLE_COMBO_OPT_A: NodeId = hash_node_id("insp_sample_combo_opt_a");
pub const INSP_SAMPLE_COMBO_OPT_B: NodeId = hash_node_id("insp_sample_combo_opt_b");
pub const INSP_SAMPLE_COMBO_OPT_C: NodeId = hash_node_id("insp_sample_combo_opt_c");
pub const INSP_SAMPLE_NUMBER: NodeId = hash_node_id("insp_sample_number");
pub const INSP_SAMPLE_SLIDER: NodeId = hash_node_id("insp_sample_slider");
pub const INSP_SAMPLE_SLIDER_CHIP: NodeId = hash_node_id("insp_sample_slider_chip");
pub const INSP_SAMPLE_CHECKBOX: NodeId = hash_node_id("insp_sample_checkbox");
pub const INSP_SAMPLE_TOGGLE: NodeId = hash_node_id("insp_sample_toggle");
pub const INSP_SAMPLE_RADIO_A: NodeId = hash_node_id("insp_sample_radio_a");
pub const INSP_SAMPLE_RADIO_B: NodeId = hash_node_id("insp_sample_radio_b");
pub const INSP_SAMPLE_RADIO_C: NodeId = hash_node_id("insp_sample_radio_c");
pub const INSP_SAMPLE_DROPDOWN: NodeId = hash_node_id("insp_sample_dropdown");
pub const INSP_SAMPLE_DD_OPT_A: NodeId = hash_node_id("insp_sample_dd_opt_a");
pub const INSP_SAMPLE_DD_OPT_B: NodeId = hash_node_id("insp_sample_dd_opt_b");
pub const INSP_SAMPLE_DD_OPT_C: NodeId = hash_node_id("insp_sample_dd_opt_c");
pub const INSP_SAMPLE_TAB_A: NodeId = hash_node_id("insp_sample_tab_a");
pub const INSP_SAMPLE_TAB_B: NodeId = hash_node_id("insp_sample_tab_b");
pub const INSP_SAMPLE_TAB_C: NodeId = hash_node_id("insp_sample_tab_c");
pub const INSP_SAMPLE_TREE_ROOT: NodeId = hash_node_id("insp_sample_tree_root");
pub const INSP_SAMPLE_TREE_LEAF_A: NodeId = hash_node_id("insp_sample_tree_leaf_a");
pub const INSP_SAMPLE_TREE_LEAF_B: NodeId = hash_node_id("insp_sample_tree_leaf_b");
pub const INSP_SAMPLE_V3_X: NodeId = hash_node_id("insp_sample_v3_x");
pub const INSP_SAMPLE_V3_Y: NodeId = hash_node_id("insp_sample_v3_y");
pub const INSP_SAMPLE_V3_Z: NodeId = hash_node_id("insp_sample_v3_z");
pub const INSP_SAMPLE_SWATCH: NodeId = hash_node_id("insp_sample_swatch");
pub const INSP_SAMPLE_BTN_PRIMARY: NodeId = hash_node_id("insp_sample_btn_primary");
pub const INSP_SAMPLE_BTN_SECONDARY: NodeId = hash_node_id("insp_sample_btn_secondary");
pub const INSP_SAMPLE_BTN_DANGER: NodeId = hash_node_id("insp_sample_btn_danger");
pub const INSP_SAMPLE_BTN_ICON: NodeId = hash_node_id("insp_sample_btn_icon");
pub const INSP_SAMPLE_LIST_ITEM: NodeId = hash_node_id("insp_sample_list_item");
pub const INSP_SAMPLE_TAG_REMOVE: NodeId = hash_node_id("insp_sample_tag_remove");

// Section header ids — clicking toggles the section's collapsed
// state on the WidgetStore. Each maps 1:1 to the corresponding
// `paint_*_section` function in `inspector.rs`.
pub const INSP_SECTION_INPUTS: NodeId = hash_node_id("insp_section_inputs");
pub const INSP_SECTION_SLIDER: NodeId = hash_node_id("insp_section_slider");
pub const INSP_SECTION_SWITCHES: NodeId = hash_node_id("insp_section_switches");
pub const INSP_SECTION_LISTS: NodeId = hash_node_id("insp_section_lists");
pub const INSP_SECTION_VECTOR: NodeId = hash_node_id("insp_section_vector");
pub const INSP_SECTION_STATUS: NodeId = hash_node_id("insp_section_status");
pub const INSP_SECTION_COLOR: NodeId = hash_node_id("insp_section_color");
pub const INSP_SECTION_ACTIONS: NodeId = hash_node_id("insp_section_actions");
pub const INSP_SECTION_IDENTITY: NodeId = hash_node_id("insp_section_identity");
pub const INSP_SECTION_CARD: NodeId = hash_node_id("insp_section_card");

// Section header color-circle hit ids. Each section displays a
// small colored circle on the right of its title (replacing the
// old count chip); clicking the circle opens the global color
// picker for that section. Index ordering matches `SECTION_IDS`.
pub const INSP_SECTION_INPUTS_COLOR: NodeId = hash_node_id("insp_section_inputs_color");
pub const INSP_SECTION_SLIDER_COLOR: NodeId = hash_node_id("insp_section_slider_color");
pub const INSP_SECTION_SWITCHES_COLOR: NodeId = hash_node_id("insp_section_switches_color");
pub const INSP_SECTION_LISTS_COLOR: NodeId = hash_node_id("insp_section_lists_color");
pub const INSP_SECTION_VECTOR_COLOR: NodeId = hash_node_id("insp_section_vector_color");
pub const INSP_SECTION_STATUS_COLOR: NodeId = hash_node_id("insp_section_status_color");
pub const INSP_SECTION_COLOR_COLOR: NodeId = hash_node_id("insp_section_color_color");
pub const INSP_SECTION_ACTIONS_COLOR: NodeId = hash_node_id("insp_section_actions_color");
pub const INSP_SECTION_IDENTITY_COLOR: NodeId = hash_node_id("insp_section_identity_color");
pub const INSP_SECTION_CARD_COLOR: NodeId = hash_node_id("insp_section_card_color");

// Pre-allocated note hit-slot ids. Each note in `notes_per_panel`
// gets one of these slots assigned by position. Right-clicking a
// slot opens the `NoteBackground` context menu for that index.
pub const INSP_NOTE_SLOT_0: NodeId = hash_node_id("insp_note_slot_0");
pub const INSP_NOTE_SLOT_1: NodeId = hash_node_id("insp_note_slot_1");
pub const INSP_NOTE_SLOT_2: NodeId = hash_node_id("insp_note_slot_2");
pub const INSP_NOTE_SLOT_3: NodeId = hash_node_id("insp_note_slot_3");
pub const INSP_NOTE_SLOT_4: NodeId = hash_node_id("insp_note_slot_4");
pub const INSP_NOTE_SLOT_5: NodeId = hash_node_id("insp_note_slot_5");
pub const INSP_NOTE_SLOT_6: NodeId = hash_node_id("insp_note_slot_6");
pub const INSP_NOTE_SLOT_7: NodeId = hash_node_id("insp_note_slot_7");
pub const INSP_NOTE_SLOT_8: NodeId = hash_node_id("insp_note_slot_8");
pub const INSP_NOTE_SLOT_9: NodeId = hash_node_id("insp_note_slot_9");
pub const INSP_NOTE_SLOT_10: NodeId = hash_node_id("insp_note_slot_10");
pub const INSP_NOTE_SLOT_11: NodeId = hash_node_id("insp_note_slot_11");

// Editable text fields for each note slot. NOTE_TITLE_N and
// NOTE_BODY_N are the title TextInput and body TextArea for note N.
// Click → focus → type to edit. Double-click → select all.
pub const INSP_NOTE_TITLE_0: NodeId = hash_node_id("insp_note_title_0");
pub const INSP_NOTE_TITLE_1: NodeId = hash_node_id("insp_note_title_1");
pub const INSP_NOTE_TITLE_2: NodeId = hash_node_id("insp_note_title_2");
pub const INSP_NOTE_TITLE_3: NodeId = hash_node_id("insp_note_title_3");
pub const INSP_NOTE_TITLE_4: NodeId = hash_node_id("insp_note_title_4");
pub const INSP_NOTE_TITLE_5: NodeId = hash_node_id("insp_note_title_5");
pub const INSP_NOTE_TITLE_6: NodeId = hash_node_id("insp_note_title_6");
pub const INSP_NOTE_TITLE_7: NodeId = hash_node_id("insp_note_title_7");
pub const INSP_NOTE_TITLE_8: NodeId = hash_node_id("insp_note_title_8");
pub const INSP_NOTE_TITLE_9: NodeId = hash_node_id("insp_note_title_9");
pub const INSP_NOTE_TITLE_10: NodeId = hash_node_id("insp_note_title_10");
pub const INSP_NOTE_TITLE_11: NodeId = hash_node_id("insp_note_title_11");
pub const INSP_NOTE_BODY_0: NodeId = hash_node_id("insp_note_body_0");
pub const INSP_NOTE_BODY_1: NodeId = hash_node_id("insp_note_body_1");
pub const INSP_NOTE_BODY_2: NodeId = hash_node_id("insp_note_body_2");
pub const INSP_NOTE_BODY_3: NodeId = hash_node_id("insp_note_body_3");
pub const INSP_NOTE_BODY_4: NodeId = hash_node_id("insp_note_body_4");
pub const INSP_NOTE_BODY_5: NodeId = hash_node_id("insp_note_body_5");
pub const INSP_NOTE_BODY_6: NodeId = hash_node_id("insp_note_body_6");
pub const INSP_NOTE_BODY_7: NodeId = hash_node_id("insp_note_body_7");
pub const INSP_NOTE_BODY_8: NodeId = hash_node_id("insp_note_body_8");
pub const INSP_NOTE_BODY_9: NodeId = hash_node_id("insp_note_body_9");
pub const INSP_NOTE_BODY_10: NodeId = hash_node_id("insp_note_body_10");
pub const INSP_NOTE_BODY_11: NodeId = hash_node_id("insp_note_body_11");

// ── Context menu item ids ──────────────────────────────────────────────────
// The right-click context menu reuses these stable ids across both
// inspector and hierarchy. Click dispatch routes by id to the
// inspector's `apply_event`.
pub const CTX_MENU_CREATE_NOTE: NodeId = hash_node_id("ctx_menu_create_note");
pub const CTX_MENU_OUTLINE_NONE: NodeId = hash_node_id("ctx_menu_outline_none");
pub const CTX_MENU_OUTLINE_0: NodeId = hash_node_id("ctx_menu_outline_0");
pub const CTX_MENU_OUTLINE_1: NodeId = hash_node_id("ctx_menu_outline_1");
pub const CTX_MENU_OUTLINE_2: NodeId = hash_node_id("ctx_menu_outline_2");
pub const CTX_MENU_OUTLINE_3: NodeId = hash_node_id("ctx_menu_outline_3");
pub const CTX_MENU_OUTLINE_4: NodeId = hash_node_id("ctx_menu_outline_4");
// Theme selector menu items — opened by clicking TOPBAR_THEME.
pub const CTX_MENU_THEME_FORGE: NodeId = hash_node_id("ctx_menu_theme_forge");
pub const CTX_MENU_THEME_PAINT: NodeId = hash_node_id("ctx_menu_theme_paint");
pub const CTX_MENU_THEME_SUNSTONE: NodeId = hash_node_id("ctx_menu_theme_sunstone");
pub const CTX_MENU_THEME_BLUEPRINT: NodeId = hash_node_id("ctx_menu_theme_blueprint");
// Corner-radius scale presets — also exposed via the theme menu.
pub const CTX_MENU_RADIUS_SHARP: NodeId = hash_node_id("ctx_menu_radius_sharp");
pub const CTX_MENU_RADIUS_DEFAULT: NodeId = hash_node_id("ctx_menu_radius_default");
pub const CTX_MENU_RADIUS_ROUND: NodeId = hash_node_id("ctx_menu_radius_round");
/// "Mirror UI" entry in the theme menu — toggles
/// `HeroScreen::ui_mirrored`, which swaps Hierarchy ↔ Inspector
/// horizontally.
pub const CTX_MENU_MIRROR_UI: NodeId = hash_node_id("ctx_menu_mirror_ui");
/// "Show Statistics" entry in the theme menu — toggles
/// `HeroScreen::stats_visible`, which gates the bottom HUD.
pub const CTX_MENU_SHOW_STATS: NodeId = hash_node_id("ctx_menu_show_stats");
/// "Show Grid" entry in the theme menu — toggles
/// `HeroScreen::grid_visible`, which gates the world-space grid
/// overlay (ADR-0025 M14.4b).
pub const CTX_MENU_SHOW_GRID: NodeId = hash_node_id("ctx_menu_show_grid");
// Save-button context menu (Save / Save As).
pub const CTX_MENU_SAVE: NodeId = hash_node_id("ctx_menu_save");
pub const CTX_MENU_SAVE_AS: NodeId = hash_node_id("ctx_menu_save_as");
// Open-button context menu (Open Project / Import…).
pub const CTX_MENU_OPEN_PROJECT: NodeId = hash_node_id("ctx_menu_open_project");
pub const CTX_MENU_IMPORT: NodeId = hash_node_id("ctx_menu_import");

// Pixels-per-meter presets — opened from the Settings cluster (gear).
// Drives `HeroScreen.project.pixels_per_meter`. The values are the
// canonical presets surfaced as labels in `SettingsMenu`.
//
// Pre-PR-11.3 these sat at hand-picked integers 940..944 to dodge
// the SceneList popover rows (CTX_SCENE_ROW_*) — exactly the type
// of cross-cluster collision the M14.4d audit caught when an early
// draft reused 930..934. Now hash-derived; the regression test in
// `tests/architecture/node_id_collisions.rs` catches reuse mechanically.
pub const CTX_MENU_PPM_16: NodeId = hash_node_id("ctx_menu_ppm_16");
pub const CTX_MENU_PPM_32: NodeId = hash_node_id("ctx_menu_ppm_32");
pub const CTX_MENU_PPM_100: NodeId = hash_node_id("ctx_menu_ppm_100");
pub const CTX_MENU_PPM_256: NodeId = hash_node_id("ctx_menu_ppm_256");
pub const CTX_MENU_PPM_1024: NodeId = hash_node_id("ctx_menu_ppm_1024");

/// M14.7 polish (6.3): top-level Settings cascade entry that opens
/// the Pixels-per-meter submenu.
pub const CTX_MENU_SETTINGS_PPM: NodeId = hash_node_id("ctx_menu_settings_ppm");

/// Top-level Settings entry that opens the Display-unit submenu
/// (Meters / Pixels). Companion of `CTX_MENU_SETTINGS_PPM`.
pub const CTX_MENU_SETTINGS_UNIT: NodeId = hash_node_id("ctx_menu_settings_unit");
pub const CTX_MENU_UNIT_METERS: NodeId = hash_node_id("ctx_menu_unit_meters");
pub const CTX_MENU_UNIT_PIXELS: NodeId = hash_node_id("ctx_menu_unit_pixels");

// M14.6 F: per-row Hierarchy context menu entries. Triggered by a
// secondary (right-button) click on any hierarchy row in live mode;
// `ContextMenuKind::HierarchyRow { row }` carries the target row's
// NodeId so dispatch can attach the action to the right entity when
// any of these ids fires.
pub const CTX_MENU_HIER_DUPLICATE: NodeId = hash_node_id("ctx_menu_hier_duplicate");
pub const CTX_MENU_HIER_DELETE: NodeId = hash_node_id("ctx_menu_hier_delete");
pub const CTX_MENU_HIER_RESET_TRANSFORM: NodeId = hash_node_id("ctx_menu_hier_reset_transform");
pub const CTX_MENU_HIER_ADD_CHILD: NodeId = hash_node_id("ctx_menu_hier_add_child");
/// M14.7 polish: per-row "Rename..." entry. Opens inline rename
/// mode (the row's name turns into a TextInput).
pub const CTX_MENU_HIER_RENAME: NodeId = hash_node_id("ctx_menu_hier_rename");
// Project-chip Scene List popover (search input + up to 8 result rows).
pub const CTX_SCENE_SEARCH: NodeId = hash_node_id("ctx_scene_search");
pub const CTX_SCENE_ROW_0: NodeId = hash_node_id("ctx_scene_row_0");
pub const CTX_SCENE_ROW_1: NodeId = hash_node_id("ctx_scene_row_1");
pub const CTX_SCENE_ROW_2: NodeId = hash_node_id("ctx_scene_row_2");
pub const CTX_SCENE_ROW_3: NodeId = hash_node_id("ctx_scene_row_3");
pub const CTX_SCENE_ROW_4: NodeId = hash_node_id("ctx_scene_row_4");
pub const CTX_SCENE_ROW_5: NodeId = hash_node_id("ctx_scene_row_5");
pub const CTX_SCENE_ROW_6: NodeId = hash_node_id("ctx_scene_row_6");
pub const CTX_SCENE_ROW_7: NodeId = hash_node_id("ctx_scene_row_7");
pub const CTX_SCENE_ROWS: [NodeId; 8] = [
    CTX_SCENE_ROW_0,
    CTX_SCENE_ROW_1,
    CTX_SCENE_ROW_2,
    CTX_SCENE_ROW_3,
    CTX_SCENE_ROW_4,
    CTX_SCENE_ROW_5,
    CTX_SCENE_ROW_6,
    CTX_SCENE_ROW_7,
];
/// Floating `BlenderColorPicker` parent id. The picker is painted
/// over the canvas (not inside the Inspector) — the historical
/// `INSP_` prefix is kept to avoid churning every side-table key.
pub const INSP_BLENDER_PICKER: NodeId = hash_node_id("insp_blender_picker");

// BlenderColorPicker sub-control hit ids — registered by
// `color_picker_demo::paint_blender_picker_demo` every frame,
// dispatched by `dispatch_pointer` into store mutations on
// `INSP_BLENDER_PICKER`.
pub const BLENDER_WHEEL: NodeId = hash_node_id("blender_wheel");
pub const BLENDER_VALUE_SLIDER: NodeId = hash_node_id("blender_value_slider");

// BlenderColorPicker extension hit ids (range 600-699).
// Channel sliders (4 rows: R/H, G/S, B/V, A).
pub const BLENDER_CHANNEL_0: NodeId = hash_node_id("blender_channel_0");
pub const BLENDER_CHANNEL_1: NodeId = hash_node_id("blender_channel_1");
pub const BLENDER_CHANNEL_2: NodeId = hash_node_id("blender_channel_2");
pub const BLENDER_CHANNEL_3: NodeId = hash_node_id("blender_channel_3");
// Hex `#RRGGBBAA` TextInput.
pub const BLENDER_HEX: NodeId = hash_node_id("blender_hex");
// Segmented toggle ids.
pub const BLENDER_INTERP_LINEAR: NodeId = hash_node_id("blender_interp_linear");
pub const BLENDER_INTERP_PERCEPTUAL: NodeId = hash_node_id("blender_interp_perceptual");
pub const BLENDER_CHANNEL_RGB: NodeId = hash_node_id("blender_channel_rgb");
pub const BLENDER_CHANNEL_HSV: NodeId = hash_node_id("blender_channel_hsv");
// Channel value chips — interactive `NumberInput`s mirrored
// to the channel sliders. Display the current channel value
// (R/G/B/A or H/S/V/A depending on `channel_mode`) in 0..1.
pub const BLENDER_NUM_0: NodeId = hash_node_id("blender_num_0");
pub const BLENDER_NUM_1: NodeId = hash_node_id("blender_num_1");
pub const BLENDER_NUM_2: NodeId = hash_node_id("blender_num_2");
pub const BLENDER_NUM_3: NodeId = hash_node_id("blender_num_3");
// "+ swatch" button (appends current value to palette).
pub const BLENDER_ADD_SWATCH: NodeId = hash_node_id("blender_add_swatch");
// Eyedropper button (enters pixel-pick mode).
pub const BLENDER_EYEDROPPER: NodeId = hash_node_id("blender_eyedropper");
// Drag handle bar at the top of the picker — drag to move.
pub const BLENDER_DRAG_HANDLE: NodeId = hash_node_id("blender_drag_handle");
// Palette swatch slots 0..26 — first 12 are the default palette,
// remaining 15 cover user "+ swatch" additions. Hard cap at 27 to
// keep registration static; `blender_palette_push` rejects beyond
// (and the painter hides the "+" tile when the palette is full).
pub const BLENDER_SWATCH_0: NodeId = hash_node_id("blender_swatch_0");
pub const BLENDER_SWATCH_1: NodeId = hash_node_id("blender_swatch_1");
pub const BLENDER_SWATCH_2: NodeId = hash_node_id("blender_swatch_2");
pub const BLENDER_SWATCH_3: NodeId = hash_node_id("blender_swatch_3");
pub const BLENDER_SWATCH_4: NodeId = hash_node_id("blender_swatch_4");
pub const BLENDER_SWATCH_5: NodeId = hash_node_id("blender_swatch_5");
pub const BLENDER_SWATCH_6: NodeId = hash_node_id("blender_swatch_6");
pub const BLENDER_SWATCH_7: NodeId = hash_node_id("blender_swatch_7");
pub const BLENDER_SWATCH_8: NodeId = hash_node_id("blender_swatch_8");
pub const BLENDER_SWATCH_9: NodeId = hash_node_id("blender_swatch_9");
pub const BLENDER_SWATCH_10: NodeId = hash_node_id("blender_swatch_10");
pub const BLENDER_SWATCH_11: NodeId = hash_node_id("blender_swatch_11");
pub const BLENDER_SWATCH_12: NodeId = hash_node_id("blender_swatch_12");
pub const BLENDER_SWATCH_13: NodeId = hash_node_id("blender_swatch_13");
pub const BLENDER_SWATCH_14: NodeId = hash_node_id("blender_swatch_14");
pub const BLENDER_SWATCH_15: NodeId = hash_node_id("blender_swatch_15");
pub const BLENDER_SWATCH_16: NodeId = hash_node_id("blender_swatch_16");
pub const BLENDER_SWATCH_17: NodeId = hash_node_id("blender_swatch_17");
pub const BLENDER_SWATCH_18: NodeId = hash_node_id("blender_swatch_18");
pub const BLENDER_SWATCH_19: NodeId = hash_node_id("blender_swatch_19");
pub const BLENDER_SWATCH_20: NodeId = hash_node_id("blender_swatch_20");
pub const BLENDER_SWATCH_21: NodeId = hash_node_id("blender_swatch_21");
pub const BLENDER_SWATCH_22: NodeId = hash_node_id("blender_swatch_22");
pub const BLENDER_SWATCH_23: NodeId = hash_node_id("blender_swatch_23");
pub const BLENDER_SWATCH_24: NodeId = hash_node_id("blender_swatch_24");
pub const BLENDER_SWATCH_25: NodeId = hash_node_id("blender_swatch_25");
pub const BLENDER_SWATCH_26: NodeId = hash_node_id("blender_swatch_26");

/// Hierarchy panel container — wheel-scroll key.
pub const HIER_PANEL: NodeId = hash_node_id("hier_panel");
pub const HIER_PLAYER: NodeId = NodeId(400);
pub const HIER_SPRITE_IDLE: NodeId = NodeId(401);
pub const HIER_COLLIDER_BOX: NodeId = NodeId(402);
pub const HIER_SCRIPT_PLAYER: NodeId = NodeId(403);
pub const HIER_RIGIDBODY: NodeId = NodeId(404);
pub const HIER_TILEMAP_GROUND: NodeId = NodeId(405);
pub const HIER_TILEMAP_DECOR: NodeId = NodeId(406);
pub const HIER_SLIME_01: NodeId = NodeId(407);
pub const HIER_SLIME_02: NodeId = NodeId(408);
pub const HIER_TRIGGER_ZONE_A: NodeId = NodeId(409);
pub const HIER_AMBIENT_LIGHT: NodeId = NodeId(410);
pub const HIER_MAIN_CAMERA: NodeId = NodeId(411);

/// M14.6 E: search/filter TextInput in the Hierarchy header. Empty
/// query shows every row; non-empty case-insensitively filters by
/// `name.contains(query)` with ancestor-path preservation (a parent
/// stays visible if any descendant matches, so the user sees where
/// the hit lives in the tree).
pub const HIER_SEARCH: NodeId = hash_node_id("hier_search");

/// M14.7 polish: inline rename TextInput on a hierarchy row.
/// Painted only when `HeroScreen.hierarchy.rename_target_row` is `Some(id)`
/// — replaces the matching row's name label with an editable input.
pub const HIER_RENAME_INPUT: NodeId = hash_node_id("hier_rename_input");

/// M14.5 inspector phase (6.4/§9): "Reimport at current px/m" button
/// shown in the Render Source section when the selected sprite has a
/// live atlas-backed source. Click → `HeroScreen.pending_reimport`
/// gets set with the entity bits the host then drains to recompute
/// `Sprite.size` against the current `ProjectSettings.pixels_per_meter`.
pub const INSP_RENDER_SOURCE_REIMPORT: NodeId = hash_node_id("insp_render_source_reimport");

/// M14.5 inspector phase: pixel-format segmented picker in the
/// Render Source section. Pressed = current choice, Normal = the
/// alternative. RGBA16 is `Disabled` until the asset crate gains
/// half-float / 16-bit-channel storage (currently only `ImageRgba8`).
/// Reimport reads the pressed button to decide the target format.
pub const INSP_RENDER_FORMAT_RGBA8: NodeId = hash_node_id("insp_render_format_rgba8");
pub const INSP_RENDER_FORMAT_RGBA16: NodeId = hash_node_id("insp_render_format_rgba16");

/// Map fixture entity name to canonical hierarchy `NodeId`. The
/// placeholder fixture currently exposes only "Scene Root"; the
/// other `HIER_*` ids are kept reserved for the pilot project's
/// real entities.
pub fn hierarchy_id(name: &str) -> Option<NodeId> {
    Some(match name {
        "Scene Root" => HIER_PLAYER,
        _ => return None,
    })
}

/// Map a hierarchy `NodeId` back to its fixture entity name. Inverse
/// of [`hierarchy_id`].
pub fn hierarchy_label_for_id(id: NodeId) -> Option<&'static str> {
    Some(match id {
        x if x == HIER_PLAYER => "Scene Root",
        _ => return None,
    })
}

/// Best-effort 3-letter "kind" badge for the selection tag.
/// Placeholder fixture has a single Scene Root; pilot replaces.
pub fn hierarchy_kind_for_label(_label: &str) -> &'static str {
    "ENT"
}

/// M14.6A: high-bit offset for the eye-toggle companion NodeId on
/// each hierarchy row. The row's primary NodeId is allocated by the
/// host bridge in the low 32-bit range
/// ([`shells/desktop/src/hero_bridge.rs`](shells/desktop/src/hero_bridge.rs)
/// uses `BASE_NODE_ID = 100_000`); the eye companion sits at
/// `row_id.0 | EYE_TOGGLE_BIT` so dispatch can recognize a click as
/// an eye-toggle without an explicit `BlenderHit` registration in
/// the `WidgetStore`.
pub const EYE_TOGGLE_BIT: u64 = 1u64 << 62;

/// M14.6C: high-bit offset for the expand/collapse chevron companion
/// NodeId on parent rows in the Hierarchy panel. Uses a different
/// bit from `EYE_TOGGLE_BIT` so the two companions stay
/// distinguishable in `apply_event`.
pub const EXPAND_TOGGLE_BIT: u64 = 1u64 << 61;

/// Wave 2 PR 11.3 guard: companion detection only fires when the
/// un-masked id is below this threshold (i.e., looks like a real
/// hierarchy row id, allocated linearly from `BASE_NODE_ID = 100_000`
/// by `hero_bridge::EntityNodeMap`). Hash-derived chrome ids
/// (`hash_node_id(...)`) are uniformly distributed across 64 bits and
/// have a ~25% chance of accidentally setting bit 61 or 62; without
/// this threshold they'd be misrouted as row-companion clicks.
///
/// 2^32 (~4 G) is far above any realistic hierarchy size (fixture
/// has 12 rows; pilot project ~100s; even a stress-test 10 M rows
/// stay well below). The regression test
/// `tests/node_id_collisions.rs::no_chrome_id_has_companion_bits`
/// asserts every chrome const stays out of the companion-bit range
/// after hashing.
const COMPANION_ROW_ID_MAX: u64 = 1u64 << 32;

/// Derive the eye-toggle companion NodeId for a hierarchy row.
/// Inverse: [`hier_eye_companion_to_row`].
#[inline]
pub fn hier_eye_companion(row_id: NodeId) -> NodeId {
    NodeId(row_id.0 | EYE_TOGGLE_BIT)
}

/// Recognize whether `id` is an eye-toggle companion and recover the
/// row NodeId. Returns `Some(row_id)` only when the high bit is set
/// AND the un-masked portion falls in the valid hierarchy-row id
/// space (< [`COMPANION_ROW_ID_MAX`]); used by `apply_event` to
/// route eye clicks without touching the BlenderHit pattern.
#[inline]
pub fn hier_eye_companion_to_row(id: NodeId) -> Option<NodeId> {
    if id.0 & EYE_TOGGLE_BIT != 0 {
        let masked = id.0 & !EYE_TOGGLE_BIT;
        if masked < COMPANION_ROW_ID_MAX {
            return Some(NodeId(masked));
        }
    }
    None
}

/// M14.6C: derive the chevron-companion NodeId for the
/// expand/collapse hit-rect at the start of a hierarchy row.
#[inline]
pub fn hier_expand_companion(row_id: NodeId) -> NodeId {
    NodeId(row_id.0 | EXPAND_TOGGLE_BIT)
}

/// Inverse of [`hier_expand_companion`]. Returns `Some(row_id)` only
/// when [`EXPAND_TOGGLE_BIT`] is set AND the un-masked portion falls
/// in the valid hierarchy-row id space (< [`COMPANION_ROW_ID_MAX`]).
#[inline]
pub fn hier_expand_companion_to_row(id: NodeId) -> Option<NodeId> {
    if id.0 & EXPAND_TOGGLE_BIT != 0 {
        let masked = id.0 & !EXPAND_TOGGLE_BIT;
        if masked < COMPANION_ROW_ID_MAX {
            return Some(NodeId(masked));
        }
    }
    None
}

// ─── Wave 6+7 Phase 2: consolidated section + grid-snap ids that
// dispatch (in editor-core) needs to query. Definitions originally
// lived in `screens/hero/inspector/mod.rs` and `grid_snap/ids.rs`;
// moved here so editor-core can reference them without depending
// back on ph2d-editor. Inspector + grid_snap re-export for legacy
// import-path stability.

/// Stable id list for every collapsible section header in the
/// Inspector (Widget Gallery showcase mode).
pub const SECTION_IDS: [NodeId; 10] = [
    INSP_SECTION_INPUTS,
    INSP_SECTION_SLIDER,
    INSP_SECTION_SWITCHES,
    INSP_SECTION_LISTS,
    INSP_SECTION_VECTOR,
    INSP_SECTION_STATUS,
    INSP_SECTION_COLOR,
    INSP_SECTION_ACTIONS,
    INSP_SECTION_IDENTITY,
    INSP_SECTION_CARD,
];

/// Live Inspector section headers (Name / Visibility / Transform /
/// Render). Right-click opens the SectionOutline context menu.
pub const LIVE_SECTION_IDS: [NodeId; 4] = [
    INSP_LIVE_NAME_SECTION,
    INSP_LIVE_VISIBILITY_SECTION,
    INSP_LIVE_TRANSFORM_SECTION,
    INSP_LIVE_RENDER_SECTION,
];

/// Grid-snap floating panel root id.
pub const GS_PANEL: NodeId = NodeId(1000);

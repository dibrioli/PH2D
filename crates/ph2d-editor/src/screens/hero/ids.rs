//! Stable [`NodeId`] constants for the hero screen's interactive
//! widgets + helper mappings between fixture entity names and ids.
//!
//! Pre-populated in [`crate::interaction::WidgetStore`] at
//! construction time so the dispatcher always finds an entry on
//! hit-test. Numeric ranges:
//!
//! - 100..199 — TopBar buttons + Hierarchy add
//! - 200..299 — LeftRail tools
//! - 300..399 — Inspector slots (currently only the panel container;
//!   the placeholder field ids were removed when the inspector was
//!   emptied for the next sample-loading phase)
//! - 400..499 — Hierarchy entity rows
//! - 600..699 — Floating BlenderColorPicker sub-controls

use ph2d_a11y::NodeId;

pub const TOPBAR_THEME: NodeId = NodeId(101);
pub const TOPBAR_SAVE: NodeId = NodeId(102);
pub const TOPBAR_PROJECT: NodeId = NodeId(103);
pub const TOPBAR_PLAY_TOGGLE: NodeId = NodeId(104);
pub const TOPBAR_PLAY_BUTTON: NodeId = NodeId(105);
pub const TOPBAR_RIGHT_LAYERS: NodeId = NodeId(106);
pub const TOPBAR_RIGHT_ASSETS: NodeId = NodeId(107);
pub const TOPBAR_RIGHT_SCRIPT: NodeId = NodeId(108);
pub const TOPBAR_PAUSE: NodeId = NodeId(109);
pub const TOPBAR_RESET: NodeId = NodeId(110);
pub const TOPBAR_SAVE_AS: NodeId = NodeId(111);
pub const TOPBAR_OPEN: NodeId = NodeId(112);
/// Settings cluster (gear icon) — opens the SettingsMenu context menu
/// with project-level toggles (pixels-per-meter presets, future
/// global config). Added M14.4d retrofit.
pub const TOPBAR_SETTINGS: NodeId = NodeId(113);
/// Image Tools cluster — toggle entry-point for the image-editing
/// action row (Trim Transparency in V1; BG Removal / Equalize / etc.
/// to follow). Click flips the TopBar between Edit mode and
/// ImageTools mode; the state lives on
/// [`crate::screens::HeroScreen::image_tools_mode`].
pub const TOPBAR_IMAGE_TOOLS: NodeId = NodeId(114);
/// Widget Gallery cluster — toggles the floating reference panel
/// that showcases every canonical widget (Inputs / Slider /
/// Switches / Lists / Vector / Status / Color / Actions / Identity /
/// Card). Peripheral agents open this from the live app as the
/// single in-app source of truth for UI decoration. State lives on
/// [`crate::screens::HeroScreen::widget_gallery_visible`].
pub const TOPBAR_WIDGET_GALLERY: NodeId = NodeId(116);

/// Image Tools action — Trim Transparency pill. Lives in the action
/// row that replaces the right-side TopBar clusters when
/// `image_tools_mode` is on. Click is no-op for now — wiring to the
/// `ph2d_editor::trim_transparency()` algorithm on a selected sprite
/// requires the live asset model (out of scope for this PR).
pub const IMAGE_ACTION_TRIM: NodeId = NodeId(115);

pub const HIERARCHY_ADD: NodeId = NodeId(150);

pub const TOOL_TRANSLATE: NodeId = NodeId(201);
pub const TOOL_ROTATE: NodeId = NodeId(202);
pub const TOOL_SCALE: NodeId = NodeId(203);
pub const TOOL_PIVOT: NodeId = NodeId(204);
pub const TOOL_SPACE: NodeId = NodeId(205);
pub const TOOL_PROJECTION: NodeId = NodeId(206);
pub const TOOL_HOME: NodeId = NodeId(207);
pub const TOOL_UNDO: NodeId = NodeId(208);
pub const TOOL_REDO: NodeId = NodeId(209);
/// Show/Hide toggles for the side panels — top of the left rail.
/// `Pressed` state == panel currently visible.
pub const RAIL_SHOW_INSPECTOR: NodeId = NodeId(210);
pub const RAIL_SHOW_HIERARCHY: NodeId = NodeId(211);

/// Inspector panel container — used as the wheel-scroll key.
pub const INSP_PANEL: NodeId = NodeId(371);
/// Drag handle at the top of the Inspector — click+drag moves the
/// panel. Registered as `BlenderHit { parent: INSP_PANEL, kind:
/// DragHandle }` so the existing picker-drag dispatch infra
/// (panel-agnostic on parent NodeId) drives it.
pub const INSP_DRAG_HANDLE: NodeId = NodeId(372);
/// Resize gripper at the Inspector's bottom-right corner. Registered
/// as `BlenderHit { parent: INSP_PANEL, kind: ResizeHandle }`.
pub const INSP_RESIZE_HANDLE: NodeId = NodeId(373);
/// Widget Gallery floating panel — root id. The gallery is a Procreate-
/// style floating reference panel that hosts the canonical widget
/// showcase. Toggle visibility via [`TOPBAR_WIDGET_GALLERY`].
pub const GAL_PANEL: NodeId = NodeId(950);
/// Drag handle pill at the top of the Widget Gallery panel.
pub const GAL_DRAG_HANDLE: NodeId = NodeId(951);
/// Resize gripper at the Widget Gallery's bottom-right corner.
pub const GAL_RESIZE_HANDLE: NodeId = NodeId(952);
/// Close (X) button at the top-right of the Widget Gallery — alternate
/// way to dismiss the panel beyond clicking the TopBar palette pill.
pub const GAL_CLOSE: NodeId = NodeId(953);
/// Drag handle at the top of the Hierarchy.
pub const HIER_DRAG_HANDLE: NodeId = NodeId(398);
/// Resize gripper at the Hierarchy's bottom-right corner.
pub const HIER_RESIZE_HANDLE: NodeId = NodeId(397);

// ── Inspector widget samples ───────────────────────────────────────────────
// One of each canonical widget, parented to the Inspector panel.
// These are *demonstration* widgets; their state lives on the store
// but is not wired to any simulation. The placeholder fixture-driven
// rows that used to live in 300..370 were removed pre-samples.
pub const INSP_SAMPLE_TEXT: NodeId = NodeId(300);
pub const INSP_SAMPLE_TEXTAREA: NodeId = NodeId(301);
pub const INSP_SAMPLE_COMBO: NodeId = NodeId(302);
pub const INSP_SAMPLE_COMBO_OPT_A: NodeId = NodeId(303);
pub const INSP_SAMPLE_COMBO_OPT_B: NodeId = NodeId(304);
pub const INSP_SAMPLE_COMBO_OPT_C: NodeId = NodeId(305);
pub const INSP_SAMPLE_NUMBER: NodeId = NodeId(306);
pub const INSP_SAMPLE_SLIDER: NodeId = NodeId(307);
pub const INSP_SAMPLE_SLIDER_CHIP: NodeId = NodeId(308);
pub const INSP_SAMPLE_CHECKBOX: NodeId = NodeId(309);
pub const INSP_SAMPLE_TOGGLE: NodeId = NodeId(310);
pub const INSP_SAMPLE_RADIO_A: NodeId = NodeId(312);
pub const INSP_SAMPLE_RADIO_B: NodeId = NodeId(313);
pub const INSP_SAMPLE_RADIO_C: NodeId = NodeId(314);
pub const INSP_SAMPLE_DROPDOWN: NodeId = NodeId(315);
pub const INSP_SAMPLE_DD_OPT_A: NodeId = NodeId(316);
pub const INSP_SAMPLE_DD_OPT_B: NodeId = NodeId(317);
pub const INSP_SAMPLE_DD_OPT_C: NodeId = NodeId(318);
pub const INSP_SAMPLE_TAB_A: NodeId = NodeId(319);
pub const INSP_SAMPLE_TAB_B: NodeId = NodeId(320);
pub const INSP_SAMPLE_TAB_C: NodeId = NodeId(321);
pub const INSP_SAMPLE_TREE_ROOT: NodeId = NodeId(322);
pub const INSP_SAMPLE_TREE_LEAF_A: NodeId = NodeId(323);
pub const INSP_SAMPLE_TREE_LEAF_B: NodeId = NodeId(324);
pub const INSP_SAMPLE_V3_X: NodeId = NodeId(325);
pub const INSP_SAMPLE_V3_Y: NodeId = NodeId(326);
pub const INSP_SAMPLE_V3_Z: NodeId = NodeId(327);
pub const INSP_SAMPLE_SWATCH: NodeId = NodeId(328);
pub const INSP_SAMPLE_BTN_PRIMARY: NodeId = NodeId(329);
pub const INSP_SAMPLE_BTN_SECONDARY: NodeId = NodeId(330);
pub const INSP_SAMPLE_BTN_DANGER: NodeId = NodeId(331);
pub const INSP_SAMPLE_BTN_ICON: NodeId = NodeId(332);
pub const INSP_SAMPLE_LIST_ITEM: NodeId = NodeId(333);
pub const INSP_SAMPLE_TAG_REMOVE: NodeId = NodeId(334);

// Section header ids — clicking toggles the section's collapsed
// state on the WidgetStore. Each maps 1:1 to the corresponding
// `paint_*_section` function in `inspector.rs`.
pub const INSP_SECTION_INPUTS: NodeId = NodeId(350);
pub const INSP_SECTION_SLIDER: NodeId = NodeId(351);
pub const INSP_SECTION_SWITCHES: NodeId = NodeId(352);
pub const INSP_SECTION_LISTS: NodeId = NodeId(353);
pub const INSP_SECTION_VECTOR: NodeId = NodeId(354);
pub const INSP_SECTION_STATUS: NodeId = NodeId(355);
pub const INSP_SECTION_COLOR: NodeId = NodeId(356);
pub const INSP_SECTION_ACTIONS: NodeId = NodeId(357);
pub const INSP_SECTION_IDENTITY: NodeId = NodeId(358);
pub const INSP_SECTION_CARD: NodeId = NodeId(359);

// Section header color-circle hit ids. Each section displays a
// small colored circle on the right of its title (replacing the
// old count chip); clicking the circle opens the global color
// picker for that section. Index ordering matches `SECTION_IDS`.
pub const INSP_SECTION_INPUTS_COLOR: NodeId = NodeId(360);
pub const INSP_SECTION_SLIDER_COLOR: NodeId = NodeId(361);
pub const INSP_SECTION_SWITCHES_COLOR: NodeId = NodeId(362);
pub const INSP_SECTION_LISTS_COLOR: NodeId = NodeId(363);
pub const INSP_SECTION_VECTOR_COLOR: NodeId = NodeId(364);
pub const INSP_SECTION_STATUS_COLOR: NodeId = NodeId(365);
pub const INSP_SECTION_COLOR_COLOR: NodeId = NodeId(366);
pub const INSP_SECTION_ACTIONS_COLOR: NodeId = NodeId(367);
pub const INSP_SECTION_IDENTITY_COLOR: NodeId = NodeId(368);
pub const INSP_SECTION_CARD_COLOR: NodeId = NodeId(369);

// Pre-allocated note hit-slot ids. Each note in `notes_per_panel`
// gets one of these slots assigned by position. Right-clicking a
// slot opens the `NoteBackground` context menu for that index.
pub const INSP_NOTE_SLOT_0: NodeId = NodeId(800);
pub const INSP_NOTE_SLOT_1: NodeId = NodeId(801);
pub const INSP_NOTE_SLOT_2: NodeId = NodeId(802);
pub const INSP_NOTE_SLOT_3: NodeId = NodeId(803);
pub const INSP_NOTE_SLOT_4: NodeId = NodeId(804);
pub const INSP_NOTE_SLOT_5: NodeId = NodeId(805);
pub const INSP_NOTE_SLOT_6: NodeId = NodeId(806);
pub const INSP_NOTE_SLOT_7: NodeId = NodeId(807);
pub const INSP_NOTE_SLOT_8: NodeId = NodeId(808);
pub const INSP_NOTE_SLOT_9: NodeId = NodeId(809);
pub const INSP_NOTE_SLOT_10: NodeId = NodeId(810);
pub const INSP_NOTE_SLOT_11: NodeId = NodeId(811);

// Editable text fields for each note slot. NOTE_TITLE_N and
// NOTE_BODY_N are the title TextInput and body TextArea for note N.
// Click → focus → type to edit. Double-click → select all.
pub const INSP_NOTE_TITLE_0: NodeId = NodeId(830);
pub const INSP_NOTE_TITLE_1: NodeId = NodeId(831);
pub const INSP_NOTE_TITLE_2: NodeId = NodeId(832);
pub const INSP_NOTE_TITLE_3: NodeId = NodeId(833);
pub const INSP_NOTE_TITLE_4: NodeId = NodeId(834);
pub const INSP_NOTE_TITLE_5: NodeId = NodeId(835);
pub const INSP_NOTE_TITLE_6: NodeId = NodeId(836);
pub const INSP_NOTE_TITLE_7: NodeId = NodeId(837);
pub const INSP_NOTE_TITLE_8: NodeId = NodeId(838);
pub const INSP_NOTE_TITLE_9: NodeId = NodeId(839);
pub const INSP_NOTE_TITLE_10: NodeId = NodeId(840);
pub const INSP_NOTE_TITLE_11: NodeId = NodeId(841);
pub const INSP_NOTE_BODY_0: NodeId = NodeId(850);
pub const INSP_NOTE_BODY_1: NodeId = NodeId(851);
pub const INSP_NOTE_BODY_2: NodeId = NodeId(852);
pub const INSP_NOTE_BODY_3: NodeId = NodeId(853);
pub const INSP_NOTE_BODY_4: NodeId = NodeId(854);
pub const INSP_NOTE_BODY_5: NodeId = NodeId(855);
pub const INSP_NOTE_BODY_6: NodeId = NodeId(856);
pub const INSP_NOTE_BODY_7: NodeId = NodeId(857);
pub const INSP_NOTE_BODY_8: NodeId = NodeId(858);
pub const INSP_NOTE_BODY_9: NodeId = NodeId(859);
pub const INSP_NOTE_BODY_10: NodeId = NodeId(860);
pub const INSP_NOTE_BODY_11: NodeId = NodeId(861);

// ── Context menu item ids ──────────────────────────────────────────────────
// The right-click context menu reuses these stable ids across both
// inspector and hierarchy. Click dispatch routes by id to the
// inspector's `apply_event`.
pub const CTX_MENU_CREATE_NOTE: NodeId = NodeId(900);
pub const CTX_MENU_OUTLINE_NONE: NodeId = NodeId(901);
pub const CTX_MENU_OUTLINE_0: NodeId = NodeId(902);
pub const CTX_MENU_OUTLINE_1: NodeId = NodeId(903);
pub const CTX_MENU_OUTLINE_2: NodeId = NodeId(904);
pub const CTX_MENU_OUTLINE_3: NodeId = NodeId(905);
pub const CTX_MENU_OUTLINE_4: NodeId = NodeId(906);
// Theme selector menu items — opened by clicking TOPBAR_THEME.
pub const CTX_MENU_THEME_FORGE: NodeId = NodeId(910);
pub const CTX_MENU_THEME_PAINT: NodeId = NodeId(911);
pub const CTX_MENU_THEME_SUNSTONE: NodeId = NodeId(912);
pub const CTX_MENU_THEME_BLUEPRINT: NodeId = NodeId(913);
// Corner-radius scale presets — also exposed via the theme menu.
pub const CTX_MENU_RADIUS_SHARP: NodeId = NodeId(914);
pub const CTX_MENU_RADIUS_DEFAULT: NodeId = NodeId(915);
pub const CTX_MENU_RADIUS_ROUND: NodeId = NodeId(916);
/// "Mirror UI" entry in the theme menu — toggles
/// `HeroScreen::ui_mirrored`, which swaps Hierarchy ↔ Inspector
/// horizontally.
pub const CTX_MENU_MIRROR_UI: NodeId = NodeId(917);
/// "Show Statistics" entry in the theme menu — toggles
/// `HeroScreen::stats_visible`, which gates the bottom HUD.
pub const CTX_MENU_SHOW_STATS: NodeId = NodeId(918);
/// "Show Grid" entry in the theme menu — toggles
/// `HeroScreen::grid_visible`, which gates the world-space grid
/// overlay (ADR-0025 M14.4b).
pub const CTX_MENU_SHOW_GRID: NodeId = NodeId(919);
// Save-button context menu (Save / Save As).
pub const CTX_MENU_SAVE: NodeId = NodeId(920);
pub const CTX_MENU_SAVE_AS: NodeId = NodeId(921);
// Open-button context menu (Open Project / Import…).
pub const CTX_MENU_OPEN_PROJECT: NodeId = NodeId(922);
pub const CTX_MENU_IMPORT: NodeId = NodeId(923);

// Pixels-per-meter presets — opened from the Settings cluster (gear).
// Drives `HeroScreen.project.pixels_per_meter`. The values are the
// canonical presets surfaced as labels in `SettingsMenu`.
//
// Range 940..945 — sits PAST `CTX_SCENE_ROW_0..7` (931-938) to
// avoid the NodeId collision that the M14.4d audit caught: an
// initial draft of these IDs reused 930-934, which mirrored the
// SceneList popover rows and made dispatch ambiguous (a click on
// `CTX_MENU_PPM_16` would also fire `CTX_SCENE_SEARCH`).
pub const CTX_MENU_PPM_16: NodeId = NodeId(940);
pub const CTX_MENU_PPM_32: NodeId = NodeId(941);
pub const CTX_MENU_PPM_100: NodeId = NodeId(942);
pub const CTX_MENU_PPM_256: NodeId = NodeId(943);
pub const CTX_MENU_PPM_1024: NodeId = NodeId(944);

/// M14.7 polish (6.3): top-level Settings cascade entry that opens
/// the Pixels-per-meter submenu. Sits between `CTX_MENU_HIER_*`
/// (945-948) and the SAVE/OPEN range — id 949.
pub const CTX_MENU_SETTINGS_PPM: NodeId = NodeId(949);

// M14.6 F: per-row Hierarchy context menu entries. Triggered by a
// secondary (right-button) click on any hierarchy row in live mode;
// `ContextMenuKind::HierarchyRow { row }` carries the target row's
// NodeId so dispatch can attach the action to the right entity when
// any of these ids fires.
pub const CTX_MENU_HIER_DUPLICATE: NodeId = NodeId(945);
pub const CTX_MENU_HIER_DELETE: NodeId = NodeId(946);
pub const CTX_MENU_HIER_RESET_TRANSFORM: NodeId = NodeId(947);
pub const CTX_MENU_HIER_ADD_CHILD: NodeId = NodeId(948);
/// M14.7 polish: per-row "Rename..." entry. Opens inline rename
/// mode (the row's name turns into a TextInput).
pub const CTX_MENU_HIER_RENAME: NodeId = NodeId(970);
// Project-chip Scene List popover (search input + up to 8 result rows).
pub const CTX_SCENE_SEARCH: NodeId = NodeId(930);
pub const CTX_SCENE_ROW_0: NodeId = NodeId(931);
pub const CTX_SCENE_ROW_1: NodeId = NodeId(932);
pub const CTX_SCENE_ROW_2: NodeId = NodeId(933);
pub const CTX_SCENE_ROW_3: NodeId = NodeId(934);
pub const CTX_SCENE_ROW_4: NodeId = NodeId(935);
pub const CTX_SCENE_ROW_5: NodeId = NodeId(936);
pub const CTX_SCENE_ROW_6: NodeId = NodeId(937);
pub const CTX_SCENE_ROW_7: NodeId = NodeId(938);
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
pub const INSP_BLENDER_PICKER: NodeId = NodeId(380);

// BlenderColorPicker sub-control hit ids — registered by
// `color_picker_demo::paint_blender_picker_demo` every frame,
// dispatched by `dispatch_pointer` into store mutations on
// `INSP_BLENDER_PICKER`.
pub const BLENDER_WHEEL: NodeId = NodeId(381);
pub const BLENDER_VALUE_SLIDER: NodeId = NodeId(382);

// BlenderColorPicker extension hit ids (range 600-699).
// Channel sliders (4 rows: R/H, G/S, B/V, A).
pub const BLENDER_CHANNEL_0: NodeId = NodeId(600);
pub const BLENDER_CHANNEL_1: NodeId = NodeId(601);
pub const BLENDER_CHANNEL_2: NodeId = NodeId(602);
pub const BLENDER_CHANNEL_3: NodeId = NodeId(603);
// Hex `#RRGGBBAA` TextInput.
pub const BLENDER_HEX: NodeId = NodeId(604);
// Segmented toggle ids.
pub const BLENDER_INTERP_LINEAR: NodeId = NodeId(610);
pub const BLENDER_INTERP_PERCEPTUAL: NodeId = NodeId(611);
pub const BLENDER_CHANNEL_RGB: NodeId = NodeId(612);
pub const BLENDER_CHANNEL_HSV: NodeId = NodeId(613);
// Channel value chips — interactive `NumberInput`s mirrored
// to the channel sliders. Display the current channel value
// (R/G/B/A or H/S/V/A depending on `channel_mode`) in 0..1.
pub const BLENDER_NUM_0: NodeId = NodeId(640);
pub const BLENDER_NUM_1: NodeId = NodeId(641);
pub const BLENDER_NUM_2: NodeId = NodeId(642);
pub const BLENDER_NUM_3: NodeId = NodeId(643);
// "+ swatch" button (appends current value to palette).
pub const BLENDER_ADD_SWATCH: NodeId = NodeId(644);
// Eyedropper button (enters pixel-pick mode).
pub const BLENDER_EYEDROPPER: NodeId = NodeId(645);
// Drag handle bar at the top of the picker — drag to move.
pub const BLENDER_DRAG_HANDLE: NodeId = NodeId(646);
// Palette swatch slots 0..26 — first 12 are the default palette,
// remaining 15 cover user "+ swatch" additions. Hard cap at 27 to
// keep registration static; `blender_palette_push` rejects beyond
// (and the painter hides the "+" tile when the palette is full).
pub const BLENDER_SWATCH_0: NodeId = NodeId(620);
pub const BLENDER_SWATCH_1: NodeId = NodeId(621);
pub const BLENDER_SWATCH_2: NodeId = NodeId(622);
pub const BLENDER_SWATCH_3: NodeId = NodeId(623);
pub const BLENDER_SWATCH_4: NodeId = NodeId(624);
pub const BLENDER_SWATCH_5: NodeId = NodeId(625);
pub const BLENDER_SWATCH_6: NodeId = NodeId(626);
pub const BLENDER_SWATCH_7: NodeId = NodeId(627);
pub const BLENDER_SWATCH_8: NodeId = NodeId(628);
pub const BLENDER_SWATCH_9: NodeId = NodeId(629);
pub const BLENDER_SWATCH_10: NodeId = NodeId(630);
pub const BLENDER_SWATCH_11: NodeId = NodeId(631);
pub const BLENDER_SWATCH_12: NodeId = NodeId(632);
pub const BLENDER_SWATCH_13: NodeId = NodeId(633);
pub const BLENDER_SWATCH_14: NodeId = NodeId(634);
pub const BLENDER_SWATCH_15: NodeId = NodeId(635);
pub const BLENDER_SWATCH_16: NodeId = NodeId(636);
pub const BLENDER_SWATCH_17: NodeId = NodeId(637);
pub const BLENDER_SWATCH_18: NodeId = NodeId(638);
pub const BLENDER_SWATCH_19: NodeId = NodeId(639);
pub const BLENDER_SWATCH_20: NodeId = NodeId(650);
pub const BLENDER_SWATCH_21: NodeId = NodeId(651);
pub const BLENDER_SWATCH_22: NodeId = NodeId(652);
pub const BLENDER_SWATCH_23: NodeId = NodeId(653);
pub const BLENDER_SWATCH_24: NodeId = NodeId(654);
pub const BLENDER_SWATCH_25: NodeId = NodeId(655);
pub const BLENDER_SWATCH_26: NodeId = NodeId(656);

/// Hierarchy panel container — wheel-scroll key.
pub const HIER_PANEL: NodeId = NodeId(399);
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
pub const HIER_SEARCH: NodeId = NodeId(412);

/// M14.7 polish: inline rename TextInput on a hierarchy row.
/// Painted only when `HeroScreen.rename_target_row` is `Some(id)`
/// — replaces the matching row's name label with an editable input.
pub const HIER_RENAME_INPUT: NodeId = NodeId(413);

/// M14.5 inspector phase (6.4/§9): "Reimport at current px/m" button
/// shown in the Render Source section when the selected sprite has a
/// live atlas-backed source. Click → `HeroScreen.pending_reimport`
/// gets set with the entity bits the host then drains to recompute
/// `Sprite.size` against the current `ProjectSettings.pixels_per_meter`.
pub const INSP_RENDER_SOURCE_REIMPORT: NodeId = NodeId(414);

/// M14.5 inspector phase: pixel-format segmented picker in the
/// Render Source section. Pressed = current choice, Normal = the
/// alternative. RGBA16 is `Disabled` until the asset crate gains
/// half-float / 16-bit-channel storage (currently only `ImageRgba8`).
/// Reimport reads the pressed button to decide the target format.
pub const INSP_RENDER_FORMAT_RGBA8: NodeId = NodeId(415);
pub const INSP_RENDER_FORMAT_RGBA16: NodeId = NodeId(416);

/// Map fixture entity name to canonical hierarchy `NodeId`. The
/// placeholder fixture currently exposes only "Scene Root"; the
/// other `HIER_*` ids are kept reserved for the pilot project's
/// real entities.
pub(crate) fn hierarchy_id(name: &str) -> Option<NodeId> {
    Some(match name {
        "Scene Root" => HIER_PLAYER,
        _ => return None,
    })
}

/// Map a hierarchy `NodeId` back to its fixture entity name. Inverse
/// of [`hierarchy_id`].
pub(crate) fn hierarchy_label_for_id(id: NodeId) -> Option<&'static str> {
    Some(match id {
        x if x == HIER_PLAYER => "Scene Root",
        _ => return None,
    })
}

/// Best-effort 3-letter "kind" badge for the selection tag.
/// Placeholder fixture has a single Scene Root; pilot replaces.
pub(crate) fn hierarchy_kind_for_label(_label: &str) -> &'static str {
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

/// Derive the eye-toggle companion NodeId for a hierarchy row.
/// Inverse: [`hier_eye_companion_to_row`].
#[inline]
pub fn hier_eye_companion(row_id: NodeId) -> NodeId {
    NodeId(row_id.0 | EYE_TOGGLE_BIT)
}

/// Recognize whether `id` is an eye-toggle companion and recover the
/// row NodeId. Returns `Some(row_id)` if the high bit is set,
/// `None` otherwise. Used by `apply_event` to route eye clicks
/// without touching the BlenderHit pattern.
#[inline]
pub fn hier_eye_companion_to_row(id: NodeId) -> Option<NodeId> {
    if id.0 & EYE_TOGGLE_BIT != 0 {
        Some(NodeId(id.0 & !EYE_TOGGLE_BIT))
    } else {
        None
    }
}

/// M14.6C: derive the chevron-companion NodeId for the
/// expand/collapse hit-rect at the start of a hierarchy row.
#[inline]
pub fn hier_expand_companion(row_id: NodeId) -> NodeId {
    NodeId(row_id.0 | EXPAND_TOGGLE_BIT)
}

/// Inverse of [`hier_expand_companion`]. Returns `Some(row_id)` when
/// the high `EXPAND_TOGGLE_BIT` is set, `None` otherwise.
#[inline]
pub fn hier_expand_companion_to_row(id: NodeId) -> Option<NodeId> {
    if id.0 & EXPAND_TOGGLE_BIT != 0 {
        Some(NodeId(id.0 & !EXPAND_TOGGLE_BIT))
    } else {
        None
    }
}

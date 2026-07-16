//! Regression: every chrome `NodeId` declared in
//! [`ph2d_editor_core::screens::hero::ids`] (Wave 2 PR 11.3) must be unique.
//!
//! The ids are derived from FNV-1a-hashed slug strings via
//! `hash_node_id`. The function is collision-resistant in principle
//! (64-bit output, ~200-id chrome surface = < 2e-15 birthday prob),
//! but a typo that gives two consts the same slug would silently
//! short-circuit hit-test routing in production. This test enumerates
//! every public chrome const and asserts pairwise uniqueness so any
//! future regression — slug typo, copy-paste mistake, accidental
//! string-table merge — is caught at `cargo test` time.
//!
//! Pre-PR-11.3 the file allocated ids by hand and six numeric
//! collisions had already slipped in (380, 381, 382, 853, 854, 855).
//! Those are the reason this test exists.

use ph2d_a11y::NodeId;
use ph2d_editor_core::screens::hero::ids;

/// Every chrome [`NodeId`] const exported by [`ids`], paired with its
/// const identifier for error reporting. Hand-maintained — extending
/// chrome means adding a row here so the new id participates in the
/// uniqueness check.
const CHROME_IDS: &[(&str, NodeId)] = &[
    // TopBar
    ("TOPBAR_THEME", ids::TOPBAR_THEME),
    ("TOPBAR_SAVE", ids::TOPBAR_SAVE),
    ("TOPBAR_PROJECT", ids::TOPBAR_PROJECT),
    ("TOPBAR_PLAY_TOGGLE", ids::TOPBAR_PLAY_TOGGLE),
    ("TOPBAR_PLAY_BUTTON", ids::TOPBAR_PLAY_BUTTON),
    ("TOPBAR_RIGHT_LAYERS", ids::TOPBAR_RIGHT_LAYERS),
    ("TOPBAR_RIGHT_ASSETS", ids::TOPBAR_RIGHT_ASSETS),
    ("TOPBAR_RIGHT_SCRIPT", ids::TOPBAR_RIGHT_SCRIPT),
    ("TOPBAR_PAUSE", ids::TOPBAR_PAUSE),
    ("TOPBAR_RESET", ids::TOPBAR_RESET),
    ("TOPBAR_SAVE_AS", ids::TOPBAR_SAVE_AS),
    ("TOPBAR_OPEN", ids::TOPBAR_OPEN),
    ("TOPBAR_SETTINGS", ids::TOPBAR_SETTINGS),
    ("TOPBAR_IMAGE_TOOLS", ids::TOPBAR_IMAGE_TOOLS),
    ("TOPBAR_WIDGET_GALLERY", ids::TOPBAR_WIDGET_GALLERY),
    ("TOPBAR_GRID_SETTINGS", ids::TOPBAR_GRID_SETTINGS),
    ("IMAGE_ACTION_TRIM", ids::IMAGE_ACTION_TRIM),
    ("IMAGE_ACTION_MAKE_SQUARE", ids::IMAGE_ACTION_MAKE_SQUARE),
    ("IMAGE_ACTION_BGREMOVAL", ids::IMAGE_ACTION_BGREMOVAL),
    ("IMAGE_ACTION_REAL_SIZE", ids::IMAGE_ACTION_REAL_SIZE),
    ("IMAGE_ACTION_PADDING", ids::IMAGE_ACTION_PADDING),
    ("PAD_PANEL", ids::PAD_PANEL),
    ("PAD_TOP", ids::PAD_TOP),
    ("PAD_RIGHT", ids::PAD_RIGHT),
    ("PAD_BOTTOM", ids::PAD_BOTTOM),
    ("PAD_LEFT", ids::PAD_LEFT),
    ("PAD_APPLY", ids::PAD_APPLY),
    ("PAD_CANCEL", ids::PAD_CANCEL),
    ("HIERARCHY_ADD", ids::HIERARCHY_ADD),
    // LeftRail
    ("TOOL_TRANSLATE", ids::TOOL_TRANSLATE),
    ("TOOL_ROTATE", ids::TOOL_ROTATE),
    ("TOOL_SCALE", ids::TOOL_SCALE),
    ("TOOL_PIVOT", ids::TOOL_PIVOT),
    ("TOOL_SPACE", ids::TOOL_SPACE),
    ("TOOL_PROJECTION", ids::TOOL_PROJECTION),
    ("TOOL_HOME", ids::TOOL_HOME),
    ("TOOL_UNDO", ids::TOOL_UNDO),
    ("TOOL_REDO", ids::TOOL_REDO),
    ("RAIL_SHOW_INSPECTOR", ids::RAIL_SHOW_INSPECTOR),
    ("RAIL_SHOW_HIERARCHY", ids::RAIL_SHOW_HIERARCHY),
    // Inspector chrome
    ("INSP_PANEL", ids::INSP_PANEL),
    ("INSP_DRAG_HANDLE", ids::INSP_DRAG_HANDLE),
    ("INSP_RESIZE_HANDLE", ids::INSP_RESIZE_HANDLE),
    ("INSP_TRANSFORM_SECTION", ids::INSP_TRANSFORM_SECTION),
    ("INSP_TRANSFORM_POS_X", ids::INSP_TRANSFORM_POS_X),
    ("INSP_TRANSFORM_POS_Y", ids::INSP_TRANSFORM_POS_Y),
    ("INSP_TRANSFORM_ROT", ids::INSP_TRANSFORM_ROT),
    ("INSP_TRANSFORM_SCALE_X", ids::INSP_TRANSFORM_SCALE_X),
    ("INSP_TRANSFORM_SCALE_Y", ids::INSP_TRANSFORM_SCALE_Y),
    ("INSP_TRANSFORM_RESET", ids::INSP_TRANSFORM_RESET),
    ("INSP_VISIBILITY_CHECK", ids::INSP_VISIBILITY_CHECK),
    (
        "INSP_RENDER_STRATEGY_ATLAS",
        ids::INSP_RENDER_STRATEGY_ATLAS,
    ),
    (
        "INSP_RENDER_STRATEGY_INDIVIDUAL",
        ids::INSP_RENDER_STRATEGY_INDIVIDUAL,
    ),
    (
        "INSP_RENDER_STRATEGY_HANDPACKED",
        ids::INSP_RENDER_STRATEGY_HANDPACKED,
    ),
    ("INSP_ENTITY_NAME", ids::INSP_ENTITY_NAME),
    (
        "INSP_RENDER_SOURCE_REIMPORT",
        ids::INSP_RENDER_SOURCE_REIMPORT,
    ),
    ("INSP_RENDER_FORMAT_RGBA8", ids::INSP_RENDER_FORMAT_RGBA8),
    ("INSP_RENDER_FORMAT_RGBA16", ids::INSP_RENDER_FORMAT_RGBA16),
    // Inspector sample widgets
    ("INSP_SAMPLE_TEXT", ids::INSP_SAMPLE_TEXT),
    ("INSP_SAMPLE_TEXTAREA", ids::INSP_SAMPLE_TEXTAREA),
    ("INSP_SAMPLE_COMBO", ids::INSP_SAMPLE_COMBO),
    ("INSP_SAMPLE_COMBO_OPT_A", ids::INSP_SAMPLE_COMBO_OPT_A),
    ("INSP_SAMPLE_COMBO_OPT_B", ids::INSP_SAMPLE_COMBO_OPT_B),
    ("INSP_SAMPLE_COMBO_OPT_C", ids::INSP_SAMPLE_COMBO_OPT_C),
    ("INSP_SAMPLE_NUMBER", ids::INSP_SAMPLE_NUMBER),
    ("INSP_SAMPLE_SLIDER", ids::INSP_SAMPLE_SLIDER),
    ("INSP_SAMPLE_SLIDER_CHIP", ids::INSP_SAMPLE_SLIDER_CHIP),
    ("INSP_SAMPLE_CHECKBOX", ids::INSP_SAMPLE_CHECKBOX),
    ("INSP_SAMPLE_TOGGLE", ids::INSP_SAMPLE_TOGGLE),
    ("INSP_SAMPLE_RADIO_A", ids::INSP_SAMPLE_RADIO_A),
    ("INSP_SAMPLE_RADIO_B", ids::INSP_SAMPLE_RADIO_B),
    ("INSP_SAMPLE_RADIO_C", ids::INSP_SAMPLE_RADIO_C),
    ("INSP_SAMPLE_DROPDOWN", ids::INSP_SAMPLE_DROPDOWN),
    ("INSP_SAMPLE_DD_OPT_A", ids::INSP_SAMPLE_DD_OPT_A),
    ("INSP_SAMPLE_DD_OPT_B", ids::INSP_SAMPLE_DD_OPT_B),
    ("INSP_SAMPLE_DD_OPT_C", ids::INSP_SAMPLE_DD_OPT_C),
    ("INSP_SAMPLE_TAB_A", ids::INSP_SAMPLE_TAB_A),
    ("INSP_SAMPLE_TAB_B", ids::INSP_SAMPLE_TAB_B),
    ("INSP_SAMPLE_TAB_C", ids::INSP_SAMPLE_TAB_C),
    ("INSP_SAMPLE_TREE_ROOT", ids::INSP_SAMPLE_TREE_ROOT),
    ("INSP_SAMPLE_TREE_LEAF_A", ids::INSP_SAMPLE_TREE_LEAF_A),
    ("INSP_SAMPLE_TREE_LEAF_B", ids::INSP_SAMPLE_TREE_LEAF_B),
    ("INSP_SAMPLE_V3_X", ids::INSP_SAMPLE_V3_X),
    ("INSP_SAMPLE_V3_Y", ids::INSP_SAMPLE_V3_Y),
    ("INSP_SAMPLE_V3_Z", ids::INSP_SAMPLE_V3_Z),
    ("INSP_SAMPLE_SWATCH", ids::INSP_SAMPLE_SWATCH),
    ("INSP_SAMPLE_BTN_PRIMARY", ids::INSP_SAMPLE_BTN_PRIMARY),
    ("INSP_SAMPLE_BTN_SECONDARY", ids::INSP_SAMPLE_BTN_SECONDARY),
    ("INSP_SAMPLE_BTN_DANGER", ids::INSP_SAMPLE_BTN_DANGER),
    ("INSP_SAMPLE_BTN_ICON", ids::INSP_SAMPLE_BTN_ICON),
    ("INSP_SAMPLE_LIST_ITEM", ids::INSP_SAMPLE_LIST_ITEM),
    ("INSP_SAMPLE_TAG_REMOVE", ids::INSP_SAMPLE_TAG_REMOVE),
    // Inspector section headers + color chips
    ("INSP_SECTION_INPUTS", ids::INSP_SECTION_INPUTS),
    ("INSP_SECTION_SLIDER", ids::INSP_SECTION_SLIDER),
    ("INSP_SECTION_SWITCHES", ids::INSP_SECTION_SWITCHES),
    ("INSP_SECTION_LISTS", ids::INSP_SECTION_LISTS),
    ("INSP_SECTION_VECTOR", ids::INSP_SECTION_VECTOR),
    ("INSP_SECTION_STATUS", ids::INSP_SECTION_STATUS),
    ("INSP_SECTION_COLOR", ids::INSP_SECTION_COLOR),
    ("INSP_SECTION_ACTIONS", ids::INSP_SECTION_ACTIONS),
    ("INSP_SECTION_IDENTITY", ids::INSP_SECTION_IDENTITY),
    ("INSP_SECTION_CARD", ids::INSP_SECTION_CARD),
    ("INSP_SECTION_INPUTS_COLOR", ids::INSP_SECTION_INPUTS_COLOR),
    ("INSP_SECTION_SLIDER_COLOR", ids::INSP_SECTION_SLIDER_COLOR),
    (
        "INSP_SECTION_SWITCHES_COLOR",
        ids::INSP_SECTION_SWITCHES_COLOR,
    ),
    ("INSP_SECTION_LISTS_COLOR", ids::INSP_SECTION_LISTS_COLOR),
    ("INSP_SECTION_VECTOR_COLOR", ids::INSP_SECTION_VECTOR_COLOR),
    ("INSP_SECTION_STATUS_COLOR", ids::INSP_SECTION_STATUS_COLOR),
    ("INSP_SECTION_COLOR_COLOR", ids::INSP_SECTION_COLOR_COLOR),
    (
        "INSP_SECTION_ACTIONS_COLOR",
        ids::INSP_SECTION_ACTIONS_COLOR,
    ),
    (
        "INSP_SECTION_IDENTITY_COLOR",
        ids::INSP_SECTION_IDENTITY_COLOR,
    ),
    ("INSP_SECTION_CARD_COLOR", ids::INSP_SECTION_CARD_COLOR),
    // Notes
    ("INSP_NOTE_SLOT_0", ids::INSP_NOTE_SLOT_0),
    ("INSP_NOTE_SLOT_1", ids::INSP_NOTE_SLOT_1),
    ("INSP_NOTE_SLOT_2", ids::INSP_NOTE_SLOT_2),
    ("INSP_NOTE_SLOT_3", ids::INSP_NOTE_SLOT_3),
    ("INSP_NOTE_SLOT_4", ids::INSP_NOTE_SLOT_4),
    ("INSP_NOTE_SLOT_5", ids::INSP_NOTE_SLOT_5),
    ("INSP_NOTE_SLOT_6", ids::INSP_NOTE_SLOT_6),
    ("INSP_NOTE_SLOT_7", ids::INSP_NOTE_SLOT_7),
    ("INSP_NOTE_SLOT_8", ids::INSP_NOTE_SLOT_8),
    ("INSP_NOTE_SLOT_9", ids::INSP_NOTE_SLOT_9),
    ("INSP_NOTE_SLOT_10", ids::INSP_NOTE_SLOT_10),
    ("INSP_NOTE_SLOT_11", ids::INSP_NOTE_SLOT_11),
    ("INSP_NOTE_TITLE_0", ids::INSP_NOTE_TITLE_0),
    ("INSP_NOTE_TITLE_1", ids::INSP_NOTE_TITLE_1),
    ("INSP_NOTE_TITLE_2", ids::INSP_NOTE_TITLE_2),
    ("INSP_NOTE_TITLE_3", ids::INSP_NOTE_TITLE_3),
    ("INSP_NOTE_TITLE_4", ids::INSP_NOTE_TITLE_4),
    ("INSP_NOTE_TITLE_5", ids::INSP_NOTE_TITLE_5),
    ("INSP_NOTE_TITLE_6", ids::INSP_NOTE_TITLE_6),
    ("INSP_NOTE_TITLE_7", ids::INSP_NOTE_TITLE_7),
    ("INSP_NOTE_TITLE_8", ids::INSP_NOTE_TITLE_8),
    ("INSP_NOTE_TITLE_9", ids::INSP_NOTE_TITLE_9),
    ("INSP_NOTE_TITLE_10", ids::INSP_NOTE_TITLE_10),
    ("INSP_NOTE_TITLE_11", ids::INSP_NOTE_TITLE_11),
    ("INSP_NOTE_BODY_0", ids::INSP_NOTE_BODY_0),
    ("INSP_NOTE_BODY_1", ids::INSP_NOTE_BODY_1),
    ("INSP_NOTE_BODY_2", ids::INSP_NOTE_BODY_2),
    ("INSP_NOTE_BODY_3", ids::INSP_NOTE_BODY_3),
    ("INSP_NOTE_BODY_4", ids::INSP_NOTE_BODY_4),
    ("INSP_NOTE_BODY_5", ids::INSP_NOTE_BODY_5),
    ("INSP_NOTE_BODY_6", ids::INSP_NOTE_BODY_6),
    ("INSP_NOTE_BODY_7", ids::INSP_NOTE_BODY_7),
    ("INSP_NOTE_BODY_8", ids::INSP_NOTE_BODY_8),
    ("INSP_NOTE_BODY_9", ids::INSP_NOTE_BODY_9),
    ("INSP_NOTE_BODY_10", ids::INSP_NOTE_BODY_10),
    ("INSP_NOTE_BODY_11", ids::INSP_NOTE_BODY_11),
    // Context menus
    ("CTX_MENU_CREATE_NOTE", ids::CTX_MENU_CREATE_NOTE),
    ("CTX_MENU_OUTLINE_NONE", ids::CTX_MENU_OUTLINE_NONE),
    ("CTX_MENU_OUTLINE_0", ids::CTX_MENU_OUTLINE_0),
    ("CTX_MENU_OUTLINE_1", ids::CTX_MENU_OUTLINE_1),
    ("CTX_MENU_OUTLINE_2", ids::CTX_MENU_OUTLINE_2),
    ("CTX_MENU_OUTLINE_3", ids::CTX_MENU_OUTLINE_3),
    ("CTX_MENU_OUTLINE_4", ids::CTX_MENU_OUTLINE_4),
    ("CTX_MENU_THEME_FORGE", ids::CTX_MENU_THEME_FORGE),
    ("CTX_MENU_THEME_PAINT", ids::CTX_MENU_THEME_PAINT),
    ("CTX_MENU_THEME_SUNSTONE", ids::CTX_MENU_THEME_SUNSTONE),
    ("CTX_MENU_THEME_BLUEPRINT", ids::CTX_MENU_THEME_BLUEPRINT),
    ("CTX_MENU_RADIUS_SHARP", ids::CTX_MENU_RADIUS_SHARP),
    ("CTX_MENU_RADIUS_DEFAULT", ids::CTX_MENU_RADIUS_DEFAULT),
    ("CTX_MENU_RADIUS_ROUND", ids::CTX_MENU_RADIUS_ROUND),
    ("CTX_MENU_MIRROR_UI", ids::CTX_MENU_MIRROR_UI),
    ("CTX_MENU_SHOW_STATS", ids::CTX_MENU_SHOW_STATS),
    ("CTX_MENU_SHOW_GRID", ids::CTX_MENU_SHOW_GRID),
    ("CTX_MENU_SAVE", ids::CTX_MENU_SAVE),
    ("CTX_MENU_SAVE_AS", ids::CTX_MENU_SAVE_AS),
    ("CTX_MENU_OPEN_PROJECT", ids::CTX_MENU_OPEN_PROJECT),
    ("CTX_MENU_IMPORT", ids::CTX_MENU_IMPORT),
    ("CTX_MENU_PPM_16", ids::CTX_MENU_PPM_16),
    ("CTX_MENU_PPM_32", ids::CTX_MENU_PPM_32),
    ("CTX_MENU_PPM_100", ids::CTX_MENU_PPM_100),
    ("CTX_MENU_PPM_256", ids::CTX_MENU_PPM_256),
    ("CTX_MENU_PPM_1024", ids::CTX_MENU_PPM_1024),
    ("CTX_MENU_SETTINGS_PPM", ids::CTX_MENU_SETTINGS_PPM),
    ("CTX_MENU_SETTINGS_UNIT", ids::CTX_MENU_SETTINGS_UNIT),
    ("CTX_MENU_UNIT_METERS", ids::CTX_MENU_UNIT_METERS),
    ("CTX_MENU_UNIT_PIXELS", ids::CTX_MENU_UNIT_PIXELS),
    ("CTX_MENU_SETTINGS_FILTER", ids::CTX_MENU_SETTINGS_FILTER),
    ("CTX_MENU_FILTER_PIXELART", ids::CTX_MENU_FILTER_PIXELART),
    ("CTX_MENU_FILTER_SMOOTH", ids::CTX_MENU_FILTER_SMOOTH),
    ("CTX_MENU_SETTINGS_DISPLAY", ids::CTX_MENU_SETTINGS_DISPLAY),
    ("CTX_MENU_DISPLAY_VSYNC", ids::CTX_MENU_DISPLAY_VSYNC),
    (
        "CTX_MENU_DISPLAY_IMMEDIATE",
        ids::CTX_MENU_DISPLAY_IMMEDIATE,
    ),
    ("CTX_MENU_HIER_DUPLICATE", ids::CTX_MENU_HIER_DUPLICATE),
    ("CTX_MENU_HIER_DELETE", ids::CTX_MENU_HIER_DELETE),
    (
        "CTX_MENU_HIER_RESET_TRANSFORM",
        ids::CTX_MENU_HIER_RESET_TRANSFORM,
    ),
    ("CTX_MENU_HIER_ADD_CHILD", ids::CTX_MENU_HIER_ADD_CHILD),
    ("CTX_MENU_HIER_RENAME", ids::CTX_MENU_HIER_RENAME),
    // Project chip scene list
    ("CTX_SCENE_SEARCH", ids::CTX_SCENE_SEARCH),
    ("CTX_SCENE_ROW_0", ids::CTX_SCENE_ROW_0),
    ("CTX_SCENE_ROW_1", ids::CTX_SCENE_ROW_1),
    ("CTX_SCENE_ROW_2", ids::CTX_SCENE_ROW_2),
    ("CTX_SCENE_ROW_3", ids::CTX_SCENE_ROW_3),
    ("CTX_SCENE_ROW_4", ids::CTX_SCENE_ROW_4),
    ("CTX_SCENE_ROW_5", ids::CTX_SCENE_ROW_5),
    ("CTX_SCENE_ROW_6", ids::CTX_SCENE_ROW_6),
    ("CTX_SCENE_ROW_7", ids::CTX_SCENE_ROW_7),
    // BlenderColorPicker
    ("INSP_BLENDER_PICKER", ids::INSP_BLENDER_PICKER),
    ("BLENDER_WHEEL", ids::BLENDER_WHEEL),
    ("BLENDER_VALUE_SLIDER", ids::BLENDER_VALUE_SLIDER),
    ("BLENDER_CHANNEL_0", ids::BLENDER_CHANNEL_0),
    ("BLENDER_CHANNEL_1", ids::BLENDER_CHANNEL_1),
    ("BLENDER_CHANNEL_2", ids::BLENDER_CHANNEL_2),
    ("BLENDER_CHANNEL_3", ids::BLENDER_CHANNEL_3),
    ("BLENDER_HEX", ids::BLENDER_HEX),
    ("BLENDER_INTERP_LINEAR", ids::BLENDER_INTERP_LINEAR),
    ("BLENDER_INTERP_PERCEPTUAL", ids::BLENDER_INTERP_PERCEPTUAL),
    ("BLENDER_CHANNEL_RGB", ids::BLENDER_CHANNEL_RGB),
    ("BLENDER_CHANNEL_HSV", ids::BLENDER_CHANNEL_HSV),
    ("BLENDER_NUM_0", ids::BLENDER_NUM_0),
    ("BLENDER_NUM_1", ids::BLENDER_NUM_1),
    ("BLENDER_NUM_2", ids::BLENDER_NUM_2),
    ("BLENDER_NUM_3", ids::BLENDER_NUM_3),
    ("BLENDER_ADD_SWATCH", ids::BLENDER_ADD_SWATCH),
    ("BLENDER_EYEDROPPER", ids::BLENDER_EYEDROPPER),
    ("BLENDER_DRAG_HANDLE", ids::BLENDER_DRAG_HANDLE),
    ("BLENDER_SWATCH_0", ids::BLENDER_SWATCH_0),
    ("BLENDER_SWATCH_1", ids::BLENDER_SWATCH_1),
    ("BLENDER_SWATCH_2", ids::BLENDER_SWATCH_2),
    ("BLENDER_SWATCH_3", ids::BLENDER_SWATCH_3),
    ("BLENDER_SWATCH_4", ids::BLENDER_SWATCH_4),
    ("BLENDER_SWATCH_5", ids::BLENDER_SWATCH_5),
    ("BLENDER_SWATCH_6", ids::BLENDER_SWATCH_6),
    ("BLENDER_SWATCH_7", ids::BLENDER_SWATCH_7),
    ("BLENDER_SWATCH_8", ids::BLENDER_SWATCH_8),
    ("BLENDER_SWATCH_9", ids::BLENDER_SWATCH_9),
    ("BLENDER_SWATCH_10", ids::BLENDER_SWATCH_10),
    ("BLENDER_SWATCH_11", ids::BLENDER_SWATCH_11),
    ("BLENDER_SWATCH_12", ids::BLENDER_SWATCH_12),
    ("BLENDER_SWATCH_13", ids::BLENDER_SWATCH_13),
    ("BLENDER_SWATCH_14", ids::BLENDER_SWATCH_14),
    ("BLENDER_SWATCH_15", ids::BLENDER_SWATCH_15),
    ("BLENDER_SWATCH_16", ids::BLENDER_SWATCH_16),
    ("BLENDER_SWATCH_17", ids::BLENDER_SWATCH_17),
    ("BLENDER_SWATCH_18", ids::BLENDER_SWATCH_18),
    ("BLENDER_SWATCH_19", ids::BLENDER_SWATCH_19),
    ("BLENDER_SWATCH_20", ids::BLENDER_SWATCH_20),
    ("BLENDER_SWATCH_21", ids::BLENDER_SWATCH_21),
    ("BLENDER_SWATCH_22", ids::BLENDER_SWATCH_22),
    ("BLENDER_SWATCH_23", ids::BLENDER_SWATCH_23),
    ("BLENDER_SWATCH_24", ids::BLENDER_SWATCH_24),
    ("BLENDER_SWATCH_25", ids::BLENDER_SWATCH_25),
    ("BLENDER_SWATCH_26", ids::BLENDER_SWATCH_26),
    // Widget Gallery
    ("GAL_PANEL", ids::GAL_PANEL),
    ("GAL_DRAG_HANDLE", ids::GAL_DRAG_HANDLE),
    ("GAL_RESIZE_HANDLE", ids::GAL_RESIZE_HANDLE),
    ("GAL_CLOSE", ids::GAL_CLOSE),
    // Hierarchy chrome (panel-level, not row-level — row ids kept numeric)
    ("HIER_DRAG_HANDLE", ids::HIER_DRAG_HANDLE),
    ("HIER_RESIZE_HANDLE", ids::HIER_RESIZE_HANDLE),
    ("HIER_PANEL", ids::HIER_PANEL),
    ("HIER_SEARCH", ids::HIER_SEARCH),
    ("HIER_RENAME_INPUT", ids::HIER_RENAME_INPUT),
    // Bg Removal — eyedropper + protection brush + extra-colour swatch pool.
    ("BGR_EYEDROPPER", ids::BGR_EYEDROPPER),
    ("BGR_PROTECT", ids::BGR_PROTECT),
    ("BGR_PROTECT_CLEAR", ids::BGR_PROTECT_CLEAR),
    ("BGR_SHOW_MASK", ids::BGR_SHOW_MASK),
    ("BGR_BRUSH_SIZE", ids::BGR_BRUSH_SIZE),
    ("BGR_BRUSH_SIZE_NUM", ids::BGR_BRUSH_SIZE_NUM),
    ("BGR_FALLOFF_SMOOTH", ids::BGR_FALLOFF_SMOOTH),
    ("BGR_FALLOFF_SPHERE", ids::BGR_FALLOFF_SPHERE),
    ("BGR_FALLOFF_SHARP", ids::BGR_FALLOFF_SHARP),
    ("BGR_FALLOFF_CONSTANT", ids::BGR_FALLOFF_CONSTANT),
    ("BGR_SWATCH_0", ids::BGR_SWATCH_0),
    ("BGR_SWATCH_1", ids::BGR_SWATCH_1),
    ("BGR_SWATCH_2", ids::BGR_SWATCH_2),
    ("BGR_SWATCH_3", ids::BGR_SWATCH_3),
    ("BGR_SWATCH_4", ids::BGR_SWATCH_4),
    ("BGR_SWATCH_5", ids::BGR_SWATCH_5),
    ("BGR_SWATCH_6", ids::BGR_SWATCH_6),
    ("BGR_SWATCH_7", ids::BGR_SWATCH_7),
    ("BGR_SWATCH_8", ids::BGR_SWATCH_8),
    ("BGR_SWATCH_9", ids::BGR_SWATCH_9),
    ("BGR_SWATCH_10", ids::BGR_SWATCH_10),
    ("BGR_SWATCH_11", ids::BGR_SWATCH_11),
    // Painter chrome (W3 audit-2 B.3 — were absent from the uniqueness set).
    ("PAINTER_SIDEBAR_PANEL", ids::PAINTER_SIDEBAR_PANEL),
    (
        "PAINTER_SIDEBAR_SIZE_SLIDER",
        ids::PAINTER_SIDEBAR_SIZE_SLIDER,
    ),
    ("PAINTER_SIDEBAR_SIZE_CHIP", ids::PAINTER_SIDEBAR_SIZE_CHIP),
    (
        "PAINTER_SIDEBAR_OPACITY_SLIDER",
        ids::PAINTER_SIDEBAR_OPACITY_SLIDER,
    ),
    (
        "PAINTER_SIDEBAR_OPACITY_CHIP",
        ids::PAINTER_SIDEBAR_OPACITY_CHIP,
    ),
    (
        "PAINTER_SIDEBAR_UNDO_BUTTON",
        ids::PAINTER_SIDEBAR_UNDO_BUTTON,
    ),
    (
        "PAINTER_SIDEBAR_REDO_BUTTON",
        ids::PAINTER_SIDEBAR_REDO_BUTTON,
    ),
    (
        "PAINTER_SIDEBAR_MODIFIER_SQUARE",
        ids::PAINTER_SIDEBAR_MODIFIER_SQUARE,
    ),
    ("PAINTER_SIDEBAR_CLOSE", ids::PAINTER_SIDEBAR_CLOSE),
    (
        "PAINTER_SIDEBAR_TOGGLE_DOCK",
        ids::PAINTER_SIDEBAR_TOGGLE_DOCK,
    ),
    ("PAINTER_COLOR_THUMB", ids::PAINTER_COLOR_THUMB),
    ("PAINTER_APPLY", ids::PAINTER_APPLY),
    ("PAINTER_LAYERS_PANEL", ids::PAINTER_LAYERS_PANEL),
    ("PAINTER_LAYERS_CLOSE", ids::PAINTER_LAYERS_CLOSE),
    ("PAINTER_LAYERS_ADD", ids::PAINTER_LAYERS_ADD),
    (
        "PAINTER_LAYERS_TOGGLE_DOCK",
        ids::PAINTER_LAYERS_TOGGLE_DOCK,
    ),
    // Vector Inspector (W2 T2.4 + §4.2). The PANEL/CLOSE/FILL_SWATCH trio
    // pre-dates this row but was never enrolled in the uniqueness check —
    // closed here alongside the new Shape-kind picker option ids.
    ("VECTOR_INSPECTOR_PANEL", ids::VECTOR_INSPECTOR_PANEL),
    ("VECTOR_INSPECTOR_CLOSE", ids::VECTOR_INSPECTOR_CLOSE),
    (
        "VECTOR_INSPECTOR_FILL_SWATCH",
        ids::VECTOR_INSPECTOR_FILL_SWATCH,
    ),
    (
        "VECTOR_INSPECTOR_SHAPE_KIND",
        ids::VECTOR_INSPECTOR_SHAPE_KIND,
    ),
    (
        "VECTOR_INSPECTOR_SHAPE_RECT",
        ids::VECTOR_INSPECTOR_SHAPE_RECT,
    ),
    (
        "VECTOR_INSPECTOR_SHAPE_ELLIPSE",
        ids::VECTOR_INSPECTOR_SHAPE_ELLIPSE,
    ),
    (
        "VECTOR_INSPECTOR_SHAPE_POLYGON",
        ids::VECTOR_INSPECTOR_SHAPE_POLYGON,
    ),
    (
        "VECTOR_INSPECTOR_SHAPE_STAR",
        ids::VECTOR_INSPECTOR_SHAPE_STAR,
    ),
    (
        "VECTOR_INSPECTOR_SHAPE_SPIRAL",
        ids::VECTOR_INSPECTOR_SHAPE_SPIRAL,
    ),
    // Vector tool Style panel (ADR-0108 docked `ph2d-panel-vector`).
    ("VECTOR_PANEL", ids::VECTOR_PANEL),
    ("VECTOR_CLOSE", ids::VECTOR_CLOSE),
    ("VECTOR_WIDTH", ids::VECTOR_WIDTH),
    ("VECTOR_WIDTH_NUM", ids::VECTOR_WIDTH_NUM),
    ("VECTOR_STROKE_SWATCH", ids::VECTOR_STROKE_SWATCH),
    ("VECTOR_FILL_SWATCH", ids::VECTOR_FILL_SWATCH),
    ("VECTOR_FILL_KIND_SOLID", ids::VECTOR_FILL_KIND_SOLID),
    ("VECTOR_FILL_KIND_LINEAR", ids::VECTOR_FILL_KIND_LINEAR),
    ("VECTOR_FILL_KIND_RADIAL", ids::VECTOR_FILL_KIND_RADIAL),
    ("VECTOR_FILL_KIND_MULTI", ids::VECTOR_FILL_KIND_MULTI),
    ("VECTOR_GRAD_ANGLE", ids::VECTOR_GRAD_ANGLE),
    ("VECTOR_GRAD_ANGLE_NUM", ids::VECTOR_GRAD_ANGLE_NUM),
    ("VECTOR_GRAD_ADD_POINT", ids::VECTOR_GRAD_ADD_POINT),
    ("VECTOR_GRAD_REMOVE_POINT", ids::VECTOR_GRAD_REMOVE_POINT),
    ("VECTOR_GRAD_INFLUENCE", ids::VECTOR_GRAD_INFLUENCE),
    ("VECTOR_GRAD_INFLUENCE_NUM", ids::VECTOR_GRAD_INFLUENCE_NUM),
    ("VECTOR_GRAD_JITTER", ids::VECTOR_GRAD_JITTER),
    ("VECTOR_GRAD_JITTER_NUM", ids::VECTOR_GRAD_JITTER_NUM),
    ("VECTOR_GRAD_ADD_STOP", ids::VECTOR_GRAD_ADD_STOP),
    ("VECTOR_GRAD_REMOVE_STOP", ids::VECTOR_GRAD_REMOVE_STOP),
    ("VECTOR_ALIGN_LEFT", ids::VECTOR_ALIGN_LEFT),
    ("VECTOR_ALIGN_HCENTER", ids::VECTOR_ALIGN_HCENTER),
    ("VECTOR_ALIGN_RIGHT", ids::VECTOR_ALIGN_RIGHT),
    ("VECTOR_ALIGN_TOP", ids::VECTOR_ALIGN_TOP),
    ("VECTOR_ALIGN_VCENTER", ids::VECTOR_ALIGN_VCENTER),
    ("VECTOR_ALIGN_BOTTOM", ids::VECTOR_ALIGN_BOTTOM),
    ("VECTOR_DISTRIBUTE_H", ids::VECTOR_DISTRIBUTE_H),
    ("VECTOR_DISTRIBUTE_V", ids::VECTOR_DISTRIBUTE_V),
    ("VECTOR_PIVOT_EDIT", ids::VECTOR_PIVOT_EDIT),
    ("VECTOR_STROKE_OPACITY", ids::VECTOR_STROKE_OPACITY),
    ("VECTOR_STROKE_OPACITY_NUM", ids::VECTOR_STROKE_OPACITY_NUM),
    ("VECTOR_FILL_OPACITY", ids::VECTOR_FILL_OPACITY),
    ("VECTOR_FILL_OPACITY_NUM", ids::VECTOR_FILL_OPACITY_NUM),
    ("VECTOR_CAP_BUTT", ids::VECTOR_CAP_BUTT),
    ("VECTOR_CAP_ROUND", ids::VECTOR_CAP_ROUND),
    ("VECTOR_CAP_SQUARE", ids::VECTOR_CAP_SQUARE),
    ("VECTOR_JOIN_MITER", ids::VECTOR_JOIN_MITER),
    ("VECTOR_JOIN_ROUND", ids::VECTOR_JOIN_ROUND),
    ("VECTOR_JOIN_BEVEL", ids::VECTOR_JOIN_BEVEL),
    ("VECTOR_DASH", ids::VECTOR_DASH),
    ("VECTOR_DASH_NUM", ids::VECTOR_DASH_NUM),
    ("VECTOR_GAP", ids::VECTOR_GAP),
    ("VECTOR_GAP_NUM", ids::VECTOR_GAP_NUM),
    // Stroke markers (arrowheads): the two dropdown chips. Their popover OPTIONS
    // are runtime ids (`vector_marker_option_id`), like the shape catalogue's —
    // outside this const table by construction.
    ("VECTOR_MARKER_START_DD", ids::VECTOR_MARKER_START_DD),
    ("VECTOR_MARKER_END_DD", ids::VECTOR_MARKER_END_DD),
    ("VECTOR_MODE_SELECT", ids::VECTOR_MODE_SELECT),
    ("VECTOR_MODE_NODE", ids::VECTOR_MODE_NODE),
    ("VECTOR_MODE_PEN", ids::VECTOR_MODE_PEN),
    ("VECTOR_TEXT_SIZE", ids::VECTOR_TEXT_SIZE),
    ("VECTOR_TEXT_SIZE_NUM", ids::VECTOR_TEXT_SIZE_NUM),
    ("VECTOR_TEXT_WEIGHT", ids::VECTOR_TEXT_WEIGHT),
    ("VECTOR_TEXT_WEIGHT_NUM", ids::VECTOR_TEXT_WEIGHT_NUM),
    ("VECTOR_TEXT_FONT_PREV", ids::VECTOR_TEXT_FONT_PREV),
    ("VECTOR_TEXT_FONT_NEXT", ids::VECTOR_TEXT_FONT_NEXT),
    ("VECTOR_TEXT_FONT_IMPORT", ids::VECTOR_TEXT_FONT_IMPORT),
    ("VECTOR_TEXT_FONT_DD", ids::VECTOR_TEXT_FONT_DD),
    ("VECTOR_TEXT_ALIGN_LEFT", ids::VECTOR_TEXT_ALIGN_LEFT),
    ("VECTOR_TEXT_ALIGN_CENTER", ids::VECTOR_TEXT_ALIGN_CENTER),
    ("VECTOR_TEXT_ALIGN_RIGHT", ids::VECTOR_TEXT_ALIGN_RIGHT),
    ("VECTOR_TEXT_LINE_HEIGHT", ids::VECTOR_TEXT_LINE_HEIGHT),
    (
        "VECTOR_TEXT_LINE_HEIGHT_NUM",
        ids::VECTOR_TEXT_LINE_HEIGHT_NUM,
    ),
    ("VECTOR_TEXT_TRACKING", ids::VECTOR_TEXT_TRACKING),
    ("VECTOR_TEXT_TRACKING_NUM", ids::VECTOR_TEXT_TRACKING_NUM),
    ("VECTOR_CONVERT_TO_CURVES", ids::VECTOR_CONVERT_TO_CURVES),
    ("VECTOR_BOOL_UNION", ids::VECTOR_BOOL_UNION),
    ("VECTOR_SECTION_BLEND", ids::VECTOR_SECTION_BLEND),
    ("VECTOR_BLEND_RUN", ids::VECTOR_BLEND_RUN),
    ("VECTOR_MORPH_RUN", ids::VECTOR_MORPH_RUN),
    ("VECTOR_MORPH_T", ids::VECTOR_MORPH_T),
    ("VECTOR_MORPH_T_NUM", ids::VECTOR_MORPH_T_NUM),
    ("VECTOR_BLEND_STEPS", ids::VECTOR_BLEND_STEPS),
    ("VECTOR_BLEND_STEPS_NUM", ids::VECTOR_BLEND_STEPS_NUM),
    ("VECTOR_BLEND_STACK_UP", ids::VECTOR_BLEND_STACK_UP),
    ("VECTOR_BLEND_RESET_SPINE", ids::VECTOR_BLEND_RESET_SPINE),
    ("VECTOR_BLEND_EXPAND", ids::VECTOR_BLEND_EXPAND),
    ("VECTOR_BLEND_RELEASE", ids::VECTOR_BLEND_RELEASE),
    ("VECTOR_MODE_PICKBLEND", ids::VECTOR_MODE_PICKBLEND),
    ("VECTOR_BOOL_SUBTRACT", ids::VECTOR_BOOL_SUBTRACT),
    ("VECTOR_BOOL_INTERSECT", ids::VECTOR_BOOL_INTERSECT),
    ("VECTOR_BOOL_EXCLUDE", ids::VECTOR_BOOL_EXCLUDE),
    ("VECTOR_VERT_CORNER", ids::VECTOR_VERT_CORNER),
    ("VECTOR_VERT_SMOOTH", ids::VECTOR_VERT_SMOOTH),
    ("VECTOR_VERT_SYMMETRIC", ids::VECTOR_VERT_SYMMETRIC),
    ("VECTOR_VERT_DELETE", ids::VECTOR_VERT_DELETE),
    ("VECTOR_ARRANGE_DUPLICATE", ids::VECTOR_ARRANGE_DUPLICATE),
    ("VECTOR_ARRANGE_TO_BACK", ids::VECTOR_ARRANGE_TO_BACK),
    ("VECTOR_ARRANGE_BACKWARD", ids::VECTOR_ARRANGE_BACKWARD),
    ("VECTOR_ARRANGE_FORWARD", ids::VECTOR_ARRANGE_FORWARD),
    ("VECTOR_ARRANGE_TO_FRONT", ids::VECTOR_ARRANGE_TO_FRONT),
    ("VECTOR_ARRANGE_FLIP_H", ids::VECTOR_ARRANGE_FLIP_H),
    ("VECTOR_ARRANGE_FLIP_V", ids::VECTOR_ARRANGE_FLIP_V),
    ("VECTOR_ARRANGE_ROTATE_CW", ids::VECTOR_ARRANGE_ROTATE_CW),
    ("VECTOR_ARRANGE_ROTATE_CCW", ids::VECTOR_ARRANGE_ROTATE_CCW),
    ("VECTOR_TRANSFORM_X", ids::VECTOR_TRANSFORM_X),
    ("VECTOR_TRANSFORM_Y", ids::VECTOR_TRANSFORM_Y),
    ("VECTOR_TRANSFORM_W", ids::VECTOR_TRANSFORM_W),
    ("VECTOR_TRANSFORM_H", ids::VECTOR_TRANSFORM_H),
    ("VECTOR_TRANSFORM_R", ids::VECTOR_TRANSFORM_R),
    ("VECTOR_PATH_SMOOTH", ids::VECTOR_PATH_SMOOTH),
    ("VECTOR_PATH_SHARPEN", ids::VECTOR_PATH_SHARPEN),
    ("VECTOR_PATH_SIMPLIFY", ids::VECTOR_PATH_SIMPLIFY),
    ("VECTOR_PATH_SUBDIVIDE", ids::VECTOR_PATH_SUBDIVIDE),
    ("VECTOR_PATH_CLOSE", ids::VECTOR_PATH_CLOSE),
    ("VECTOR_COMPOUND_MAKE", ids::VECTOR_COMPOUND_MAKE),
    ("VECTOR_COMPOUND_RELEASE", ids::VECTOR_COMPOUND_RELEASE),
    ("VECTOR_FILL_RULE_NONZERO", ids::VECTOR_FILL_RULE_NONZERO),
    ("VECTOR_FILL_RULE_EVENODD", ids::VECTOR_FILL_RULE_EVENODD),
    ("VECTOR_SNAP_OFF", ids::VECTOR_SNAP_OFF),
    ("VECTOR_SNAP_ON", ids::VECTOR_SNAP_ON),
    // Vector panel UI rework: the 5th mode pill, the shape-category dropdown chip
    // and the 17 collapsible section headers.
    ("VECTOR_MODE_SHAPE", ids::VECTOR_MODE_SHAPE),
    // O 6º pill: o CONECTOR (a linha que gruda em duas formas e as segue).
    ("VECTOR_MODE_CONNECT", ids::VECTOR_MODE_CONNECT),
    ("VECTOR_SHAPE_GROUP_DD", ids::VECTOR_SHAPE_GROUP_DD),
    ("VECTOR_SECTION_TOOL", ids::VECTOR_SECTION_TOOL),
    ("VECTOR_SECTION_SHAPE", ids::VECTOR_SECTION_SHAPE),
    (
        "VECTOR_SECTION_SHAPE_PARAMS",
        ids::VECTOR_SECTION_SHAPE_PARAMS,
    ),
    ("VECTOR_SECTION_STROKE", ids::VECTOR_SECTION_STROKE),
    ("VECTOR_SECTION_FILL", ids::VECTOR_SECTION_FILL),
    ("VECTOR_SECTION_FILL_TYPE", ids::VECTOR_SECTION_FILL_TYPE),
    ("VECTOR_SECTION_SNAP", ids::VECTOR_SECTION_SNAP),
    ("VECTOR_SECTION_TRANSFORM", ids::VECTOR_SECTION_TRANSFORM),
    ("VECTOR_SECTION_VERTEX", ids::VECTOR_SECTION_VERTEX),
    ("VECTOR_SECTION_BOOLEAN", ids::VECTOR_SECTION_BOOLEAN),
    ("VECTOR_SECTION_ALIGN", ids::VECTOR_SECTION_ALIGN),
    ("VECTOR_SECTION_ARRANGE", ids::VECTOR_SECTION_ARRANGE),
    ("VECTOR_SECTION_PATH", ids::VECTOR_SECTION_PATH),
    ("VECTOR_SECTION_TEXT", ids::VECTOR_SECTION_TEXT),
    ("VECTOR_SECTION_FONT", ids::VECTOR_SECTION_FONT),
    ("VECTOR_SECTION_PARAGRAPH", ids::VECTOR_SECTION_PARAGRAPH),
    ("VECTOR_SECTION_AXES", ids::VECTOR_SECTION_AXES),
    // O CONECTOR: a seção + os três campos da relação (Route / Jetty / Spread).
    ("VECTOR_SECTION_CONNECTOR", ids::VECTOR_SECTION_CONNECTOR),
    ("VECTOR_CONNECTOR_ROUTE", ids::VECTOR_CONNECTOR_ROUTE),
    ("VECTOR_CONNECTOR_JETTY", ids::VECTOR_CONNECTOR_JETTY),
    ("VECTOR_CONNECTOR_SPREAD", ids::VECTOR_CONNECTOR_SPREAD),
    // Flip tool Style panel (ADR-0114 W2 docked `ph2d-panel-flip`).
    ("FLIP_PANEL", ids::FLIP_PANEL),
    ("FLIP_CLOSE", ids::FLIP_CLOSE),
    ("FLIP_MODE_SELECT", ids::FLIP_MODE_SELECT),
    ("FLIP_MODE_DRAW", ids::FLIP_MODE_DRAW),
    ("FLIP_MODE_ERASE", ids::FLIP_MODE_ERASE),
    ("FLIP_SIZE", ids::FLIP_SIZE),
    ("FLIP_SIZE_NUM", ids::FLIP_SIZE_NUM),
    ("FLIP_HARDNESS", ids::FLIP_HARDNESS),
    ("FLIP_HARDNESS_NUM", ids::FLIP_HARDNESS_NUM),
    ("FLIP_OPACITY", ids::FLIP_OPACITY),
    ("FLIP_OPACITY_NUM", ids::FLIP_OPACITY_NUM),
    ("FLIP_SMOOTHING", ids::FLIP_SMOOTHING),
    ("FLIP_SMOOTHING_NUM", ids::FLIP_SMOOTHING_NUM),
    ("FLIP_STROKE_SWATCH", ids::FLIP_STROKE_SWATCH),
    ("FLIP_ERASE_SOFT", ids::FLIP_ERASE_SOFT),
    ("FLIP_ERASE_HARD", ids::FLIP_ERASE_HARD),
    ("FLIP_ERASE_STROKE", ids::FLIP_ERASE_STROKE),
    ("FLIP_MODE_FILL", ids::FLIP_MODE_FILL),
    ("FLIP_MODE_EDIT", ids::FLIP_MODE_EDIT),
    ("FLIP_FALLOFF", ids::FLIP_FALLOFF),
    ("FLIP_EDIT_DELETE", ids::FLIP_EDIT_DELETE),
    ("FLIP_EDIT_DESELECT", ids::FLIP_EDIT_DESELECT),
    ("FLIP_EDIT_SELECT_ALL", ids::FLIP_EDIT_SELECT_ALL),
    ("FLIP_FILL_SWATCH", ids::FLIP_FILL_SWATCH),
    ("FLIP_FILL_PAINT", ids::FLIP_FILL_PAINT),
    ("FLIP_FILL_BEHIND", ids::FLIP_FILL_BEHIND),
    ("FLIP_FILL_UNPAINT", ids::FLIP_FILL_UNPAINT),
    ("FLIP_GAP", ids::FLIP_GAP),
    ("FLIP_GAP_NUM", ids::FLIP_GAP_NUM),
    ("FLIP_GROW", ids::FLIP_GROW),
    ("FLIP_GROW_NUM", ids::FLIP_GROW_NUM),
    ("FLIP_PRECISION", ids::FLIP_PRECISION),
    ("FLIP_PRECISION_NUM", ids::FLIP_PRECISION_NUM),
    ("FLIP_LAYER_ADD", ids::FLIP_LAYER_ADD),
    ("FLIP_LAYER_DELETE", ids::FLIP_LAYER_DELETE),
    // Frame strip (W3) — the bottom-docked cells + transport + tween.
    ("FLIP_STRIP_PANEL", ids::FLIP_STRIP_PANEL),
    ("FLIP_STRIP_CLOSE", ids::FLIP_STRIP_CLOSE),
    ("FLIP_SCRUB", ids::FLIP_SCRUB),
    ("FLIP_PLAY", ids::FLIP_PLAY),
    ("FLIP_PREV_DRAWING", ids::FLIP_PREV_DRAWING),
    ("FLIP_NEXT_DRAWING", ids::FLIP_NEXT_DRAWING),
    ("FLIP_FPS_NUM", ids::FLIP_FPS_NUM),
    ("FLIP_GHOST", ids::FLIP_GHOST),
    ("FLIP_GHOST_BEFORE_NUM", ids::FLIP_GHOST_BEFORE_NUM),
    ("FLIP_GHOST_AFTER_NUM", ids::FLIP_GHOST_AFTER_NUM),
    ("FLIP_AUTOKEY", ids::FLIP_AUTOKEY),
    ("FLIP_ADDITIVE", ids::FLIP_ADDITIVE),
    ("FLIP_KEY_ADD", ids::FLIP_KEY_ADD),
    ("FLIP_KEY_DUP", ids::FLIP_KEY_DUP),
    ("FLIP_KEY_INSTANCE", ids::FLIP_KEY_INSTANCE),
    ("FLIP_KEY_UNLINK", ids::FLIP_KEY_UNLINK),
    ("FLIP_KEY_DELETE", ids::FLIP_KEY_DELETE),
    ("FLIP_HOLD_NUM", ids::FLIP_HOLD_NUM),
    ("FLIP_KEY_LEFT", ids::FLIP_KEY_LEFT),
    ("FLIP_KEY_RIGHT", ids::FLIP_KEY_RIGHT),
    ("FLIP_TWEEN_NUM", ids::FLIP_TWEEN_NUM),
    ("FLIP_TWEEN_ADD", ids::FLIP_TWEEN_ADD),
    ("FLIP_CYCLE_DD", ids::FLIP_CYCLE_DD),
];

/// Pairwise uniqueness across every chrome [`NodeId`]. O(n²) over ~200
/// entries = trivial in test time.
#[test]
fn chrome_node_ids_are_pairwise_unique() {
    // Sort by id so any collision lands in adjacent pairs.
    let mut sorted: Vec<(NodeId, &'static str)> =
        CHROME_IDS.iter().map(|(n, id)| (*id, *n)).collect();
    // The timeline segment menu publishes its rows as tables rather than as
    // hand-listed consts, so pull them in from the tables themselves: a row added
    // there can never slip past this gate the way a forgotten CHROME_IDS entry
    // would (the label doubles as the collision report's name).
    sorted.extend(
        ids::TIMELINE_SEGMENT_MENU
            .iter()
            .chain(ids::TIMELINE_EASE_MENU.iter())
            .chain(ids::TIMELINE_TRACK_MENU.iter())
            .chain(ids::TIMELINE_STRIP_MENU.iter())
            .chain(ids::TIMELINE_LANE_MENU.iter())
            .map(|(id, label, _)| (*id, *label)),
    );
    sorted.sort_by_key(|(id, _)| id.0);

    for w in sorted.windows(2) {
        let (a_id, a_name) = w[0];
        let (b_id, b_name) = w[1];
        assert_ne!(
            a_id.0, b_id.0,
            "NodeId collision: `{}` and `{}` both hash to {:#018x}. \
             Rename one slug in `screens/hero/ids.rs` to disambiguate.",
            a_name, b_name, a_id.0
        );
    }
}

/// Reserved a11y root NodeId must not be shadowed by any chrome const.
/// `hash_node_id` defends against this with a `== 0 → 1` fixup, but a
/// hand-allocated const could still hit it.
#[test]
fn no_chrome_id_is_root() {
    for (name, id) in CHROME_IDS {
        assert_ne!(
            id.0,
            ph2d_a11y::NodeId::ROOT.0,
            "Chrome const `{}` collides with a11y NodeId::ROOT — must be > 0.",
            name
        );
    }
}

/// Eye/expand companion-bit fixture: no chrome id may be mistaken for
/// a row companion. Companion-detection in `ids.rs` requires BOTH the
/// high bit (61 or 62) set AND the un-masked low portion to fall in
/// the row-id range (< 2^32). A hashed chrome id will set the high
/// bits ~50% of the time, but the un-masked 62-bit residue is
/// uniformly random over a 4-billion-x-larger space, so the
/// probability that any one chrome id is misdetected is < 2^-30 per
/// bit. This test asserts the property holds for the actual set of
/// chrome consts shipped — if a future slug change pushes one into
/// the danger zone, this fails loudly at `cargo test` time.
#[test]
fn no_chrome_id_is_companion_misread() {
    for (name, id) in CHROME_IDS {
        assert!(
            ids::hier_eye_companion_to_row(*id).is_none(),
            "Chrome const `{}` (id {:#018x}) is misdetected as an eye-toggle row companion.",
            name,
            id.0
        );
        assert!(
            ids::hier_expand_companion_to_row(*id).is_none(),
            "Chrome const `{}` (id {:#018x}) is misdetected as an expand-toggle row companion.",
            name,
            id.0
        );
    }
}

/// W3 audit-2 B.3: the DYNAMIC painter row/blend ids (derived at runtime via
/// `fnv_node_id_runtime`) must collide neither with any fixed chrome const nor
/// with each other. A slug-scheme change that aliased a dynamic id onto a chrome
/// id would misroute a production click — the exact failure this file guards,
/// extended to the per-row painter id space.
#[test]
fn painter_dynamic_ids_dont_collide_with_chrome_or_each_other() {
    use ids::PainterLayerWidget::{Blend, MoveDown, MoveUp, Opacity, OpacityChip, Row, Visibility};

    let chrome: std::collections::BTreeSet<u64> = CHROME_IDS.iter().map(|(_, id)| id.0).collect();
    let mut seen: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();

    let kinds = [
        Row,
        Visibility,
        Opacity,
        OpacityChip,
        Blend,
        MoveUp,
        MoveDown,
    ];
    // Dense small ids + sparse/large runtime ids (LayerId is a u64 monotonic).
    let layer_ids = [0u64, 1, 2, 3, 7, 42, 255, 1000, 0x_dead_beef, u64::MAX];

    for &lid in &layer_ids {
        for &kind in &kinds {
            let id = ids::painter_layer_widget_id(lid, kind).0;
            assert!(
                !chrome.contains(&id),
                "painter_layer_widget_id({lid}, {kind:?}) (id {id:#018x}) collides with a chrome const",
            );
            assert!(
                seen.insert(id),
                "painter_layer_widget_id({lid}, {kind:?}) (id {id:#018x}) collides with another dynamic id",
            );
        }
        // 22 W3C blend modes today; sample a margin past that.
        for mode in 0u8..28 {
            let id = ids::painter_layer_blend_option_id(lid, mode).0;
            assert!(
                !chrome.contains(&id),
                "painter_layer_blend_option_id({lid}, {mode}) (id {id:#018x}) collides with a chrome const",
            );
            assert!(
                seen.insert(id),
                "painter_layer_blend_option_id({lid}, {mode}) (id {id:#018x}) collides with another dynamic id",
            );
        }
    }
}

/// The Flip layers panel's per-row ids (ADR-0114 W2, from `flip_layer_widget_id`)
/// must collide neither with any fixed chrome const nor with each other — same
/// guard the painter dynamic ids get, extended to the Flip per-row id space.
#[test]
fn flip_dynamic_ids_dont_collide_with_chrome_or_each_other() {
    use ids::FlipLayerWidget::{Blend, Lock, MoveDown, MoveUp, Opacity, Row, Visibility};

    let chrome: std::collections::BTreeSet<u64> = CHROME_IDS.iter().map(|(_, id)| id.0).collect();
    let mut seen: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();

    let kinds = [Row, Visibility, Lock, Opacity, Blend, MoveUp, MoveDown];
    let layer_ids = [0u64, 1, 2, 3, 7, 42, 255, 1000, 0x_dead_beef, u64::MAX];

    for &lid in &layer_ids {
        for &kind in &kinds {
            let id = ids::flip_layer_widget_id(lid, kind).0;
            assert!(
                !chrome.contains(&id),
                "flip_layer_widget_id({lid}, {kind:?}) (id {id:#018x}) collides with a chrome const",
            );
            assert!(
                seen.insert(id),
                "flip_layer_widget_id({lid}, {kind:?}) (id {id:#018x}) collides with another dynamic id",
            );
        }
        for mode in 0u8..28 {
            let id = ids::flip_layer_blend_option_id(lid, mode).0;
            assert!(
                !chrome.contains(&id),
                "flip_layer_blend_option_id({lid}, {mode}) (id {id:#018x}) collides with a chrome const",
            );
            assert!(
                seen.insert(id),
                "flip_layer_blend_option_id({lid}, {mode}) (id {id:#018x}) collides with another dynamic id",
            );
        }
    }
}

/// The timeline's six dynamic id families (key diamonds, row twirls,
/// graph-height grips, curve anchors, bézier handles, Summary columns) share one FNV-1a body,
/// differing only by their domain seed. Prove that a twirl for target `T` never
/// lands on a key hit for target `T` (which a shared seed would have made
/// possible), nor on any chrome const. The anchor family matters most here: one
/// key owns BOTH a diamond and an anchor, keyed by the same `(target, key)`.
#[test]
fn timeline_dynamic_ids_dont_collide_with_chrome_or_each_other() {
    let chrome: std::collections::BTreeSet<u64> = CHROME_IDS.iter().map(|(_, id)| id.0).collect();
    let mut seen: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    let raw = [0u64, 1, 2, 3, 7, 42, 255, 1000, 0x_dead_beef, u64::MAX];

    let mut check = |what: String, id: NodeId| {
        assert!(
            !chrome.contains(&id.0),
            "{what} (id {:#018x}) collides with a chrome const",
            id.0
        );
        assert!(
            seen.insert(id.0),
            "{what} (id {:#018x}) collides with another dynamic timeline id",
            id.0
        );
    };

    for &target in &raw {
        check(
            format!("timeline_twirl_id({target})"),
            ids::timeline_twirl_id(target),
        );
        check(
            format!("timeline_graph_resize_id({target})"),
            ids::timeline_graph_resize_id(target),
        );
        check(
            format!("timeline_summary_hit_id({target})"),
            ids::timeline_summary_hit_id(target),
        );
        for &key in &raw {
            check(
                format!("timeline_key_hit_id({target}, {key})"),
                ids::timeline_key_hit_id(target, key),
            );
            check(
                format!("timeline_anchor_hit_id({target}, {key})"),
                ids::timeline_anchor_hit_id(target, key),
            );
            for which in 0..2u8 {
                check(
                    format!("timeline_handle_hit_id({target}, {key}, {which})"),
                    ids::timeline_handle_hit_id(target, key, which),
                );
            }
        }
    }
    // Loop-range braces: three singletons (start / end / body), not per-target.
    for edge in 0..3u8 {
        check(
            format!("timeline_loop_brace_id({edge})"),
            ids::timeline_loop_brace_id(edge),
        );
    }
    // Marker pennants, keyed by storage index.
    for index in [0usize, 1, 2, 7, 42, 1000] {
        check(
            format!("timeline_marker_hit_id({index})"),
            ids::timeline_marker_hit_id(index),
        );
    }
}

/// The Vector font-dropdown option ids (one per pickable family, keyed by list
/// index) must not collide with a chrome const nor with each other — a collision
/// would route a family pick to the wrong widget. Mirrors the painter / timeline
/// dynamic-id guards. Indices span dense (small) + sparse (large) family counts.
#[test]
fn vector_dynamic_ids_dont_collide_with_chrome_or_each_other() {
    let chrome: std::collections::BTreeSet<u64> = CHROME_IDS.iter().map(|(_, id)| id.0).collect();
    let mut seen: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    for index in [0usize, 1, 2, 3, 7, 42, 255, 1000, 100_000] {
        let id = ids::vector_text_font_option_id(index).0;
        assert!(
            !chrome.contains(&id),
            "vector_text_font_option_id({index}) (id {id:#018x}) collides with a chrome const",
        );
        assert!(
            seen.insert(id),
            "vector_text_font_option_id({index}) (id {id:#018x}) collides with another dynamic id",
        );
    }
    // Variation-axis fields (index into the current font's non-wght axes).
    for index in 0..ids::MAX_TEXT_VARIATION_AXES {
        let id = ids::vector_text_axis_id(index).0;
        assert!(
            !chrome.contains(&id),
            "vector_text_axis_id({index}) (id {id:#018x}) collides with a chrome const",
        );
        assert!(
            seen.insert(id),
            "vector_text_axis_id({index}) (id {id:#018x}) collides with another dynamic id",
        );
    }
}

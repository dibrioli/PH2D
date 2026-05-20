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
];

/// Pairwise uniqueness across every chrome [`NodeId`]. O(n²) over ~200
/// entries = trivial in test time.
#[test]
fn chrome_node_ids_are_pairwise_unique() {
    // Sort by id so any collision lands in adjacent pairs.
    let mut sorted: Vec<(NodeId, &'static str)> =
        CHROME_IDS.iter().map(|(n, id)| (*id, *n)).collect();
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

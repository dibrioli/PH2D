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

/// Inspector panel container — used as the wheel-scroll key.
pub const INSP_PANEL: NodeId = NodeId(371);

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

/// Map fixture entity name to canonical hierarchy `NodeId`.
pub(crate) fn hierarchy_id(name: &str) -> Option<NodeId> {
    Some(match name {
        "Player" => HIER_PLAYER,
        "Sprite_idle" => HIER_SPRITE_IDLE,
        "Collider_box" => HIER_COLLIDER_BOX,
        "Script_player" => HIER_SCRIPT_PLAYER,
        "RigidBody" => HIER_RIGIDBODY,
        "Tilemap_ground" => HIER_TILEMAP_GROUND,
        "Tilemap_decor" => HIER_TILEMAP_DECOR,
        "Slime_01" => HIER_SLIME_01,
        "Slime_02" => HIER_SLIME_02,
        "Trigger_zoneA" => HIER_TRIGGER_ZONE_A,
        "Ambient_light" => HIER_AMBIENT_LIGHT,
        "Main_Camera" => HIER_MAIN_CAMERA,
        _ => return None,
    })
}

/// Map a hierarchy `NodeId` back to its fixture entity name. Inverse
/// of [`hierarchy_id`].
pub(crate) fn hierarchy_label_for_id(id: NodeId) -> Option<&'static str> {
    Some(match id {
        x if x == HIER_PLAYER => "Player",
        x if x == HIER_SPRITE_IDLE => "Sprite_idle",
        x if x == HIER_COLLIDER_BOX => "Collider_box",
        x if x == HIER_SCRIPT_PLAYER => "Script_player",
        x if x == HIER_RIGIDBODY => "RigidBody",
        x if x == HIER_TILEMAP_GROUND => "Tilemap_ground",
        x if x == HIER_TILEMAP_DECOR => "Tilemap_decor",
        x if x == HIER_SLIME_01 => "Slime_01",
        x if x == HIER_SLIME_02 => "Slime_02",
        x if x == HIER_TRIGGER_ZONE_A => "Trigger_zoneA",
        x if x == HIER_AMBIENT_LIGHT => "Ambient_light",
        x if x == HIER_MAIN_CAMERA => "Main_Camera",
        _ => return None,
    })
}

/// Best-effort 3-letter "kind" badge for the selection tag. Mirrors
/// the badges shown by the hierarchy row painter (PRF / UNI / OUT /
/// CAM).
pub(crate) fn hierarchy_kind_for_label(label: &str) -> &'static str {
    match label {
        "Player" => "OUT",
        "Sprite_idle" => "SPR",
        "Collider_box" | "Script_player" | "RigidBody" => "UNI",
        "Slime_01" | "Slime_02" => "PRF",
        "Main_Camera" => "CAM",
        "Tilemap_ground" | "Tilemap_decor" => "TIL",
        "Trigger_zoneA" => "TRG",
        "Ambient_light" => "LGT",
        _ => "ENT",
    }
}


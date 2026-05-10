//! Stable [`NodeId`] constants for the hero screen's interactive
//! widgets + helper mappings between fixture entity names and ids.
//!
//! Pre-populated in [`crate::interaction::WidgetStore`] at
//! construction time so the dispatcher always finds an entry on
//! hit-test. Numeric ranges:
//!
//! - 100..199 — TopBar buttons + Hierarchy add
//! - 200..299 — LeftRail tools
//! - 300..399 — Inspector fields
//! - 400..499 — Hierarchy entity rows
//! - 500..599 — Components Showcase widgets (RESERVED)

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

pub const INSP_MOVE_SPEED: NodeId = NodeId(300);
pub const INSP_JUMP_HEIGHT: NodeId = NodeId(301);
pub const INSP_FRICTION: NodeId = NodeId(302);
pub const INSP_DAMPING: NodeId = NodeId(303);
pub const INSP_DEBUG_SELECT: NodeId = NodeId(310);
pub const INSP_LINK_DISTANCE: NodeId = NodeId(320);
pub const INSP_LINK_MATERIAL: NodeId = NodeId(321);
pub const INSP_CAM_YAW: NodeId = NodeId(330);
pub const INSP_CAM_PITCH: NodeId = NodeId(331);

// Inspector polish (Phase 1) extra ids:
pub const INSP_TAB_PROPS: NodeId = NodeId(340);
pub const INSP_TAB_LAYERS: NodeId = NodeId(341);
pub const INSP_TAB_MATERIALS: NodeId = NodeId(342);
pub const INSP_NUM_MOVE_SPEED: NodeId = NodeId(350);
pub const INSP_NUM_JUMP_HEIGHT: NodeId = NodeId(351);
pub const INSP_NUM_FRICTION: NodeId = NodeId(352);
pub const INSP_NUM_DAMPING: NodeId = NodeId(353);
pub const INSP_NUM_CAM_YAW: NodeId = NodeId(354);
pub const INSP_HOT_RELOAD_CHECK: NodeId = NodeId(360);
pub const INSP_SNAP_GRID_TOGGLE: NodeId = NodeId(361);
pub const INSP_TINT_SWATCH: NodeId = NodeId(370);
pub const INSP_BLENDER_PICKER: NodeId = NodeId(380);

// BlenderColorPicker sub-control hit ids — registered by the
// showcase painter every frame, dispatched by `dispatch_pointer`
// into store mutations on `INSP_BLENDER_PICKER`.
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
// Components Showcase panel — drag handle reuses the picker's
// `BlenderHitKind::DragHandle` mechanism (panel-agnostic on the
// `parent` NodeId).
pub const SHOWCASE_PANEL: NodeId = NodeId(660);
pub const SHOWCASE_DRAG_HANDLE: NodeId = NodeId(661);
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

// ── Components Showcase (500..599) ─────────────────────────────────────────
// Text fields
pub const SHOWCASE_TEXT_INPUT_NAME: NodeId = NodeId(500);
pub const SHOWCASE_TEXT_AREA_NOTES: NodeId = NodeId(501);
// Combobox + its options
pub const SHOWCASE_COMBOBOX_ASSET: NodeId = NodeId(502);
pub const SHOWCASE_COMBOBOX_OPT_SPIKE: NodeId = NodeId(512);
pub const SHOWCASE_COMBOBOX_OPT_BLOCK: NodeId = NodeId(513);
// Checkbox
pub const SHOWCASE_CHECKBOX_LOCK: NodeId = NodeId(503);
// Dropdown + its options
pub const SHOWCASE_DROPDOWN_VIEW: NodeId = NodeId(504);
pub const SHOWCASE_DROPDOWN_OPT_FRONT: NodeId = NodeId(514);
pub const SHOWCASE_DROPDOWN_OPT_SIDE: NodeId = NodeId(515);
// RadioGroup + options
pub const SHOWCASE_RADIO_MODE: NodeId = NodeId(505);
pub const SHOWCASE_RADIO_SHADED: NodeId = NodeId(506);
pub const SHOWCASE_RADIO_WIRE: NodeId = NodeId(507);
pub const SHOWCASE_RADIO_SOLID: NodeId = NodeId(508);
// Vertical slider
pub const SHOWCASE_SLIDER_VERTICAL: NodeId = NodeId(509);
// Tags (removable → need a hit rect)
pub const SHOWCASE_TAG_DRAFT: NodeId = NodeId(510);
pub const SHOWCASE_TAG_DONE: NodeId = NodeId(511);
// Vector3Editor (position) — container + three NumberInput sub-fields
pub const SHOWCASE_V3_POS: NodeId = NodeId(516);
pub const SHOWCASE_V3_X: NodeId = NodeId(517);
pub const SHOWCASE_V3_Y: NodeId = NodeId(518);
pub const SHOWCASE_V3_Z: NodeId = NodeId(519);
// SectionHeader
pub const SHOWCASE_SECTION_ADVANCED: NodeId = NodeId(520);
// Modal buttons
pub const SHOWCASE_MODAL_CANCEL: NodeId = NodeId(521);
pub const SHOWCASE_MODAL_CONFIRM: NodeId = NodeId(522);
// Popover surface
pub const SHOWCASE_POPOVER: NodeId = NodeId(523);
// ContextMenu container + items
pub const SHOWCASE_CTX_MENU: NodeId = NodeId(524);
pub const SHOWCASE_CTX_ITEM_CUT: NodeId = NodeId(525);
pub const SHOWCASE_CTX_ITEM_COPY: NodeId = NodeId(526);
pub const SHOWCASE_CTX_DIVIDER: NodeId = NodeId(527);
pub const SHOWCASE_CTX_ITEM_DELETE: NodeId = NodeId(528);
// Card + list items + divider inside it
pub const SHOWCASE_CARD_QUICK_ACTIONS: NodeId = NodeId(529);
pub const SHOWCASE_LIST_OPEN: NodeId = NodeId(530);
pub const SHOWCASE_LIST_SAVE: NodeId = NodeId(531);
pub const SHOWCASE_LIST_EXPORT: NodeId = NodeId(532);
pub const SHOWCASE_CARD_DIVIDER: NodeId = NodeId(533);
// Decorative non-interactive widgets — still need stable ids for a11y
pub const SHOWCASE_PROGRESS_DET: NodeId = NodeId(540);
pub const SHOWCASE_PROGRESS_IND: NodeId = NodeId(541);
pub const SHOWCASE_SPINNER: NodeId = NodeId(542);
pub const SHOWCASE_AVATAR_CIRCLE: NodeId = NodeId(543);
pub const SHOWCASE_AVATAR_SQUARE: NodeId = NodeId(544);
// Primitives — canonical "one of each" gallery at the bottom
// of the showcase. New widgets added by the M13 audit round.
pub const SHOWCASE_PRIM_SLIDER: NodeId = NodeId(545);
pub const SHOWCASE_PRIM_SLIDER_CHIP: NodeId = NodeId(546);
pub const SHOWCASE_PRIM_BTN_PRIMARY: NodeId = NodeId(547);
pub const SHOWCASE_PRIM_BTN_SECONDARY: NodeId = NodeId(548);
pub const SHOWCASE_PRIM_BTN_DANGER: NodeId = NodeId(549);
pub const SHOWCASE_PRIM_BTN_ICON: NodeId = NodeId(550);
pub const SHOWCASE_PRIM_TOGGLE: NodeId = NodeId(551);
pub const SHOWCASE_PRIM_TABS_A: NodeId = NodeId(552);
pub const SHOWCASE_PRIM_TABS_B: NodeId = NodeId(553);
pub const SHOWCASE_PRIM_TABS_C: NodeId = NodeId(554);
pub const SHOWCASE_PRIM_SWATCH: NodeId = NodeId(555);
pub const SHOWCASE_PRIM_NUMBER: NodeId = NodeId(556);
pub const SHOWCASE_PRIM_TREE: NodeId = NodeId(557);
pub const SHOWCASE_PRIM_TREE_ROOT_A: NodeId = NodeId(558);
pub const SHOWCASE_PRIM_TREE_LEAF_A1: NodeId = NodeId(559);
pub const SHOWCASE_PRIM_TREE_LEAF_A2: NodeId = NodeId(580);
pub const SHOWCASE_PRIM_TOOLTIP: NodeId = NodeId(581);

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

/// Map a fixture-label to the canonical interactive id for that
/// inspector field. `None` when the field is non-interactive.
pub(crate) fn inspector_field_id(label: &str) -> Option<NodeId> {
    Some(match label {
        "Move Speed" => INSP_MOVE_SPEED,
        "Jump Height" => INSP_JUMP_HEIGHT,
        "Friction" => INSP_FRICTION,
        "Damping" => INSP_DAMPING,
        "Cam Yaw" => INSP_CAM_YAW,
        "Cam Pitch" => INSP_CAM_PITCH,
        "Debug" => INSP_DEBUG_SELECT,
        "Distance" => INSP_LINK_DISTANCE,
        "Material" => INSP_LINK_MATERIAL,
        _ => return None,
    })
}

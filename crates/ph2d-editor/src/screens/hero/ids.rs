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

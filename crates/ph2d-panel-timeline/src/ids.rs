//! Re-export of this panel's chrome NodeIds (defined centrally in
//! `ph2d-editor-core::ids::chrome::timeline`, so dispatch + paint share them).

pub use ph2d_editor_core::ids::{
    TIMELINE_ADD_MARKER, TIMELINE_AUTOKEY, TIMELINE_CLOSE, TIMELINE_FRAME_NUM, TIMELINE_GO_END,
    TIMELINE_GO_START, TIMELINE_LOOP, TIMELINE_MOTION_PATH, TIMELINE_NEXT_FRAME,
    TIMELINE_ONION_SETTINGS, TIMELINE_PANEL, TIMELINE_PHYSICS, TIMELINE_PINGPONG, TIMELINE_PLAY,
    TIMELINE_PREV_FRAME, TIMELINE_RECORD, TIMELINE_RULER, TIMELINE_SNAP, TIMELINE_TIME_NUM,
};

pub use ph2d_editor_core::ids::{
    CTX_MENU_TL_AUTO_ORIENT, CTX_MENU_TL_DELETE_TRACK, CTX_MENU_TL_TO_AXES, CTX_MENU_TL_TO_PATH,
};
// Per-track extrapolation menu (crown-jewels plan §6): the two cascade rows, the
// four mode leaves, and the side wire-encodings.
pub use ph2d_editor_core::ids::{
    CTX_MENU_TL_EXTRAP_CONTINUE, CTX_MENU_TL_EXTRAP_HOLD, CTX_MENU_TL_EXTRAP_LOOP,
    CTX_MENU_TL_EXTRAP_PINGPONG, CTX_MENU_TL_EXTRAP_POST, CTX_MENU_TL_EXTRAP_PRE,
};
// Marker right-click menu (ADR-0143): the pennant's whole edit surface.
pub use ph2d_editor_core::ids::{
    CTX_MENU_TL_DELETE_MARKER, CTX_MENU_TL_RENAME_MARKER, CTX_MENU_TL_SET_SIGNAL,
    TIMELINE_MARKER_MENU,
};
pub use ph2d_editor_core::ids::{
    CTX_MENU_TL_LANE_ADDITIVE, CTX_MENU_TL_LANE_DELETE, CTX_MENU_TL_LANE_OVERRIDE,
    CTX_MENU_TL_LANE_RENAME, TIMELINE_LANE_MENU,
};
pub use ph2d_editor_core::ids::{
    CTX_MENU_TL_STRIP_DELETE, CTX_MENU_TL_STRIP_DUPLICATE, CTX_MENU_TL_STRIP_ENTER,
    CTX_MENU_TL_STRIP_LOOP, CTX_MENU_TL_STRIP_ONCE, CTX_MENU_TL_STRIP_PINGPONG,
    CTX_MENU_TL_STRIP_RESET_SPEED, TIMELINE_STRIP_MENU,
};

/// The "+Track" property buttons paired with their [`ph2d_timeline::PropKind`]:
/// the six scene properties in `PropKind::ALL` order, then **Time Remap** (the
/// per-object clock — not a pose prop, so it lives outside `ALL`). Used by
/// paint (the label comes from `prop_label`, the single i18n source) + the
/// hit/populate wiring (which ignores the second field).
///
/// **Position** fecha a lista (ADR-0141): é o modo de TRAJETÓRIA, a alternativa a
/// TranslationX + TranslationY, e por isso fica fora de `ALL` como o Time Remap. Entra
/// no FIM porque a ordem desta tabela é a ordem que o artista lê, e as seis da pose são
/// o que ele procura primeiro.
pub const ADDPROP_BUTTONS: [(ph2d_a11y::NodeId, ph2d_timeline::PropKind); 13] = [
    (TIMELINE_ADDPROP_TX, ph2d_timeline::PropKind::TranslationX),
    (TIMELINE_ADDPROP_TY, ph2d_timeline::PropKind::TranslationY),
    (TIMELINE_ADDPROP_ROT, ph2d_timeline::PropKind::Rotation),
    (TIMELINE_ADDPROP_SX, ph2d_timeline::PropKind::ScaleX),
    (TIMELINE_ADDPROP_SY, ph2d_timeline::PropKind::ScaleY),
    (TIMELINE_ADDPROP_OPACITY, ph2d_timeline::PropKind::Opacity),
    (TIMELINE_ADDPROP_TIME, ph2d_timeline::PropKind::TimeRemap),
    (TIMELINE_ADDPROP_POS, ph2d_timeline::PropKind::Position),
    (TIMELINE_ADDPROP_MORPH, ph2d_timeline::PropKind::Morph),
    (
        TIMELINE_ADDPROP_JOINT_MOTOR_TARGET,
        ph2d_timeline::PropKind::JointMotorTarget,
    ),
    (
        TIMELINE_ADDPROP_JOINT_MOTOR_SPEED,
        ph2d_timeline::PropKind::JointMotorSpeed,
    ),
    (
        TIMELINE_ADDPROP_JOINT_REST_LENGTH,
        ph2d_timeline::PropKind::JointRestLength,
    ),
    (
        TIMELINE_ADDPROP_JOINT_MAX_LENGTH,
        ph2d_timeline::PropKind::JointMaxLength,
    ),
];

mod menus_timeline;
pub use menus_timeline::*;
mod timeline;
pub use timeline::*;

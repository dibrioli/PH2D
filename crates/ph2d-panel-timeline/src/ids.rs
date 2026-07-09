//! Re-export of this panel's chrome NodeIds (defined centrally in
//! `ph2d-editor-core::ids::chrome::timeline`, so dispatch + paint share them).

pub use ph2d_editor_core::ids::{
    TIMELINE_ADD_TRACK, TIMELINE_ADDPROP_OPACITY, TIMELINE_ADDPROP_ROT, TIMELINE_ADDPROP_SX,
    TIMELINE_ADDPROP_SY, TIMELINE_ADDPROP_TX, TIMELINE_ADDPROP_TY, TIMELINE_AUTOKEY,
    TIMELINE_CLOSE, TIMELINE_FRAME_NUM, TIMELINE_GO_END, TIMELINE_GO_START, TIMELINE_LANES,
    TIMELINE_LOOP, TIMELINE_NEXT_FRAME, TIMELINE_PANEL, TIMELINE_PLAY, TIMELINE_PREV_FRAME,
    TIMELINE_RULER, TIMELINE_SCROLLBAR, TIMELINE_SNAP, TIMELINE_TIME_NUM,
};

/// The six "+Track" property buttons paired with their [`ph2d_timeline::PropKind`],
/// in `PropKind::ALL` order. Used by paint (labels) + the shell (id → PropKind).
pub const ADDPROP_BUTTONS: [(ph2d_a11y::NodeId, &str); 6] = [
    (TIMELINE_ADDPROP_TX, "X"),
    (TIMELINE_ADDPROP_TY, "Y"),
    (TIMELINE_ADDPROP_ROT, "R"),
    (TIMELINE_ADDPROP_SX, "Sx"),
    (TIMELINE_ADDPROP_SY, "Sy"),
    (TIMELINE_ADDPROP_OPACITY, "Op"),
];

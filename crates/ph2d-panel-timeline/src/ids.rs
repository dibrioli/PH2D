//! Re-export of this panel's chrome NodeIds (defined centrally in
//! `ph2d-editor-core::ids::chrome::timeline`, so dispatch + paint share them).

pub use ph2d_editor_core::ids::{
    TIMELINE_ADD_CLIP, TIMELINE_ADD_MARKER, TIMELINE_ADD_TRACK, TIMELINE_ADDPROP_OPACITY,
    TIMELINE_ADDPROP_ROT, TIMELINE_ADDPROP_SX, TIMELINE_ADDPROP_SY, TIMELINE_ADDPROP_TIME,
    TIMELINE_ADDPROP_TX, TIMELINE_ADDPROP_TY, TIMELINE_AUTOKEY, TIMELINE_CLIP_DD,
    TIMELINE_CLIP_OPT, TIMELINE_CLIP_RENAME_INPUT, TIMELINE_CLOSE, TIMELINE_DELETE_CLIP,
    TIMELINE_FRAME_NUM, TIMELINE_GO_END, TIMELINE_GO_START, TIMELINE_LABEL_SPLIT, TIMELINE_LANES,
    TIMELINE_LOOP, TIMELINE_MARKER_RENAME_INPUT, TIMELINE_NEXT_FRAME, TIMELINE_PANEL,
    TIMELINE_PLAY, TIMELINE_PREV_FRAME, TIMELINE_RECORD, TIMELINE_RENAME_CLIP, TIMELINE_RESIZE_B,
    TIMELINE_RESIZE_BL, TIMELINE_RESIZE_BR, TIMELINE_RESIZE_L, TIMELINE_RESIZE_R,
    TIMELINE_RESIZE_T, TIMELINE_RESIZE_TL, TIMELINE_RESIZE_TR, TIMELINE_RULER, TIMELINE_SCROLLBAR,
    TIMELINE_SNAP, TIMELINE_SPEED, TIMELINE_SUMMARY_LOCK, TIMELINE_TIME_NUM,
    timeline_anchor_hit_id, timeline_graph_resize_id, timeline_handle_hit_id, timeline_key_hit_id,
    timeline_loop_brace_id, timeline_marker_hit_id, timeline_row_id, timeline_summary_hit_id,
    timeline_twirl_id,
};

pub use ph2d_editor_core::ids::CTX_MENU_TL_DELETE_TRACK;

/// The "+Track" property buttons paired with their [`ph2d_timeline::PropKind`]:
/// the six scene properties in `PropKind::ALL` order, then **Time Remap** (the
/// per-object clock — not a pose prop, so it lives outside `ALL`). Used by
/// paint (the label comes from `prop_label`, the single i18n source) + the
/// hit/populate wiring (which ignores the second field).
pub const ADDPROP_BUTTONS: [(ph2d_a11y::NodeId, ph2d_timeline::PropKind); 7] = [
    (TIMELINE_ADDPROP_TX, ph2d_timeline::PropKind::TranslationX),
    (TIMELINE_ADDPROP_TY, ph2d_timeline::PropKind::TranslationY),
    (TIMELINE_ADDPROP_ROT, ph2d_timeline::PropKind::Rotation),
    (TIMELINE_ADDPROP_SX, ph2d_timeline::PropKind::ScaleX),
    (TIMELINE_ADDPROP_SY, ph2d_timeline::PropKind::ScaleY),
    (TIMELINE_ADDPROP_OPACITY, ph2d_timeline::PropKind::Opacity),
    (TIMELINE_ADDPROP_TIME, ph2d_timeline::PropKind::TimeRemap),
];

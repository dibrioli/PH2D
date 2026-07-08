//! Re-export of this panel's chrome NodeIds (defined centrally in
//! `ph2d-editor-core::ids::chrome::timeline`, so dispatch + paint share them).

pub use ph2d_editor_core::ids::{
    TIMELINE_ADD_TRACK, TIMELINE_AUTOKEY, TIMELINE_CLOSE, TIMELINE_FRAME_NUM, TIMELINE_GO_END,
    TIMELINE_GO_START, TIMELINE_LANES, TIMELINE_LOOP, TIMELINE_NEXT_FRAME, TIMELINE_PANEL,
    TIMELINE_PLAY, TIMELINE_PREV_FRAME, TIMELINE_RULER, TIMELINE_SCROLLBAR, TIMELINE_SNAP,
    TIMELINE_TIME_NUM,
};

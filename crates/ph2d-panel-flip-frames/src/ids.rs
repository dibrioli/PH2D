//! Os `NodeId`s da tira — re-export dos ids canônicos do editor-core (a fonte
//! única; o z-order walk e o teste de colisão os enumeram lá).

pub use ph2d_editor_core::ids::{
    FLIP_ADDITIVE, FLIP_AUTOKEY, FLIP_CYCLE_DD, FLIP_FALLOFF, FLIP_FPS_NUM, FLIP_GHOST,
    FLIP_GHOST_AFTER_NUM, FLIP_GHOST_BEFORE_NUM, FLIP_HOLD_NUM, FLIP_KEY_ADD, FLIP_KEY_DELETE,
    FLIP_KEY_DUP, FLIP_KEY_INSTANCE, FLIP_KEY_LEFT, FLIP_KEY_RIGHT, FLIP_KEY_UNLINK,
    FLIP_NEXT_DRAWING, FLIP_PLAY, FLIP_PREV_DRAWING, FLIP_SCRUB, FLIP_STRIP_CLOSE,
    FLIP_STRIP_PANEL, FLIP_TWEEN_ADD, FLIP_TWEEN_NUM, flip_cell_id, flip_cycle_option_id,
};

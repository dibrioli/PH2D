//! Widget `NodeId`s for the Flip Style panel.
//!
//! Like the other panel crates, the ids stay defined in editor-core
//! (`ph2d_editor_core::ids`) — the layout + z-order walk + `node_id_collisions`
//! arch test all reference them, and re-defining them here would fork the
//! source of truth. This module is a convenience re-export so the panel's
//! internal modules can write `crate::ids::FLIP_*`.

pub use ph2d_editor_core::ids::{
    FLIP_CLOSE, FLIP_EDIT_DELETE, FLIP_EDIT_DESELECT, FLIP_EDIT_SELECT_ALL, FLIP_ERASE_HARD,
    FLIP_ERASE_SOFT, FLIP_ERASE_STROKE, FLIP_FILL_BEHIND, FLIP_FILL_PAINT, FLIP_FILL_SWATCH,
    FLIP_FILL_UNPAINT, FLIP_GAP, FLIP_GAP_NUM, FLIP_GROW, FLIP_GROW_NUM, FLIP_HARDNESS,
    FLIP_HARDNESS_NUM, FLIP_LAYER_ADD, FLIP_LAYER_DELETE, FLIP_MODE_DRAW, FLIP_MODE_EDIT,
    FLIP_MODE_ERASE, FLIP_MODE_FILL, FLIP_MODE_RESHAPE, FLIP_MODE_SELECT, FLIP_OPACITY,
    FLIP_OPACITY_NUM, FLIP_PANEL, FLIP_PRECISION, FLIP_PRECISION_NUM, FLIP_RESHAPE_KIND_IDS,
    FLIP_RS_GRAB, FLIP_RS_PINCH, FLIP_RS_PUSH, FLIP_RS_RANDOMIZE, FLIP_RS_SMOOTH, FLIP_RS_STRENGTH,
    FLIP_RS_THICKNESS, FLIP_RS_TWIST, FLIP_SHAPE_FILLED, FLIP_SHAPE_LINE, FLIP_SIZE, FLIP_SIZE_NUM,
    FLIP_SMOOTHING, FLIP_SMOOTHING_NUM, FLIP_STROKE_SWATCH, FlipLayerWidget,
    flip_layer_blend_option_id, flip_layer_widget_id,
};

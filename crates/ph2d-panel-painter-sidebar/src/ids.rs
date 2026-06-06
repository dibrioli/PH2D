//! Widget NodeIds para o Painter sidebar — re-export do canon em
//! `ph2d_editor_core::ids`.
//!
//! Convenção (mesma BgRemoval): IDs vivem em editor-core (single source
//! of truth) para que PainterTool::handle_panel_event possa referenciá-los
//! sem cycle dep (tool → panel-sidebar).

pub use ph2d_editor_core::ids::{
    PAINTER_SIDEBAR_ACCUMULATE_TOGGLE as ACCUMULATE_TOGGLE,
    PAINTER_SIDEBAR_GRAIN_TOGGLE as GRAIN_TOGGLE,
    PAINTER_SIDEBAR_MODIFIER_SQUARE as MODIFIER_SQUARE,
    PAINTER_SIDEBAR_OPACITY_CHIP as OPACITY_CHIP, PAINTER_SIDEBAR_OPACITY_SLIDER as OPACITY_SLIDER,
    PAINTER_SIDEBAR_PIGMENT_TOGGLE as PIGMENT_TOGGLE, PAINTER_SIDEBAR_REDO_BUTTON as REDO_BUTTON,
    PAINTER_SIDEBAR_SIZE_CHIP as SIZE_CHIP, PAINTER_SIDEBAR_SIZE_SLIDER as SIZE_SLIDER,
    PAINTER_SIDEBAR_UNDO_BUTTON as UNDO_BUTTON,
};

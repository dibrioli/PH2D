//! Widget NodeIds para o Painter layers panel — re-export do canon em
//! `ph2d_editor_core::ids`.
//!
//! Convenção (mesma sidebar/BgRemoval): IDs vivem em editor-core (single
//! source of truth) para que `PainterTool::handle_panel_event` possa
//! referenciá-los sem cycle dep (tool → panel-layers).
//!
//! **SCAFFOLD:** só o panel container + close existem hoje. O Implementador
//! adiciona `PAINTER_LAYERS_*` para os widgets das rows (visibility,
//! opacity slider/chip, blend dropdown, reorder handle) em editor-core
//! `ids.rs` e re-exporta aqui.

pub use ph2d_editor_core::ids::{
    PAINTER_LAYERS_CLOSE as CLOSE, PAINTER_LAYERS_PANEL as PANEL,
};

//! Widget NodeIds para o Painter layers panel — re-export do canon em
//! `ph2d_editor_core::ids`.
//!
//! Convenção (mesma sidebar/BgRemoval): IDs vivem em editor-core (single
//! source of truth) para que `PainterTool::handle_panel_event` possa
//! referenciá-los sem cycle dep (tool → panel-layers).
//!
//! Container + close are fixed; the "+ Layer" footer button and the header
//! dock-toggle are also fixed ids. The per-row widget ids (visibility / opacity
//! slider+chip / blend chip / row-select) are *dynamic* — derived at paint time
//! via [`ph2d_editor_core::ids::painter_layer_widget_id`] — so they aren't
//! re-exported here.

pub use ph2d_editor_core::ids::{
    PAINTER_LAYERS_ADD as ADD, PAINTER_LAYERS_CLOSE as CLOSE, PAINTER_LAYERS_PANEL as PANEL,
    PAINTER_LAYERS_TOGGLE_DOCK as TOGGLE_DOCK,
};

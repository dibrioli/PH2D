#![forbid(unsafe_code)]
//! ph2d-editor — Procreate-style canvas-first editor (M12, ADR-0023).
//!
//! Foundation:
//! - [`zones::Layout`] — 4-zone canonical positioning (top-left
//!   creates / top-right edits / sidebar modulates / center 100 %
//!   canvas). Per ADR-0023 §3.
//! - [`floating_panel::FloatingPanel`] — Procreate-style draggable
//!   tool drawer primitive. Per ADR-0023 §5.
//! - [`widget`] — primitives (Button, Slider, Toggle, RadioGroup,
//!   ColorSwatch). Each follows the same pattern: data + state enum
//!   + tokens + a11y::Node + colocated `paint_X` helper.
//! - [`tool::Tool`] + [`tool::ToolRegistry`] — canonical contract
//!   every editor tool implements (id / label / icon / build_panel
//!   / activate hooks). Registry tracks the active tool.
//! - [`tools`] — seed implementations ([`tools::BrushTool`],
//!   [`tools::MoveTool`]) proving the trait shape.
//! - [`zen::ZenMode`] — Tab-toggle workspace state. Per ADR-0023 §2.
//! - [`toast::ToastQueue`] — non-modal notification stream. Per
//!   ADR-0023 §2 ("Notificações flutuantes não-modais").
//! - [`paint`] — Vello lowering (`Paint` trait + `paint_text`
//!   helper). M11 widget paint pass + this PR's text rendering.
//!
//! Out of scope (M13+):
//! - QuickMenu radial (ADR-0023 §6)
//! - Gesture-mapping editor UI (ADR-0023 §4)
//! - Single-Touch Companion overlay

pub mod floating_panel;
pub mod paint;
pub mod toast;
pub mod tool;
pub mod tools;
pub mod widget;
pub mod zen;
pub mod zones;

pub use floating_panel::{FloatingPanel, PanelAction, PanelAnchor, PanelControl, PanelTab, ToolId};
pub use paint::{Paint, PaintCtx, paint_button, paint_text, paint_text_centered, resolve};
pub use toast::{Toast, ToastQueue, ToastSeverity};
pub use tool::{PanelEvent, Tool, ToolRegistry};
// Re-export so the shell can name the dragging node id without
// taking a direct ph2d-a11y dep just for one type.
pub use ph2d_a11y::NodeId;
pub use tools::{BrushTool, MoveTool};
pub use widget::{
    Button, ColorSwatch, RadioGroup, RadioOption, Slider, Toggle, paint_color_swatch,
    paint_radio_group, paint_slider, paint_toggle,
};
pub use zen::ZenMode;
pub use zones::{Layout, Zone};

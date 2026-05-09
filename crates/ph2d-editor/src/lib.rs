#![forbid(unsafe_code)]
//! ph2d-editor — Procreate-style canvas-first editor (M12, ADR-0023).
//!
//! Pure data + a11y. Rendering is the shell's job (egui owns paint
//! after the egui-migration pivot — see ADR-0024).
//!
//! Foundation:
//! - [`zones::Layout`] — 4-zone canonical positioning (top-left
//!   edits / top-right creates / sidebar modulates / center 100 %
//!   canvas). Per ADR-0023 §3.
//! - [`floating_panel::FloatingPanel`] — Procreate-style panel data
//!   (tabs + actions + controls). Per ADR-0023 §5.
//! - [`widget`] — primitives (Button, Slider, Toggle, RadioGroup,
//!   ColorSwatch) with AccessKit nodes.
//! - [`tool::Tool`] + [`tool::ToolRegistry`] — canonical contract
//!   every editor tool implements (id / label / icon / build_panel
//!   / activate hooks / handle_panel_event).
//! - [`tools`] — seed implementations ([`tools::BrushTool`],
//!   [`tools::MoveTool`]).
//! - [`zen::ZenMode`] — Tab-toggle workspace state. Per ADR-0023 §2.
//! - [`toast::ToastQueue`] — non-modal notification stream.
//!
//! Out of scope (M13+):
//! - QuickMenu radial (ADR-0023 §6)
//! - Gesture-mapping editor UI (ADR-0023 §4)
//! - Single-Touch Companion overlay

pub mod floating_panel;
pub mod toast;
pub mod tool;
pub mod tools;
pub mod widget;
pub mod zen;
pub mod zones;

pub use floating_panel::{FloatingPanel, PanelAction, PanelAnchor, PanelControl, PanelTab, ToolId};
pub use toast::{Toast, ToastQueue, ToastSeverity};
pub use tool::{PanelEvent, Tool, ToolRegistry};
// Re-export so the shell can name node ids without a direct ph2d-a11y dep.
pub use ph2d_a11y::NodeId;
pub use tools::{BrushTool, MoveTool};
pub use widget::{Button, ColorSwatch, RadioGroup, RadioOption, Slider, Toggle};
pub use zen::ZenMode;
pub use zones::{Layout, Zone};

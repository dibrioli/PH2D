#![forbid(unsafe_code)]
//! ph2d-editor — Procreate-style canvas-first editor (M12, ADR-0023).
//!
//! Foundation in this skeleton:
//! - [`zones::Layout`] — 4-zone canonical positioning (top-left
//!   creates / top-right edits / sidebar modulates / center 100 %
//!   canvas). Per ADR-0023 §3.
//! - [`floating_panel::FloatingPanel`] — Procreate-style draggable
//!   tool drawer primitive. Per ADR-0023 §5.
//! - [`widget::Button`] — example widget showing the
//!   `tokens` + `a11y::Node` integration pattern every future
//!   widget follows.
//! - [`zen::ZenMode`] — Tab-toggle workspace state. Per ADR-0023 §2.
//! - [`toast::ToastQueue`] — non-modal notification stream. Per
//!   ADR-0023 §2 ("Notificações flutuantes não-modais").
//!
//! Out of scope this skeleton (lands in M12 follow-ups / M13):
//! - Vello rendering of widgets (only data shapes here; rendering
//!   path lands when shells/desktop wires the editor in)
//! - QuickMenu radial (M13+ per ADR-0023 §6)
//! - Gesture-mapping editor UI (M13+ per ADR-0023 §4)
//! - Single-Touch Companion overlay (M13+)
//! - Selection panel demo with all 8 actions (this skeleton ships
//!   the `FloatingPanel` primitive + 1 minimal demo proving the API)

pub mod floating_panel;
pub mod paint;
pub mod toast;
pub mod widget;
pub mod zen;
pub mod zones;

pub use floating_panel::{FloatingPanel, PanelAnchor, PanelTab};
pub use paint::{Paint, PaintCtx, paint_button, resolve};
pub use toast::{Toast, ToastQueue, ToastSeverity};
pub use widget::Button;
pub use zen::ZenMode;
pub use zones::{Layout, Zone};

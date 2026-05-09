#![forbid(unsafe_code)]
//! ph2d-a11y — AccessKit cross-platform a11y (M12, ADR-0023).
//!
//! Wraps [AccessKit](https://github.com/AccessKit/accesskit) — the
//! canonical Rust abstraction over Mac VoiceOver, Windows Narrator,
//! iPadOS VoiceOver, and Linux AT-SPI. Each editor widget exports
//! a node; the shell drives the global tree update via the
//! platform-specific accesskit_<os> adapter.
//!
//! ### What this crate exposes
//!
//! - [`Tree`] — light wrapper around `accesskit::Tree` + node store.
//!   Insert, update, remove nodes by [`NodeId`]; serialize to a
//!   `TreeUpdate` for the shell adapter.
//! - [`NodeBuilder`] — convenience builder around `accesskit::Node`
//!   with PH2D defaults (live region for toasts, focusable for
//!   buttons, etc).
//! - [`Live`] — re-export of `accesskit::Live` for live-region
//!   toast announcements (per ADR-0023 §10).
//!
//! ### What this crate does NOT do (M12 scope)
//!
//! - Adapter to native OS — `shells/desktop` adds `accesskit_winit`
//!   when M12 wires the editor into the running window
//! - Single-Touch Companion overlay — primitive lands in
//!   `ph2d-editor` (M12+ as the gesture system materializes)
//! - WCAG audit lints (contrast, ARIA-equivalent role coverage) —
//!   `ph2d-tokens` already validates color contrast; widget-shape
//!   lints are M13+

pub mod node;
pub mod tree;

pub use node::{NodeBuilder, NodeId};
pub use tree::Tree;

// Re-export AccessKit primitives that show up in the public API.
// Keep the surface small per SKILL §7 anti-pattern "acoplar API
// pública a tipos externos" — only what callers genuinely need.
pub use accesskit::{Action, Live, Node, NodeId as AkNodeId, Role, Toggled};

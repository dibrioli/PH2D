//! Test helpers for `ph2d-editor-core` consumers (panel crates,
//! integration tests, downstream shell tests).
//!
//! Wave 8 Phase 1 — `HeroScreen::new` is a pure constructor and no
//! longer auto-installs the panel registry (audit B1). Production
//! callers wire `ph2d_panel_registry_init::register_all_panels()`
//! at boot; tests that just want a working HeroScreen call
//! [`ensure_panel_registry`].
//!
//! ADR-0029 Phase D — the legacy fn-pointer registry was deleted, and
//! the orchestrator now reaches the typed registry through
//! [`crate::panel::with_registry_opt`] which tolerates an empty
//! `OnceLock`. So this helper is intentionally a no-op now: tests that
//! construct a [`crate::HeroScreen`] and only exercise the host /
//! chrome paths need no registry at all, and tests that need a
//! specific panel installed (see `ph2d-panel-*/tests/*.rs`) manage
//! their own [`crate::panel::install_panel_registry`] call. Kept as a
//! function so existing callers continue to compile.
//!
//! Exposed via `#[doc(hidden)] pub mod test_support` so integration
//! tests can reach it, but it is **not** part of the stable public
//! API.

/// No-op since ADR-0029 Phase D. See module docs.
pub fn ensure_panel_registry() {}

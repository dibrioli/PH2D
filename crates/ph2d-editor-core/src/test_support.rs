//! Test helpers for `ph2d-editor` consumers (panel crates,
//! integration tests, downstream shell tests).
//!
//! Wave 8 Phase 1 — `HeroScreen::new` is a pure constructor and no
//! longer auto-installs the panel registry (audit B1). Production
//! callers wire `ph2d_panel_registry_init::register_all_panels()`
//! at boot; tests that just want a working HeroScreen call
//! [`ensure_panel_registry`].
//!
//! Exposed via `#[doc(hidden)] pub mod test_support` so integration
//! tests can reach it, but it is **not** part of the stable public
//! API.

use std::sync::Once;

/// Install the bundled `default_panel_registry()` (all 4 in-tree
/// panels) into the process-wide `PANEL_REGISTRY` exactly once per
/// process. Subsequent calls are a cheap no-op via [`Once`], so
/// every `#[test]` can call this without coordination.
///
/// Use this in any test that constructs a [`crate::HeroScreen`] and
/// either paints, dispatches events, or calls
/// [`crate::panel_registry::panels`] directly.
pub fn ensure_panel_registry() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = crate::panel_registry::install_panel_registry(
            crate::panel_registry::default_panel_registry(),
        );
    });
}

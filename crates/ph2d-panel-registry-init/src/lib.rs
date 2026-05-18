//! ph2d-panel-registry-init — append-only point of contact for
//! panel registration. Wave 7 Phase 5 scaffold.
//!
//! Hosts that want feature-gated panel selection call
//! [`register_all_panels`] at boot BEFORE constructing the first
//! `HeroScreen`. The default (all 4 in-tree panels) lives inside
//! `ph2d_editor::panel_registry::default_panel_registry`, which
//! `HeroScreen::new` auto-installs as a fallback when no registry
//! is present.
//!
//! ## Cargo features
//!
//! - `default = ["panel-widget-gallery", "panel-hierarchy",
//!   "panel-inspector", "panel-grid-snap"]` — bundles every panel.
//! - `--no-default-features` — empty registry (host renders chrome
//!   only; useful for lite-build distributions).
//! - `--no-default-features --features panel-inspector` — single
//!   panel; everything else stripped.
//!
//! Wave 7 Phase 3 (panel-as-crate extraction) is the future step
//! that physically moves each panel to its own `crates/ph2d-panel-*`
//! crate; this init crate's API stays the same — only its `[dependencies]`
//! block in Cargo.toml swaps the path-dep from `ph2d-editor` to the
//! per-panel crate.

#![forbid(unsafe_code)]

use ph2d_editor::panel_registry::{PanelRegistry, install_panel_registry};

/// Aggregate the cargo-feature-gated panel manifests + install
/// them into the process-wide registry. Idempotent — calling
/// twice is a no-op (second registry silently dropped per
/// `OnceLock::set` semantics).
///
/// Returns `true` on first install, `false` if a registry was
/// already installed (matches `install_panel_registry`).
pub fn register_all_panels() -> bool {
    install_panel_registry(build_registry())
}

/// Build the registry without installing. Useful for tests that
/// want to inspect the manifest list before install.
pub fn build_registry() -> PanelRegistry {
    #[allow(unused_mut)]
    let mut reg = PanelRegistry::new_empty();
    // Wave 7 Stage 2: every panel is its own crate. widget_gallery
    // is fully physical-extracted; the others alias the in-tree
    // `ph2d_editor::*::PANEL_MANIFEST` until Wave 8 promotes them.
    #[cfg(feature = "panel-widget-gallery")]
    reg.push(&ph2d_panel_widget_gallery::PANEL_MANIFEST);
    #[cfg(feature = "panel-hierarchy")]
    reg.push(&ph2d_panel_hierarchy::PANEL_MANIFEST);
    #[cfg(feature = "panel-inspector")]
    reg.push(&ph2d_panel_inspector::PANEL_MANIFEST);
    #[cfg(feature = "panel-grid-snap")]
    reg.push(&ph2d_panel_grid_snap::PANEL_MANIFEST);
    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Expected panel count = sum of enabled cargo features. Computed
    /// at compile time so the test passes for any feature combination
    /// (default-all, `--no-default-features`, single-panel selection).
    const EXPECTED_PANELS: usize = {
        let mut n = 0;
        #[cfg(feature = "panel-widget-gallery")]
        {
            n += 1;
        }
        #[cfg(feature = "panel-hierarchy")]
        {
            n += 1;
        }
        #[cfg(feature = "panel-inspector")]
        {
            n += 1;
        }
        #[cfg(feature = "panel-grid-snap")]
        {
            n += 1;
        }
        n
    };

    #[test]
    fn build_registry_matches_enabled_features() {
        let reg = build_registry();
        assert_eq!(reg.manifests().len(), EXPECTED_PANELS);
    }
}

//! ph2d-panel-registry-init — append-only point of contact for
//! panel registration.
//!
//! ADR-0029 Phase C.4 closed the typed migration: every in-tree panel
//! (Inspector / Hierarchy / Widget Gallery / Grid Snap) now lives in
//! `ph2d_editor_core::panel::PANEL_REGISTRY` as a typed
//! `Panel<State>`. The legacy fn-pointer registry
//! (`ph2d_editor_core::panel_registry`) ships empty by default but
//! continues to exist so 3rd-party panels (future) can opt into the
//! pre-ADR contract.
//!
//! `register_all_panels` installs BOTH atomically. Hosts call it once
//! at boot before `HeroScreen::new`.
//!
//! ## Cargo features
//!
//! - `default = ["panel-widget-gallery", "panel-hierarchy",
//!   "panel-inspector", "panel-grid-snap"]` — bundles every panel.
//! - `--no-default-features` — empty registries.
//! - `--no-default-features --features panel-inspector` — only the
//!   typed Inspector; chrome only otherwise.

#![forbid(unsafe_code)]

use ph2d_editor_core::panel::{ErasedPanel, install_panel_registry as install_typed_registry};
use ph2d_editor_core::panel_registry::{
    PanelRegistry as LegacyRegistry, install_panel_registry as install_legacy_registry,
};

/// Aggregate the cargo-feature-gated panel manifests + install them
/// into both process-wide registries. Idempotent.
///
/// Returns `true` only when BOTH installs succeeded on first try.
/// Subsequent calls return `false` (matches `OnceLock::set` semantics).
pub fn register_all_panels() -> bool {
    let legacy_ok = install_legacy_registry(build_legacy_registry());
    let typed_ok = install_typed_registry(build_typed_registry());
    legacy_ok && typed_ok
}

/// Build the legacy fn-pointer registry without installing it.
/// Post-Phase-C.4 this always returns an empty registry — every
/// in-tree panel migrated to the typed registry below. Kept around
/// so future 3rd-party panels can `push(&manifest)` if they prefer
/// the legacy shape.
pub fn build_legacy_registry() -> LegacyRegistry {
    LegacyRegistry::new_empty()
}

/// Build the typed `Panel<State>` registry without installing it.
pub fn build_typed_registry() -> ph2d_editor_core::panel::PanelRegistry {
    #[allow(unused_mut)]
    let mut reg = ph2d_editor_core::panel::PanelRegistry::new_empty();
    #[cfg(feature = "panel-inspector")]
    reg.push(ErasedPanel::new::<ph2d_panel_inspector::InspectorPanel>());
    #[cfg(feature = "panel-hierarchy")]
    reg.push(ErasedPanel::new::<ph2d_panel_hierarchy::HierarchyPanel>());
    #[cfg(feature = "panel-widget-gallery")]
    reg.push(ErasedPanel::new::<
        ph2d_panel_widget_gallery::WidgetGalleryPanel,
    >());
    #[cfg(feature = "panel-grid-snap")]
    reg.push(ErasedPanel::new::<ph2d_panel_grid_snap::GridSnapPanel>());
    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Expected legacy panel count post-Phase-C.4 — always zero (no
    /// in-tree panel lives on the fn-pointer registry anymore).
    const EXPECTED_LEGACY: usize = 0;

    /// Expected typed panel count (Inspector + Hierarchy + Widget
    /// Gallery + Grid Snap after C.4).
    const EXPECTED_TYPED: usize = {
        let mut n = 0;
        #[cfg(feature = "panel-inspector")]
        {
            n += 1;
        }
        #[cfg(feature = "panel-hierarchy")]
        {
            n += 1;
        }
        #[cfg(feature = "panel-widget-gallery")]
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
    fn build_registries_match_enabled_features() {
        let legacy = build_legacy_registry();
        assert_eq!(legacy.manifests().len(), EXPECTED_LEGACY);
        let typed = build_typed_registry();
        assert_eq!(typed.panels().len(), EXPECTED_TYPED);
    }
}

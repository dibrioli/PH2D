//! ph2d-panel-registry-init — append-only point of contact for
//! panel registration.
//!
//! ADR-0029 Phase C.1 split this init into two registries:
//! - **Legacy fn-pointer registry** (`ph2d_editor_core::panel_registry`)
//!   for panels still on the pre-ADR shape (grid_snap as of C.3;
//!   migrates to typed in C.4).
//! - **Typed `Panel<State>` registry** (`ph2d_editor_core::panel`)
//!   for migrated panels (Inspector after C.1, Hierarchy after C.2,
//!   Widget Gallery after C.3).
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
pub fn build_legacy_registry() -> LegacyRegistry {
    #[allow(unused_mut)]
    let mut reg = LegacyRegistry::new_empty();
    #[cfg(feature = "panel-grid-snap")]
    reg.push(&ph2d_panel_grid_snap::PANEL_MANIFEST);
    reg
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
    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Expected legacy panel count (grid_snap while it remains a
    /// fn-pointer manifest post-C.3).
    const EXPECTED_LEGACY: usize = {
        let mut n = 0;
        #[cfg(feature = "panel-grid-snap")]
        {
            n += 1;
        }
        n
    };

    /// Expected typed panel count (Inspector + Hierarchy + Widget
    /// Gallery after C.3).
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

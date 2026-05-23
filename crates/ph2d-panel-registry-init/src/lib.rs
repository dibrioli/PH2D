//! ph2d-panel-registry-init — append-only point of contact for
//! panel registration.
//!
//! ADR-0029 Phase D closed the migration: every in-tree panel
//! (Inspector / Hierarchy / Widget Gallery / Grid Snap) lives in
//! `ph2d_editor_core::panel::PANEL_REGISTRY` as a typed
//! `Panel<State>`. The legacy fn-pointer registry was deleted in
//! Phase D — this crate installs only the typed registry. Hosts call
//! it once at boot before `HeroScreen::new`.
//!
//! ## Cargo features
//!
//! - `default = ["panel-widget-gallery", "panel-hierarchy",
//!   "panel-inspector", "panel-grid-snap"]` — bundles every panel.
//! - `--no-default-features` — empty registry.
//! - `--no-default-features --features panel-inspector` — only the
//!   typed Inspector; chrome only otherwise.

#![forbid(unsafe_code)]

use ph2d_editor_core::panel::{ErasedPanel, install_panel_registry};

/// Aggregate the cargo-feature-gated panel manifests + install them
/// into the process-wide typed registry. Idempotent.
///
/// Returns `true` on first install, `false` if a registry was already
/// installed (matches `OnceLock::set` semantics).
pub fn register_all_panels() -> bool {
    install_panel_registry(build_typed_registry())
}

/// Build the typed `Panel<State>` registry without installing it.
pub fn build_typed_registry() -> ph2d_editor_core::panel::PanelRegistry {
    #[allow(unused_mut)]
    let mut reg = ph2d_editor_core::panel::PanelRegistry::new_empty();
    #[cfg(feature = "panel-bgremoval")]
    reg.push(ErasedPanel::new::<ph2d_panel_bgremoval::BgRemovalPanel>());
    #[cfg(feature = "panel-color-equalization")]
    reg.push(ErasedPanel::new::<
        ph2d_panel_color_equalization::ColorEqualizationPanel,
    >());
    #[cfg(feature = "panel-equalize-sizes")]
    reg.push(ErasedPanel::new::<
        ph2d_panel_equalize_sizes::EqualizeSizesPanel,
    >());
    #[cfg(feature = "panel-padding")]
    reg.push(ErasedPanel::new::<ph2d_panel_padding::PaddingPanel>());
    #[cfg(feature = "panel-upscale")]
    reg.push(ErasedPanel::new::<ph2d_panel_upscale::UpscalePanel>());
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

    /// Expected typed panel count (Inspector + Hierarchy + Widget
    /// Gallery + Grid Snap after Phase C.4).
    const EXPECTED_TYPED: usize = {
        let mut n = 0;
        #[cfg(feature = "panel-bgremoval")]
        {
            n += 1;
        }
        #[cfg(feature = "panel-color-equalization")]
        {
            n += 1;
        }
        #[cfg(feature = "panel-equalize-sizes")]
        {
            n += 1;
        }
        #[cfg(feature = "panel-padding")]
        {
            n += 1;
        }
        #[cfg(feature = "panel-upscale")]
        {
            n += 1;
        }
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
    fn build_typed_registry_matches_enabled_features() {
        let typed = build_typed_registry();
        assert_eq!(typed.panels().len(), EXPECTED_TYPED);
    }
}

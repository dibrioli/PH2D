#![forbid(unsafe_code)]
//! ph2d-tool-equalize-sizes — multi-sprite canvas-size normalization.
//!
//! Image Tools row, order 100. Sabor 3 (stateful + docked panel) with one
//! **cross-sprite** twist: Apply iterates `hero.gizmo.iter_selected()`
//! instead of the chrome broadcasting 1 [`OneShotImageOp`] per sprite.
//! The shell collects per-entity [`algorithm::SpriteInput`]s, calls
//! [`tool::EqualizeSizesTool::run_full_resolution_multi`] in one batch
//! via `as_any_mut` downcast (BgRemoval/Padding template), and writes the
//! results back through `texture_edit::commit_edited_texture` + the
//! sprite `Transform`.
//!
//! Shape of this island (mirrors `ph2d-tool-padding`):
//! - [`algorithm`] — pure cross-sprite math + Lanczos3 / Nearest /
//!   Mitchell-Netravali kernels (zero external dep on `image`).
//! - [`icon`] — `BezPath` glyph for the chrome pill (mirrors
//!   `docs/design/icons/equalize-sizes.svg`).
//! - [`ids`] — panel-widget [`NodeId`](ph2d_a11y::NodeId) consts.
//! - [`manifest`] — `ToolManifest` declaration.
//! - [`params`] — `UiEdit` / `UiSnapshot` / `Params` + the single-site
//!   `apply_ui_edit` clamp + slider↔px helpers.
//! - [`tool`] — `EqualizeSizesTool` (`impl Tool`).
//!
//! [`OneShotImageOp`]: ph2d_editor_core::action_bus::EditorAction::OneShotImageOp

pub mod algorithm;
pub mod icon;
pub mod ids;
pub mod manifest;
pub mod params;
pub mod tool;

pub use algorithm::{SpriteInput, SpriteOutput, run_equalize_sizes};
pub use icon::equalize_sizes_bezpath;
pub use manifest::MANIFEST;
pub use params::{
    EQS_MAX_FIXED_DIM, EQS_MAX_GRID_UNIT, EqualizeSizesParams, EqualizeSizesUiEdit,
    EqualizeSizesUiSnapshot, TargetMode, UpscaleAlgorithm,
};
pub use tool::EqualizeSizesTool;

/// Register the Equalize Sizes manifest with the runtime registry.
/// Codegen target for `ph2d-tool-sync` (`register_all`).
pub fn register(reg: &mut ph2d_tool_registry::Registry) {
    reg.register(&MANIFEST);
}

/// Construct the Equalize Sizes tool as a boxed trait object for the
/// behavior `ToolRegistry`. Codegen target for `ph2d-tool-sync`
/// (`register_all_tools`).
pub fn make() -> Box<dyn ph2d_editor_core::tool::Tool> {
    Box::new(EqualizeSizesTool::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_attaches_manifest_to_registry() {
        let mut reg = ph2d_tool_registry::Registry::default();
        register(&mut reg);
        reg.build()
            .expect("registry should build with equalize_sizes");
        let m = reg.by_id("equalize_sizes").expect("registered by id");
        assert_eq!(m.id, "equalize_sizes");
        assert_eq!(m.label_key, "tool.equalize_sizes.label");
        assert_eq!(m.cluster, "image_tools");
        assert_eq!(m.order, 100);
    }

    #[test]
    fn register_is_idempotent() {
        let mut reg = ph2d_tool_registry::Registry::default();
        register(&mut reg);
        register(&mut reg);
        reg.build().expect("idempotent registration should build");
        assert_eq!(reg.manifests().len(), 1);
    }

    #[test]
    fn make_builds_a_tool_with_canonical_id() {
        let t = make();
        assert_eq!(
            t.id(),
            ph2d_editor_core::floating_panel::ToolId::new("equalize_sizes")
        );
    }
}

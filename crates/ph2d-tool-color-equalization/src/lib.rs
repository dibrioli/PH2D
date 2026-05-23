#![forbid(unsafe_code)]
//! ph2d-tool-color-equalization — Color Equalization: CLAHE + tonal
//! adjustments + auto white balance as an isolated tool crate (ADR-0040).
//!
//! Stateful Tool — active in the TopBar `image_tools` cluster, presents a
//! docked panel with six controls (clip limit, tile grid size, brightness,
//! contrast, saturation, auto-WB toggle) plus Apply / Cancel. Pixel work
//! happens at full resolution on commit; live preview runs on a 512²
//! aspect-fit thumbnail to keep slider drags smooth.
//!
//! All kernels are 100% own implementation, no external image deps. The
//! algorithm pipeline is:
//!
//! 1. **CLAHE** (Zuiderveld 1994, *Graphics Gems IV* pp. 474-485) — per-tile
//!    contrast-limited histogram equalization in sRGB luminance, with
//!    bilinear interpolation of the per-tile LUTs at the pixel level.
//! 2. **Brightness / Contrast / Saturation** — applied in linear-light sRGB
//!    after CLAHE (sRGB → linearize → adjust → delinearize).
//! 3. **Auto White Balance** (optional) — Gray-World channel gains in sRGB.
//!
//! Shape mirrors `ph2d-tool-bgremoval` / `ph2d-tool-padding` (sabor 3 da
//! DIRETRIZ §3.8.3): MANIFEST (data) + `ColorEqualizationTool`
//! (`impl ph2d_editor_core::tool::Tool`) + pure [`algorithm`] +
//! [`params`] vocab + [`ids`] (panel NodeIds via `hash_node_id`) +
//! [`icon`] glyph. The companion `ph2d-panel-color-equalization` reads
//! its snapshot type from [`params::ColorEqualizationUiSnapshot`].

pub mod algorithm;
pub mod icon;
pub mod ids;
pub mod manifest;
pub mod params;
pub mod tool;

pub use manifest::MANIFEST;
pub use tool::ColorEqualizationTool;

/// Register the Color Equalization manifest with the runtime registry.
/// Codegen target for `ph2d-tool-sync` (`register_all`).
pub fn register(reg: &mut ph2d_tool_registry::Registry) {
    reg.register(&MANIFEST);
}

/// Construct the Color Equalization tool as a boxed trait object for the
/// behavior `ToolRegistry`. Codegen target for `ph2d-tool-sync`
/// (`register_all_tools`).
pub fn make() -> Box<dyn ph2d_editor_core::tool::Tool> {
    Box::new(ColorEqualizationTool::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_attaches_manifest_to_registry() {
        let mut reg = ph2d_tool_registry::Registry::default();
        register(&mut reg);
        reg.build()
            .expect("registry should build with color_equalization");
        let found = reg
            .by_id("color_equalization")
            .expect("color_equalization registered by id");
        assert_eq!(found.id, "color_equalization");
        assert_eq!(found.label_key, "tool.color_equalization.label");
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
    fn make_returns_tool_with_matching_id() {
        let t = make();
        assert_eq!(
            t.id(),
            ph2d_editor_core::floating_panel::ToolId::new("color_equalization")
        );
    }
}

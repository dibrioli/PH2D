//! Global image-filter mode — the single source of truth for how the
//! app samples every sprite/texture and the Vello preview.
//!
//! Lives in `ph2d-host` because it is the lowest-level crate shared by
//! both `ph2d-render` (which maps it to `wgpu::FilterMode` for the
//! sprite samplers) and `ph2d-editor-core` (which maps it to
//! `peniko::ImageQuality` for the Background-Removal Vello preview).
//! Keeping the enum here — a zero-dependency, `#![forbid(unsafe_code)]`
//! crate — avoids a circular dep and keeps the type free of any GPU /
//! vector-renderer baggage. The `wgpu` ↔ `peniko` mapping helpers live
//! next to their consumers (see `ph2d_render::image_filter` and
//! `ph2d_editor_core`), so this crate stays dependency-light.
//!
//! ## Why a single mode
//!
//! Before this type the atlas sampler hardcoded `FilterMode::Linear`
//! (smooth), the individual-texture sampler hardcoded
//! `FilterMode::Nearest` (pixelated), and the Vello preview used the
//! peniko default. The same sprite looked smooth in the BG-removal
//! preview but pixelated after Apply (which bakes into an Individual
//! texture). One mode, chosen once in Settings, threaded everywhere,
//! removes the divergence.

/// The app-wide image-sampling mode. Chosen by the user in
/// Config → "Image filter" and applied to **every** texture sample
/// (atlas + individual sprite textures) and the Vello image preview.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum ImageFilterMode {
    /// Nearest-neighbor sampling — crisp, blocky pixels. Maps to
    /// `wgpu::FilterMode::Nearest` and `peniko::ImageQuality::Low`.
    PixelArt,
    /// Bilinear sampling — smooth, anti-aliased edges. The default,
    /// matching `ProjectSettings::default().image_filter`. Maps to
    /// `wgpu::FilterMode::Linear` and `peniko::ImageQuality::High`.
    /// Enio 2026-05-25: "Smooth deve ser o padrão" — antes o
    /// renderer abria em PixelArt enquanto o Settings marcava Smooth,
    /// divergência que aparecia no canvas com sampling Nearest na
    /// primeira frame.
    #[default]
    Smooth,
}

impl ImageFilterMode {
    // ⛔⛔ **Aqui viveu um ``ImageFilterMode::label()`` ORFAO, apagado em 2026-09-19.** O doc dizia *«stable string label for menu rows / telemetry»* e nenhum menu o lia.
    //
    // ⚠️ A sonda foi o COMPILADOR: renomeada a funcao, a workspace inteira compilou — o unico
    // vermelho veio de um teste desta crate. ⇒ e' a terceira especie do `CLAUDE.md` §5.0 (um orfao
    // le'-se igual a um morto), e traduzi-lo poria uma SEGUNDA palavra por variante na tabela de
    // strings, ao lado da que quem pinta ja' usa.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_smooth() {
        // Matches `ProjectSettings::default().image_filter` so the
        // canvas sampler and the Settings menu checkmark agree on
        // first paint (Enio 2026-05-25 fix).
        assert_eq!(ImageFilterMode::default(), ImageFilterMode::Smooth);
    }
}

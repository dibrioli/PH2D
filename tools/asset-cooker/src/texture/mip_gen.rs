//! W1.T7 — Mip pyramid generation pre-cook (ADR-0055-v4).
//!
//! Gera chain de mip levels (level 0 = source, level N = downsampled 2:1)
//! para textures que vão ser cookadas com mipmaps embedded no KTX2. Filtros
//! disponíveis: Box (fast, baseline), Lanczos (high quality), Point (pixel art).
//!
//! Determinismo: `image::imageops::resize` é puro Rust + arithmetic determinístic
//! (não usa `f32::sin/cos` non-deterministic — `Lanczos3` usa polynomial
//! approximation of sinc bounded por filter window). HR-6 compliant intra-machine;
//! cross-machine validado em W1.T10 canonical runner CI quando wired.
//!
//! API:
//! - [`MipFilter`] — escolha de filtro (Box/Lanczos/Point).
//! - [`generate_mip_chain`] — produce Vec<RgbaImage> chain (level 0 = source).
//! - [`mip_levels_for_dimensions`] — calcula quantos levels uma texture dada teria.
//!
//! Integration com cook: este módulo é STANDALONE. Caller pode iterar chain e
//! cookar cada level separadamente OR (W1.T7.1, future) `cook_with_mips` helper
//! passará chain inteiro pra `ctt::Image { surfaces: vec![chain] }` em single
//! `ctt::convert` call. ctt 0.4.0 já suporta multi-mip Image input.

use image::imageops::FilterType;
use image::{ImageBuffer, RgbaImage};

/// Filtro de downsample para mip generation. Mapeia pra `image::imageops::FilterType`
/// internamente mas expõe API mais semantic e PH2D-specific.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MipFilter {
    /// **Box** (= Triangle/bilinear no `image` crate): rápido, baseline.
    /// Apropriado pra sprites coloridos onde quality não é crítica.
    Box,
    /// **Lanczos3**: high quality (sinc 3-lobe), mais lento. Apropriado pra
    /// SDF fonts, normal maps, photo-realistic textures.
    Lanczos,
    /// **Point** (= Nearest): NO interpolation — preserva pixel-art / palette.
    /// Apropriado pra pixel art, palette-restricted assets, icons low-res.
    Point,
}

impl MipFilter {
    /// Mapeia pra `image::imageops::FilterType` correspondente.
    #[must_use]
    pub fn to_image_filter(self) -> FilterType {
        match self {
            // Triangle = bilinear (2-tap, equivalent à Box para 2:1 downsample exato).
            Self::Box => FilterType::Triangle,
            Self::Lanczos => FilterType::Lanczos3,
            Self::Point => FilterType::Nearest,
        }
    }
}

/// Calcula quantos mip levels uma texture `(width, height)` produz.
///
/// Convenção GPU standard: chain vai até level onde max(w, h) = 1. Ex: 256×256
/// → 9 levels (256, 128, 64, 32, 16, 8, 4, 2, 1). Para 100×80, segue floor(/2)
/// e stop quando ambos == 1: 100×80 → 50×40 → 25×20 → 12×10 → 6×5 → 3×2 → 1×1
/// = 7 levels.
#[must_use]
pub fn mip_levels_for_dimensions(width: u32, height: u32) -> usize {
    if width == 0 || height == 0 {
        return 0;
    }
    let mut w = width;
    let mut h = height;
    let mut levels = 1; // level 0 = source
    while w > 1 || h > 1 {
        w = (w / 2).max(1);
        h = (h / 2).max(1);
        levels += 1;
    }
    levels
}

/// Gera chain completo de mip levels a partir de `source` (level 0). Cada level
/// subsequente é downsampled 2:1 do anterior usando `filter`.
///
/// `max_levels = Some(N)` cap'a chain em N levels (level 0 + N-1 downsampled).
/// `None` gera chain completo até max(w, h) = 1.
///
/// Mip chain serve como input pra `ctt::Image` multi-level antes do cook —
/// ctt grava todos os levels no KTX2 container.
#[must_use]
pub fn generate_mip_chain(
    source: &RgbaImage,
    filter: MipFilter,
    max_levels: Option<usize>,
) -> Vec<RgbaImage> {
    let (w, h) = source.dimensions();
    if w == 0 || h == 0 {
        return Vec::new();
    }
    let total = mip_levels_for_dimensions(w, h);
    let capped = max_levels.map(|n| n.min(total)).unwrap_or(total);
    let image_filter = filter.to_image_filter();

    let mut chain: Vec<RgbaImage> = Vec::with_capacity(capped);
    chain.push(source.clone());

    while chain.len() < capped {
        let prev = chain.last().expect("chain has level 0");
        let (pw, ph) = prev.dimensions();
        let nw = (pw / 2).max(1);
        let nh = (ph / 2).max(1);
        // Early stop: já 1×1, não há mais downsample possível.
        if nw == pw && nh == ph {
            break;
        }
        // `resize` produz `ImageBuffer<Rgba<u8>, Vec<u8>>` que é `RgbaImage`.
        let resized: RgbaImage = ImageBuffer::from_raw(
            nw,
            nh,
            image::imageops::resize(prev, nw, nh, image_filter).into_raw(),
        )
        .expect("resize output dims match nw×nh×4 bytes");
        chain.push(resized);
    }

    chain
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texture::fixtures::brush_atlas_256_r8;

    /// Decode fixture PNG bytes → RgbaImage (test helper, evita boilerplate).
    fn fixture_to_rgba(png_bytes: &[u8]) -> RgbaImage {
        let dynamic = image::ImageReader::new(std::io::Cursor::new(png_bytes))
            .with_guessed_format()
            .expect("guess format")
            .decode()
            .expect("decode PNG");
        dynamic.to_rgba8()
    }

    #[test]
    fn mip_levels_for_256_square_is_9() {
        // 256, 128, 64, 32, 16, 8, 4, 2, 1 = 9 levels.
        assert_eq!(mip_levels_for_dimensions(256, 256), 9);
    }

    #[test]
    fn mip_levels_for_1x1_is_1() {
        assert_eq!(mip_levels_for_dimensions(1, 1), 1);
    }

    #[test]
    fn mip_levels_for_zero_dim_is_0() {
        assert_eq!(mip_levels_for_dimensions(0, 0), 0);
        assert_eq!(mip_levels_for_dimensions(256, 0), 0);
        assert_eq!(mip_levels_for_dimensions(0, 256), 0);
    }

    #[test]
    fn mip_levels_for_non_power_of_two_100x80() {
        // 100×80 → 50×40 → 25×20 → 12×10 → 6×5 → 3×2 → 1×1 = 7 levels.
        assert_eq!(mip_levels_for_dimensions(100, 80), 7);
    }

    #[test]
    fn mip_levels_for_rectangular_2x1() {
        // 2×1 → 1×1 = 2 levels.
        assert_eq!(mip_levels_for_dimensions(2, 1), 2);
    }

    #[test]
    fn generate_chain_256_box_has_9_levels() {
        let png = brush_atlas_256_r8();
        let src = fixture_to_rgba(&png);
        let chain = generate_mip_chain(&src, MipFilter::Box, None);
        assert_eq!(chain.len(), 9);
    }

    #[test]
    fn generate_chain_dimensions_halve_each_level() {
        let png = brush_atlas_256_r8();
        let src = fixture_to_rgba(&png);
        let chain = generate_mip_chain(&src, MipFilter::Box, None);
        let expected = [256, 128, 64, 32, 16, 8, 4, 2, 1];
        for (i, level) in chain.iter().enumerate() {
            assert_eq!(
                level.dimensions(),
                (expected[i], expected[i]),
                "level {i}: dimensions mismatch"
            );
        }
    }

    #[test]
    fn generate_chain_respects_max_levels_cap() {
        let png = brush_atlas_256_r8();
        let src = fixture_to_rgba(&png);
        let chain = generate_mip_chain(&src, MipFilter::Box, Some(4));
        assert_eq!(chain.len(), 4);
        let dims: Vec<_> = chain.iter().map(|l| l.dimensions()).collect();
        assert_eq!(dims, vec![(256, 256), (128, 128), (64, 64), (32, 32)]);
    }

    #[test]
    fn generate_chain_lanczos3_distinct_from_box() {
        // Sanity: filtro real é dispatched (não silenciosamente colapsa). Em
        // mid-frequency content (brush atlas radial soft falloff), Lanczos3
        // produz pixels diferentes de Triangle no nível 1.
        let png = brush_atlas_256_r8();
        let src = fixture_to_rgba(&png);
        let box_chain = generate_mip_chain(&src, MipFilter::Box, Some(2));
        let lanczos_chain = generate_mip_chain(&src, MipFilter::Lanczos, Some(2));
        let box_level1 = &box_chain[1];
        let lanczos_level1 = &lanczos_chain[1];
        assert_eq!(box_level1.dimensions(), lanczos_level1.dimensions());
        assert_ne!(
            box_level1.as_raw(),
            lanczos_level1.as_raw(),
            "Box vs Lanczos3 deveriam produzir pixels distintos em content radial"
        );
    }

    #[test]
    fn generate_chain_intra_machine_determinism() {
        let png = brush_atlas_256_r8();
        let src = fixture_to_rgba(&png);
        let a = generate_mip_chain(&src, MipFilter::Lanczos, None);
        let b = generate_mip_chain(&src, MipFilter::Lanczos, None);
        assert_eq!(a.len(), b.len());
        for (level, (la, lb)) in a.iter().zip(b.iter()).enumerate() {
            assert_eq!(
                la.as_raw(),
                lb.as_raw(),
                "level {level}: intra-machine byte-identity violation (HR-6)"
            );
        }
    }

    #[test]
    fn generate_chain_non_power_of_two_steps() {
        // 100×80 source → 50×40 → 25×20 → 12×10 → 6×5 → 3×2 → 1×1 = 7 levels.
        let src = RgbaImage::new(100, 80);
        let chain = generate_mip_chain(&src, MipFilter::Box, None);
        let dims: Vec<_> = chain.iter().map(|l| l.dimensions()).collect();
        assert_eq!(
            dims,
            vec![(100, 80), (50, 40), (25, 20), (12, 10), (6, 5), (3, 2), (1, 1)]
        );
    }

    #[test]
    fn generate_chain_1x1_source_has_single_level() {
        let src = RgbaImage::new(1, 1);
        let chain = generate_mip_chain(&src, MipFilter::Point, None);
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].dimensions(), (1, 1));
    }

    #[test]
    fn mip_filter_maps_to_image_filter_canonical() {
        assert_eq!(MipFilter::Box.to_image_filter(), FilterType::Triangle);
        assert_eq!(MipFilter::Lanczos.to_image_filter(), FilterType::Lanczos3);
        assert_eq!(MipFilter::Point.to_image_filter(), FilterType::Nearest);
    }
}

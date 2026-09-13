//! Testes de `schema.rs` — irmão por tecto de LOC (`workspace_src_files_under_loc_cap`).
//!
//! Corte mecânico: o `mod tests` saiu inteiro, verbatim, do ficheiro que o continha.

use super::*;
use ph2d_color::SrgbRgba;

/// Audit-8 Lens M (2026-05-26): boundary check for the J-3 walker.
/// Without these, a refactor that loosens the per-buffer dimension
/// check regresses an OOM defence silently.
#[test]
fn validate_dimensions_v1_accepts_exact_max_raster() {
    let buf = ImageBufferSrgbV1 {
        width: ph2d_imageio::MAX_RASTER_DIMENSION,
        height: ph2d_imageio::MAX_RASTER_DIMENSION,
        pixels: Vec::new(),
        color_profile: ColorProfileV1::Srgb,
    };
    let content = Ph2dContentV1::Flat(buf);
    assert!(validate_v1_inner_versions(&content).is_ok());
}

#[test]
fn validate_dimensions_v1_rejects_above_max_raster() {
    let buf = ImageBufferSrgbV1 {
        width: ph2d_imageio::MAX_RASTER_DIMENSION + 1,
        height: 1,
        pixels: Vec::new(),
        color_profile: ColorProfileV1::Srgb,
    };
    let content = Ph2dContentV1::Flat(buf);
    assert!(matches!(
        validate_v1_inner_versions(&content),
        Err(Error::DimensionExceedsLimit)
    ));
}

#[test]
fn validate_dimensions_v1_rejects_height_above_max_raster() {
    let buf = ImageBufferSrgbV1 {
        width: 1,
        height: ph2d_imageio::MAX_RASTER_DIMENSION + 1,
        pixels: Vec::new(),
        color_profile: ColorProfileV1::Srgb,
    };
    let content = Ph2dContentV1::Flat(buf);
    assert!(matches!(
        validate_v1_inner_versions(&content),
        Err(Error::DimensionExceedsLimit)
    ));
}

/// Pixel field unused in the assertion — silence warning.
#[allow(dead_code)]
fn _silence(_: SrgbRgba) {}

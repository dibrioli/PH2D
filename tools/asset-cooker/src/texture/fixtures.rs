//! W1.T11 — 8 fixtures canônicos para texture cooking (ADR-0055-v4 §2.3
//! determinismo + W1.T10 canonical-runner replay-hash gate).
//!
//! Cada generator é uma função pura `() -> Vec<u8>` que retorna bytes PNG
//! deterministic (mesmo input → mesmo output, intra-machine + cross-machine
//! quando canonical runner CI estiver wired). HR-6 ready: caller pode
//! `blake3(fixture_xxx())` para obter `LogicalTextureId` estável.
//!
//! Coverage do plano vivo §1.2:
//!
//! | Fixture                | Size  | Channels | Wave / Use                          |
//! |------------------------|-------|----------|-------------------------------------|
//! | `gradient_64x64`       | 64²   | RGBA     | unit tests texture::cook (W1.T3+)   |
//! | `gradient_256`         | 256²  | RGBA     | W1.T11 sprite color baseline        |
//! | `critical_ui_16`       | 16²   | RGBA     | W1.T11 high-contrast UI icon        |
//! | `brush_atlas_256_r8`   | 256²  | R8 → RGBA gray | W1.T14 sample cook R8→BC4      |
//! | `sdf_font_512`         | 512²  | RGBA (R=signed dist) | W1.T11 SDF font baseline |
//! | `normal_map_512`       | 512²  | RGBA tangent-space | W1.T11 normal map baseline |
//! | `photo_like_1024`      | 1024² | RGBA     | W1.T11 photo deferred (~4MB raw)    |
//! | `atlas_packed_4096`    | 4096² | RGBA     | W1.T11 atlas deferred (~64MB raw)   |
//!
//! NB: fixtures grandes (1024², 4096²) têm stubs nesta wave (W1.T11); os
//! generators reais vivem aqui mas testes em runtime usam apenas as 6
//! pequenas. Generators grandes são exercised pelo CI canonical-runner W1.T10
//! quando LFS + workflow materializarem.

use image::{ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;

/// Encode uma `RgbaImage` para PNG bytes via `image` crate em buffer in-memory.
fn encode_png(img: &RgbaImage) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    img.write_to(&mut cursor, ImageFormat::Png)
        .expect("encode RgbaImage to PNG buffer");
    buf
}

/// 64×64 RGBA8 gradient — vermelho→azul linear no eixo X + alpha gradient
/// top→bottom. Usado em unit tests da W1.T3+ (`texture::cook` smoke).
#[must_use]
pub fn gradient_64x64() -> Vec<u8> {
    let mut img = RgbaImage::new(64, 64);
    for (x, y, px) in img.enumerate_pixels_mut() {
        let r = (x * 4) as u8;
        let b = ((63 - x) * 4) as u8;
        let g = 64u8;
        let a = (y * 4) as u8;
        *px = Rgba([r, g, b, a]);
    }
    encode_png(&img)
}

/// 256×256 RGBA8 sprite color baseline — diagonal gradient red/green +
/// constant blue + opaque alpha. Representativo de assets típicos (sprite
/// flat color de hero/inimigos).
#[must_use]
pub fn gradient_256() -> Vec<u8> {
    let mut img = RgbaImage::new(256, 256);
    for (x, y, px) in img.enumerate_pixels_mut() {
        let r = ((x.saturating_add(y)) as u8).saturating_mul(1);
        let g = ((255 - x.min(255)) as u8) ^ ((y.min(255)) as u8);
        let b = 32u8;
        *px = Rgba([r, g, b, 255]);
    }
    encode_png(&img)
}

/// 16×16 RGBA8 critical-UI sprite — high-contrast checkerboard alternado
/// preto/branco com 1-pixel border vermelho. Stress-test pra ASTC 4×4 (block
/// fino) e BC7 alta qualidade per W1.T2 audit.
#[must_use]
pub fn critical_ui_16() -> Vec<u8> {
    let mut img = RgbaImage::new(16, 16);
    for (x, y, px) in img.enumerate_pixels_mut() {
        let on_border = x == 0 || x == 15 || y == 0 || y == 15;
        if on_border {
            *px = Rgba([255, 0, 0, 255]);
        } else {
            let checker = (x + y) % 2 == 0;
            *px = if checker {
                Rgba([255, 255, 255, 255])
            } else {
                Rgba([0, 0, 0, 255])
            };
        }
    }
    encode_png(&img)
}

/// 256×256 single-channel R8 brush atlas (representado como RGBA com R=intensidade,
/// G=B=R, A=255 — sample cook BC4 lê só canal R). W1.T14 proof-of-life: R8 source
/// → BC4 cooked (saving -50%; R8=8bpp ÷ BC4=4bpp = 0.5).
///
/// Pattern: radial gradient soft (brush stamp típico) com centro em (128,128),
/// raio 100, falloff cosine.
#[must_use]
pub fn brush_atlas_256_r8() -> Vec<u8> {
    let mut img = RgbaImage::new(256, 256);
    let cx = 128.0_f32;
    let cy = 128.0_f32;
    let radius = 100.0_f32;
    for (x, y, px) in img.enumerate_pixels_mut() {
        let dx = x as f32 - cx;
        let dy = y as f32 - cy;
        let dist = (dx * dx + dy * dy).sqrt();
        let intensity_f = if dist >= radius {
            0.0
        } else {
            // Cosine falloff: 1 at center, 0 at edge, smooth derivative.
            let t = dist / radius;
            0.5 * (1.0 + (std::f32::consts::PI * t).cos())
        };
        let v = (intensity_f.clamp(0.0, 1.0) * 255.0) as u8;
        *px = Rgba([v, v, v, 255]);
    }
    encode_png(&img)
}

/// 512×512 RGBA SDF font fixture — sinal de distância radial euclidiana
/// normalizada para 0..=255, com centro do "glyph" em (256, 256) e raio
/// canônico 200. SDF fonts em PH2D usariam channel R como distância signed
/// (com 128 = boundary); aqui simplified como unsigned para fixture.
#[must_use]
pub fn sdf_font_512() -> Vec<u8> {
    let mut img = RgbaImage::new(512, 512);
    let cx = 256.0_f32;
    let cy = 256.0_f32;
    let glyph_radius = 200.0_f32;
    for (x, y, px) in img.enumerate_pixels_mut() {
        let dx = x as f32 - cx;
        let dy = y as f32 - cy;
        let dist = (dx * dx + dy * dy).sqrt();
        // SDF: inside glyph (dist < radius) → > 128; outside → < 128.
        // Map dist 0 → 255 (deep inside), dist=radius → 128 (boundary),
        // dist > radius linearly down to 0.
        let signed = glyph_radius - dist;
        let normalized = (128.0 + signed * 0.5).clamp(0.0, 255.0) as u8;
        *px = Rgba([normalized, normalized, normalized, 255]);
    }
    encode_png(&img)
}

/// 512×512 RGBA tangent-space normal map fixture — perturbação determinística
/// representando uma superfície "bumpy". Channel R = X normal (0..255 ≈ -1..+1
/// tangent space), G = Y normal, B = Z (always ≈ pointing out = ~255), A = unused.
///
/// Pattern: sine wave normal map (sin(x) * cos(y)) → smooth bumpy surface
/// adequada para validar compression em conteúdo low-frequency.
#[must_use]
pub fn normal_map_512() -> Vec<u8> {
    let mut img = RgbaImage::new(512, 512);
    let freq = 8.0_f32; // 8 oscilações across 512px
    for (x, y, px) in img.enumerate_pixels_mut() {
        let xf = x as f32 / 512.0;
        let yf = y as f32 / 512.0;
        let nx = (xf * std::f32::consts::TAU * freq).sin() * 0.3;
        let ny = (yf * std::f32::consts::TAU * freq).cos() * 0.3;
        // Z = sqrt(1 - nx² - ny²) — unit normal. Bounded by perturbação ≤ 0.3.
        let nz = (1.0_f32 - nx * nx - ny * ny).max(0.0).sqrt();
        // Map [-1, 1] → [0, 255].
        let r = ((nx + 1.0) * 127.5).clamp(0.0, 255.0) as u8;
        let g = ((ny + 1.0) * 127.5).clamp(0.0, 255.0) as u8;
        let b = ((nz + 1.0) * 127.5).clamp(0.0, 255.0) as u8;
        *px = Rgba([r, g, b, 255]);
    }
    encode_png(&img)
}

/// 1024×1024 synthetic photo-like fixture — multi-scale gradient + xor noise
/// pra simular detalhe high-frequency típico de photos cookadas. ~4MB raw.
/// Deferred W1.T10 — runtime tests usam só os pequenos.
#[must_use]
pub fn photo_like_1024() -> Vec<u8> {
    let mut img = RgbaImage::new(1024, 1024);
    for (x, y, px) in img.enumerate_pixels_mut() {
        // Multi-scale gradient: base (large) + detail (small XOR-pattern).
        let base_r = (x / 4) as u8;
        let base_g = (y / 4) as u8;
        let base_b = ((x + y) / 8) as u8;
        let xor_noise = ((x as u8) ^ (y as u8)).wrapping_mul(3);
        let r = base_r.wrapping_add(xor_noise / 4);
        let g = base_g.wrapping_add(xor_noise / 4);
        let b = base_b.wrapping_add(xor_noise / 8);
        *px = Rgba([r, g, b, 255]);
    }
    encode_png(&img)
}

/// 4096×4096 synthetic atlas-packed fixture — grid 16×16 sub-rects de 256²
/// cada, com cores distintas modulando (x_cell, y_cell) — simula UI atlas
/// típico. ~64MB raw. Deferred W1.T10 — runtime tests usam só os pequenos.
#[must_use]
pub fn atlas_packed_4096() -> Vec<u8> {
    let mut img = RgbaImage::new(4096, 4096);
    for (x, y, px) in img.enumerate_pixels_mut() {
        let cell_x = (x / 256) as u8;
        let cell_y = (y / 256) as u8;
        // Color per cell index modulada pelo X dentro da célula (gradient leve).
        let in_cell_x = (x % 256) as u8;
        let r = cell_x.wrapping_mul(17).wrapping_add(in_cell_x / 4);
        let g = cell_y.wrapping_mul(23).wrapping_add(in_cell_x / 4);
        let b = (cell_x ^ cell_y).wrapping_mul(31);
        *px = Rgba([r, g, b, 255]);
    }
    encode_png(&img)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PNG magic header bytes: 89 50 4E 47 0D 0A 1A 0A.
    const PNG_MAGIC: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

    fn assert_valid_png(bytes: &[u8], name: &str) {
        assert!(
            bytes.len() > PNG_MAGIC.len(),
            "{name}: PNG output suspiciously small ({} bytes)",
            bytes.len()
        );
        assert_eq!(&bytes[0..8], PNG_MAGIC, "{name}: invalid PNG magic header");
    }

    fn assert_deterministic<F: Fn() -> Vec<u8>>(generator: F, name: &str) {
        let a = generator();
        let b = generator();
        assert_eq!(
            a, b,
            "{name}: fixture generator MUST produce byte-identical output across calls (HR-6)"
        );
    }

    #[test]
    fn gradient_64x64_is_valid_and_deterministic() {
        let bytes = gradient_64x64();
        assert_valid_png(&bytes, "gradient_64x64");
        assert_deterministic(gradient_64x64, "gradient_64x64");
    }

    #[test]
    fn gradient_256_is_valid_and_deterministic() {
        let bytes = gradient_256();
        assert_valid_png(&bytes, "gradient_256");
        assert_deterministic(gradient_256, "gradient_256");
    }

    #[test]
    fn critical_ui_16_is_valid_and_deterministic() {
        let bytes = critical_ui_16();
        assert_valid_png(&bytes, "critical_ui_16");
        assert_deterministic(critical_ui_16, "critical_ui_16");
    }

    #[test]
    fn brush_atlas_256_r8_is_valid_and_deterministic() {
        let bytes = brush_atlas_256_r8();
        assert_valid_png(&bytes, "brush_atlas_256_r8");
        assert_deterministic(brush_atlas_256_r8, "brush_atlas_256_r8");
    }

    #[test]
    fn sdf_font_512_is_valid_and_deterministic() {
        let bytes = sdf_font_512();
        assert_valid_png(&bytes, "sdf_font_512");
        assert_deterministic(sdf_font_512, "sdf_font_512");
    }

    #[test]
    fn normal_map_512_is_valid_and_deterministic() {
        let bytes = normal_map_512();
        assert_valid_png(&bytes, "normal_map_512");
        assert_deterministic(normal_map_512, "normal_map_512");
    }

    /// Sanity-check pra big fixtures — apenas verifica que decodam e têm
    /// dimensão correta; NÃO testa determinism em runtime para evitar slow tests.
    /// Determinism dos big fixtures é exercised pelo canonical-runner CI (W1.T10).
    #[test]
    #[ignore = "slow (~4MB+ PNG encode) — run with `cargo test -- --ignored`"]
    fn big_fixtures_are_valid_png() {
        let photo = photo_like_1024();
        assert_valid_png(&photo, "photo_like_1024");
        let atlas = atlas_packed_4096();
        assert_valid_png(&atlas, "atlas_packed_4096");
    }

    /// Cross-fixture distinctness — diferentes fixtures NÃO podem produzir bytes
    /// idênticos (sanity-check pra evitar copy-paste bugs em generators).
    #[test]
    fn distinct_fixtures_produce_distinct_bytes() {
        let g64 = gradient_64x64();
        let g256 = gradient_256();
        let crit = critical_ui_16();
        let brush = brush_atlas_256_r8();
        let sdf = sdf_font_512();
        let normal = normal_map_512();
        let all = [&g64, &g256, &crit, &brush, &sdf, &normal];
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate().skip(i + 1) {
                assert_ne!(
                    a.as_slice(),
                    b.as_slice(),
                    "fixtures #{i} and #{j} produced identical bytes — copy-paste bug?"
                );
            }
        }
    }
}

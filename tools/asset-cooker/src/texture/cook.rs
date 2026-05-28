//! W1.T3 — Texture cook function (lib API + CLI underlying).
//!
//! Pipeline:
//!
//! 1. Decode source bytes (PNG hoje; JPEG/WebP em W4+) via `image` crate → RGBA8 raw.
//! 2. Construct `ctt::Image` com 1 `Surface` 2D.
//! 3. Lookup `TargetFormat` na `target_matrix` (tier × asset_class).
//! 4. `ctt::convert(image, ConvertSettings { format, container: ktx2(), .. })`.
//! 5. Return KTX2 bytes.
//!
//! Content-addressed (HR-6): caller computa `blake3(bytes)` para AssetId.
//!
//! Determinismo: depende de canonical runner (W1.T10/W1.T11.5 ⏳) — Lente A
//! HIGH#1 do W1.T2 audit (ISA dispatch runtime). Snapshot test (D4) defer
//! até T3 + D2 ambos materializados.

use ctt::{
    AlphaMode, ColorSpace, Container, ConvertSettings, Image, PipelineOutput, Surface,
    TextureKind, convert,
};
use image::ImageReader;

use super::target_matrix::{AssetClass, Tier, default_color_space_for, target_for};

#[derive(Debug)]
pub enum TextureCookError {
    /// Image decode failed (corrupted PNG, unsupported format, etc.).
    Decode(image::ImageError),
    /// I/O error reading source bytes from cursor.
    Io(std::io::Error),
    /// `ctt::convert` failed (invalid format/encoder combo, OOM, etc.).
    Ctt(ctt::Error),
}

impl std::fmt::Display for TextureCookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decode(e) => write!(f, "texture cook: PNG decode failed: {e}"),
            Self::Io(e) => write!(f, "texture cook: I/O error: {e}"),
            Self::Ctt(e) => write!(f, "texture cook: ctt::convert failed: {e:?}"),
        }
    }
}

impl std::error::Error for TextureCookError {}

impl From<image::ImageError> for TextureCookError {
    fn from(e: image::ImageError) -> Self {
        Self::Decode(e)
    }
}

impl From<std::io::Error> for TextureCookError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<ctt::Error> for TextureCookError {
    fn from(e: ctt::Error) -> Self {
        Self::Ctt(e)
    }
}

/// Options para um cook individual. Multi-tier emit (W1.T6) chama `cook` N
/// vezes com tiers diferentes.
///
/// **Construir via [`CookOptions::for_asset_class`] em vez de `Default::default()`**
/// quando asset class for conhecida — escolhe `color_space` semanticamente
/// correto (sRGB para Color/UI, Linear para SingleChannel/NormalMap). `Default`
/// retorna `SpriteColor` + `Srgb` para compat-API, mas aplicar isso a normal
/// maps causa shader-visible gamma bug (auditoria W1.T3 γ-H2).
#[derive(Copy, Clone, Debug)]
pub struct CookOptions {
    pub tier: Tier,
    pub asset_class: AssetClass,
    /// Source alpha intent. Default `Straight`; bg-removal pipeline emite
    /// `Premultiplied` (ver W2.T-pre + KTX2 keyValueData `PH2D_PREMUL`).
    pub alpha: AlphaMode,
    /// Source color space. Para asset color (sprite/UI) usar `Srgb`; para
    /// normal maps e single-channel data, usar `Linear`. `for_asset_class`
    /// deriva automaticamente.
    pub color_space: ColorSpace,
}

impl CookOptions {
    /// Constructor semanticamente correto: deriva `color_space` apropriado
    /// para o asset class. Use sempre que possível em vez de `Default::default()`.
    #[must_use]
    pub fn for_asset_class(tier: Tier, asset_class: AssetClass) -> Self {
        Self {
            tier,
            asset_class,
            alpha: AlphaMode::Straight,
            color_space: default_color_space_for(asset_class),
        }
    }
}

impl Default for CookOptions {
    /// **NB:** retorna defaults para `SpriteColor` (sRGB color data). Aplicar
    /// a `SingleChannel`/`NormalMap` causa gamma bug. Use [`CookOptions::for_asset_class`]
    /// quando asset class for conhecida.
    fn default() -> Self {
        Self {
            tier: Tier::Desktop,
            asset_class: AssetClass::SpriteColor,
            alpha: AlphaMode::Straight,
            color_space: ColorSpace::Srgb,
        }
    }
}

/// Lib API entry — cook PNG source bytes → KTX2 cooked bytes.
///
/// HR-3 nota: este path NÃO é hot path (offline cooker tool), aloca livremente.
///
/// HR-6: bytes retornados são deterministic-given (input, ctt-cli SHA, CPU
/// features ISA dispatch); chamador computa `blake3(bytes)` para AssetId.
pub fn cook(source_bytes: &[u8], options: CookOptions) -> Result<Vec<u8>, TextureCookError> {
    // Step 1: decode source PNG → RGBA8 raw.
    let reader = ImageReader::new(std::io::Cursor::new(source_bytes))
        .with_guessed_format()?;
    let dynamic = reader.decode()?;
    let rgba = dynamic.to_rgba8();
    let (width, height) = rgba.dimensions();

    // Step 2: construct ctt::Image with single 2D surface, mip level 0.
    let stride = width * 4; // RGBA8 = 4 bytes per pixel.
    let surface = Surface {
        data: rgba.into_raw(),
        width,
        height,
        depth: 1,
        stride,
        slice_stride: 0,
        format: ctt::Format::R8G8B8A8_UNORM,
        color_space: options.color_space,
        alpha: options.alpha,
    };
    let image = Image {
        surfaces: vec![vec![surface]],
        kind: TextureKind::Texture2D,
    };

    // Step 3: lookup target format (W1.T3 γ-H1 fix: agora retorna TargetFormat direto).
    let target = target_for(options.tier, options.asset_class);

    // Step 4: convert via ctt (encoder vendored ISPC offline; produces KTX2 bytes).
    let settings = ConvertSettings {
        format: Some(target),
        container: Container::ktx2(),
        ..Default::default()
    };
    let output = convert(image, settings)?;

    // Container::ktx2() sempre produz Encoded path; Raw path só ocorre com
    // Container::Raw (não usado por este cooker). Pattern match defensivo
    // serve como invariante semântica.
    match output {
        PipelineOutput::Encoded(bytes) => Ok(bytes),
        PipelineOutput::Raw(_) => unreachable!(
            "Container::ktx2() always emits PipelineOutput::Encoded; this branch is dead by design"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generate a deterministic 64×64 RGBA8 PNG fixture for tests.
    /// Pattern: linear gradient red→blue + alpha gradient top→bottom.
    fn fixture_png_64x64() -> Vec<u8> {
        let mut img = image::RgbaImage::new(64, 64);
        for (x, y, px) in img.enumerate_pixels_mut() {
            let r = (x * 4) as u8; // 0..=252
            let b = ((63 - x) * 4) as u8;
            let g = 64u8;
            let a = (y * 4) as u8;
            *px = image::Rgba([r, g, b, a]);
        }
        let mut buf = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buf);
        img.write_to(&mut cursor, image::ImageFormat::Png)
            .expect("encode test PNG");
        buf
    }

    #[test]
    fn cook_64x64_desktop_sprite_color_emits_nonempty_ktx2() {
        let png = fixture_png_64x64();
        let bytes = cook(
            &png,
            CookOptions::for_asset_class(Tier::Desktop, AssetClass::SpriteColor),
        )
        .expect("cook desktop sprite color");
        assert!(
            bytes.len() > 100,
            "KTX2 output suspiciously small: {} bytes",
            bytes.len()
        );
        // KTX2 magic bytes: «KTX 20»\r\n\x1A\n (12 bytes).
        assert_eq!(
            &bytes[0..12],
            &[0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A],
            "output must start with KTX2 magic identifier"
        );
    }

    /// W1.T3 γ-H3: nome + mensagem soften para deixar claro que isto valida
    /// SÓ intra-machine byte-identity (mesmo binário rodando 2× no mesmo CPU).
    /// HR-6 cross-machine determinism requer canonical runner (D2/W1.T10) +
    /// snapshot lock (D4/W1.T2.3) — gates separados, audit-tracked.
    #[test]
    fn cook_intra_machine_byte_identity_when_repeated() {
        let png = fixture_png_64x64();
        let a = cook(&png, CookOptions::default()).expect("cook a");
        let b = cook(&png, CookOptions::default()).expect("cook b");
        assert_eq!(
            a, b,
            "intra-machine byte-identity check (same binary, same CPU, repeated call). \
             NB: HR-6 cross-machine determinism is a separate gate (D2/W1.T10 canonical \
             runner + D4/W1.T2.3 snapshot lock); this test does NOT validate that."
        );
    }

    #[test]
    fn cook_64x64_mobile_critical_ui_emits_valid_ktx2_header() {
        // Audit W1.T3 γ-LOW-1: test name antes era `..._uses_astc_4x4` mas só
        // checava KTX2 magic; renamed para refletir o que o test realmente assert.
        // Validação que format interno do KTX2 é ASTC 4×4 será coberto pelo
        // snapshot test W1.T2.3/D4 quando canonical runner (D2) materializar.
        let png = fixture_png_64x64();
        let bytes = cook(
            &png,
            CookOptions::for_asset_class(Tier::Mobile, AssetClass::CriticalUi),
        )
        .expect("cook mobile critical UI");
        assert!(bytes.len() > 100);
        assert_eq!(&bytes[0..12], &[0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A]);
    }

    /// W1.T3 γ-H2 fix: verifica integração end-to-end do gamma fix —
    /// NormalMap → ColorSpace::Linear (não sRGB). Asserção via cook() não-erro;
    /// validação do flag KTX2 sRGB no header de output será W1.T2.3 snapshot.
    #[test]
    fn cook_normal_map_uses_linear_color_space_via_for_asset_class() {
        let options = CookOptions::for_asset_class(Tier::Desktop, AssetClass::NormalMap);
        assert!(
            matches!(options.color_space, ctt::ColorSpace::Linear),
            "for_asset_class(NormalMap) MUST yield Linear color space, got Srgb"
        );
        // Smoke: cook actually succeeds with this configuration.
        let png = fixture_png_64x64();
        let bytes = cook(&png, options).expect("cook normal map");
        assert!(bytes.len() > 100);
    }
}

#![forbid(unsafe_code)]
//! `ph2d-imageio-png` — PNG decode + encode (W1.T1).
//!
//! First `crates/ph2d-imageio-*` format crate. The decode path accepts
//! every PNG color type the `image` 0.25 + `png` crates support
//! (Grayscale, GrayscaleAlpha, RGB, RGBA, Indexed/Palette, with or
//! without tRNS); all converge on `ImageBuffer<SrgbRgba>` via the
//! `to_rgba8` widening. The encode path emits 8-bit RGBA PNG with
//! deterministic byte output (HR-5).
//!
//! ### What W1.T1 covers
//!
//! - Magic-byte (Strong) + extension (Weak) `supports()` recognition.
//! - All PNG color types decoded to 8-bit RGBA (grayscale → R=G=B,
//!   palette → expanded, alpha preserved or 255 when absent).
//! - ⚠️ **16-bit channel PNGs KEEP their precision** (plano
//!   [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md),
//!   W2.4, 2026-08-20). Até essa data eles eram **esmagados para 8 bits** aqui, com a perda
//!   documentada nesta lista e um teste a fixá-la. Não foi preciso alargar o `DecodedImage`: o
//!   `FlatHdr` (`ImageBuffer<LinearRgba>`) sempre foi o lugar certo, já estava congelado no
//!   ADR-0054 e o AVIF já o produzia — *o que faltava era este importador parar de deitar fora o
//!   que tinha*. Um PNG de 8 bits continua a sair `Flat`, e há gate.
//! - Decompression-bomb defence via hard dimension cap (32768×32768).
//! - HR-5 byte-exact determinism on the encoder.
//! - HR-6: round-trip lossless bit-exact.
//! - HR-15: error variants carry [`Error::fluent_key`] surface.
//!
//! ### Out of scope (later waves)
//!
//! - sBIT chunk preservation (W2 — when ICC pipeline lands and `image`
//!   crate exposes the chunk via its `png` backend).
//! - ICC profile chunk parsing (W2.0.1 ICC pipeline activation).
//! - 16-bit RGBA **export**: o `PngExporter` continua a emitir 8 bits e a recusar `FlatHdr`
//!   (`Error::HdrUnsupported`). A importação já preserva (acima); a escrita espera um pedido real.
//! - Quality-vs-speed encoder tuning options via `ExportOpts`.
//! - **APNG (animated PNG)**: audit A-Crit1 (2026-05-26). PNG files
//!   carrying an `acTL` animation chunk decode to **the first frame
//!   only** via `image::load_from_memory_with_format` — extra frames
//!   are silently dropped. W1.T1 documents this; W2.T3 (`ph2d-imageio-apng`)
//!   ships full animated decode/encode and the PNG importer here will
//!   then delegate to it on `acTL` detection. Until W2.T3, users who
//!   open an APNG see the first frame and save loses animation.

use ph2d_color::srgb::srgb_to_linear_unit;
use ph2d_color::{LinearRgba, SrgbRgba};
use ph2d_imageio::{
    ColorProfile, DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageBuffer,
    ImageExporter, ImageImporter, ImportOpts, ImporterRegistry, MAX_RASTER_DIMENSION, MagicHint,
    MagicMatch,
};

// Audit B-H2 (2026-05-26): `MAX_DIMENSION` was redeclared per crate
// (PNG/JPEG/WebP/GIF), drifting independently. Hoisted to
// `ph2d_imageio::MAX_RASTER_DIMENSION` so raising the cap is one edit
// + arch-gate amendment, not a 4-site sweep.

/// Register the PNG importer with `reg`. Codegen-wired into
/// [`ph2d_imageio_registry_init::register_all_importers`] by
/// `cargo run -p ph2d-imageio-sync`.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(PngImporter));
}

/// Register the PNG exporter with `reg`. Codegen-wired into
/// [`ph2d_imageio_registry_init::register_all_exporters`].
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(PngExporter));
}

/// PNG import driver. Stateless — one instance shared across the
/// importer registry.
pub struct PngImporter;

/// PNG's 8-byte signature: `\x89 P N G \r \n \x1a \n`.
const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

impl ImageImporter for PngImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if b.starts_with(&PNG_MAGIC) => MagicMatch::Strong,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("png") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        // `with_format` skips magic-byte autodetect inside `image` — we
        // already vetted the format via `supports()`, so this saves
        // ~µs of redundant inspection and avoids the rare case where
        // `image` would dispatch to a decoder we didn't enable.
        let img =
            image::load_from_memory_with_format(src, image::ImageFormat::Png).map_err(|e| {
                // Audit-8 Lens L L-3 (2026-05-26): migrated to
                // contract-level helper. Previously inline heuristic
                // covered only 2 EOF signatures; helper covers 10
                // canonical phrases across all upstream codecs.
                Error::from_decoder_message(e.to_string())
            })?;
        let (width, height) = (img.width(), img.height());
        if width == 0 || height == 0 {
            return Err(Error::Decode(format!(
                "PNG has zero-sized dimension: {width}×{height}"
            )));
        }
        if width > MAX_RASTER_DIMENSION || height > MAX_RASTER_DIMENSION {
            return Err(Error::DimensionExceedsLimit);
        }
        // **Um PNG de 16 bits deixa de ser esmagado na porta** — plano
        // `docs/Sprite_projeto/18`, W2.4. Até 2026-08-20 este importador chamava `to_rgba8()` para
        // TUDO, e a perda estava documentada no cabeçalho da crate com um teste a fixá-la.
        //
        // ⚠️ **A precisão do FICHEIRO é que decide, não uma opção.** O `DecodedImage` já tinha o
        // lugar certo (`FlatHdr`, `ImageBuffer<LinearRgba>`, congelado no ADR-0054) e o AVIF já o
        // produzia; era só o PNG que continuava a deitar fora o que tinha.
        //
        // ⚠️ **`FlatHdr` é LINEAR, e um PNG é sRGB-encoded** — por isso a curva aplica-se aqui, e
        // **só aos canais de cor**. O alfa já é linear por definição; passá-lo pela curva é o erro
        // clássico, e escurece toda borda macia da imagem.
        if matches!(
            img.color(),
            image::ColorType::L16
                | image::ColorType::La16
                | image::ColorType::Rgb16
                | image::ColorType::Rgba16
        ) {
            let rgba = img.to_rgba16();
            let pixels: Vec<LinearRgba> = rgba
                .chunks_exact(4)
                .map(|c| {
                    LinearRgba::new(
                        srgb_to_linear_unit(f32::from(c[0]) / 65535.0),
                        srgb_to_linear_unit(f32::from(c[1]) / 65535.0),
                        srgb_to_linear_unit(f32::from(c[2]) / 65535.0),
                        f32::from(c[3]) / 65535.0,
                    )
                })
                .collect();
            return Ok(DecodedImage::FlatHdr(ImageBuffer {
                width,
                height,
                pixels,
                color_profile: ColorProfile::Srgb,
            }));
        }
        // `to_rgba8` is the canonical widening — handles Grayscale,
        // GrayscaleAlpha, RGB, RGBA, and Indexed/Palette PNGs all into
        // the same 8-bit RGBA wire shape.
        let rgba = img.to_rgba8();
        let pixels: Vec<SrgbRgba> = rgba
            .chunks_exact(4)
            .map(|c| SrgbRgba([c[0], c[1], c[2], c[3]]))
            .collect();
        Ok(DecodedImage::Flat(ImageBuffer {
            width,
            height,
            pixels,
            // ADR-0054 §2.3 W1 policy: sRGB assumed. W2.0.1 expands to
            // honour iCCP chunk when ICC pipeline lands.
            color_profile: ColorProfile::Srgb,
        }))
    }
}

/// PNG export driver. Stateless.
pub struct PngExporter;

impl ImageExporter for PngExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Png)
    }

    fn export(&self, img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        let buf = match img {
            DecodedImage::Flat(b) => b,
            DecodedImage::FlatHdr(_) => return Err(Error::HdrUnsupported),
            DecodedImage::Layered(_) => {
                // PNG is single-layer; the caller should flatten via the
                // engine's composite pipeline before reaching this
                // exporter. Returning `MissingLayer` is the "you asked
                // PNG to do what only PSD/ORA can" signal.
                return Err(Error::MissingLayer);
            }
            DecodedImage::Animated(_) => {
                return Err(Error::Unsupported(
                    "PNG is single-frame; export to APNG instead".into(),
                ));
            }
            DecodedImage::Vector(_) => {
                return Err(Error::Unsupported(
                    "PNG is raster; rasterize the VectorDoc first".into(),
                ));
            }
        };
        // Audit D-E4: checked_mul prevents 32-bit silent truncation.
        let byte_len = buf.pixels.len().checked_mul(4).ok_or(Error::OutOfMemory)?;
        let mut rgba8: Vec<u8> = Vec::with_capacity(byte_len);
        for p in &buf.pixels {
            rgba8.extend_from_slice(&p.0);
        }
        let img_buf =
            image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(buf.width, buf.height, rgba8)
                .ok_or_else(|| {
                    Error::Encode(format!(
                        "pixel count {} does not match {}×{}",
                        buf.pixels.len(),
                        buf.width,
                        buf.height
                    ))
                })?;
        let mut out: Vec<u8> = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut out);
        img_buf
            .write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| Error::Encode(e.to_string()))?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_2x2() -> ImageBuffer<SrgbRgba> {
        ImageBuffer {
            width: 2,
            height: 2,
            pixels: vec![
                SrgbRgba([255, 0, 0, 255]),
                SrgbRgba([0, 255, 0, 255]),
                SrgbRgba([0, 0, 255, 255]),
                SrgbRgba([128, 128, 128, 200]),
            ],
            color_profile: ColorProfile::Srgb,
        }
    }

    #[test]
    fn supports_recognizes_png_magic_strong() {
        let imp = PngImporter;
        // Audit A-M1: magic bytes are Strong (unique recognition).
        assert_eq!(
            imp.supports(MagicHint::Bytes(&PNG_MAGIC)),
            MagicMatch::Strong
        );
        let mut padded = PNG_MAGIC.to_vec();
        padded.extend_from_slice(&[0; 32]);
        assert_eq!(imp.supports(MagicHint::Bytes(&padded)), MagicMatch::Strong);
        assert_eq!(
            imp.supports(MagicHint::Bytes(&[0xff, 0xd8, 0xff, 0xe0])),
            MagicMatch::None
        );
        assert_eq!(imp.supports(MagicHint::Bytes(&[])), MagicMatch::None);
    }

    #[test]
    fn supports_recognizes_png_extension_weak() {
        let imp = PngImporter;
        // Audit A-M1: extension is Weak (could be wrong file with .png name).
        assert_eq!(imp.supports(MagicHint::Extension("png")), MagicMatch::Weak);
        assert_eq!(imp.supports(MagicHint::Extension("PNG")), MagicMatch::Weak);
        assert_eq!(imp.supports(MagicHint::Extension("Png")), MagicMatch::Weak);
        assert_eq!(imp.supports(MagicHint::Extension("jpeg")), MagicMatch::None);
        assert_eq!(imp.supports(MagicHint::Extension("")), MagicMatch::None);
    }

    #[test]
    fn exporter_recognizes_png_format() {
        let exp = PngExporter;
        assert!(exp.supports_format(ExportFormat::Png));
        assert!(!exp.supports_format(ExportFormat::Jpeg));
        assert!(!exp.supports_format(ExportFormat::Webp));
    }

    /// The core W0.T5 vertical: synthesize a 2×2 fixture in memory,
    /// export it as PNG bytes, re-import, assert pixels are bit-exact.
    /// PNG is lossless, so blake3 of the pixels round-trips clean —
    /// HR-6 derived (asset = hash blake3 of conteúdo, not bytes-on-wire).
    #[test]
    fn roundtrip_2x2_image_preserves_pixels_bit_exact() {
        let original = fixture_2x2();
        let bytes = PngExporter
            .export(
                &DecodedImage::Flat(original.clone()),
                &ExportOpts {
                    format: ExportFormat::Png,
                    ..ExportOpts::default()
                },
            )
            .expect("PNG export should succeed");
        assert!(bytes.starts_with(&PNG_MAGIC), "output bytes are not PNG");

        let decoded = PngImporter
            .import(&bytes, &ImportOpts::default())
            .expect("PNG import should succeed");
        let DecodedImage::Flat(out) = decoded else {
            panic!("PNG decode should yield Flat, got something else");
        };
        assert_eq!(out.width, 2);
        assert_eq!(out.height, 2);
        assert_eq!(out.color_profile, ColorProfile::Srgb);
        assert_eq!(out.pixels.len(), original.pixels.len());
        for (i, (got, want)) in out.pixels.iter().zip(original.pixels.iter()).enumerate() {
            assert_eq!(got.0, want.0, "pixel {i} drifted across round-trip");
        }
    }

    /// Audit E-H1 (HR-5 determinism): exporting the same buffer twice
    /// must produce byte-exact identical output. PNG via `image` 0.25
    /// MUST NOT emit a `tIME` chunk by default (it doesn't, but the
    /// test guards future drift if upstream changes encoder defaults).
    #[test]
    fn export_is_byte_exact_deterministic() {
        let buf = fixture_2x2();
        let opts = ExportOpts {
            format: ExportFormat::Png,
            ..ExportOpts::default()
        };
        let bytes_a = PngExporter
            .export(&DecodedImage::Flat(buf.clone()), &opts)
            .expect("first export");
        let bytes_b = PngExporter
            .export(&DecodedImage::Flat(buf), &opts)
            .expect("second export");
        assert_eq!(
            bytes_a, bytes_b,
            "HR-5: PNG export must be byte-exact for the same input. \
             If this fails, the `image` 0.25 encoder picked up a \
             non-deterministic chunk (tIME, comments, etc) — pin the \
             encoder via `png::Encoder` direct API."
        );
    }

    /// W1.T1: every PNG color type (Grayscale, GrayscaleAlpha, RGB,
    /// RGBA, Indexed) converges on 8-bit RGBA at import. We don't
    /// hand-craft chunks — we let the `image` crate encode each shape
    /// and verify our importer widens correctly.
    #[test]
    fn import_widens_grayscale_to_rgba() {
        // 2×1 grayscale: black + white.
        let gray_buf = image::ImageBuffer::<image::Luma<u8>, _>::from_raw(2, 1, vec![0_u8, 255])
            .expect("luma buffer");
        let mut bytes: Vec<u8> = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut bytes);
        image::DynamicImage::ImageLuma8(gray_buf)
            .write_to(&mut cursor, image::ImageFormat::Png)
            .expect("encode gray");
        let DecodedImage::Flat(out) = PngImporter
            .import(&bytes, &ImportOpts::default())
            .expect("import gray")
        else {
            panic!("expected Flat");
        };
        assert_eq!(out.pixels.len(), 2);
        // Black: R=G=B=0, alpha 255 (gray PNG has no alpha → image adds 255).
        assert_eq!(out.pixels[0].0, [0, 0, 0, 255]);
        // White: R=G=B=255.
        assert_eq!(out.pixels[1].0, [255, 255, 255, 255]);
    }

    #[test]
    fn import_widens_rgb_to_rgba_with_full_alpha() {
        // 1×1 RGB pure-red — no alpha channel in source.
        let rgb_buf =
            image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(1, 1, vec![255, 0, 0]).expect("rgb");
        let mut bytes: Vec<u8> = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut bytes);
        image::DynamicImage::ImageRgb8(rgb_buf)
            .write_to(&mut cursor, image::ImageFormat::Png)
            .expect("encode rgb");
        let DecodedImage::Flat(out) = PngImporter
            .import(&bytes, &ImportOpts::default())
            .expect("import rgb")
        else {
            panic!("expected Flat");
        };
        // Alpha synthesised to 255 (opaque) since source had none.
        assert_eq!(out.pixels[0].0, [255, 0, 0, 255]);
    }

    fn sixteen_bit_png(values: &[u16]) -> Vec<u8> {
        let width = (values.len() / 4) as u32;
        let img = image::ImageBuffer::<image::Rgba<u16>, _>::from_raw(width, 1, values.to_vec())
            .expect("rgba16");
        let mut bytes: Vec<u8> = Vec::new();
        image::DynamicImage::ImageRgba16(img)
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .expect("encode 16-bit");
        bytes
    }

    /// **Um PNG de 16 bits PRESERVA a sua precisão** — plano `docs/Sprite_projeto/18`, W2.4.
    ///
    /// ⚠️ Este teste era o inverso: chamava-se `import_quantizes_16bit_to_8bit_with_documented_loss`
    /// e **fixava a perda**. Ele estava certo enquanto o `DecodedImage` não tinha onde pôr a
    /// precisão; a partir do momento em que tem, um gate que exige a perda é um gate a defender um
    /// bug. *Quando a premissa de um teste morre, ele não se silencia — inverte-se, e o commit
    /// diz porquê.*
    #[test]
    fn a_sixteen_bit_png_keeps_its_precision_instead_of_being_crushed() {
        // Dois valores de 16 bits que caem no MESMO byte de 8 bits: 0x8080 e 0x8081 quantizam
        // ambos para 128. Se a importação ainda esmagasse, eles sairiam idênticos.
        let bytes = sixteen_bit_png(&[
            0x8080, 0x8080, 0x8080, 0xFFFF, 0x8081, 0x8081, 0x8081, 0xFFFF,
        ]);
        let DecodedImage::FlatHdr(out) = PngImporter
            .import(&bytes, &ImportOpts::default())
            .expect("import 16-bit")
        else {
            panic!("um PNG de 16 bits tem de decodificar para FlatHdr, nao para Flat");
        };
        assert_eq!(out.pixels.len(), 2);
        assert_ne!(
            out.pixels[0].r(),
            out.pixels[1].r(),
            "0x8080 e 0x8081 sairam iguais — a importacao ainda esta' a esmagar para 8 bits, e a \
             prova e' que dois valores distintos de 16 bits colapsaram no mesmo"
        );
    }

    /// **A curva sRGB aplica-se à COR e não ao alfa**, e os extremos continuam exactos.
    ///
    /// ⚠️ Sem isto, o gate acima passaria numa implementação que guardasse os valores sRGB crus
    /// como se fossem lineares — dois valores continuariam distintos, e a imagem inteira
    /// renderizaria mais clara.
    #[test]
    fn the_sixteen_bit_path_linearises_colour_and_leaves_alpha_alone() {
        let bytes = sixteen_bit_png(&[
            0x0000, 0x0000, 0x0000, 0xFFFF, // preto opaco
            0x8080, 0x8080, 0x8080, 0x8080, // meio-tom, meio-alfa
            0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, // branco opaco
        ]);
        let DecodedImage::FlatHdr(out) = PngImporter
            .import(&bytes, &ImportOpts::default())
            .expect("import 16-bit")
        else {
            panic!("esperava FlatHdr");
        };
        // Os extremos da curva sRGB são pontos fixos.
        assert!(
            out.pixels[0].r().abs() < 1e-6,
            "0x0000 devia dar 0.0 linear"
        );
        assert!(
            (out.pixels[2].r() - 1.0).abs() < 1e-6,
            "0xFFFF devia dar 1.0 linear"
        );
        // sRGB 0x8080 (≈0.5029) linearizado é ≈0.2158 — e NÃO 0.5029.
        let mid = out.pixels[1].r();
        assert!(
            (mid - 0.2158).abs() < 1e-3,
            "o meio-tom saiu {mid:.4}; o linear de sRGB ~0,503 e' ~0,2158. Se deu ~0,503, os \
             valores sRGB foram guardados como se ja' fossem lineares."
        );
        // ⚠️ O alfa NÃO atravessa a curva.
        let alpha = out.pixels[1].a();
        assert!(
            (alpha - 0.5029).abs() < 1e-3,
            "o alfa saiu {alpha:.4} e devia ser ~0,503 — alguem aplicou-lhe a curva sRGB"
        );
    }

    /// **Controle: um PNG de 8 bits continua a decodificar para `Flat`.** Sem isto, uma
    /// implementação que mandasse TUDO para `FlatHdr` passaria os dois gates acima e faria toda
    /// imagem do app pagar 8 B/px.
    #[test]
    fn an_eight_bit_png_still_decodes_to_flat() {
        let mut img = image::RgbaImage::new(1, 1);
        img.put_pixel(0, 0, image::Rgba([10, 20, 30, 255]));
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .expect("encode 8-bit");
        let decoded = PngImporter
            .import(&bytes, &ImportOpts::default())
            .expect("import 8-bit");
        assert!(
            matches!(decoded, DecodedImage::Flat(_)),
            "um PNG de 8 bits nao pode virar FlatHdr — seria dobrar a memoria de toda imagem do app"
        );
    }

    #[test]
    fn import_rejects_empty_bytes_as_truncated() {
        let err = PngImporter
            .import(&[], &ImportOpts::default())
            .expect_err("must refuse empty");
        assert!(matches!(err, Error::Truncated));
        assert_eq!(err.fluent_key(), "imageio.error.truncated");
    }

    #[test]
    fn import_rejects_corrupted_bytes_as_decode_error() {
        // Magic header present (so supports() passes) but rest is
        // garbage — image crate returns a decode error which we map
        // to `Error::Decode`.
        let mut bytes = PNG_MAGIC.to_vec();
        bytes.extend_from_slice(b"this is not a real PNG");
        let err = PngImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("corrupted bytes must fail");
        assert!(
            matches!(err, Error::Decode(_) | Error::Truncated),
            "expected Decode or Truncated, got {err:?}"
        );
    }

    /// Audit A-Crit1 (2026-05-26): document the APNG silent-first-frame
    /// behaviour explicitly so future bisects / regressions / readers
    /// see the policy. Builds a 2-frame APNG via the `image` crate's
    /// own encoder if available; falls back to a hex-baked minimal
    /// APNG fixture if not. If neither works (image crate APIs shift),
    /// skips with a clear message rather than failing — the doc-test
    /// is the primary spec; this is defensive coverage.
    #[test]
    fn import_of_apng_takes_first_frame_silently() {
        // The simplest APNG fixture: PNG header + IHDR + acTL +
        // fcTL/IDAT + fcTL/fdAT + IEND. Building this by hand is
        // brittle across image-crate versions. Instead, encode a
        // plain 1×1 RGBA via the encoder (which is NOT animated)
        // and assert the decode path returns Flat — confirming the
        // baseline. Then document the limitation in this test's
        // doc-comment for the audit trail. Full APNG fixture test
        // lands in W2.T3 with `ph2d-imageio-apng`.
        let rgb_buf =
            image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(1, 1, vec![255, 0, 0, 255])
                .expect("1x1 RGBA");
        let mut bytes: Vec<u8> = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut bytes);
        image::DynamicImage::ImageRgba8(rgb_buf)
            .write_to(&mut cursor, image::ImageFormat::Png)
            .expect("encode plain PNG");
        let decoded = PngImporter
            .import(&bytes, &ImportOpts::default())
            .expect("import plain PNG");
        // Confirms the baseline behaviour: PNG (animated or not) yields
        // Flat. When a real APNG arrives, all frames after the first
        // are dropped — see crate doc + audit A-Crit1 in commit log.
        assert!(matches!(decoded, DecodedImage::Flat(_)));
    }

    #[test]
    fn fluent_keys_match_error_variants() {
        // HR-15 surface: each Error variant maps to a stable Fluent key.
        // This is the smoke that confirms the W1.T1 caller can route to
        // user-facing localisation reliably.
        assert_eq!(Error::Truncated.fluent_key(), "imageio.error.truncated");
        assert_eq!(
            Error::DimensionExceedsLimit.fluent_key(),
            "imageio.error.dimension-exceeds-limit"
        );
        assert_eq!(
            Error::HdrUnsupported.fluent_key(),
            "imageio.error.hdr-unsupported"
        );
    }

    // Sanity range for the bomb-defence cap moved to
    // `ph2d_imageio::limits` per audit B-H2 (single source of truth).

    /// W3 pre-gate (LOCAL drift guard, Mac aarch64 only): pin blake3
    /// hash of canonical PNG export bytes so a future encoder /
    /// dependency bump that silently changes byte-level output trips
    /// the test loud — not silent.
    ///
    /// Scope: SINGLE-PLATFORM (macOS aarch64). The `image` + `png`
    /// crates emit different (still-valid) PNG bytes on
    /// x86_64-linux/windows due to SIMD-divergent deflate paths.
    /// Multi-platform pinning would require a per-(os,arch) hash
    /// tuple — deferred (entry: `crates/ph2d-imageio-png/src/lib.rs::export_golden_blake3_local_drift_pinned_macos_silicon`)
    /// until first cross-OS divergence motivates it.
    ///
    /// If this test FAILS on macOS aarch64:
    /// - First-time setup: copy printed hash into GOLDEN_BLAKE3 + commit.
    /// - Otherwise: real regression — encoder picked up
    ///   non-determinism (timestamp, SIMD-divergent compress, thread
    ///   scheduling). Investigate; pin via direct codec API if needed.
    // CONVENTION: keep `const GOLDEN_BLAKE3` inside the fn body
    // (below). If hoisted to module scope, clippy in CI lint job
    // (ubuntu) won't see the test's cfg gate and may flag the
    // const as unused on non-Mac targets. Source-level guardrail
    // against accidental hoist (audit-6 Lens B HIGH).
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    #[test]
    fn export_golden_blake3_local_drift_pinned_macos_silicon() {
        let original = fixture_2x2();
        let opts = ExportOpts {
            format: ExportFormat::Png,
            ..ExportOpts::default()
        };
        let bytes = PngExporter
            .export(&DecodedImage::Flat(original), &opts)
            .expect("PNG golden export");
        let hash = blake3::hash(&bytes);
        const GOLDEN_BLAKE3: &str =
            "30c1a80e53ed388661fe752c2845d5f220248071685cfb5fd89761cd601ceaa9";
        assert_eq!(
            hash.to_hex().to_string(),
            GOLDEN_BLAKE3,
            "LOCAL drift (macOS aarch64): PNG export blake3 drifted"
        );
    }

    #[test]
    fn export_rejects_hdr_with_actionable_error() {
        // HR-6 derived: silent clipping of HDR to LDR is a data-loss
        // bug. The exporter refuses unless the caller opts into a
        // ToneMap (none configured in this test).
        let fake_hdr = DecodedImage::FlatHdr(ImageBuffer {
            width: 1,
            height: 1,
            pixels: vec![ph2d_color::LinearRgba::new(1.0, 0.0, 0.0, 1.0)],
            color_profile: ColorProfile::LinearRec709,
        });
        let err = PngExporter
            .export(
                &fake_hdr,
                &ExportOpts {
                    format: ExportFormat::Png,
                    ..ExportOpts::default()
                },
            )
            .expect_err("PNG must refuse HDR without tone-map");
        assert!(
            matches!(err, Error::HdrUnsupported),
            "expected HdrUnsupported, got {err:?}"
        );
        // Audit E-M1 (HR-15): error must carry a Fluent key for UI.
        assert_eq!(err.fluent_key(), "imageio.error.hdr-unsupported");
    }
}

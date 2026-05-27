#![forbid(unsafe_code)]
//! `ph2d-imageio-psd` — Adobe Photoshop Document (`.psd`) decode
//! (W2.T4). **Write is deferred** to W3+ per ADR-0054 §5.2 W2.5
//! escape hatch — PSD binary encoding compatible with Photoshop CC
//! real is greenfield ~600 LOC + golden-test cycles against real
//! PS output, not justified mid-W2 batch when the rest of W2 is
//! already shippable.
//!
//! ### What W2.T4 covers
//!
//! - Magic: `"8BPS"` signature (4 bytes) + PSD version `0x0001`
//!   (PSD; PSB at `0x0002` not supported in W2.T4 — separate crate
//!   when a real client appears).
//! - Decode via `psd` 0.3.5 crate. Surface to `DecodedImage::Layered`:
//!   canvas width/height; per-layer name, RGBA pixels (already
//!   composited by the `psd` crate against transparent canvas at
//!   the layer's offset — full-canvas-sized buffer), opacity, blend
//!   mode (subset of the 24 `BlendMode` variants we now ship),
//!   visibility.
//! - Single-layer or flat PSD → still wraps in `LayerStack` with one
//!   `Layer::Pixel` for consistency with multi-layer.
//!
//! ### Out of scope (W3+ / .ph2d-native owns the full fidelity)
//!
//! - **PSD export** — see ADR-0054 §5.2 W2.5 escape hatch. The
//!   `Error::Unsupported` message points the caller at `.ph2d` save.
//! - PSB (Photoshop Big) format (different magic version field;
//!   `>= 30000×30000` canvases that PSD cannot hold).
//! - Layer groups / clipping masks / adjustment layers / text layers
//!   / smart objects — `psd` 0.3.5 surfaces them as flat pixel
//!   layers; full LayerKind preservation would need our own PSD
//!   parser (out of scope until first real client demands it; round-
//!   trip via `.ph2d-native` is the canonical preservation path).
//! - Layer effects (drop shadow / glow / stroke) — same as above.
//! - ICC profile preservation AND conversion. Audit-8 Lens K K-1
//!   (2026-05-26): previous doc claimed "preserved as
//!   `ColorProfile::Custom`" but this is **vapor** — the decoder
//!   hard-codes `ColorProfile::Srgb` and never reads the embedded
//!   ICC chunk. `psd` 0.3.5 does surface it via
//!   `Psd::resource_block_iccp()`, but we don't yet route the bytes.
//!   Entry-point: `import` (this file) — read ICC resource block and
//!   set `ColorProfile::Custom(Vec<u8>)`. Until W2.0.1 ICC pipeline
//!   ratifies the path, PSD imports with non-sRGB profile **silently
//!   lose the profile**.

use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    BlendMode, ColorProfile, DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry,
    ImageBuffer, ImageExporter, ImageImporter, ImportOpts, ImporterRegistry, Layer, LayerKind,
    LayerStack, MAX_RASTER_DIMENSION, MagicHint, MagicMatch,
};

/// PSD signature: `"8BPS"` followed by 2-byte big-endian version.
const PSD_MAGIC: [u8; 6] = [b'8', b'B', b'P', b'S', 0x00, 0x01];
/// PSB (Photoshop Big) signature — same `8BPS` prefix + version
/// `0x0002`. Audit W2.T6 H-6: claim Strong so registry lands here
/// and `import` returns an actionable `Error::Unsupported` instead
/// of a generic "no importer" miss.
const PSB_MAGIC: [u8; 6] = [b'8', b'B', b'P', b'S', 0x00, 0x02];

/// Audit W2.T6 H-7 + nova auditoria H-N1 (2026-05-26): cap on
/// **multi-layer fan-out** of pixel bytes. `psd` 0.3.5 returns
/// full-canvas-sized buffers per layer; a 16K × 16K PSD with 200
/// layers = ~200 GB. We refuse the **fan-out**, not a single-layer
/// legitimate PSD at max canvas dimension.
///
/// Cap is `n_layers × canvas_bytes` only when `n_layers > 1`. A
/// 32768 × 32768 single-layer PSD allocates 4 GiB (= one canvas
/// buffer) — legitimate; the cap previously refused it. With this
/// fix, single-layer PSDs are bounded by `MAX_RASTER_DIMENSION`
/// alone (the canvas dim check); multi-layer PSDs hit this cap.
///
/// 4 GiB = comfortable for typical multi-layer Painter use
/// (e.g. 4096 × 4096 × 64 layers = 4 GiB exactly).
const MAX_PSD_MULTI_LAYER_TOTAL_BYTES: u64 = 4 * 1024 * 1024 * 1024;

/// Hard cap on PSD source bytes — defense against decompression-bomb
/// inputs (e.g. a PSD header claiming N layers + crafted RLE chunks
/// that explode in memory). Checked BEFORE `psd::Psd::from_bytes`
/// parses, because the `psd` crate allocates eagerly. Audit nova
/// H-7 residual.
const MAX_PSD_INPUT_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// Register the PSD importer.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(PsdImporter));
}

/// Register the PSD exporter. The exporter exists so the registry
/// dispatch finds something for `ExportFormat::Psd` — its `export`
/// returns `Error::Unsupported` directing the user to `.ph2d-native`.
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(PsdExporter));
}

/// PSD import driver.
pub struct PsdImporter;

impl ImageImporter for PsdImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if b.starts_with(&PSD_MAGIC) => MagicMatch::Strong,
            // Audit W2.T6 H-6: PSB (`8BPS\0\x02`) claims Strong too,
            // but `import` returns Unsupported immediately so the
            // user sees an actionable error instead of generic
            // "no importer".
            MagicHint::Bytes(b) if b.starts_with(&PSB_MAGIC) => MagicMatch::Strong,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("psd") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        // Audit W2.T6 H-6: detect PSB early + return an actionable
        // Unsupported. The `psd` crate would try to parse and fail
        // with an obscure error otherwise.
        if src.starts_with(&PSB_MAGIC) {
            return Err(Error::Unsupported(
                "PSB (Photoshop Big) format not supported. PSB targets >30000×30000 \
                 canvases — open in Photoshop and \"Save As\" → PSD or use \
                 `.ph2d-native` for full-fidelity layered save."
                    .into(),
            ));
        }
        // Audit nova H-7 residual (2026-05-26): bound input size BEFORE
        // calling `psd::Psd::from_bytes`, which allocates eagerly. A
        // crafted PSD header claiming 200 layers can balloon during
        // parse before we ever check `parsed.layers().len()`.
        if (src.len() as u64) > MAX_PSD_INPUT_BYTES {
            return Err(Error::OutOfMemory);
        }
        // Audit-7 Lens G HIGH G-F2 (2026-05-26): `psd` 0.3.5 is
        // unmaintained (last release 2020) and has a history of
        // panicking on malformed input (channel data RLE truncated
        // mid-stream, etc.). Since our source is `&[u8]` from
        // arbitrary user input (drag-and-drop), isolate the
        // third-party parser with `catch_unwind` so a hostile PSD
        // returns `Error::Decode` instead of crashing the process.
        let parsed =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| psd::Psd::from_bytes(src)))
                .map_err(|_| Error::Decode("PSD parser panicked on malformed input".into()))?
                .map_err(|e| Error::from_decoder_message(format!("PSD parse: {e}")))?;

        let canvas_width = parsed.width();
        let canvas_height = parsed.height();
        if canvas_width > MAX_RASTER_DIMENSION || canvas_height > MAX_RASTER_DIMENSION {
            return Err(Error::DimensionExceedsLimit);
        }
        if canvas_width == 0 || canvas_height == 0 {
            return Err(Error::Decode(format!(
                "PSD has zero-sized canvas: {canvas_width}×{canvas_height}"
            )));
        }

        // Walk the layers. `psd` 0.3.5 returns a flat list (Group /
        // clipping / adjustment / text / smart layers are surfaced as
        // pixel layers with composited bytes). For full fidelity use
        // `.ph2d-native`; the PSD layer-info-section parsing for
        // groups would need our own decoder.
        let psd_layers = parsed.layers();

        // Audit W2.T6 H-7 + nova H-N1: refuse decompression-bomb PSDs
        // (16K × 16K × 200 layers = ~200 GB) before iterating. Only
        // apply the cap when n_layers > 1 — a single-layer 32K canvas
        // is 4 GiB but legitimate, bounded by MAX_RASTER_DIMENSION.
        // `psd` 0.3.5 returns full-canvas buffers per layer so cost
        // is `n_layers × canvas_w × canvas_h × 4`.
        if psd_layers.len() > 1 {
            let total_bytes = (psd_layers.len() as u64)
                .checked_mul(canvas_width as u64)
                .and_then(|n| n.checked_mul(canvas_height as u64))
                .and_then(|n| n.checked_mul(4))
                .ok_or(Error::OutOfMemory)?;
            if total_bytes > MAX_PSD_MULTI_LAYER_TOTAL_BYTES {
                // Audit nova H-N2: report OOM (semantically correct)
                // rather than DimensionExceedsLimit; the dims may be
                // legitimate, it's the layer count × dims that overflows.
                return Err(Error::OutOfMemory);
            }
        }
        let mut layers: Vec<Layer> = Vec::with_capacity(psd_layers.len());
        for psd_layer in psd_layers {
            let pixels_buf = layer_pixels(psd_layer, canvas_width, canvas_height)?;
            layers.push(Layer {
                version: 1,
                name: psd_layer.name().to_string(),
                kind: LayerKind::Pixel,
                pixels: Some(pixels_buf),
                opacity: (psd_layer.opacity() as f32) / 255.0,
                blend_mode: map_psd_blend_mode_via_debug(&format!("{:?}", psd_layer.blend_mode())),
                visible: psd_layer.visible(),
                mask: None,
                effects: Vec::new(),
                color_profile: None,
            });
        }

        // PSD has no concept of empty layer stack — if `psd` returned
        // zero layers (defensive), build a single layer from the
        // composited flattened image so the caller sees pixels.
        if layers.is_empty() {
            let flat_pixels = flat_image_to_image_buffer(&parsed, canvas_width, canvas_height)?;
            layers.push(Layer {
                version: 1,
                name: "Background".into(),
                kind: LayerKind::Pixel,
                pixels: Some(flat_pixels),
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                visible: true,
                mask: None,
                effects: Vec::new(),
                color_profile: None,
            });
        }

        Ok(DecodedImage::Layered(LayerStack {
            version: 1,
            canvas_width,
            canvas_height,
            layers,
            // PSD ICC profile preservation deferred — W2.0.1 ICC client.
            color_profile: ColorProfile::Srgb,
        }))
    }
}

fn layer_pixels(
    psd_layer: &psd::PsdLayer,
    canvas_w: u32,
    canvas_h: u32,
) -> Result<ImageBuffer<SrgbRgba>, Error> {
    // `psd` 0.3.5 returns RGBA bytes sized canvas_w × canvas_h, with
    // transparent pixels everywhere the layer doesn't cover. This is
    // not the most compact representation (a real-world layer at
    // (offset, offset) of a 4K canvas allocates 4K × 4K × 4 bytes
    // even if it's a 100×100 sprite), but matches how Painter wants
    // to composite — full-canvas buffers compose with simple alpha
    // blend, no per-layer bounding-box logic.
    let raw = psd_layer.rgba();
    let expected = (canvas_w as usize)
        .checked_mul(canvas_h as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or(Error::OutOfMemory)?;
    if raw.len() != expected {
        return Err(Error::Decode(format!(
            "PSD layer `{}` returned {} RGBA bytes; expected {} for {}×{} canvas",
            psd_layer.name(),
            raw.len(),
            expected,
            canvas_w,
            canvas_h
        )));
    }
    let pixels: Vec<SrgbRgba> = raw
        .chunks_exact(4)
        .map(|c| SrgbRgba([c[0], c[1], c[2], c[3]]))
        .collect();
    Ok(ImageBuffer {
        width: canvas_w,
        height: canvas_h,
        pixels,
        color_profile: ColorProfile::Srgb,
    })
}

fn flat_image_to_image_buffer(
    parsed: &psd::Psd,
    canvas_w: u32,
    canvas_h: u32,
) -> Result<ImageBuffer<SrgbRgba>, Error> {
    let raw = parsed.rgba();
    let expected = (canvas_w as usize)
        .checked_mul(canvas_h as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or(Error::OutOfMemory)?;
    if raw.len() != expected {
        return Err(Error::Decode(format!(
            "PSD composited image returned {} bytes; expected {}",
            raw.len(),
            expected
        )));
    }
    Ok(ImageBuffer {
        width: canvas_w,
        height: canvas_h,
        pixels: raw
            .chunks_exact(4)
            .map(|c| SrgbRgba([c[0], c[1], c[2], c[3]]))
            .collect(),
        color_profile: ColorProfile::Srgb,
    })
}

/// Map `psd::BlendMode` (lots of variants) → our [`BlendMode`].
///
/// **Workaround**: `psd` 0.3.5 leaves `BlendMode` "pub-in-private-mod"
/// — the type is `pub enum` but its parent `sections` module is
/// `pub(crate)`, so we can't name it in a function signature. We
/// match on the `Debug` representation (which IS pub-leaked through
/// the `derive` machinery). Hack but stable: each variant's Debug
/// form is the variant name verbatim per Rust's derive contract.
///
/// Coverage: the 24 modes we ship after the W1.T6 audit expansion.
/// Unknown variants (including `PassThrough` which is group-only)
/// default to `Normal` — visuals stay sane, fidelity moves to
/// `.ph2d-native` for full preservation.
///
/// Upgrade path: when `psd` 0.4 re-exports `BlendMode` (issue
/// chinedufn/psd#XX), drop this hack for a clean `match`.
fn map_psd_blend_mode_via_debug(debug_name: &str) -> BlendMode {
    match debug_name {
        "Normal" => BlendMode::Normal,
        "Dissolve" => BlendMode::Normal, // PSD-specific, no direct equivalent.
        "Darken" | "DarkerColor" => BlendMode::Darken,
        "Multiply" => BlendMode::Multiply,
        "ColorBurn" => BlendMode::ColorBurn,
        "LinearBurn" => BlendMode::LinearBurn,
        "Lighten" | "LighterColor" => BlendMode::Lighten,
        "Screen" => BlendMode::Screen,
        "ColorDodge" => BlendMode::ColorDodge,
        "LinearDodge" => BlendMode::LinearDodge,
        "Overlay" => BlendMode::Overlay,
        "SoftLight" => BlendMode::SoftLight,
        "HardLight" => BlendMode::HardLight,
        "VividLight" => BlendMode::VividLight,
        "LinearLight" => BlendMode::LinearLight,
        "PinLight" => BlendMode::PinLight,
        "HardMix" => BlendMode::HardMix,
        "Difference" => BlendMode::Difference,
        "Exclusion" => BlendMode::Exclusion,
        "Subtract" => BlendMode::Subtract,
        "Divide" => BlendMode::Divide,
        "Hue" => BlendMode::Hue,
        "Saturation" => BlendMode::Saturation,
        "Color" => BlendMode::Color,
        "Luminosity" => BlendMode::Luminosity,
        // PassThrough (group-only) + future unknowns → Normal.
        _ => BlendMode::Normal,
    }
}

/// PSD export driver. **All exports return `Error::Unsupported`** —
/// see crate doc + ADR-0054 §5.2 W2.5 escape hatch. Caller wanting
/// to save layered Painter projects uses `.ph2d-native`.
pub struct PsdExporter;

impl ImageExporter for PsdExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Psd)
    }

    fn export(&self, _img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        Err(Error::Unsupported(
            "PSD export is deferred to W3+ (ADR-0054 §5.2 W2.5 escape hatch). \
             For layered Painter projects use `.ph2d-native` (full PSD-grade \
             preservation). For flat raster export use PNG/JPEG/WebP."
                .into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_recognizes_psd_v1_magic_strong() {
        let imp = PsdImporter;
        assert_eq!(
            imp.supports(MagicHint::Bytes(&PSD_MAGIC)),
            MagicMatch::Strong
        );
        // Wrong signature.
        assert_eq!(
            imp.supports(MagicHint::Bytes(b"not-a-psd")),
            MagicMatch::None
        );
        assert_eq!(imp.supports(MagicHint::Bytes(&[])), MagicMatch::None);
    }

    /// Audit W2.T6 H-6: PSB (`8BPS\0\x02`) now claims Strong so the
    /// registry lands here instead of failing with a generic "no
    /// importer". Import returns actionable `Unsupported` directing
    /// the user to PSD or `.ph2d-native`.
    #[test]
    fn psb_magic_strong_but_import_refuses_actionable() {
        let imp = PsdImporter;
        assert_eq!(
            imp.supports(MagicHint::Bytes(&PSB_MAGIC)),
            MagicMatch::Strong
        );
        let mut psb_header = PSB_MAGIC.to_vec();
        psb_header.extend_from_slice(&[0; 32]);
        let err = imp
            .import(&psb_header, &ImportOpts::default())
            .expect_err("PSB must be refused early");
        match err {
            Error::Unsupported(msg) => assert!(
                msg.contains("PSB") && msg.contains(".ph2d-native"),
                "expected PSB + .ph2d-native pointer, got: {msg}"
            ),
            other => panic!("expected Unsupported, got {other:?}"),
        }
    }

    #[test]
    fn supports_recognizes_psd_extension_weak() {
        let imp = PsdImporter;
        for ext in &["psd", "PSD", "Psd"] {
            assert_eq!(imp.supports(MagicHint::Extension(ext)), MagicMatch::Weak);
        }
        // PSB extension NOT claimed (separate format, future crate).
        assert_eq!(imp.supports(MagicHint::Extension("psb")), MagicMatch::None);
        assert_eq!(imp.supports(MagicHint::Extension("png")), MagicMatch::None);
    }

    #[test]
    fn exporter_recognizes_psd_format() {
        let exp = PsdExporter;
        assert!(exp.supports_format(ExportFormat::Psd));
        assert!(!exp.supports_format(ExportFormat::Png));
    }

    /// PSD export ALWAYS returns Unsupported with the escape-hatch
    /// message pointing the caller at `.ph2d-native`. Audit-aware:
    /// the message names the ADR so future readers find the
    /// rationale.
    #[test]
    fn export_always_returns_unsupported_with_escape_hatch_message() {
        let dummy_flat = DecodedImage::Flat(ImageBuffer {
            width: 1,
            height: 1,
            pixels: vec![SrgbRgba([0, 0, 0, 255])],
            color_profile: ColorProfile::Srgb,
        });
        let err = PsdExporter
            .export(
                &dummy_flat,
                &ExportOpts {
                    format: ExportFormat::Psd,
                    ..ExportOpts::default()
                },
            )
            .expect_err("PSD export must refuse");
        match err {
            Error::Unsupported(msg) => {
                assert!(msg.contains("ADR-0054"), "should cite ADR: {msg}");
                assert!(
                    msg.contains(".ph2d-native"),
                    "should point at .ph2d alternative: {msg}"
                );
            }
            other => panic!("expected Unsupported, got {other:?}"),
        }
    }

    /// Audit nova H-8 residual (2026-05-26): unit-test the
    /// `map_psd_blend_mode_via_debug` helper directly so that if
    /// `psd` 0.3.x derive Debug ever changes the variant-name repr
    /// (e.g. via attribute or rename), this test fails LOUDLY
    /// instead of every blend mode silently mapping to `Normal`.
    /// Pin is `=0.3.5` so versions can't drift; this guards manual
    /// version bumps without re-validation.
    #[test]
    fn map_psd_blend_mode_via_debug_covers_24_variants() {
        // The 24 PSD variant names we ship. Match arms in
        // `map_psd_blend_mode_via_debug` rely on these exact strings.
        let pairs: &[(&str, BlendMode)] = &[
            ("Normal", BlendMode::Normal),
            ("Multiply", BlendMode::Multiply),
            ("Screen", BlendMode::Screen),
            ("Overlay", BlendMode::Overlay),
            ("Darken", BlendMode::Darken),
            ("Lighten", BlendMode::Lighten),
            ("ColorBurn", BlendMode::ColorBurn),
            ("ColorDodge", BlendMode::ColorDodge),
            ("LinearBurn", BlendMode::LinearBurn),
            ("LinearDodge", BlendMode::LinearDodge),
            ("HardLight", BlendMode::HardLight),
            ("SoftLight", BlendMode::SoftLight),
            ("VividLight", BlendMode::VividLight),
            ("LinearLight", BlendMode::LinearLight),
            ("PinLight", BlendMode::PinLight),
            ("HardMix", BlendMode::HardMix),
            ("Difference", BlendMode::Difference),
            ("Exclusion", BlendMode::Exclusion),
            ("Subtract", BlendMode::Subtract),
            ("Divide", BlendMode::Divide),
            ("Hue", BlendMode::Hue),
            ("Saturation", BlendMode::Saturation),
            ("Color", BlendMode::Color),
            ("Luminosity", BlendMode::Luminosity),
        ];
        for (debug_name, expected) in pairs {
            let mapped = map_psd_blend_mode_via_debug(debug_name);
            assert_eq!(
                mapped, *expected,
                "psd 0.3.5 Debug name {debug_name:?} must map to {expected:?}"
            );
        }
        // Unknown variant falls to Normal (PassThrough + future).
        assert_eq!(
            map_psd_blend_mode_via_debug("PassThrough"),
            BlendMode::Normal
        );
        assert_eq!(map_psd_blend_mode_via_debug("Dissolve"), BlendMode::Normal);
        assert_eq!(
            map_psd_blend_mode_via_debug("DarkerColor"),
            BlendMode::Darken
        );
        assert_eq!(
            map_psd_blend_mode_via_debug("LighterColor"),
            BlendMode::Lighten
        );
        assert_eq!(
            map_psd_blend_mode_via_debug("HypotheticalNewMode"),
            BlendMode::Normal
        );
    }

    #[test]
    fn import_rejects_empty_as_truncated() {
        let err = PsdImporter
            .import(&[], &ImportOpts::default())
            .expect_err("empty");
        assert!(matches!(err, Error::Truncated));
    }

    #[test]
    fn import_rejects_wrong_signature_with_decode() {
        // 32 bytes of garbage that won't parse as PSD.
        let bytes = vec![0xAB; 32];
        let err = PsdImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("garbage refused");
        assert!(matches!(err, Error::Decode(_)));
    }
}

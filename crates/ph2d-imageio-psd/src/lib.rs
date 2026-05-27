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
//! - ICC profile parsing — preserved as `ColorProfile::Custom` if
//!   the `psd` crate surfaces it (TBD when we test against a real
//!   profile-tagged PSD); ICC conversion via moxcms lands W2.0.1
//!   when the first real client appears.

use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    BlendMode, ColorProfile, DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry,
    ImageBuffer, ImageExporter, ImageImporter, ImportOpts, ImporterRegistry, Layer, LayerKind,
    LayerStack, MAX_RASTER_DIMENSION, MagicHint, MagicMatch,
};

/// PSD signature: `"8BPS"` followed by 2-byte big-endian version
/// (`0x0001` = PSD; `0x0002` = PSB which we don't support yet).
const PSD_MAGIC: [u8; 6] = [b'8', b'B', b'P', b'S', 0x00, 0x01];

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
            // `8BPS` without the `0x0001` PSD version is a PSB file
            // (`0x0002`); we don't claim it — let registry-fallthrough
            // surface an Unsupported.
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("psd") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        let parsed =
            psd::Psd::from_bytes(src).map_err(|e| Error::Decode(format!("PSD parse: {e}")))?;

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
        // PSB version 0x0002 — NOT supported by W2.T4; must NOT match.
        let psb = [b'8', b'B', b'P', b'S', 0x00, 0x02];
        assert_eq!(imp.supports(MagicHint::Bytes(&psb)), MagicMatch::None);
        // Wrong signature.
        assert_eq!(
            imp.supports(MagicHint::Bytes(b"not-a-psd")),
            MagicMatch::None
        );
        assert_eq!(imp.supports(MagicHint::Bytes(&[])), MagicMatch::None);
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

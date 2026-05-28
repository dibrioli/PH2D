#![forbid(unsafe_code)]
//! `ph2d-imageio-svg` — SVG decode + (deferred) encode (W3.T5).
//!
//! Pure-Rust via `usvg` 0.43 (parser + simplifier). W3.T5 ships
//! parse-only — the resulting tree validates the SVG is well-formed
//! and within size bounds, but the canonical `VectorDoc` body is
//! reserved for W3+ when `ph2d-vector` lands the `kurbo::BezPath`
//! paint stack + paint server + transform amendment.
//!
//! ### What W3.T5 covers
//!
//! - Magic: heuristic on first 256 bytes containing `<svg` or
//!   `<?xml` followed by `<svg` (handles BOM + DOCTYPE prefixes).
//! - Decode: `usvg::Tree::from_data` validates the SVG, returns
//!   `DecodedImage::Vector(VectorDoc::default())`. The VectorDoc
//!   body is intentionally empty (W3+ amendment).
//! - HR-1: pure-Rust, no C bindings.
//! - HR-13: size cap via `ph2d_imageio::MAX_ARCHIVE_TEXT_BYTES`
//!   (SVG is text/XML; 16 MiB ceiling covers any plausible icon
//!   set, blocks billion-laughs / external-entity attacks at the
//!   read boundary).
//!
//! Audit-13 (2026-05-27): fan-out drop-crate per ADR-0054 §3.8.

use ph2d_imageio::{
    DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageExporter, ImageImporter,
    ImportOpts, ImporterRegistry, MAX_ARCHIVE_TEXT_BYTES, MagicHint, MagicMatch, VectorDoc,
};

/// Register the SVG importer.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(SvgImporter));
}

/// Register the SVG exporter.
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(SvgExporter));
}

/// SVG import driver.
pub struct SvgImporter;

/// Heuristic: does the first 256 bytes look like SVG? Handles BOM,
/// XML prolog, DOCTYPE preamble, and direct `<svg` opening.
fn is_svg_heuristic(b: &[u8]) -> bool {
    let head = &b[..b.len().min(256)];
    let head_lower: Vec<u8> = head.iter().map(|c| c.to_ascii_lowercase()).collect();
    // `<svg` anywhere in the first 256 bytes — covers `<svg xmlns=…>`,
    // `<?xml …?><!DOCTYPE …><svg …>`, BOM + XML prolog, etc.
    head_lower.windows(4).any(|w| w == b"<svg")
}

impl ImageImporter for SvgImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if is_svg_heuristic(b) => MagicMatch::Strong,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("svg") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        // HR-13: cap source size before usvg parses. SVG is text/XML
        // so the cap is large enough for any plausible icon set
        // (~16 MiB) but blocks billion-laughs / hostile expansion.
        if (src.len() as u64) > MAX_ARCHIVE_TEXT_BYTES {
            return Err(Error::Decode(format!(
                "SVG source {} bytes > MAX_ARCHIVE_TEXT_BYTES={MAX_ARCHIVE_TEXT_BYTES} \
                 (DoS defence)",
                src.len()
            )));
        }
        // Parse via usvg. Default options resolve <use>, <style>,
        // gradients into a flat tree; the result is dropped here
        // because VectorDoc body is reserved for W3+ ph2d-vector
        // amendment.
        let _tree = usvg::Tree::from_data(src, &usvg::Options::default())
            .map_err(|e| Error::from_decoder_message(format!("SVG parse: {e}")))?;
        // W3.T5 ships parse-only validation. The actual vector
        // tree → BezPath conversion lands in W3+ when ph2d-vector
        // is callable from imageio.
        Ok(DecodedImage::Vector(VectorDoc::default()))
    }
}

/// SVG export driver. Deferred until ph2d-vector canonicalises.
pub struct SvgExporter;

impl ImageExporter for SvgExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Svg)
    }

    fn export(&self, _img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        Err(Error::Unsupported(
            "SVG export deferred to W3+ — ph2d-vector canonical types \
             (kurbo::BezPath paint stack) need to land first. Round-trip \
             via `.ph2d-native` preserves vector content losslessly today."
                .into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_recognizes_svg_open_tag_strong() {
        assert_eq!(
            SvgImporter.supports(MagicHint::Bytes(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>")),
            MagicMatch::Strong
        );
    }

    #[test]
    fn supports_recognizes_svg_with_xml_prolog_strong() {
        assert_eq!(
            SvgImporter.supports(MagicHint::Bytes(
                b"<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\"/>"
            )),
            MagicMatch::Strong
        );
    }

    #[test]
    fn supports_rejects_html_with_none() {
        assert_eq!(
            SvgImporter.supports(MagicHint::Bytes(b"<!DOCTYPE html><html></html>")),
            MagicMatch::None
        );
    }

    #[test]
    fn supports_recognizes_svg_extension_weak() {
        assert_eq!(
            SvgImporter.supports(MagicHint::Extension("svg")),
            MagicMatch::Weak
        );
    }

    #[test]
    fn import_rejects_empty_as_truncated() {
        let err = SvgImporter
            .import(&[], &ImportOpts::default())
            .expect_err("empty");
        assert!(matches!(err, Error::Truncated));
    }

    #[test]
    fn import_minimal_svg_returns_vector() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"/>"#;
        let decoded = SvgImporter
            .import(svg, &ImportOpts::default())
            .expect("minimal SVG must parse");
        assert!(matches!(decoded, DecodedImage::Vector(_)));
    }

    #[test]
    fn import_rejects_malformed_svg_with_decode() {
        let bad = b"<svg this is not closed";
        let err = SvgImporter
            .import(bad, &ImportOpts::default())
            .expect_err("malformed");
        assert!(matches!(err, Error::Decode(_) | Error::Truncated));
    }

    #[test]
    fn exporter_returns_unsupported() {
        let err = SvgExporter
            .export(&DecodedImage::Vector(VectorDoc::default()), &ExportOpts {
                format: ExportFormat::Svg,
                ..ExportOpts::default()
            })
            .expect_err("export deferred");
        assert!(matches!(err, Error::Unsupported(_)));
    }
}

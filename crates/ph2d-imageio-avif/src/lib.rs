#![forbid(unsafe_code)]
//! `ph2d-imageio-avif` — AVIF magic recognition + (deferred) decode
//! + encode (W3.T4).
//!
//! W3.T4 placeholder: recognizes AVIF magic via `ftyp` box detection
//! so the registry dispatches here for `.avif` files; both import
//! and export return `Error::Unsupported` with actionable messages
//! pointing to the next wave when a real client (Painter HDR import
//! demo) triggers full wire-up.
//!
//! ### Why still a stub (audit-15 deship history)
//!
//! `272d99d` (2026-05-27) shipped real decode via
//! `avif-decode = "1"` but **audit-15** caught 1 CRITICAL + 6 HIGH
//! + 8 MEDIUM in that dep tree:
//!
//! - **CRITICAL** RUSTSEC-2022-0040 `owning_ref` use-after-free
//!   (no fix upstream) via `avif-decode → aom-decode →
//!   owning_ref 0.4.1`. CI hard-fails `cargo audit`.
//! - **HIGH** `libaom-sys 0.17` vendora 26 MB C source + cmake
//!   build-script (+30-60s clean build × 3 platforms; WASM
//!   impossible).
//! - **HIGH** silent HDR PQ/HLG quantisation (asymmetry with the
//!   JXL audit-14 fix that explicitly rejects HDR via
//!   `hdr_type()`); plus `ColorProfile::Srgb` hardcoded mislabel
//!   for Display-P3/BT.2020 wide-gamut.
//! - **HIGH** `assert!` panic in `avif-parse` UUID-box parser
//!   (hostile input).
//! - **HIGH** `Vec::with_capacity(w*h)` allocated pre-decode from
//!   AV1 sequence-header dims — 100k×100k claim OOMs before our
//!   `MAX_RASTER_DIMENSION` post-decode cap.
//! - **MEDIUM** upstream `unprem()` mathematically wrong
//!   (`val/alpha/256 = 0` for `alpha < 255`) — premultiplied
//!   AVIF turns black. Bug in the dep, not patchable here.
//! - **MEDIUM** ~160 unsafe blocks across the transitive tree
//!   (`lodepng = 97`, `owning_ref = 42`, `libaom-sys = 9`,
//!   `aom-decode = 11`).
//! - **MEDIUM** maintainer bus-factor = 1 (Kornel single-author
//!   chain: avif-decode/aom-decode/avif-parse/yuv/imgref/rgb).
//!
//! Per
//! [`feedback-no-industrial-claims-without-verification`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_no_industrial_claims_without_verification.md)
//! and
//! [`feedback-perfection-no-deferrals`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md):
//! 1 unfixable RUSTSEC + 1 unfixable upstream math bug + 1 wave-2
//! HDR asymmetry that wasn't even attempted is unshippable. The
//! real decode commit was reverted; the magic-only stub stays
//! until a better pure-Rust decoder candidate appears.
//!
//! **Real-decode candidates to re-evaluate** (in priority order):
//! 1. `image = { features = ["avif-native"] }` — different dep
//!    tree (uses `mp4parse` + `dav1d`-Rust); needs verification
//!    that `owning_ref` is NOT pulled.
//! 2. Wait for `avif-decode` 2.x to migrate to `safer_owning_ref`
//!    + fix `unprem()` upstream.
//! 3. Direct `libavif-sys` (C FFI, fastest but unsafe ABI surface).
//!
//! Audit-13 (2026-05-27): fan-out drop-crate per ADR-0054 §3.8.
//! Audit-15 (2026-05-28): deship of `272d99d`. Vide ADR §5.17.

use ph2d_imageio::{
    DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageExporter, ImageImporter,
    ImportOpts, ImporterRegistry, MagicHint, MagicMatch,
};

/// AVIF magic: ISOBMFF `ftyp` box starting at byte 4, brand `avif`
/// or `avis` (still / sequence) at byte 8.
fn is_avif_magic(b: &[u8]) -> bool {
    if b.len() < 12 {
        return false;
    }
    // Bytes 4-7 = "ftyp".
    if &b[4..8] != b"ftyp" {
        return false;
    }
    // Bytes 8-11 = "avif" (still) or "avis" (sequence).
    matches!(&b[8..12], b"avif" | b"avis")
}

/// Register the AVIF importer.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(AvifImporter));
}

/// Register the AVIF exporter.
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(AvifExporter));
}

/// AVIF import driver. W3.T4 placeholder — recognizes magic but
/// returns Unsupported pointing the caller at the next wave.
pub struct AvifImporter;

impl ImageImporter for AvifImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if is_avif_magic(b) => MagicMatch::Strong,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("avif") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        Err(Error::Unsupported(
            "AVIF decode deferred to W3+ — pure-Rust `avif-decode = \"1\"` \
             wire-up lands when first real client (Painter HDR import demo) \
             appears. Convert to PNG/WebP/EXR via external tool (cavif-rs, \
             ffmpeg) for now."
                .into(),
        ))
    }
}

/// AVIF export driver. W3.T4 placeholder.
pub struct AvifExporter;

impl ImageExporter for AvifExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Avif)
    }

    fn export(&self, _img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        Err(Error::Unsupported(
            "AVIF encode deferred to W3+ — pure-Rust `ravif = \"0.11\"` \
             (rav1e backend) wire-up lands when first real client \
             (Painter HDR export demo) appears."
                .into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_recognizes_avif_ftyp_strong() {
        let bytes = [
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'a', b'v', b'i', b'f', 0x00, 0x00,
        ];
        assert_eq!(
            AvifImporter.supports(MagicHint::Bytes(&bytes)),
            MagicMatch::Strong
        );
    }

    #[test]
    fn supports_recognizes_avis_sequence_strong() {
        let bytes = [
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'a', b'v', b'i', b's', 0x00, 0x00,
        ];
        assert_eq!(
            AvifImporter.supports(MagicHint::Bytes(&bytes)),
            MagicMatch::Strong
        );
    }

    #[test]
    fn supports_rejects_heif_with_none() {
        // HEIF same container but brand=mif1/heic — must NOT match.
        let bytes = [
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'm', b'i', b'f', b'1', 0x00, 0x00,
        ];
        assert_eq!(
            AvifImporter.supports(MagicHint::Bytes(&bytes)),
            MagicMatch::None
        );
    }

    #[test]
    fn supports_recognizes_avif_extension_weak() {
        assert_eq!(
            AvifImporter.supports(MagicHint::Extension("avif")),
            MagicMatch::Weak
        );
    }

    #[test]
    fn import_rejects_empty_as_truncated() {
        let err = AvifImporter
            .import(&[], &ImportOpts::default())
            .expect_err("empty");
        assert!(matches!(err, Error::Truncated));
    }

    #[test]
    fn import_returns_unsupported_with_actionable_message() {
        let bytes = [
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'a', b'v', b'i', b'f',
        ];
        let err = AvifImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("decode deferred");
        let msg = match err {
            Error::Unsupported(s) => s,
            other => panic!("expected Unsupported, got {other:?}"),
        };
        assert!(msg.contains("avif-decode"), "actionable: {msg}");
    }

    /// Audit-13 Lens FF F4 (2026-05-27): cover `avis` sequence brand
    /// import path too — previous test only exercised still `avif`.
    #[test]
    fn import_avis_sequence_returns_unsupported_deferred() {
        let bytes = [
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'a', b'v', b'i', b's',
        ];
        let err = AvifImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("avis sequence decode deferred");
        assert!(matches!(err, Error::Unsupported(_)));
    }

    #[test]
    fn exporter_returns_unsupported() {
        // `AvifExporter::export` returns Unsupported BEFORE looking
        // at the image payload — `DecodedImage::Animated(vec![])` is
        // the cheapest variant to construct in a test that doesn't
        // import ph2d-color.
        let err = AvifExporter.export(
            &DecodedImage::Animated(vec![]),
            &ExportOpts {
                format: ExportFormat::Avif,
                ..ExportOpts::default()
            },
        );
        assert!(matches!(err, Err(Error::Unsupported(_))));
    }

    #[test]
    fn exporter_recognizes_avif_format() {
        assert!(AvifExporter.supports_format(ExportFormat::Avif));
        assert!(!AvifExporter.supports_format(ExportFormat::Png));
    }
}

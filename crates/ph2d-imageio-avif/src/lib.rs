// NOTE: this crate intentionally does **not** carry
// `#![forbid(unsafe_code)]` (every sibling format crate does). AVIF
// real decode/encode + HDR/wide-gamut requires the `libavif` C
// reference codec via `libavif-sys` raw FFI, whose `nclx` colour box,
// bit depth, and float RGB output the safe `libavif` wrapper hides.
// All `unsafe` lives in [`decode`] / [`encode`], each block carrying a
// `// SAFETY:` rationale; `unsafe_op_in_unsafe_fn` is denied so every
// op is individually justified. Decision: ADR-0054 §5.18 (Path C),
// ratified 2026-05-28 ("o melhor possível, sem custo" — HDR real beats
// keeping the crate `forbid(unsafe)` at the cost of an LDR-only stub).
#![deny(unsafe_op_in_unsafe_fn)]
//! `ph2d-imageio-avif` — AVIF (AV1 Image File) real decode + encode +
//! HDR/wide-gamut (W3.T4 re-ship, Path C).
//!
//! Backed by `libavif-sys` (the AOM `libavif` C reference library)
//! with `codec-dav1d` (decode) + `codec-rav1e` (pure-Rust encode).
//!
//! ### What W3.T4 covers
//!
//! - **Magic**: ISOBMFF `ftyp` box, brand `avif` (still) / `avis`
//!   (sequence).
//! - **Decode** ([`decode`]): first image → `Flat` (8-bit SDR) or
//!   `FlatHdr` (scene-linear, for PQ/HLG/linear transfer, >8-bit, or
//!   BT.2020 primaries). `nclx` → [`ColorProfile`]
//!   (ph2d_imageio::ColorProfile). Grid images are decoded as the
//!   composited primary image (libavif stitches the grid). Animation
//!   (`avis`) decodes the first frame — multi-frame `Animated` bridge
//!   is W3+ (documented, not silent: `imageCount > 1` is logged).
//! - **Encode** ([`encode`]): `Flat`/`FlatHdr` → AVIF via rav1e,
//!   quality/speed knobs, `nclx` written from the source profile so
//!   HDR/wide-gamut round-trips.
//! - **HR-13**: pre-decode dimension cap (`imageDimensionLimit` set on
//!   the C decoder *before* parse) + a redundant post-parse check.
//! - **Hostile input**: the C parser boundary is wrapped in
//!   `catch_unwind` so a malformed file returns `Error::Decode`
//!   instead of crashing the process.
//!
//! ### Deship history (audit-15) → re-ship (Path C)
//!
//! `272d99d` shipped real decode via `avif-decode = "1"`; audit-15
//! deshipped it in `f034e9a` over an `owning_ref` UAF
//! (RUSTSEC-2022-0040) plus an upstream `unprem()` math bug. Path C
//! replaces that dep tree entirely (zero `owning_ref`, 0 RUSTSEC —
//! verified 2026-05-28). See `Cargo.toml` header + ADR-0054 §5.18.

mod color;
mod decode;
mod encode;

pub use decode::AvifImporter;
pub use encode::AvifExporter;

use ph2d_imageio::{ExporterRegistry, ImporterRegistry};

/// AVIF magic: ISOBMFF `ftyp` box at byte 4, brand `avif` (still) or
/// `avis` (sequence) at byte 8. Used by `supports()` for dispatch; the
/// authoritative recognition is libavif's own `ftyp` parse at decode.
pub(crate) fn is_avif_magic(b: &[u8]) -> bool {
    if b.len() < 12 {
        return false;
    }
    if &b[4..8] != b"ftyp" {
        return false;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_recognizes_avif_and_avis() {
        let avif = [
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'a', b'v', b'i', b'f',
        ];
        let avis = [
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'a', b'v', b'i', b's',
        ];
        assert!(is_avif_magic(&avif));
        assert!(is_avif_magic(&avis));
    }

    #[test]
    fn magic_rejects_heif_and_short() {
        // HEIF shares the container but brand=mif1/heic.
        let heif = [
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'm', b'i', b'f', b'1',
        ];
        assert!(!is_avif_magic(&heif));
        assert!(!is_avif_magic(b"\x00\x00\x00"));
    }
}

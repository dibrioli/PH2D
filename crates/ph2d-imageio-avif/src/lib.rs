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
//!   (`avis`) decodes the **first frame only** — multi-frame `Animated`
//!   bridge is W3+. This is a documented limitation, not a runtime
//!   signal: the crate has no logging facility, so the dropped frames
//!   are silent to the caller (the comment at the decode site marks it).
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

/// AVIF magic: an ISOBMFF `ftyp` box whose **major brand OR any
/// compatible brand** is `avif`/`avis`. This mirrors libavif's own
/// `avifFileTypeHasBrand` (read.c) — checking only the major-brand slot
/// (offset 8) gives false negatives, because the AVIF/MIAF spec lets a
/// file carry `major_brand = mif1`/`msf1`/`iso8` and list `avif` only
/// among the compatible brands (real encoders emit this). Such a file
/// is a valid AVIF and must dispatch here.
pub(crate) fn is_avif_magic(b: &[u8]) -> bool {
    // ftyp box: [u32 size][b"ftyp"][major_brand:4][minor_version:4]
    //           [compatible_brands: 4·n]. `size == 1` would signal a
    //           64-bit largesize (8 extra bytes before the body) — ftyp
    //           never uses it in practice, so we bail rather than risk a
    //           mis-parse.
    if b.len() < 12 || &b[4..8] != b"ftyp" {
        return false;
    }
    let box_size = u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize;
    if box_size == 1 {
        return false; // 64-bit largesize ftyp — unheard-of; reject.
    }
    let is_avif_brand = |brand: &[u8]| matches!(brand, b"avif" | b"avis");
    // Major brand at offset 8.
    if is_avif_brand(&b[8..12]) {
        return true;
    }
    // Compatible brands run from offset 16 to the box end (or the end of
    // the peeked slice, whichever is smaller — the peek window is ≥ 32
    // bytes per the MagicHint contract, enough for the usual brand list).
    // `min` (not `clamp`) avoids a panic when box_size < the slice or the
    // loop start: the `off + 4 <= end` guard handles end < 16 by simply
    // not iterating.
    let end = box_size.min(b.len());
    let mut off = 16;
    while off + 4 <= end {
        if is_avif_brand(&b[off..off + 4]) {
            return true;
        }
        off += 4;
    }
    false
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

    #[test]
    fn magic_recognizes_avif_in_compatible_brands() {
        // Spec-valid AVIF whose MAJOR brand is `mif1` but lists `avif`
        // among the compatible brands — must still match (audit-16 HIGH:
        // checking only the major-brand slot gave false negatives).
        // [size=28][ftyp][major=mif1][minor=0000][compat: mif1, avif]
        let bytes = [
            0x00, 0x00, 0x00, 0x1c, b'f', b't', b'y', b'p', b'm', b'i', b'f', b'1', 0x00, 0x00,
            0x00, 0x00, b'm', b'i', b'f', b'1', b'a', b'v', b'i', b'f', 0x00, 0x00, 0x00, 0x00,
        ];
        assert!(is_avif_magic(&bytes));
    }
}

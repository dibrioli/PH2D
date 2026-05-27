//! Bomb-defence + budget caps shared by every format crate.
//!
//! Audit B-H2 / D-E6 (2026-05-26): each raster format crate had its
//! own private `MAX_DIMENSION = 32_768` const, drifting independently
//! if any were raised; HR-13 also wants the budget surface
//! discoverable (not buried in private code). Hoisting here makes the
//! caps a contract-level concern with a single source of truth.

/// Maximum width or height (in pixels) a raster importer will accept.
/// Sources reporting dimensions above this trip
/// [`crate::Error::DimensionExceedsLimit`] before any pixel decoding —
/// decompression-bomb defence (a fraudulent PNG IHDR claiming
/// `65535×65535 × 4 bytes` = 16 GB is refused at the header).
///
/// 32768 covers every printer-DPI + 8K display × 4 HiDPI scenario the
/// Painter ships with comfortable headroom. Documents needing larger
/// dimensions are an ADR-0054 amendment, not a per-format silent bump.
///
/// Note: this is a *single-axis* cap. A 32K × 32K image is still
/// ~4 GB and may trip `Error::OutOfMemory` further down the pipeline
/// (the system allocator refuses); the dimension cap is the
/// first-line defence, not the last.
pub const MAX_RASTER_DIMENSION: u32 = 32_768;

/// Maximum `.ph2d-native` payload size (bytes). 4 GiB minus 1 to
/// avoid 32-bit `usize` overflow on the future ARMv7 / x86_32 target
/// (audit A-MEDIUM `.ph2d-native` `MAX_PAYLOAD_LEN` overflow).
pub const MAX_PH2D_PAYLOAD_LEN: u64 = u32::MAX as u64;

/// Maximum embedded ICC profile size (bytes) preserved in
/// [`crate::ColorProfile::Custom`]. 4 MiB covers every real-world
/// ICC v2/v4 profile (typical: 50 KB - 1 MB) with comfortable margin.
/// Caps `Custom` allocation when decoding tampered files claiming
/// huge ICC chunks. Audit `.ph2d-native` L1.
pub const MAX_ICC_PROFILE_LEN: usize = 4 * 1024 * 1024;

/// Maximum frames in an animation (APNG/GIF/WebP-animated). Audit-8
/// Lens O O-1 (2026-05-26): hoisted from private constants drifting
/// independently in `ph2d-imageio-apng` and `ph2d-imageio-gif`
/// (both = 1024 by coincidence, one `u32` + one `usize`). Now a
/// contract-level concern with a single source of truth.
///
/// 1024 covers any plausible real animation (longer sequences belong
/// in video formats, not animated rasters).
pub const MAX_ANIMATION_FRAMES: u32 = 1024;

/// Maximum pages in a multi-page document (TIFF). Audit-8 Lens O O-1
/// (2026-05-26): hoisted from `ph2d-imageio-tiff::MAX_PAGES`.
///
/// 256 covers any plausible catalogue / book scan (longer files
/// should split — TIFF chained IFDs of 10k pages is exclusively a
/// hostile-input scenario).
pub const MAX_DOCUMENT_PAGES: usize = 256;

/// Maximum recursion depth in nested layer trees (ORA `<stack>` and
/// future PSD groups). Audit-9 Lens T CRITICAL T-#1 (2026-05-26):
/// without this cap, a hostile `stack.xml` with 50000 nested
/// `<stack>` elements stack-overflows the 8 MiB default thread
/// frame on macOS → SIGSEGV (not catchable by `catch_unwind`). 64
/// covers any plausible artist organization (Krita ships with
/// group-trees of depth 5-10 in practice).
pub const MAX_LAYER_DEPTH: usize = 64;

/// Maximum total layer count in a layered document. Audit-9 Lens T
/// MEDIUM T-#2 (2026-05-26): defence against a hostile `stack.xml`
/// listing N references to the same PNG (read amplification → OOM
/// even within `MAX_LAYER_DEPTH`). 4096 is generous for real
/// documents (Krita ships ~10 layers per file commonly).
pub const MAX_LAYER_COUNT: usize = 4096;

/// Maximum entries in a ZIP-container format (ORA). Audit-10 Lens Z
/// HIGH #2 (2026-05-26): `ZipArchive::new` reads the entire central
/// directory in memory at construction time; a hostile ZIP of 5 MiB
/// declaring 8M false entries (each CDFH ~46 B compressed) inflates
/// to ~1.5 GiB metadata. 8192 = `MAX_LAYER_COUNT` + 2 (mimetype +
/// stack.xml) + comfortable headroom for `mergedimage.png` and
/// future Thumbnails/.
pub const MAX_ARCHIVE_ENTRIES: usize = 8192;

/// Maximum size of a single text entry inside an archive (ORA
/// `stack.xml`). Audit-10 Lens Z MEDIUM #3 (2026-05-26): hostile ORA
/// could declare a 500 MiB `stack.xml` of whitespace (valid XML but
/// blows past `String::read_to_string` without bound). 16 MiB covers
/// any plausible layer-tree manifest (Krita's biggest XMP-ish
/// manifests measured < 1 MiB) with comfortable margin.
pub const MAX_ARCHIVE_TEXT_BYTES: u64 = 16 * 1024 * 1024;

/// Compile-time sanity envelope. Catches typo regressions.
const _SANITY: () = {
    assert!(MAX_RASTER_DIMENSION >= 16_384);
    assert!(MAX_RASTER_DIMENSION <= 65_536);
    assert!(MAX_PH2D_PAYLOAD_LEN >= 1_073_741_824); // ≥ 1 GiB
    assert!(MAX_ICC_PROFILE_LEN >= 65_536); // ≥ 64 KiB
    assert!(MAX_ANIMATION_FRAMES >= 60); // ≥ 1 s @ 60 Hz
    assert!(MAX_DOCUMENT_PAGES >= 16); // ≥ 16 pages
    assert!(MAX_LAYER_DEPTH >= 8); // ≥ 8 typical artist nesting
    assert!(MAX_LAYER_COUNT >= 256); // ≥ 256 layers/file
    assert!(MAX_ARCHIVE_ENTRIES >= MAX_LAYER_COUNT + 2); // layers + mimetype + stack.xml
    assert!(MAX_ARCHIVE_TEXT_BYTES >= 1024 * 1024); // ≥ 1 MiB stack.xml
};

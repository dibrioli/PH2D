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

/// Compile-time sanity envelope. Catches typo regressions.
const _SANITY: () = {
    assert!(MAX_RASTER_DIMENSION >= 16_384);
    assert!(MAX_RASTER_DIMENSION <= 65_536);
    assert!(MAX_PH2D_PAYLOAD_LEN >= 1_073_741_824); // ≥ 1 GiB
    assert!(MAX_ICC_PROFILE_LEN >= 65_536); // ≥ 64 KiB
};

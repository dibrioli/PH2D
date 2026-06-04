// ── public limits ───────────────────────────────────────────────────

/// Largest texture edge accepted, in pixels. Matches `ph2d-render`'s
/// atlas cap ([`ATLAS_DEFAULT_SIZE_PX`] = 8192) and the PNG / JPEG
/// limits in `ph2d-asset::loader`. Anything larger is almost certainly
/// a malformed or hostile file.
///
/// [`ATLAS_DEFAULT_SIZE_PX`]: ../ph2d_render/atlas/constant.ATLAS_DEFAULT_SIZE_PX.html
pub const MAX_DIMENSION: u32 = 8192;

/// Cap on the sum of every mip level's bytes after parsing. KTX2 is a
/// container — a tiny header can claim gigabytes of mip data. 512 MiB
/// matches the `MAX_ALLOC_BYTES` guard in `ph2d-asset::loader`.
pub const MAX_TOTAL_BYTES: u64 = 512 * 1024 * 1024;

/// Cap on declared mip count. `log2(8192) + 1 = 14`; 16 leaves headroom
/// for the (rare) NPOT case while still rejecting absurd headers.
pub const MAX_LEVELS: u32 = 16;

/// W1.T9 — cap on KTX2 keyValueData entries preserved by the parser. KTX2
/// containers MAY ship arbitrary metadata (timestamps, swizzle, custom
/// keys); a malformed/hostile file could declare thousands. 64 covers all
/// realistic use cases (KTX-Software conventions, PH2D's own `PH2D_PREMUL`
/// W1.T8, glTF KHR_*, swizzle keys) with plenty of headroom.
pub const MAX_KVD_ENTRIES: usize = 64;

/// W1.T9 — cap on individual kvd value size in bytes. Most KTX2 metadata
/// values are < 64 B (strings, single bytes); 4 KiB allows complex JSON-like
/// values used by some tooling while bounding memory in pathological files.
pub const MAX_KVD_VALUE_BYTES: usize = 4 * 1024;

/// W1.T9 audit Lente ξ-F3 — cap on individual kvd *key* length in bytes.
/// Symmetric DOS defence to [`MAX_KVD_VALUE_BYTES`]: without it a hostile
/// file could ship a multi-MiB key (the value cap alone leaves the key
/// unbounded) and force a large `key.to_string()` allocation. Real KTX2
/// keys are short identifiers ("KTXorientation", "PH2D_PREMUL"); 256 B is
/// generous headroom. Aggregate worst case is therefore bounded:
/// `MAX_KVD_ENTRIES × (MAX_KVD_KEY_BYTES + MAX_KVD_VALUE_BYTES)`
/// ≈ 64 × (256 B + 4 KiB) ≈ 272 KiB.
pub const MAX_KVD_KEY_BYTES: usize = 256;

/// W1.T9 — canonical KTX2 keyValueData key used by PH2D to tag the alpha
/// intent of a cooked texture (`PremulIntent`). 1-byte value:
/// `[0] = Straight`, `[1] = Premultiplied`. Key ausente = `Unspecified`.
/// Emit deferred a W1.T8 (ctt 0.4.0 não suporta kvd write; precisa de
/// patcher post-hoc OR upstream PR).
pub const PH2D_PREMUL_KEY: &str = "PH2D_PREMUL";

// Compile-time sanity: catches anyone zeroing a limit by accident
// (clippy bans these as runtime `assert!`-on-constants — const-context
// asserts are the canonical form).
const _: () = assert!(MAX_DIMENSION > 0 && MAX_DIMENSION <= 16384);
const _: () = assert!(MAX_TOTAL_BYTES > 0);
const _: () = assert!(MAX_LEVELS > 0 && MAX_LEVELS < 32);
const _: () = assert!(MAX_KVD_ENTRIES > 0 && MAX_KVD_ENTRIES <= 256);
const _: () = assert!(MAX_KVD_VALUE_BYTES > 0 && MAX_KVD_VALUE_BYTES <= 64 * 1024);
const _: () = assert!(MAX_KVD_KEY_BYTES > 0 && MAX_KVD_KEY_BYTES <= 4 * 1024);

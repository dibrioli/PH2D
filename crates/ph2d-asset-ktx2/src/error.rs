use thiserror::Error;

// ── error type ──────────────────────────────────────────────────────

/// Failures during KTX2 decode. Each variant carries enough context
/// for the caller to either surface a precise error toast or decide on
/// a fallback path.
///
/// `#[non_exhaustive]` (W1.T9 audit Lente ν-7): future variants must
/// not be a breaking change for downstream consumers. External callers
/// match with a wildcard arm; within this crate the attribute is inert.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Ktx2Error {
    /// The bytes are not a valid KTX2 container (bad magic, truncated
    /// header, malformed level index, …). The wrapped string is the
    /// upstream `ktx2` crate's diagnostic; treat as opaque.
    #[error("invalid KTX2 container: {0}")]
    InvalidContainer(String),

    /// The container declares dimensions beyond [`MAX_DIMENSION`](crate::MAX_DIMENSION).
    /// Could be a real high-res asset (raise the cap deliberately) or
    /// a header attack (file says 2³¹ but allocates nothing).
    #[error("dimension {dim} exceeds max {max}")]
    BoundsExceeded { dim: u32, max: u32 },

    /// Sum of mip-level payload bytes exceeds [`MAX_TOTAL_BYTES`](crate::MAX_TOTAL_BYTES).
    #[error("total mip bytes {total} exceeds max {max}")]
    TotalBytesExceeded { total: u64, max: u64 },

    /// One of width / height / layer / face was zero.
    #[error("zero dimension is not allowed")]
    ZeroDimension,

    /// Container declares more mip levels than [`MAX_LEVELS`](crate::MAX_LEVELS).
    #[error("level count {count} exceeds max {max}")]
    TooManyLevels { count: u32, max: u32 },

    /// Header `format = VK_FORMAT_UNDEFINED` (0). The KTX2 spec allows
    /// this for formats described purely by the Data Format Descriptor
    /// (e.g. some Basis layouts), but we do not interpret the DFD in
    /// Fase 1 — callers must supply a file with an explicit VkFormat.
    #[error("KTX2 file has no declared VkFormat (DFD-only formats not supported in Fase 1)")]
    MissingFormat,

    /// Container declares a non-`None` supercompression scheme (zstd,
    /// BasisLZ, ZLIB, …). Decoding those is foundational work
    /// gated by an ADR; for now we reject explicitly.
    #[error("supercompression scheme {raw} is not supported in Fase 1")]
    UnsupportedSupercompression { raw: u32 },

    /// Declared mip-level data slice did not match the index.
    #[error("level {level}: data length mismatch")]
    LevelDataMismatch { level: u32 },

    /// Container declares a non-2D layout: 3D texture
    /// (`pixel_depth > 0`), cubemap (`face_count == 6`), or array
    /// texture (`layer_count > 1`). Fase 1 only wires 2D sprites
    /// — the renderer has no path for the other layouts yet.
    /// Lifting the restriction is a foundational change (Fase 2):
    /// `Ktx2Image` would need to carry layer/face arrays.
    #[error("non-2D KTX2 layout not supported in Fase 1: {reason}")]
    UnsupportedDimensionality { reason: &'static str },

    /// W1.T9 — kvd section declares mais entries que [`MAX_KVD_ENTRIES`](crate::MAX_KVD_ENTRIES).
    /// Hostile file ou tooling explorou kvd para metadata bloat.
    #[error("kvd has {count} entries, exceeds max {max}")]
    TooManyKvdEntries { count: usize, max: usize },

    /// W1.T9 — kvd entry value excede [`MAX_KVD_VALUE_BYTES`](crate::MAX_KVD_VALUE_BYTES). Hostile file
    /// pode embedar arbitrary blobs em metadata.
    #[error("kvd entry '{key}' has {size} bytes, exceeds max {max}")]
    KvdValueTooLarge {
        key: String,
        size: usize,
        max: usize,
    },

    /// W1.T9 audit Lente ξ-F3 — kvd entry key excede [`MAX_KVD_KEY_BYTES`](crate::MAX_KVD_KEY_BYTES).
    /// The offending key is intentionally NOT carried in the error — doing
    /// so would perform the very multi-MiB allocation this bound prevents.
    #[error("kvd entry key has {size} bytes, exceeds max {max}")]
    KvdKeyTooLong { size: usize, max: usize },
}

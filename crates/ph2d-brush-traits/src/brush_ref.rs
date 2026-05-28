//! Opaque brush handle.
//!
//! Per [ADR-0067 §2.2](../../../../docs/architecture/decisions/0067-brush-traits-decoupling.md).

use serde::{Deserialize, Serialize};

/// Opaque handle to a brush. Painter implementations resolve the
/// `BrushRef` to their internal `Brush` struct via lookup; consumers
/// (Vector pattern-along-path, etc.) consume the handle without
/// knowing internals.
///
/// Stable value: a blake3 hash of the `.ph2d-brush` asset bytes,
/// truncated to 64 bits. Stable across machines so `.ph2d-vector`
/// assets referencing brushes resolve identically after asset reload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BrushRef(pub u64);

impl BrushRef {
    /// Construct from a raw blake3-truncated u64.
    #[must_use]
    pub const fn from_u64(hash: u64) -> Self {
        Self(hash)
    }

    /// Sentinel handle for "no brush selected". Engines treat this as
    /// "skip stamp" rather than panicking.
    pub const NONE: Self = Self(0);

    /// Returns `true` if this is the `NONE` sentinel.
    #[must_use]
    pub const fn is_none(self) -> bool {
        self.0 == 0
    }
}

/// Type alias for the unique runtime ID of a brush instance held in a
/// brush library. Mirrors `BrushHandle` naming used by Painter
/// (ADR-0044) so consumers reading both ADRs see a familiar word.
///
/// Distinct from [`BrushRef`]: a `BrushHandle` is a *library slot*
/// (resolved at runtime from the asset DB); a `BrushRef` is a
/// *persistent content-hash identifier* serialized into `.ph2d-vector`
/// files.
pub type BrushHandle = BrushRef;

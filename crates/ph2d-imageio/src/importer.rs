//! `ImageImporter` trait — the contract every format-import crate
//! implements. **Cap FROZEN at 3 methods** by ADR-0054 §2.1 arch-gate
//! `image_importer_contract_is_capped`. Surface uses 2 (one slot of
//! headroom for a future feature like async stream decode).

use crate::{DecodedImage, Error, ImportOpts, MagicHint};

/// Implemented by each `crates/ph2d-imageio-<slug>/` to decode one
/// image format. The runtime registry holds a `Vec<Box<dyn ImageImporter>>`
/// and dispatches based on [`ImageImporter::supports`].
///
/// Implementors are `Send + Sync + 'static` because the registry is
/// constructed once at boot and shared across the editor; per HR-3 the
/// import path is **not hot** (invoked by user click, not per-frame).
pub trait ImageImporter: Send + Sync + 'static {
    /// Quick check: can this importer handle the source identified by
    /// `hint`? Should be `O(1)` — peek at magic bytes or string-compare
    /// the extension. Heavy validation belongs in [`Self::import`].
    fn supports(&self, hint: MagicHint<'_>) -> bool;

    /// Decode `src` to a [`DecodedImage`]. `opts` carry caller
    /// preferences (target color profile, preserve layers/vector); the
    /// importer is free to ignore any preference its format doesn't
    /// support — that is not an error.
    fn import(&self, src: &[u8], opts: &ImportOpts) -> Result<DecodedImage, Error>;
}

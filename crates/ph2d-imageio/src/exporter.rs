//! `ImageExporter` trait — the contract every format-export crate
//! implements. **Cap FROZEN at 3 methods** by ADR-0054 §2.1 arch-gate
//! `image_exporter_contract_is_capped`. Surface uses 2 (one slot of
//! headroom for a future feature like streaming chunked write).

use crate::{DecodedImage, Error, ExportOpts};

/// Implemented by each `crates/ph2d-imageio-<slug>/` to encode one
/// image format. The runtime registry holds a `Vec<Box<dyn ImageExporter>>`
/// and dispatches based on [`ExportOpts::format`].
///
/// Implementors are `Send + Sync + 'static` for the same reason as
/// [`crate::ImageImporter`] — registry is shared, export path is not
/// hot (user click triggered).
pub trait ImageExporter: Send + Sync + 'static {
    /// `true` if this exporter recognizes the format identifier
    /// `fmt` (lowercase, no leading dot — e.g. `"png"`, `"jpeg"`,
    /// `"webp"`). Should be `O(1)` — typically a small string-set
    /// check.
    fn supports_format(&self, fmt: &str) -> bool;

    /// Encode `img` to a byte vector using `opts`. Refuses with
    /// [`Error::HdrUnsupported`] if `img` is `FlatHdr`, the exporter is
    /// LDR-only, and `opts.tone_map == ToneMap::None`.
    fn export(&self, img: &DecodedImage, opts: &ExportOpts) -> Result<Vec<u8>, Error>;
}

//! `ImageExporter` trait — the contract every format-export crate
//! implements. **Cap FROZEN at 3 methods** by ADR-0054 §2.1 arch-gate
//! `image_exporter_contract_is_capped`. Surface uses 2 (one slot of
//! headroom — currently planned for W1.T1 chrome derivation, audit
//! C-H1).

use crate::{DecodedImage, Error, ExportFormat, ExportOpts};

/// Implemented by each `crates/ph2d-imageio-<slug>/` to encode one
/// image format. The runtime registry holds a `Vec<Box<dyn ImageExporter>>`
/// and dispatches based on [`ExportOpts::format`].
///
/// Implementors are `Send + Sync + 'static` for the same reason as
/// [`crate::ImageImporter`] — registry is shared, export path is not
/// hot (user click triggered).
pub trait ImageExporter: Send + Sync + 'static {
    /// `true` if this exporter handles `fmt`. Audit A-H1 (2026-05-26):
    /// was `&str` (typo-unsafe); promoted to [`ExportFormat`] enum so
    /// a misspelt `"jepg"` is a compile error, not a silent
    /// `find_for` miss.
    fn supports_format(&self, fmt: ExportFormat) -> bool;

    /// Encode `img` to a byte vector using `opts`. Refuses with
    /// [`Error::HdrUnsupported`] if `img` is `FlatHdr`, the exporter is
    /// LDR-only, and `opts.tone_map == ToneMap::None`.
    fn export(&self, img: &DecodedImage, opts: &ExportOpts) -> Result<Vec<u8>, Error>;
}

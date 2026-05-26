//! `ImageImporter` trait — the contract every format-import crate
//! implements. **Cap FROZEN at 3 methods** by ADR-0054 §2.1 arch-gate
//! `image_importer_contract_is_capped`. Surface uses 2 (one slot of
//! headroom — currently planned for a chrome-derivation method in
//! W1.T1 follow-up, audit C-H1).

use crate::{DecodedImage, Error, ImportOpts, MagicHint, MagicMatch};

/// Implemented by each `crates/ph2d-imageio-<slug>/` to decode one
/// image format. The runtime registry holds a `Vec<Box<dyn ImageImporter>>`
/// and dispatches based on [`ImageImporter::supports`].
///
/// Implementors are `Send + Sync + 'static` because the registry is
/// constructed once at boot and shared across the editor; per HR-3 the
/// import path is **not hot** (invoked by user click, not per-frame).
pub trait ImageImporter: Send + Sync + 'static {
    /// Quick recognition: does this importer claim `hint`, and how
    /// strongly? Should be `O(1)` — peek at magic bytes (return
    /// [`MagicMatch::Strong`]) or string-compare the extension (return
    /// [`MagicMatch::Weak`]).
    ///
    /// Audit A-M1 (2026-05-26): was `-> bool`; promoted to
    /// `MagicMatch` so [`crate::ImporterRegistry`] disambiguates
    /// overlapping formats correctly (PSB+PSD share magic; TIFF+DNG
    /// share `II*\0`).
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch;

    /// Decode `src` to a [`DecodedImage`]. `opts` carry caller
    /// preferences (target color profile, preserve layers/vector); the
    /// importer is free to ignore any preference its format does not
    /// support — that is not an error, unless
    /// [`ImportOpts::color_profile_strictness`] is `Strict` and the
    /// target profile is unreachable (then return
    /// [`Error::Unsupported`]).
    fn import(&self, src: &[u8], opts: &ImportOpts) -> Result<DecodedImage, Error>;
}

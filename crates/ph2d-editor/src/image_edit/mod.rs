//! Cross-cutting utilities for image-editing actions (Trim
//! Transparency, Background Removal, future Crop / Padding-strip /
//! Auto-Trim). Pure-math helpers that don't belong inside any single
//! tool but get reused across the Image Tools top-bar row.
//!
//! Each submodule is `pub`, free of external deps beyond `std`, and
//! exercises its public surface through `#[cfg(test)]` inline tests
//! so the documentation contract stays close to the implementation.

pub mod recenter;

pub use recenter::{PixelBounds, recenter_after_crop};

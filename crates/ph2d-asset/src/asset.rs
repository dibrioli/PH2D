//! [`Asset`] — the decoded payload behind an [`crate::AssetId`].
//!
//! M6 ships only `ImageRgba8`. Audio, font, vector, and binary blob
//! variants land as their respective milestones (M7+ as needed). The
//! enum is intentionally non-exhaustive so adding variants doesn't
//! break downstream `match`es.
//!
//! `pixels` is wrapped in `Arc<[u8]>` (not `Vec<u8>`) so two
//! consumers — e.g. the renderer's atlas builder + an MCP tool that
//! wants to introspect the data — can share the same allocation.

use std::sync::Arc;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Asset {
    ImageRgba8 {
        width: u32,
        height: u32,
        /// Tight-packed RGBA8: `len == width * height * 4`.
        pixels: Arc<[u8]>,
    },
}

impl Asset {
    /// Convenience: rough byte cost of the decoded payload.  Used
    /// later for HR-13 budget accounting.
    pub fn byte_size(&self) -> usize {
        match self {
            Self::ImageRgba8 { pixels, .. } => pixels.len(),
        }
    }
}

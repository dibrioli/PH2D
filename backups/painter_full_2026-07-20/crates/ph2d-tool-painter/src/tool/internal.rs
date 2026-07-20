//! Free-function helpers + `ToolPixelSource` for the Painter (layers) tool.
//! `pub(crate)` so the impl submodules can call them; re-exported from
//! `tool/mod.rs` via `pub(crate) use internal::*`.

use super::*;
use std::sync::Arc;

/// Blit a freshly-composited `region` (its own `bbox.w × bbox.h` RGBA8 buffer)
/// into the full-canvas composite `cache` at `bbox`, row by row.
pub(crate) fn blit_region(cache: &mut [u8], canvas_w: u32, region: &[u8], bbox: Region) {
    let row_bytes = (bbox.w * 4) as usize;
    for ry in 0..bbox.h {
        let src_off = (ry * bbox.w * 4) as usize;
        let dst_off = (((bbox.y + ry) * canvas_w + bbox.x) * 4) as usize;
        cache[dst_off..dst_off + row_bytes].copy_from_slice(&region[src_off..src_off + row_bytes]);
    }
}

/// [`LayerPixelSource`] over the tool's live buffers: the ACTIVE layer reads
/// `canvas_rgba` (the Arc working buffer — zero-copy, always current), every
/// other layer reads its `images` entry. Built transiently inside the composite
/// paths (`current_preview` / `take_preview_arc` / `run_full`).
pub(crate) struct ToolPixelSource<'a> {
    pub(crate) active_id: RtLayerId,
    pub(crate) active_rgba: &'a [u8],
    /// The NON-active layers' pixels. `Arc` because an undo snapshot must not deep-copy the pixels of
    /// layers the stroke never touched — see [`crate::undo::ModelSnapshot`]. It derefs, so the compositor
    /// below never learns about it.
    pub(crate) images: &'a BTreeMap<RtLayerId, Arc<LayerImage>>,
}

impl LayerPixelSource for ToolPixelSource<'_> {
    fn layer_rgba(&self, id: RtLayerId) -> Option<&[u8]> {
        if id == self.active_id {
            Some(self.active_rgba)
        } else {
            self.images.get(&id).map(|img| img.rgba8.as_slice())
        }
    }
}

/// Take a layer's pixels out of the shared store: free when nobody else holds them (the common case), a
/// copy when an undo snapshot does. The copy-on-write the `Arc` buys — see [`ToolPixelSource::images`].
pub(crate) fn own_image(img: Arc<LayerImage>) -> LayerImage {
    Arc::try_unwrap(img).unwrap_or_else(|shared| shared.as_ref().clone())
}

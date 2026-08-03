//! The baked-tile THUMBNAIL a card's preview moldura can show (doc 86 A5) — split from
//! `snapshot` (its `#[path]` sibling) so `snapshot.rs` stays under the panel LOC cap. `super`
//! is `snapshot`.

/// A small baked-tile image for a card's preview (doc 86 A5). Shared by refcount — the bake
/// owns the bytes and the snapshot clones the `Arc`, so a thumbnail costs a pointer per frame,
/// not a copy. Equality is by IDENTITY (same baked `Arc`), never a byte compare: two cards
/// showing the same object share the tile, and the snapshot diff must stay O(1).
#[derive(Clone)]
pub struct PreviewThumb {
    /// Straight (un-premultiplied) RGBA8, tightly packed `w * h * 4` — the format
    /// [`ph2d_vector::scene::VectorScene::draw_image_rgba`] samples.
    pub rgba: std::sync::Arc<Vec<u8>>,
    pub w: u32,
    pub h: u32,
}

impl std::fmt::Debug for PreviewThumb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PreviewThumb({}x{})", self.w, self.h)
    }
}

impl PartialEq for PreviewThumb {
    fn eq(&self, other: &Self) -> bool {
        self.w == other.w && self.h == other.h && std::sync::Arc::ptr_eq(&self.rgba, &other.rgba)
    }
}

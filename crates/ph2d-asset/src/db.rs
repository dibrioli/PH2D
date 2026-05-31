//! [`AssetDb`] — content-addressed cache, sync API.
//!
//! Internally a `RwLock<AssetDbInner>` holding two `BTreeMap`s:
//! - `by_id`: the actual asset payloads, keyed by [`AssetId`]. The
//!   asset is wrapped in `Arc` so a hot reload swap can replace the
//!   map entry without invalidating outstanding references — readers
//!   that captured the old `Arc<Asset>` keep using the old bytes
//!   until their copy drops.
//! - `by_path`: which `AssetId` a given source path currently
//!   resolves to. This is metadata, not identity. After a hot reload,
//!   the same path may point at a new id (because the bytes changed).
//!
//! HashMap is banned in simulation crates per ADR-0022; ph2d-asset
//! consumed by sim → use `BTreeMap` everywhere.

use crate::asset::Asset;
use crate::error::AssetError;
use crate::id::AssetId;
use crate::loader::{decode_image_bytes, decode_png_bytes};
use crate::prefab::PrefabDoc;
use crate::scene::SceneDoc;
use crate::tier::TierIndex;
use crate::watcher::ReloadEvent;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Default)]
struct AssetDbInner {
    by_id: BTreeMap<AssetId, Arc<Asset>>,
    by_path: BTreeMap<PathBuf, AssetId>,
}

/// Lock acquisitions use `unwrap_or_else(into_inner)` instead of
/// `expect("poisoned")`. A panic in any code path holding the lock
/// (e.g. PNG decode panicking under memory pressure) leaves the
/// state intact under the standard library's poison protocol; we
/// honor "best effort" rather than cascade-crash every subsequent
/// asset read. Worst case: callers see a snapshot from immediately
/// before the panicking write — preferable to dropping the engine.
pub struct AssetDb {
    inner: RwLock<AssetDbInner>,
}

impl AssetDb {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(AssetDbInner::default()),
        }
    }

    /// Decode `bytes` as PNG, hash it, and store under the resulting
    /// [`AssetId`]. Returns the id. Idempotent — calling with
    /// identical bytes is a no-op (existing entry untouched).
    pub fn insert_png_bytes(&self, bytes: &[u8]) -> Result<AssetId, AssetError> {
        let id = AssetId::from_bytes(bytes);
        let asset = decode_png_bytes(bytes, None)?;
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        g.by_id.entry(id).or_insert_with(|| Arc::new(asset));
        Ok(id)
    }

    /// Read the file at `path`, decode it, store, and remember the
    /// path → id mapping for future hot-reload events.
    pub fn load_png_path(&self, path: &Path) -> Result<AssetId, AssetError> {
        let bytes = std::fs::read(path).map_err(|e| AssetError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let id = AssetId::from_bytes(&bytes);
        let asset = decode_png_bytes(&bytes, Some(path))?;
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        g.by_id.entry(id).or_insert_with(|| Arc::new(asset));
        g.by_path.insert(path.to_path_buf(), id);
        Ok(id)
    }

    /// Decode `bytes` as any supported image format (PNG / WEBP /
    /// JPEG; auto-detected via `image::guess_format`), hash the
    /// raw bytes, and store under the resulting [`AssetId`].
    /// Idempotent (HR-6 content-addressed).
    ///
    /// Used by the M14.4c "Import…" button + the M14.4d drag-and-drop
    /// path. PNG-only callers can keep using
    /// [`AssetDb::insert_png_bytes`] for a tighter error type.
    pub fn insert_image_bytes(&self, bytes: &[u8]) -> Result<AssetId, AssetError> {
        let id = AssetId::from_bytes(bytes);
        let asset = decode_image_bytes(bytes, None)?;
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        g.by_id.entry(id).or_insert_with(|| Arc::new(asset));
        Ok(id)
    }

    /// Decode `bytes` as a postcard-encoded [`PrefabDoc`], hash the
    /// raw bytes, and store under the resulting [`AssetId`].
    /// Idempotent (HR-6 content-addressed).
    ///
    /// `bytes` is the **cooked** payload: the cooker
    /// (`tools/asset-cooker`) consumed a JSON5 source file and
    /// produced these bytes. They are not parsed for schema-version
    /// migration here — that responsibility lives in
    /// `crates/ph2d-asset/src/migration.rs` (lands when v2 ships).
    pub fn insert_prefab_bytes(&self, bytes: &[u8]) -> Result<AssetId, AssetError> {
        let id = AssetId::from_bytes(bytes);
        let doc: PrefabDoc = postcard::from_bytes(bytes).map_err(|e| AssetError::Decode {
            path: None,
            message: format!("PrefabDoc postcard: {e}"),
        })?;
        if doc.version != PrefabDoc::VERSION {
            return Err(AssetError::VersionMismatch {
                what: "PrefabDoc",
                got: doc.version,
                expected: PrefabDoc::VERSION,
            });
        }
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        g.by_id
            .entry(id)
            .or_insert_with(|| Arc::new(Asset::Prefab(Arc::new(doc))));
        Ok(id)
    }

    /// Same shape as [`AssetDb::insert_prefab_bytes`] but decodes a
    /// [`SceneDoc`]. Separate method (vs an `AssetKind` enum) so the
    /// type-error is reported at the call site, not deferred.
    pub fn insert_scene_bytes(&self, bytes: &[u8]) -> Result<AssetId, AssetError> {
        let id = AssetId::from_bytes(bytes);
        let doc: SceneDoc = postcard::from_bytes(bytes).map_err(|e| AssetError::Decode {
            path: None,
            message: format!("SceneDoc postcard: {e}"),
        })?;
        if doc.version != SceneDoc::VERSION {
            return Err(AssetError::VersionMismatch {
                what: "SceneDoc",
                got: doc.version,
                expected: SceneDoc::VERSION,
            });
        }
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        g.by_id
            .entry(id)
            .or_insert_with(|| Arc::new(Asset::Scene(Arc::new(doc))));
        Ok(id)
    }

    /// Store a **cooked KTX2 blob** (one tier's artifact emitted by
    /// `tools/asset-cooker`) under its content-addressed [`AssetId`]
    /// (`blake3(bytes)`, HR-6). Idempotent — re-inserting identical bytes
    /// is a no-op. The KTX2 container is **not** decoded here: `ph2d-asset`
    /// deliberately carries no `ph2d-asset-ktx2` dependency (W1.T4), so the
    /// raw bytes are stored verbatim and the renderer (W2.T4 loader) decodes
    /// then uploads them via `ph2d_asset_ktx2::decode_ktx2_bytes` at upload
    /// time (not the hot path; HR-3 ok).
    ///
    /// `tier` records which platform tier this artifact targets (Desktop=BC,
    /// Mobile=ASTC, … per the cooker target matrix); the loader resolves a
    /// [`crate::LogicalTextureId`] + the active device tier to one of these
    /// via [`crate::LogicalTextureMap`] / [`crate::logical_texture_resolve`].
    pub fn insert_ktx2_bytes(&self, tier: TierIndex, bytes: &[u8]) -> AssetId {
        let id = AssetId::from_bytes(bytes);
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        g.by_id.entry(id).or_insert_with(|| {
            Arc::new(Asset::TextureKtx2 {
                tier,
                blob: Arc::new(bytes.to_vec()),
            })
        });
        id
    }

    pub fn get(&self, id: &AssetId) -> Option<Arc<Asset>> {
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        g.by_id.get(id).cloned()
    }

    pub fn id_for_path(&self, path: &Path) -> Option<AssetId> {
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        g.by_path.get(path).copied()
    }

    /// All paths currently tracked. Order is `BTreeMap`-canonical
    /// (lexicographic on `PathBuf`).
    pub fn tracked_paths(&self) -> Vec<PathBuf> {
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        g.by_path.keys().cloned().collect()
    }

    pub fn len_assets(&self) -> usize {
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        g.by_id.len()
    }

    pub fn len_paths(&self) -> usize {
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        g.by_path.len()
    }

    /// Apply a batch of pending [`ReloadEvent`]s atomically (single
    /// write-lock acquisition) at the frame boundary. This is the
    /// HR-6 "swap atomic em <frame> boundary" point — every event is
    /// visible to readers in the same instant or none of them are.
    ///
    /// Old `Arc<Asset>` references handed out before the swap remain
    /// valid (Arc-counted). Subsequent `get(id)` will see the new
    /// content if the id changed.
    pub fn apply_pending(&self, events: Vec<ReloadEvent>) {
        if events.is_empty() {
            return;
        }
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        for ev in events {
            match ev {
                ReloadEvent::Changed { path, id, asset } => {
                    g.by_id.entry(id).or_insert_with(|| Arc::new(asset));
                    g.by_path.insert(path, id);
                }
                ReloadEvent::Removed { path } => {
                    // Drop the path mapping but keep the asset in
                    // `by_id` — direct AssetId handles must remain
                    // valid (HR-6 "path renaming não invalida
                    // handles" generalizes to "removal either").
                    g.by_path.remove(&path);
                }
                ReloadEvent::Error { .. } => {
                    // Errors are observable via the watcher's drain;
                    // the db state is unchanged. We intentionally do
                    // NOT remove the previous successful entry — the
                    // last-known-good asset stays available.
                }
            }
        }
    }
}

impl Default for AssetDb {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::watcher::ReloadEvent;

    /// Build a 1×1 transparent PNG with a single solid color. Tiny,
    /// no fixture files needed.
    fn tiny_png(r: u8, g: u8, b: u8, a: u8) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut img = image::RgbaImage::new(1, 1);
        img.put_pixel(0, 0, image::Rgba([r, g, b, a]));
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .expect("encode");
        buf
    }

    #[test]
    fn insert_png_bytes_is_idempotent() {
        let db = AssetDb::new();
        let bytes = tiny_png(255, 0, 0, 255);
        let id1 = db.insert_png_bytes(&bytes).unwrap();
        let id2 = db.insert_png_bytes(&bytes).unwrap();
        assert_eq!(id1, id2);
        assert_eq!(db.len_assets(), 1);
    }

    #[test]
    fn distinct_pngs_get_distinct_ids() {
        let db = AssetDb::new();
        let a = db.insert_png_bytes(&tiny_png(255, 0, 0, 255)).unwrap();
        let b = db.insert_png_bytes(&tiny_png(0, 255, 0, 255)).unwrap();
        assert_ne!(a, b);
        assert_eq!(db.len_assets(), 2);
    }

    #[test]
    fn get_returns_decoded_asset() {
        let db = AssetDb::new();
        let id = db.insert_png_bytes(&tiny_png(10, 20, 30, 255)).unwrap();
        let asset = db.get(&id).expect("present");
        match &*asset {
            Asset::ImageRgba8 {
                width,
                height,
                pixels,
            } => {
                assert_eq!((*width, *height), (1, 1));
                assert_eq!(&pixels[..], &[10, 20, 30, 255]);
            }
            // Other variants are out of scope for this PNG-decode test —
            // they have dedicated insert_postcard_* / cooked-texture helpers
            // and roundtrip tests in `prefab.rs` / `scene.rs` /
            // architecture_texture_ktx2.rs. The `_` also absorbs future
            // `#[non_exhaustive]` Asset variants (e.g. ADR-0055.1 evoluindo
            // TextureKtx2 → Ktx2Image) — naming Prefab/Scene/TextureKtx2
            // explicitly here made `_` unreachable (clippy).
            _ => unreachable!("insert_png_bytes can only produce Asset::ImageRgba8"),
        }
    }

    #[test]
    fn insert_ktx2_bytes_stores_blob_and_tier_idempotently() {
        let db = AssetDb::new();
        let blob = vec![0xABu8, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30]; // partial KTX2 magic
        let id1 = db.insert_ktx2_bytes(TierIndex::DESKTOP, &blob);
        let id2 = db.insert_ktx2_bytes(TierIndex::DESKTOP, &blob);
        // Content-addressed (HR-6): identical bytes → identical id, one entry.
        assert_eq!(id1, id2);
        assert_eq!(db.len_assets(), 1);

        match &*db.get(&id1).expect("present") {
            Asset::TextureKtx2 { tier, blob: got } => {
                assert_eq!(*tier, TierIndex::DESKTOP);
                assert_eq!(&got[..], &blob[..]);
            }
            _ => unreachable!("insert_ktx2_bytes only produces Asset::TextureKtx2"),
        }

        // A different tier of the SAME bytes hashes the same — content
        // addressing keys on bytes, so the first-writer tier wins (the
        // cooker emits distinct bytes per tier, so this collision can't
        // happen in practice; the `or_insert_with` keeps it deterministic).
        let id3 = db.insert_ktx2_bytes(TierIndex::MOBILE, &blob);
        assert_eq!(id3, id1);
        assert_eq!(db.len_assets(), 1);
    }

    #[test]
    fn apply_pending_changed_swaps_path_mapping() {
        let db = AssetDb::new();
        let path = PathBuf::from("/virt/sprite.png");
        let old_id = db.insert_png_bytes(&tiny_png(1, 2, 3, 255)).unwrap();
        // Manually establish the path mapping (load_png_path needs a
        // real file; this tests the apply codepath).
        {
            let mut g = db.inner.write().unwrap();
            g.by_path.insert(path.clone(), old_id);
        }

        let new_bytes = tiny_png(9, 9, 9, 255);
        let new_id = AssetId::from_bytes(&new_bytes);
        let new_asset = decode_png_bytes(&new_bytes, None).unwrap();

        db.apply_pending(vec![ReloadEvent::Changed {
            path: path.clone(),
            id: new_id,
            asset: new_asset,
        }]);

        assert_eq!(db.id_for_path(&path), Some(new_id));
        assert_ne!(new_id, old_id);
        // Both old and new live in by_id — old handles remain valid.
        assert!(db.get(&old_id).is_some());
        assert!(db.get(&new_id).is_some());
    }

    #[test]
    fn rwlock_poison_does_not_cascade_crash() {
        use std::sync::Arc as StdArc;
        let db = StdArc::new(AssetDb::new());
        let bytes = tiny_png(1, 2, 3, 255);
        let id = db.insert_png_bytes(&bytes).unwrap();

        // Spawn a thread that grabs the write lock and panics.
        // This poisons the inner RwLock per std::sync semantics.
        let db2 = StdArc::clone(&db);
        let h = std::thread::spawn(move || {
            let _g = db2.inner.write().unwrap();
            panic!("synthetic poison");
        });
        let _ = h.join(); // Result is Err — that's the point.

        // Subsequent reads must NOT panic. The lock is poisoned, but
        // our `unwrap_or_else(into_inner)` wrapper recovers the guard.
        let asset = db.get(&id);
        assert!(
            asset.is_some(),
            "post-poison read must succeed (data is intact)"
        );
        // Writes also recover.
        let new = tiny_png(9, 9, 9, 255);
        let id2 = db.insert_png_bytes(&new).unwrap();
        assert_ne!(id, id2);
        assert_eq!(db.len_assets(), 2);
    }

    #[test]
    fn apply_pending_removed_keeps_asset_for_handles() {
        let db = AssetDb::new();
        let path = PathBuf::from("/virt/x.png");
        let id = db.insert_png_bytes(&tiny_png(7, 7, 7, 255)).unwrap();
        {
            let mut g = db.inner.write().unwrap();
            g.by_path.insert(path.clone(), id);
        }
        db.apply_pending(vec![ReloadEvent::Removed { path: path.clone() }]);
        assert_eq!(db.id_for_path(&path), None);
        // Direct id handle still resolves — HR-6.
        assert!(db.get(&id).is_some());
    }
}

//! Renderer-side cache of cooked KTX2 textures (texture-compression
//! Wave 2, W2.T4 — the loader path that makes [`SpriteSource::CookedTexture`]
//! actually render).
//!
//! ## Where this sits in the pipeline
//!
//! [`SpriteSource::CookedTexture`](crate::SpriteSource::CookedTexture) stores a
//! **tier-agnostic** [`ph2d_asset::LogicalTextureId`] so a sprite stays
//! portable across devices. Turning that into pixels on the GPU is a
//! three-step resolve the W2.T4 loader drives once (NOT per frame):
//!
//! 1. **Tier** — the device's richest sampleable tier
//!    ([`CompressionFeatureSet::best_tier`](crate::CompressionFeatureSet::best_tier)),
//!    descending [`TierIndex::fallback_ladder`](ph2d_asset::TierIndex::fallback_ladder)
//!    to the RGBA8 floor when the preferred tier wasn't cooked.
//! 2. **Resolve** — `logical_id + tier → AssetId → Arc<Asset::TextureKtx2>`
//!    ([`ph2d_asset::logical_texture_resolve`]); the shell owns the
//!    [`LogicalTextureMap`](ph2d_asset::LogicalTextureMap) + [`AssetDb`](ph2d_asset::AssetDb).
//! 3. **Upload + cache** — *this* store: decode the KTX2 blob once
//!    ([`ph2d_asset_ktx2::decode_ktx2_bytes`]) and upload it through the shared
//!    [`CompressedTexturePipeline`](crate::compressed_pipeline) (W2.T3),
//!    caching the ready-to-bind [`UploadedCompressedTexture`] keyed by a
//!    renderer-assigned `texture_id` in the cooked namespace
//!    ([`RenderInstance::COOKED_TEXTURE_ID_BIT`](crate::RenderInstance::COOKED_TEXTURE_ID_BIT)).
//!
//! The extract phase then reads [`Self::texture_id`] (logical → the cached
//! `texture_id`) and stamps it into the sprite's `RenderInstance`; the
//! renderer's draw loop binds [`Self::bind_group`] for any run whose
//! `texture_id` is in the cooked namespace — exactly mirroring the
//! `IndividualTextureStore` bind path (handoff §TASK step 4).
//!
//! HR-5 / ADR-0022: all three maps are `BTreeMap`, so iteration (and the id
//! allocation it could influence) stays deterministic.

use crate::compressed_pipeline::{
    CompressedTexturePipeline, CompressedUploadError, UploadedCompressedTexture,
};
use crate::ktx2_format::CompressionFeatureSet;
use crate::sprite::RenderInstance;
use ph2d_asset::{AssetId, LogicalTextureId};
use ph2d_asset_ktx2::{Ktx2Error, decode_ktx2_bytes};
use ph2d_gpu::GpuContext;
use std::collections::BTreeMap;

/// Why a cooked KTX2 blob could not be turned into a bound GPU texture.
#[derive(Debug)]
pub enum CookedTextureError {
    /// The stored KTX2 container bytes failed to parse (truncated / corrupt
    /// artifact, or an out-of-subset VkFormat the parser rejects).
    Decode(Ktx2Error),
    /// The decoded image parsed but could not be uploaded — see
    /// [`CompressedUploadError`] (unmappable / device-unsupported format,
    /// zero mips, truncated mip payload, or a non-filterable-float format).
    /// `FormatUnsupportedByDevice` here means the loader's fallback ladder
    /// proposed a tier this GPU can't sample — the caller should descend to
    /// the next ladder rung.
    Upload(CompressedUploadError),
}

impl core::fmt::Display for CookedTextureError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Decode(e) => write!(f, "cooked KTX2 decode failed: {e}"),
            Self::Upload(e) => write!(f, "cooked KTX2 upload failed: {e}"),
        }
    }
}

impl core::error::Error for CookedTextureError {}

impl From<Ktx2Error> for CookedTextureError {
    fn from(e: Ktx2Error) -> Self {
        Self::Decode(e)
    }
}

impl From<CompressedUploadError> for CookedTextureError {
    fn from(e: CompressedUploadError) -> Self {
        Self::Upload(e)
    }
}

/// Renderer-side cache of uploaded cooked KTX2 textures.
///
/// One per [`SpriteRenderer`](crate::SpriteRenderer). The
/// [`CompressedTexturePipeline`] is built lazily on the first cooked sprite
/// (a scene with none never pays for it — zero-regression), bound to the
/// `SpritePipeline`'s `material_bgl` so a cooked texture's bind group drops
/// straight into `@group(1)` like an atlas / individual one.
pub struct CookedTextureStore {
    /// Lazily built (needs the `material_bgl` + a device feature query).
    /// `None` until the first [`Self::ensure`] uploads something.
    pipeline: Option<CompressedTexturePipeline>,
    /// Cooked `texture_id` → uploaded GPU texture (draw path: id → bind group).
    entries: BTreeMap<u32, UploadedCompressedTexture>,
    /// [`AssetId`] → cooked `texture_id` (dedup: two logical ids resolving to
    /// the same cooked artifact share ONE GPU upload).
    asset_to_id: BTreeMap<AssetId, u32>,
    /// [`LogicalTextureId`] → cooked `texture_id` (extract path: a sprite
    /// carries only the logical id; the tier + asset resolution is already
    /// folded in here at upload time).
    logical_to_id: BTreeMap<LogicalTextureId, u32>,
    /// Next id to hand out, in the [`RenderInstance::COOKED_TEXTURE_ID_BIT`]
    /// namespace.
    next_id: u32,
}

impl Default for CookedTextureStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CookedTextureStore {
    /// An empty store. The GPU pipeline is not built until the first
    /// [`Self::ensure`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            pipeline: None,
            entries: BTreeMap::new(),
            asset_to_id: BTreeMap::new(),
            logical_to_id: BTreeMap::new(),
            // First cooked id = the namespace bit | 1 (so it's never the
            // atlas sentinel `0` and never collides with an individual id).
            next_id: RenderInstance::COOKED_TEXTURE_ID_BIT | 1,
        }
    }

    /// Number of distinct cooked textures uploaded (one per [`AssetId`]).
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The compression-feature capabilities the lazily-built pipeline gates
    /// uploads against, or `None` before the first upload built it.
    #[must_use]
    pub fn feature_set(&self) -> Option<CompressionFeatureSet> {
        self.pipeline
            .as_ref()
            .map(CompressedTexturePipeline::feature_set)
    }

    /// Total GPU byte footprint of every cached cooked texture (W2.T5 HR-13
    /// accounting): the sum of
    /// [`compressed_size_per_format`](crate::compressed_pipeline::compressed_size_per_format)
    /// over each uploaded entry's `(format, width, height, mip_level_count)`.
    /// Compared against
    /// [`COMPRESSED_TEXTURE_CACHE_BUDGET_MB`](crate::compressed_pipeline::COMPRESSED_TEXTURE_CACHE_BUDGET_MB).
    /// `0` for an empty store (no GPU allocation until the first upload).
    #[must_use]
    pub fn cache_bytes(&self) -> u64 {
        self.entries
            .values()
            .map(|u| {
                crate::compressed_pipeline::compressed_size_per_format(
                    u.format,
                    u.width,
                    u.height,
                    u.mip_level_count,
                )
            })
            .sum()
    }

    /// The cooked `texture_id` a `logical_id` resolved to, or `None` if it
    /// was never [`ensure`](Self::ensure)d (not uploaded — e.g. no cooked
    /// artifact for any tier in the device's fallback ladder). The extract
    /// phase reads this to stamp the sprite's `RenderInstance.texture_id`.
    #[must_use]
    pub fn texture_id(&self, logical_id: LogicalTextureId) -> Option<u32> {
        self.logical_to_id.get(&logical_id).copied()
    }

    /// The ready-to-bind material bind group for a cooked `texture_id`
    /// (`@group(1)`), or `None` for an id this store never uploaded. The
    /// renderer's draw loop calls this for any run whose `texture_id` is in
    /// the cooked namespace.
    #[must_use]
    pub fn bind_group(&self, texture_id: u32) -> Option<&wgpu::BindGroup> {
        self.entries.get(&texture_id).map(|u| &u.bind_group)
    }

    /// Base-level `(width, height)` of a cooked `texture_id`, or `None`.
    /// Parallels [`IndividualTextureStore::dims`](crate::IndividualTextureStore::dims)
    /// (future region-rect support on cooked sprites).
    #[must_use]
    pub fn dims(&self, texture_id: u32) -> Option<(u32, u32)> {
        self.entries.get(&texture_id).map(|u| (u.width, u.height))
    }

    /// Ensure the cooked texture for `logical_id` is uploaded + cached,
    /// returning its `texture_id` in the cooked namespace.
    ///
    /// Idempotent and cheap on the hot path: a `logical_id` already resolved
    /// returns its id with two `BTreeMap` lookups and no GPU work. A new
    /// `logical_id` that resolves to an `asset_id` already uploaded (two
    /// logical ids sharing a cooked artifact) re-uses that upload. Only a
    /// genuinely-new `asset_id` decodes + uploads.
    ///
    /// `material_bgl` must be the `SpritePipeline`'s material layout so the
    /// produced bind group is pipeline-compatible; it is used only to build
    /// the shared pipeline on the first call.
    ///
    /// # Errors
    /// [`CookedTextureError::Decode`] for a corrupt KTX2 blob;
    /// [`CookedTextureError::Upload`] when the format is unmappable, the
    /// device can't sample it (loader should fall to the next tier), or a
    /// mip payload is truncated. On error nothing is cached.
    pub fn ensure(
        &mut self,
        gpu: &GpuContext,
        material_bgl: &wgpu::BindGroupLayout,
        logical_id: LogicalTextureId,
        asset_id: AssetId,
        ktx2_blob: &[u8],
    ) -> Result<u32, CookedTextureError> {
        // Fast path: this logical id already maps to an uploaded texture.
        if let Some(id) = self.logical_to_id.get(&logical_id) {
            return Ok(*id);
        }
        // Dedup: a different logical id already uploaded this exact cooked
        // artifact — share the GPU texture, just record the new alias.
        if let Some(id) = self.asset_to_id.get(&asset_id).copied() {
            self.logical_to_id.insert(logical_id, id);
            return Ok(id);
        }

        // Cold path: decode + upload. Build the shared pipeline on first use,
        // gated to what this device can sample.
        let pipeline = self.pipeline.get_or_insert_with(|| {
            let feature_set = CompressionFeatureSet::from_features(gpu.device.features());
            CompressedTexturePipeline::with_layout(gpu, feature_set, material_bgl.clone())
        });

        let image = decode_ktx2_bytes(ktx2_blob)?;
        let label = format!("ph2d-render cooked texture {logical_id}");
        let uploaded = pipeline.upload(gpu, &image, Some(&label))?;

        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("CookedTextureStore: cooked texture_id space exhausted");
        self.entries.insert(id, uploaded);
        self.asset_to_id.insert(asset_id, id);
        self.logical_to_id.insert(logical_id, id);
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_store_has_no_pipeline_and_resolves_nothing() {
        let store = CookedTextureStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
        assert!(store.feature_set().is_none());
        // No GPU allocation until the first upload → zero budget footprint.
        assert_eq!(store.cache_bytes(), 0);
        let logical = LogicalTextureId::from_source_bytes(b"hero.png");
        assert!(store.texture_id(logical).is_none());
        // Unknown id → no bind group, no dims (renderer skips the run).
        assert!(
            store
                .bind_group(RenderInstance::COOKED_TEXTURE_ID_BIT | 1)
                .is_none()
        );
        assert!(
            store
                .dims(RenderInstance::COOKED_TEXTURE_ID_BIT | 1)
                .is_none()
        );
    }

    #[test]
    fn first_cooked_id_is_in_the_cooked_namespace() {
        // The starting id must be tagged so `material_bg` routes it to this
        // store (and never collides with atlas `0` / individual `1..`).
        let store = CookedTextureStore::new();
        assert!(RenderInstance::is_cooked_texture_id(store.next_id));
        assert_eq!(store.next_id, RenderInstance::COOKED_TEXTURE_ID_BIT | 1);
        // The starting id is distinctly NOT the atlas sentinel.
        assert_ne!(store.next_id, RenderInstance::ATLAS_TEXTURE_ID);
    }

    // ── GPU end-to-end (W2.T4 loader chain; #[ignore] — no GPU on CI) ───
    //
    // Proves the full cooked path on a real adapter: encode an uncompressed
    // RGBA8 KTX2 → `ensure` decodes + uploads it → a bindable texture_id in
    // the cooked namespace, with idempotency + AssetId dedup. Mirrors the
    // `compressed_pipeline` GPU-smoke gating (run with `--ignored` on a dev
    // machine).

    fn try_headless_gpu() -> Option<ph2d_gpu::GpuContext> {
        let instance = ph2d_gpu::GpuContext::default_instance();
        ph2d_gpu::GpuContext::new(instance, None).ok()
    }

    #[test]
    #[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
    fn ensure_uploads_decodes_and_dedupes_on_real_device() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let bgl = crate::compressed_pipeline::CompressedTexturePipeline::material_bind_group_layout(
            &gpu.device,
        );
        let mut store = CookedTextureStore::new();

        // Encode a 4×4 RGBA8 KTX2 (the Constrained-tier artifact the loader
        // ladder always lands on — uploadable on every device).
        let (w, h) = (4u32, 4u32);
        let rgba = vec![200u8; (w * h * 4) as usize];
        let blob = ph2d_asset_ktx2::encode_uncompressed_rgba8(w, h, true, &rgba).expect("encode");
        let logical = LogicalTextureId::from_source_bytes(b"hero.png");
        let asset_id = ph2d_asset::AssetId::from_bytes(&blob);

        let id = store
            .ensure(&gpu, &bgl, logical, asset_id, &blob)
            .expect("decode + upload succeeds for uncompressed RGBA8");
        // The id is in the cooked namespace + binds + resolves both ways.
        assert!(RenderInstance::is_cooked_texture_id(id));
        assert!(store.bind_group(id).is_some());
        assert_eq!(store.texture_id(logical), Some(id));
        assert_eq!(store.dims(id), Some((w, h)));
        assert_eq!(store.len(), 1);
        assert!(store.cache_bytes() >= (w * h * 4) as u64, "RGBA8 footprint");
        assert!(
            store.feature_set().is_some(),
            "pipeline built on first upload"
        );

        // Idempotent: same logical id → same id, no new upload.
        let again = store.ensure(&gpu, &bgl, logical, asset_id, &blob).unwrap();
        assert_eq!(again, id);
        assert_eq!(store.len(), 1);

        // Dedup: a DIFFERENT logical id resolving to the SAME asset shares
        // the one GPU upload.
        let logical2 = LogicalTextureId::from_source_bytes(b"villain.png");
        let id2 = store.ensure(&gpu, &bgl, logical2, asset_id, &blob).unwrap();
        assert_eq!(id2, id, "same asset → same cooked id (deduped)");
        assert_eq!(store.len(), 1, "no second upload");
        assert_eq!(store.texture_id(logical2), Some(id));
    }
}

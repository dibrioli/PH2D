//! KTX2 Fase 2 (W2.T4) — cooked-texture loader pass.
//!
//! Runs once per frame BEFORE `sim_extract`, so the extract phase can resolve
//! a `SpriteSource::CookedTexture { logical_id }` sprite to a bound GPU
//! `texture_id`. For every cooked sprite NOT yet uploaded it:
//!
//! 1. takes the device's preferred tier
//!    ([`SpriteRenderer::active_device_tier`]) and walks its
//!    [`fallback_ladder`](ph2d_asset::TierIndex::fallback_ladder) toward the
//!    RGBA8 floor;
//! 2. resolves `logical_id + tier → AssetId → Arc<Asset::TextureKtx2>`
//!    (`logical_map` + `asset_db`, both shell-owned);
//! 3. hands the KTX2 blob to [`SpriteRenderer::ensure_cooked_texture`], which
//!    decodes + uploads it once and caches the `texture_id`.
//!
//! A tier the device can't actually sample (`FormatUnsupportedByDevice`) is
//! skipped and the ladder descends; `Constrained` (uncompressed RGBA8) is the
//! universal floor, so a cooked artifact registered for it always binds. A
//! logical texture with no cooked artifact for ANY ladder rung is mapped to a
//! **magenta missing-texture placeholder** (W2.T4 plan addendum — a visible
//! debug indicator, not a silent invisible drop) and is warned about ONCE
//! (deduped so a permanently-missing asset can't spam the log every frame).
//!
//! The pass is idempotent + cheap after warm-up: an already-uploaded (or
//! magenta-marked) `logical_id` short-circuits on a single map lookup, so
//! steady-state cost is one `&Sprite` query walk (same per-frame pattern as
//! `sim_extract`).

use ph2d_asset::{Asset, AssetDb, LogicalTextureId, LogicalTextureMap};
use ph2d_ecs::SimWorld;
use ph2d_render::{
    CompressedUploadError, CookedTextureError, Sprite, SpriteRenderer, SpriteSource,
};
use std::cell::RefCell;
use std::collections::BTreeSet;

thread_local! {
    /// Logical ids we've GIVEN UP on — no device-sampleable cooked tier
    /// resolved, OR every candidate tier failed to decode/upload (a corrupt
    /// artifact). Render-loop is single-threaded. Two jobs: (1) warn once
    /// per id instead of once per frame; (2) STOP retrying — a permanently-
    /// missing/corrupt blob must not re-walk the ladder + re-decode every
    /// frame (the upload cache only short-circuits *successful* loads, so
    /// without this a hard-error sprite would spin + spam forever). A future
    /// async asset-reload path would clear this set on reload.
    static GIVEN_UP: RefCell<BTreeSet<LogicalTextureId>> = const { RefCell::new(BTreeSet::new()) };
}

/// `true` if we've already given up on `logical_id` (read-only peek).
fn given_up(logical_id: LogicalTextureId) -> bool {
    GIVEN_UP.with_borrow(|s| s.contains(&logical_id))
}

/// Ensure every `SpriteSource::CookedTexture` sprite in `sim` has its KTX2
/// texture decoded + uploaded + cached on `renderer`. See the module header.
pub(super) fn ensure_uploaded(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    logical_map: &LogicalTextureMap,
) {
    let preferred = renderer.active_device_tier();
    // `World::query` needs `&mut World` to build the state; the iteration
    // then borrows `&World`. `renderer` / `asset_db` / `logical_map` are all
    // disjoint from `sim`, so the per-sprite upload calls don't conflict.
    let mut q = sim.world_mut().query::<&Sprite>();
    let world = sim.world();
    for sprite in q.iter(world) {
        let SpriteSource::CookedTexture { logical_id } = sprite.source else {
            continue;
        };
        if renderer.cooked_texture_id(logical_id).is_some() {
            continue; // already uploaded (steady state).
        }
        if given_up(logical_id) {
            continue; // permanently missing/corrupt — don't re-walk/re-decode.
        }
        // Descend the device fallback ladder, taking the first tier that has a
        // cooked artifact AND that this GPU can sample. Record the last hard
        // error (corrupt/unmappable blob) so a give-up can report a cause.
        let mut last_error: Option<String> = None;
        for tier in preferred.fallback_ladder() {
            let Some(asset_id) = logical_map.resolve(logical_id, tier) else {
                continue;
            };
            let Some(asset) = asset_db.get(&asset_id) else {
                continue;
            };
            let Asset::TextureKtx2 { blob, .. } = &*asset else {
                continue; // logical id mapped to a non-KTX2 asset (content bug).
            };
            match renderer.ensure_cooked_texture(logical_id, asset_id, blob.as_slice()) {
                Ok(_) => break,
                // This tier's format isn't sampleable here — descend the ladder.
                Err(CookedTextureError::Upload(
                    CompressedUploadError::FormatUnsupportedByDevice(_),
                )) => continue,
                // A corrupt/unmappable blob for THIS tier: a different (lower)
                // tier might still resolve, so keep walking — but remember the
                // cause in case every tier fails.
                Err(e) => {
                    last_error = Some(format!("tier {tier}: {e}"));
                    continue;
                }
            }
        }
        // Nothing bound after the whole ladder → give up. Map the id to the
        // magenta missing-texture placeholder (W2.T4 plan addendum) so the
        // sprite renders visibly magenta — a debug aid, not an invisible
        // silent drop. The placeholder mapping also short-circuits the loader
        // on later frames (cooked_texture_id → Some), so we never re-walk +
        // re-decode a missing/corrupt blob; GIVEN_UP keeps the warn to once.
        if renderer.cooked_texture_id(logical_id).is_none() {
            let magenta = renderer.mark_cooked_missing(logical_id).is_ok();
            GIVEN_UP.with_borrow_mut(|s| {
                if s.insert(logical_id) {
                    let shown = if magenta {
                        "renders magenta (missing texture)"
                    } else {
                        "renders invisible"
                    };
                    match &last_error {
                        Some(cause) => eprintln!(
                            "W2.T4: cooked KTX2 for logical texture {logical_id} failed to load \
                             ({cause}) — sprite {shown}"
                        ),
                        None => eprintln!(
                            "W2.T4: no device-sampleable cooked KTX2 tier for logical texture \
                             {logical_id} — sprite {shown}"
                        ),
                    }
                }
            });
        }
    }
}

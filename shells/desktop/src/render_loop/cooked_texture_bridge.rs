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
//! logical texture with no cooked artifact for ANY ladder rung stays
//! invisible (like a hidden/culled sprite) and is warned about ONCE (deduped
//! so a permanently-missing asset can't spam the log every frame).
//!
//! The pass is idempotent + cheap after warm-up: an already-uploaded
//! `logical_id` short-circuits on a single map lookup, so steady-state cost is
//! one `&Sprite` query walk (the same per-frame pattern `sim_extract` uses).
//!
//! **Known follow-up (W2.T4 addendum):** the plan calls for a magenta
//! missing-texture sprite when no cooked artifact resolves. This pass renders
//! such sprites *invisible* (safe, matches hidden/culled) + logs once; the
//! magenta debug indicator is a small renderer-side enhancement left for a
//! follow-up so this loader stays additive.

use ph2d_asset::{Asset, AssetDb, LogicalTextureId, LogicalTextureMap};
use ph2d_ecs::SimWorld;
use ph2d_render::{CompressedUploadError, CookedTextureError, Sprite, SpriteRenderer, SpriteSource};
use std::cell::RefCell;
use std::collections::BTreeSet;

thread_local! {
    /// Logical ids we've already warned about (no device-sampleable cooked
    /// tier). Render-loop is single-threaded; this keeps the "missing cooked
    /// texture" warning to once-per-id instead of once-per-frame.
    static WARNED_MISSING: RefCell<BTreeSet<LogicalTextureId>> = const { RefCell::new(BTreeSet::new()) };
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
        // Descend the device fallback ladder, taking the first tier that has
        // a cooked artifact AND that this GPU can sample.
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
                // A corrupt blob or other hard error: surface + stop (descending
                // wouldn't help a structurally-broken artifact for this tier, but
                // a different tier might still resolve, so keep walking).
                Err(e) => {
                    eprintln!("W2.T4 cooked texture {logical_id} (tier {tier}): {e}");
                    continue;
                }
            }
        }
        if renderer.cooked_texture_id(logical_id).is_none() {
            WARNED_MISSING.with_borrow_mut(|warned| {
                if warned.insert(logical_id) {
                    eprintln!(
                        "W2.T4: no device-sampleable cooked KTX2 tier for logical texture \
                         {logical_id} — sprite renders invisible until one is loaded"
                    );
                }
            });
        }
    }
}

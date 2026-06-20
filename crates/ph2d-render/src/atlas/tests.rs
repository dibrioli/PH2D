//! GPU-backed behavior tests for [`TextureAtlas`] (packing, free-list,
//! remove, regrow, and the Phase 2 mip chain). Most require a headless
//! adapter and silently no-op when none is available — see
//! [`try_headless_gpu`]. Split out of `mod.rs` to keep the orchestrator
//! under the module size budget.

use super::*;

/// Cached per test binary — see `game_rt::tests::try_headless_gpu`
/// for rationale. Each adapter+device request costs ~30-50 s cold;
/// the atlas suite alone has ~20 GPU-touching tests.
fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| {
            let instance = GpuContext::default_instance();
            GpuContext::new(instance, None).ok()
        })
        .clone()
}

#[test]
fn insert_returns_region_with_requested_dimensions() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 1024);
    let rgba = vec![0u8; (128 * 64 * 4) as usize];
    let region = atlas.insert(&gpu, 0, 128, 64, &rgba).expect("insert ok");
    assert_eq!(region.w, 128);
    assert_eq!(region.h, 64);
    assert_eq!(atlas.region(0), Some(region));
}

#[test]
fn insert_packs_multiple_arbitrary_sizes() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 1024);
    let sizes = [(200, 100), (50, 50), (300, 80), (10, 200)];
    for (i, &(w, h)) in sizes.iter().enumerate() {
        let rgba = vec![0u8; (w * h * 4) as usize];
        atlas
            .insert(&gpu, i as u32, w, h, &rgba)
            .expect("insert ok");
    }
    assert_eq!(atlas.region_count(), sizes.len());
    // No two regions overlap — Skyline guarantee but pin it here
    // so a future packer swap stays honest.
    let collected: Vec<AtlasRegion> = (0..sizes.len() as u32)
        .map(|k| atlas.region(k).unwrap())
        .collect();
    for (i, a) in collected.iter().enumerate() {
        for b in &collected[i + 1..] {
            let overlap = a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
            assert!(!overlap, "regions overlap: {a:?} vs {b:?}");
        }
    }
}

#[test]
fn insert_rejects_source_too_large_for_atlas() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 256);
    let rgba = vec![0u8; (512 * 16 * 4) as usize];
    let err = atlas
        .insert(&gpu, 0, 512, 16, &rgba)
        .expect_err("expected SourceTooLarge");
    assert!(matches!(err, AtlasInsertError::SourceTooLarge { .. }));
}

#[test]
fn insert_returns_full_when_packer_exhausted() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 128);
    // First 128×128 fills the atlas; second fails.
    let rgba = vec![0u8; (128 * 128 * 4) as usize];
    atlas.insert(&gpu, 0, 128, 128, &rgba).unwrap();
    let err = atlas.insert(&gpu, 1, 64, 64, &vec![0u8; (64 * 64 * 4) as usize]);
    assert!(matches!(err, Err(AtlasInsertError::AtlasFull { .. })));
}

#[test]
fn replace_same_size_succeeds_without_repack() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 256);
    let rgba_a = vec![0xAAu8; (64 * 64 * 4) as usize];
    let region_a = atlas.insert(&gpu, 7, 64, 64, &rgba_a).unwrap();
    // Same key + same size → same region (the packer is NOT
    // asked again, important for the hot-reload path).
    let rgba_b = vec![0xBBu8; (64 * 64 * 4) as usize];
    let region_b = atlas.insert(&gpu, 7, 64, 64, &rgba_b).unwrap();
    assert_eq!(region_a, region_b);
    assert_eq!(atlas.region_count(), 1);
}

#[test]
fn dummy_atlas_seeds_16_demo_keys() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let atlas = TextureAtlas::dummy(&gpu);
    assert_eq!(atlas.region_count(), DEMO_TILE_COUNT as usize);
    for i in 0..DEMO_TILE_COUNT {
        let region = atlas.region(i).unwrap_or_else(|| panic!("missing key {i}"));
        assert_eq!(region.w, DEMO_TILE_PX);
        assert_eq!(region.h, DEMO_TILE_PX);
    }
}

// ── M14.4f Atlas V2: free-list + remove + regrow ────────────────

#[test]
fn remove_returns_region_and_pushes_into_free_list() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 256);
    let rgba = vec![0u8; (64 * 64 * 4) as usize];
    let region = atlas.insert(&gpu, 7, 64, 64, &rgba).unwrap();
    assert_eq!(atlas.free_slot_count(), 0);
    let removed = atlas.remove(7).expect("region present");
    assert_eq!(removed, region);
    assert_eq!(atlas.region_count(), 0);
    assert_eq!(atlas.free_slot_count(), 1);
}

#[test]
fn remove_of_missing_key_returns_none() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 256);
    assert!(atlas.remove(123).is_none());
    assert_eq!(atlas.free_slot_count(), 0);
}

#[test]
fn insert_reuses_freed_slot_for_matching_size() {
    // After remove, an insert of the SAME dimensions must reuse
    // the freed slot — no new packer.pack() call. We verify by
    // confirming the new region equals the old region.
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 256);
    let rgba = vec![0u8; (64 * 64 * 4) as usize];
    let region_a = atlas.insert(&gpu, 1, 64, 64, &rgba).unwrap();
    atlas.remove(1).unwrap();
    let region_b = atlas.insert(&gpu, 2, 64, 64, &rgba).unwrap();
    assert_eq!(region_a, region_b, "free-list slot reused");
    assert_eq!(atlas.free_slot_count(), 0);
}

#[test]
fn insert_skips_free_list_when_size_mismatches() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 256);
    let rgba_64 = vec![0u8; (64 * 64 * 4) as usize];
    let region_64 = atlas.insert(&gpu, 1, 64, 64, &rgba_64).unwrap();
    atlas.remove(1).unwrap();
    // Insert a 32×32 — the free-list has a 64×64 entry, which is
    // ignored (no fit, no over-allocation: the slot stays in the
    // free-list waiting for another 64×64 caller).
    let rgba_32 = vec![0u8; (32 * 32 * 4) as usize];
    let region_32 = atlas.insert(&gpu, 2, 32, 32, &rgba_32).unwrap();
    assert_ne!(region_32, region_64);
    assert_eq!(atlas.free_slot_count(), 1, "64×64 slot still free");
}

#[test]
fn replace_with_different_size_releases_old_slot() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 256);
    let rgba_a = vec![0u8; (64 * 64 * 4) as usize];
    atlas.insert(&gpu, 7, 64, 64, &rgba_a).unwrap();
    // Same key, DIFFERENT size — the old 64×64 region must end
    // up in the free-list (M14.4f bug fix: previously this
    // returned AtlasFull and leaked the slot).
    let rgba_b = vec![0u8; (32 * 32 * 4) as usize];
    atlas.insert(&gpu, 7, 32, 32, &rgba_b).unwrap();
    assert_eq!(atlas.free_slot_count(), 1);
}

#[test]
fn regrow_inplace_doubles_size_and_preserves_regions() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 256);
    let rgba = vec![0u8; (64 * 64 * 4) as usize];
    atlas.insert(&gpu, 0, 64, 64, &rgba).unwrap();
    atlas.insert(&gpu, 1, 64, 64, &rgba).unwrap();
    // Fake fetch_pixels — just return zero buffers of the right
    // size. Real callers route through AssetDb.
    let new = atlas
        .regrow_inplace(&gpu, 512, |_key| Some(vec![0u8; 64 * 64 * 4]))
        .unwrap();
    assert_eq!(new, 512);
    assert_eq!(atlas.size_px, 512);
    assert_eq!(atlas.region_count(), 2);
    assert!(atlas.region(0).is_some());
    assert!(atlas.region(1).is_some());
    // Free-list cleared on regrow — old freed slots can't be
    // mapped onto the new skyline without overlap risk.
    assert_eq!(atlas.free_slot_count(), 0);
}

#[test]
fn regrow_inplace_caps_at_max_size_px() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 256);
    let cap = atlas.max_size_px();
    let actual = atlas
        .regrow_inplace(&gpu, cap * 4, |_key| Some(Vec::new()))
        .unwrap();
    assert_eq!(actual, cap, "regrow capped at max_size_px");
}

#[test]
fn regrow_inplace_drops_regions_whose_fetcher_returns_none() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 256);
    let rgba = vec![0u8; (32 * 32 * 4) as usize];
    atlas.insert(&gpu, 1, 32, 32, &rgba).unwrap();
    atlas.insert(&gpu, 2, 32, 32, &rgba).unwrap();
    // fetcher returns pixels only for key 1; key 2 should drop.
    atlas
        .regrow_inplace(&gpu, 512, |key| {
            if key == 1 {
                Some(vec![0u8; 32 * 32 * 4])
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(atlas.region_count(), 1);
    assert!(atlas.region(1).is_some());
    assert!(atlas.region(2).is_none());
}

// ── Phase 2: atlas mip chain ────────────────────────────────────

#[test]
fn atlas_texture_has_full_mip_chain() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    // 256² → floor(log2 256)+1 = 9 mip levels (256,128,…,1).
    assert_eq!(atlas.texture.mip_level_count(), 9);
}

#[test]
fn atlas_mip_chain_downsamples_region_in_linear_light() {
    // Phase 2 mirror of the individual store's linear-light mip test.
    // The first insert into a fresh atlas packs at (0,0), so a 8×8
    // half-white/half-black region's mip-3 footprint collapses to the
    // single texel (0,0) of the 32² level — the LINEAR average of
    // white+black ≈ 188 (0.5 linear → sRGB), NOT the naïve sRGB-byte
    // midpoint 128. That gap is the whole point: averaging sRGB bytes
    // directly is the classic too-dark downsample bug.
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 256);
    let (w, h) = (8u32, 8u32);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let idx = ((y * w + x) * 4) as usize;
            let v = if x < w / 2 { 255 } else { 0 }; // left white, right black
            rgba[idx] = v;
            rgba[idx + 1] = v;
            rgba[idx + 2] = v;
            rgba[idx + 3] = 255; // opaque
        }
    }
    let region = atlas.insert(&gpu, 0, w, h, &rgba).expect("insert");
    assert_eq!(
        (region.x, region.y),
        (0, 0),
        "first insert must pack at the origin for this test's mip math"
    );

    let (mw, mh, level3) = atlas.readback_mip(&gpu, 3);
    assert_eq!((mw, mh), (32, 32), "256² → mip 3 is 32²");
    let r = level3[0]; // texel (0,0) = average of the [0,8)² region
    assert!(
        (180..=196).contains(&r),
        "atlas mip-3 texel(0,0) must be the LINEAR avg of white+black ≈ 188, got {r} \
             (128 would mean a wrong sRGB-space average)"
    );
    assert_eq!(level3[3], 255, "opaque region stays opaque in its mip");
}

#[test]
fn atlas_untouched_space_is_transparent_in_mips() {
    // The transparent-clear guard: a region inserted far from the
    // origin leaves the origin texel untouched, and it must read back
    // as transparent (alpha 0) at every mip level — proof the clear
    // ran and garbage isn't bleeding into the chain.
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 256);
    // A single small opaque region; with the Skyline packer it lands
    // at (0,0), so probe a DEEP mip whose origin texel still aggregates
    // only transparent cleared space beyond the tiny region.
    let rgba = vec![255u8; (4 * 4 * 4) as usize];
    atlas.insert(&gpu, 0, 4, 4, &rgba).expect("insert");
    // mip 6 of 256² is 4²; its texel (1,1) covers original [64,128)²,
    // entirely outside the 4×4 region → must be cleared transparent.
    let (mw, mh, level6) = atlas.readback_mip(&gpu, 6);
    assert_eq!((mw, mh), (4, 4));
    let idx = ((mw + 1) * 4) as usize; // texel (1,1), alpha channel
    assert_eq!(
        level6[idx + 3],
        0,
        "untouched packing space must downsample to transparent, not garbage"
    );
}

#[test]
fn region_uv_recomputes_against_new_size_after_regrow() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 256);
    let rgba = vec![0u8; (128 * 128 * 4) as usize];
    atlas.insert(&gpu, 0, 128, 128, &rgba).unwrap();
    let uv_before = atlas.region_uv(0);
    atlas
        .regrow_inplace(&gpu, 512, |_key| Some(vec![0u8; 128 * 128 * 4]))
        .unwrap();
    let uv_after = atlas.region_uv(0);
    // Same region placement (Skyline picks (0,0) deterministically
    // for the first insert) so x/y stay zero; but the UV math
    // divides by the new `size_px = 512`, halving the
    // normalized width compared to the old `size_px = 256`.
    assert!(uv_before[2] > uv_after[2], "UV must shrink post-regrow");
    // Half-texel inset → (128 - 0.5) / 512 ≈ 0.2490.
    let expected = (128.0_f32 - 0.5) / 512.0;
    assert!(
        (uv_after[2] - expected).abs() < 1e-4,
        "uv1 = {}",
        uv_after[2]
    );
}

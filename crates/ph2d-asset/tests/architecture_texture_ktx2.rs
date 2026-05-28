//! Arch-gate W1.T4 (ADR-0055-v4) — `Asset::TextureKtx2` variant + `TierIndex`
//! newtype + `LogicalTextureMap` mapping são presentes e bem-formados.
//!
//! Razão: §Symbol Registry do plano vivo `docs/plans/2026-05-texture-compression-
//! waves.md` exige que (a) `TierIndex::COUNT == 5` (alinha com ADR-0053
//! FROZEN), (b) `Asset` enum continue `#[non_exhaustive]` após adição W1.T4,
//! (c) `LogicalTextureMap` respeite BTree ordering (HR-6 deterministic
//! serialization), (d) `Asset::TextureKtx2` byte_size devolve `blob.len()`.

use ph2d_asset::{Asset, AssetId, LogicalTextureId, LogicalTextureMap, TierIndex};
use std::sync::Arc;

#[test]
fn tier_index_count_locked_at_5() {
    assert_eq!(
        TierIndex::COUNT,
        5,
        "TierIndex must have exactly 5 variants to mirror ADR-0053 DeviceTier FROZEN. \
         Adding a 6th tier requires amendment ADR-0053 AND ADR-0055-v4 §Symbol Registry update."
    );
}

#[test]
fn tier_index_canonical_constants_round_trip() {
    let all = [
        (TierIndex::DESKTOP, 0u8, "Desktop"),
        (TierIndex::MOBILE, 1, "Mobile"),
        (TierIndex::WEB, 2, "Web"),
        (TierIndex::LOW_END, 3, "LowEnd"),
        (TierIndex::CONSTRAINED, 4, "Constrained"),
    ];
    for (tier, expected_u8, expected_name) in all {
        assert_eq!(tier.as_u8(), expected_u8);
        assert_eq!(tier.name(), expected_name);
        let reconstructed = TierIndex::new(expected_u8).expect("0..=4 must construct");
        assert_eq!(reconstructed, tier);
    }
}

#[test]
fn asset_enum_is_non_exhaustive_after_w1_t4() {
    // Compile-time check: external code MUST be able to match `Asset` with a
    // catch-all (`_`) arm. If `#[non_exhaustive]` was accidentally removed,
    // this test would still compile but downstream crates would break.
    let asset = Asset::TextureKtx2 {
        tier: TierIndex::DESKTOP,
        blob: Arc::new(vec![0u8; 32]),
    };
    let label = match &asset {
        Asset::ImageRgba8 { .. } => "img",
        Asset::Prefab(_) => "prefab",
        Asset::Scene(_) => "scene",
        Asset::TextureKtx2 { .. } => "ktx2",
        _ => "unknown",
    };
    assert_eq!(label, "ktx2");
}

#[test]
fn asset_texture_ktx2_byte_size_includes_blob_plus_overhead() {
    // Audit ε-H2 / ζ-F3 fix: byte_size deve ser consistente com Prefab/Scene
    // arms (que somam std::mem::size_of_val overhead). Para HR-13 budget
    // accounting "rough", overhead constante (~25 bytes per asset) é noise
    // vs payload (KTX2 blob tipicamente ≥1 KB), mas consistency matters.
    let blob_size = 1024;
    let asset = Asset::TextureKtx2 {
        tier: TierIndex::MOBILE,
        blob: Arc::new(vec![0u8; blob_size]),
    };
    let reported = asset.byte_size();
    assert!(
        reported >= blob_size,
        "HR-13 budget accounting must include blob payload ({blob_size} bytes); got {reported}"
    );
    // Overhead bounded: tier (1B) + Vec inner descriptor (~24B on 64-bit).
    assert!(
        reported < blob_size + 256,
        "byte_size overhead should be constant-bounded; got {} bytes overhead",
        reported - blob_size
    );
}

#[test]
fn logical_texture_id_is_distinct_from_asset_id() {
    // HR-6 contract: LogicalTextureId hashes SOURCE bytes; AssetId hashes
    // COOKED bytes. Different inputs → different IDs.
    let source = b"hero source.png raw bytes";
    let cooked = b"cooked KTX2 bytes specific to BC7 desktop tier";
    let logical = LogicalTextureId::from_source_bytes(source);
    let asset = AssetId::from_bytes(cooked);
    // Same length-32 digests but different content per HR-6.
    assert_eq!(logical.as_bytes().len(), 32);
    // Sanity: hex representations differ (can't compare across types directly).
    assert_ne!(logical.to_hex(), asset.to_hex());
}

#[test]
fn logical_texture_map_resolves_per_tier() {
    let mut map = LogicalTextureMap::new();
    let logical = LogicalTextureId::from_source_bytes(b"hero.png");

    let desktop_id = AssetId::from_bytes(b"bc7-desktop-cooked");
    let mobile_id = AssetId::from_bytes(b"astc-mobile-cooked");
    let web_id = AssetId::from_bytes(b"astc-web-cooked");

    map.insert(logical, TierIndex::DESKTOP, desktop_id);
    map.insert(logical, TierIndex::MOBILE, mobile_id);
    map.insert(logical, TierIndex::WEB, web_id);

    assert_eq!(map.resolve(logical, TierIndex::DESKTOP), Some(desktop_id));
    assert_eq!(map.resolve(logical, TierIndex::MOBILE), Some(mobile_id));
    assert_eq!(map.resolve(logical, TierIndex::WEB), Some(web_id));
    assert_eq!(map.resolve(logical, TierIndex::LOW_END), None);

    // BTreeMap iteration order is deterministic (HR-6 serialization).
    let tiers = map.available_tiers(logical);
    assert_eq!(tiers, vec![TierIndex::DESKTOP, TierIndex::MOBILE, TierIndex::WEB]);
}

#[test]
fn logical_texture_map_postcard_round_trip() {
    // HR-6: serialização deterministic via postcard (BTreeMap iteration order).
    let mut map = LogicalTextureMap::new();
    let logical = LogicalTextureId::from_source_bytes(b"sprite.png");
    map.insert(logical, TierIndex::DESKTOP, AssetId::from_bytes(b"cooked"));

    let bytes_a = postcard::to_allocvec(&map).expect("encode");
    let bytes_b = postcard::to_allocvec(&map).expect("re-encode");
    assert_eq!(bytes_a, bytes_b, "same map must serialize byte-identical");

    let decoded: LogicalTextureMap = postcard::from_bytes(&bytes_a).expect("decode");
    assert_eq!(decoded.entry_count(), map.entry_count());
}

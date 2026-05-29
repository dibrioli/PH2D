//! Determinism tests for the cooker (HR-5 + HR-6).
//!
//! Cooking the same JSON5 source twice must produce byte-identical
//! postcard output. The cooker is a pure function of (source bytes,
//! ph2d crate versions); any introduction of nondeterminism
//! (HashMap iteration, timestamps, random ids) would break HR-6
//! "asset = blake3 of content".
//!
//! The cross-platform Linux + Mac matrix lives in
//! `.github/workflows/spike.yml` — this test is the local-machine
//! sentinel.

use ph2d_asset_cooker::{cook_prefab_json5, cook_scene_json5};

const SIMPLE_PREFAB_SOURCE: &str =
    include_str!("../../../tests/fixtures/prefab/simple_sprite.json5");

const TWO_SPRITES_SCENE_SOURCE: &str =
    include_str!("../../../tests/fixtures/scene/two_sprites.json5");

#[test]
fn prefab_cook_is_deterministic() {
    let a = cook_prefab_json5(SIMPLE_PREFAB_SOURCE).unwrap();
    let b = cook_prefab_json5(SIMPLE_PREFAB_SOURCE).unwrap();
    assert_eq!(a, b, "cooker produced different bytes on repeat run");
    // Sanity: non-empty.
    assert!(!a.is_empty());
}

#[test]
fn scene_cook_is_deterministic() {
    let a = cook_scene_json5(TWO_SPRITES_SCENE_SOURCE).unwrap();
    let b = cook_scene_json5(TWO_SPRITES_SCENE_SOURCE).unwrap();
    assert_eq!(a, b, "scene cooker produced different bytes on repeat run");
    assert!(!a.is_empty());
}

#[test]
fn cooked_bytes_decode_back_to_doc() {
    use ph2d_asset::PrefabDoc;
    let bytes = cook_prefab_json5(SIMPLE_PREFAB_SOURCE).unwrap();
    let doc: PrefabDoc = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(doc.version, 1);
    assert_eq!(doc.components.len(), 3);
}

#[test]
fn scene_cooked_bytes_decode_back_to_doc() {
    use ph2d_asset::SceneDoc;
    let bytes = cook_scene_json5(TWO_SPRITES_SCENE_SOURCE).unwrap();
    let doc: SceneDoc = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(doc.version, 1);
    assert_eq!(doc.instances.len(), 2);
    assert_eq!(doc.relations.len(), 1);
    assert_eq!(doc.relations[0].parent_index, 0);
    assert_eq!(doc.relations[0].child_index, 1);
}

/// Hash the cooker output via blake3 (HR-6 identity check). If this
/// hash ever differs across machines/runs it's a determinism
/// regression — the cross-platform CI matrix in
/// `.github/workflows/spike.yml` enforces equality between Linux
/// and Mac runners.
#[test]
fn prefab_cook_hash_is_locked() {
    let bytes = cook_prefab_json5(SIMPLE_PREFAB_SOURCE).unwrap();
    let id = ph2d_asset::AssetId::from_bytes(&bytes);
    let hex = id.to_hex();
    // The locked hash is published in `tests/fixtures/scene/two_sprites.json5`
    // as a referenced prefab id. Keep them in sync.
    assert_eq!(hex.len(), 64);
    assert_eq!(
        hex, "6feb338498d6d0b6dbe4a018187c4dfb3725db624bea71301231d380dcc8afab",
        "simple_sprite.json5 cook hash changed — update scene fixtures + this assertion"
    );
    // Re-locked for the Sprite Inspector v2 v3→v4 schema bump (ADR-0070,
    // spec §10.9 "cooker salva sempre v4"): the cooked Sprite component
    // grew 5→20 serde fields (serde-json fills the new `#[serde(default)]`
    // fields from the v3-style fixture), so the postcard bytes — and thus
    // the blake3 content id — legitimately changed. Deterministic +
    // cross-platform stable (all new fields are POD f32/u32/bool, no
    // transcendentals). Prior v3 hash: 905a9b77…46b26.
}

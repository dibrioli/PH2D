//! Wave 2 PR 11.4 contract test. Validates two invariants between
//! the runtime tool [`Registry`] and the hand-allocated chrome NodeId
//! consts in `ph2d_editor::screens::hero::ids`:
//!
//! 1. **Slug parity.** Each chrome const that names a registry-derived
//!    pill must hash the SAME string slug as the matching manifest's
//!    `id` field. The Image Tools action row uses three: `TRIM` →
//!    `"trim_transparency"`, `MAKE_SQUARE` → `"make_square"`,
//!    `BGREMOVAL` → `"bgremoval"`. If a tool crate renames its `id`
//!    without updating the chrome const (or vice-versa), dispatch
//!    silently misroutes — this test catches that.
//!
//! 2. **Cluster contract.** All three image-action manifests must live
//!    in the `image_tools` cluster so a single `Registry::cluster`
//!    call returns the full set in canonical paint order.
//!
//! Both invariants are load-bearing for `screens/hero/topbar.rs::
//! image_action_pills`, which derives the painted pill list from the
//! installed registry and falls back to chrome consts in tests that
//! don't install one — they MUST be the same set.

use ph2d_tool_registry::{Registry, hash_node_id};

/// Build a fresh Registry with every manifest the desktop shell wires.
/// This is `register_all` re-implemented inline because `ph2d-editor`
/// can't depend on `ph2d-tool-registry-init` (the crate that owns
/// `register_all`) without creating the dependency cycle that PR 6.0
/// retrofit eliminated.
fn build_full_registry() -> Registry {
    let mut reg = Registry::default();
    ph2d_tool_bgremoval::register(&mut reg);
    ph2d_tool_grid_snap::register(&mut reg);
    ph2d_tool_make_square::register(&mut reg);
    ph2d_tool_padding::register(&mut reg);
    ph2d_tool_real_size::register(&mut reg);
    ph2d_tool_trim_transparency::register(&mut reg);
    reg.build().expect("registry must build");
    reg
}

/// Every registry-derived chrome pill: chrome const NodeId must equal
/// `hash_node_id(manifest.id)` so the painter (which reads the
/// registry) and the click dispatcher (which compares against
/// `ids::IMAGE_ACTION_*`) agree on the NodeId.
#[test]
fn chrome_const_hashes_match_manifest_id() {
    use ph2d_editor::screens::hero::ids;

    let cases: &[(&str, ph2d_a11y::NodeId)] = &[
        ("trim_transparency", ids::IMAGE_ACTION_TRIM),
        ("make_square", ids::IMAGE_ACTION_MAKE_SQUARE),
        ("bgremoval", ids::IMAGE_ACTION_BGREMOVAL),
        ("real_size", ids::IMAGE_ACTION_REAL_SIZE),
        ("padding", ids::IMAGE_ACTION_PADDING),
    ];
    for (manifest_id, chrome_const) in cases {
        let derived = hash_node_id(manifest_id);
        assert_eq!(
            derived.0, chrome_const.0,
            "chrome NodeId for {manifest_id:?} ({:#018x}) must equal hash_node_id({manifest_id:?}) ({:#018x})",
            chrome_const.0, derived.0,
        );
    }
}

/// All three Image Tools action-row pills live in the `image_tools`
/// cluster and arrive sorted by manifest `order` field.
#[test]
fn image_tools_cluster_contains_all_three_action_pills_in_paint_order() {
    let reg = build_full_registry();
    let cluster = reg.cluster("image_tools");
    let ids: Vec<&str> = cluster.iter().map(|m| m.id).collect();
    assert_eq!(
        ids,
        vec![
            "trim_transparency",
            "make_square",
            "bgremoval",
            "real_size",
            "padding"
        ],
        "expected trim → make_square → bgremoval → real_size → padding \
         (orders 40/50/60/70/80); got {ids:?}"
    );
}

/// Sanity: the legacy `IMAGE_ACTION_*` consts ALL resolve through the
/// registry — i.e., every chrome pill the topbar paints has a backing
/// manifest. Catches the future "added a pill, forgot the manifest"
/// regression.
#[test]
fn every_chrome_image_action_const_has_a_manifest_entry() {
    use ph2d_editor::screens::hero::ids;
    let reg = build_full_registry();
    let cluster = reg.cluster("image_tools");
    let pill_node_ids: std::collections::BTreeSet<u64> =
        cluster.iter().map(|m| hash_node_id(m.id).0).collect();
    for (label, const_id) in [
        ("IMAGE_ACTION_TRIM", ids::IMAGE_ACTION_TRIM),
        ("IMAGE_ACTION_MAKE_SQUARE", ids::IMAGE_ACTION_MAKE_SQUARE),
        ("IMAGE_ACTION_BGREMOVAL", ids::IMAGE_ACTION_BGREMOVAL),
        ("IMAGE_ACTION_REAL_SIZE", ids::IMAGE_ACTION_REAL_SIZE),
        ("IMAGE_ACTION_PADDING", ids::IMAGE_ACTION_PADDING),
    ] {
        assert!(
            pill_node_ids.contains(&const_id.0),
            "chrome const {label} ({:#018x}) has no matching manifest \
             in the `image_tools` cluster",
            const_id.0,
        );
    }
}

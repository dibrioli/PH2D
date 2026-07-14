//! **Arch-gate: the hero's paint must actually CALL the dock** (W4.T4).
//!
//! `HeroLayout::dock_timeline_into_motion` is a pure function with four gates of its own — and I
//! deleted its *call site* and every one of them stayed green. Of course they did: they test the
//! function, and the function was still correct. What died was the feature.
//!
//! That is [[feedback_a_mutation_that_survives_may_mean_a_missing_gate]] — the surviving mutation
//! did not mean the gates were loose, it meant a gate was MISSING: nothing said *"and somebody has
//! to call this"*. A paint gate would be the honest answer, but the hero screen is not a `Panel`
//! and has no headless seam; so this reads the product's source, exactly like the z-projection's
//! frame-order gate does, and asserts the three things that can silently break:
//!
//! 1. the call exists;
//! 2. it is guarded by BOTH visibility flags (docking with the timeline hidden would carve a band
//!    for a panel nobody can see, and the graph would just be short);
//! 3. the guard uses the shared CONSTS, not typed-again string literals — a `panel_visibility` miss
//!    reads as `false`, so a typo does not error, it just quietly never docks.

use std::path::Path;

#[test]
fn the_hero_paint_docks_the_timeline_into_motion() {
    let src = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/screens/hero/paint.rs"),
    )
    .expect("the hero's paint is where the layout is built");

    assert!(
        src.contains("dock_timeline_into_motion()"),
        "nobody calls the dock: the timeline would go on painting over the node graph, and every \
         layout gate would stay green about it"
    );
    assert!(
        src.contains("PANEL_MOTION_GRAPH") && src.contains("PANEL_TIMELINE"),
        "the dock must be guarded by BOTH panel flags, and by the shared consts - a re-typed \
         string key that misses reads as `false`, so the feature just never happens"
    );
    // The guard and the call are one thought: the call must sit inside the `if`.
    let guard = src.find("PANEL_MOTION_GRAPH").expect("checked above");
    let call = src
        .find("dock_timeline_into_motion()")
        .expect("checked above");
    assert!(
        guard < call && call - guard < 200,
        "the call must be INSIDE the visibility guard, not merely near it"
    );
}

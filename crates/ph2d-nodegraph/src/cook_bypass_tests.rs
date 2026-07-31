//! Bypass/mute (H) cook tests — split from `cook_tests.rs` for the HR-18 LOC cap. Declared there
//! as a `#[path]` sibling, so `super` is the test harness (the node ops + `ops()`), exactly like
//! `cook_scope_tests.rs`.

use super::*;

/// **A bypassed node passes its primary input through instead of running its op** — the H switch.
/// Normally `Scale` doubles; muted, its op never runs and `Gen`'s stream flows straight out.
/// Toggling back RECOMPUTES (the third cook), which proves `bypassed` rides the fingerprint:
/// without that field the second cook would hand back the cached `[2,4,6]` and the mute would look
/// inert. FALSIFIED two ways, both bleeding here — drop the passthrough branch in `cook_node` (the
/// op runs, `[2,4,6]`), or drop `bypassed` from the `Fingerprint` (the cache returns the stale
/// `[2,4,6]`).
#[test]
fn a_bypassed_node_passes_its_input_through_instead_of_running_its_op() {
    let mut g = Graph::new();
    let generator = g.add_node("test.gen");
    let scale = g.add_node("test.scale");
    g.connect(Edge {
        from: (generator, 0),
        to: (scale, 0),
        delayed: false,
    })
    .unwrap();
    let o = ops();
    let mut cook = Cook::new();

    // Running normally, Scale doubles its input.
    assert_eq!(
        out_scalars(&cook.cook(&g, &o, scale, 0.0).unwrap()[0]),
        vec![2.0, 4.0, 6.0]
    );

    // Switched OFF: the op never runs, so Gen's stream passes straight through.
    g.set_bypassed(scale, true);
    assert_eq!(
        out_scalars(&cook.cook(&g, &o, scale, 0.0).unwrap()[0]),
        vec![1.0, 2.0, 3.0],
        "a bypassed node passes input[0] through, not the op's doubled result"
    );

    // Un-muting recomputes: the passthrough was cache-invalidated by the bypass flag, not stuck.
    g.set_bypassed(scale, false);
    assert_eq!(
        out_scalars(&cook.cook(&g, &o, scale, 0.0).unwrap()[0]),
        vec![2.0, 4.0, 6.0],
        "un-muting must re-run the op, not return the cached passthrough"
    );
}

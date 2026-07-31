//! Tests for the view snapshot — split from `snapshot.rs` for the panel LOC cap. Declared there
//! as a `#[path]` sibling, so `super` is the `snapshot` module (its items + its imports, e.g.
//! `Graph`, `NodeRegistry`, `snapshot_from`).

use super::*;
use ph2d_nodegraph::port::Dim;

/// A single scalar is a value ○; every multi-component dim is a column ◇. The match enumerates ALL
/// `Dim` (no `_ =>`), so a new axis is a compile error here — not a socket silently drawn as the
/// wrong shape, which is exactly how the shape was lost (`draw_card` drew `fill_circle` for
/// everything while the doc claimed "shape ← Dim"). Mutation — collapsing every dim to `Value`, or
/// flipping `Scalar` to `Column` — sangra on the loop or the first assert.
#[test]
fn scalar_is_a_value_dot_and_multi_component_is_a_column_diamond() {
    assert_eq!(socket_glyph(Dim::Scalar), SocketGlyph::Value);
    for d in [Dim::Vec2, Dim::Vec3, Dim::Vec4, Dim::Mat2, Dim::Mat3, Dim::Mat4] {
        assert_eq!(socket_glyph(d), SocketGlyph::Column, "{d:?} is a column");
    }
}

/// **A bypassed node's mute reaches the VIEW** (bypass/mute — H). `snapshot_from` reads the flag
/// straight off the graph, so a muted node's card can draw dimmed with a strike; the unmuted
/// neighbour's does not. FALSIFIED by hard-coding `bypassed: false` in the build loop — the mute
/// would live in the engine and never reach the screen, the exact class of "the feature works but
/// you cannot see it" the ghost fix closed on the wire side.
#[test]
fn snapshot_from_reflects_bypass_into_the_view() {
    let mut g = Graph::new();
    let a = g.add_node("test.a");
    let b = g.add_node("test.b");
    g.set_bypassed(b, true);

    let reg = NodeRegistry::new();
    let snap = snapshot_from(&g, &reg);
    let view = |id: u32| snap.nodes.iter().find(|n| n.id == id).expect("node in view");
    assert!(!view(a.0).bypassed, "an unmuted node's view is not bypassed");
    assert!(view(b.0).bypassed, "a muted node's view IS bypassed");
}

/// **The mute strike spans the whole card**, corner to corner — the "off" gesture only reads if it
/// crosses the card, not as a dot or a stub. FALSIFIED by a strike that collapses to a point or
/// spans only part of the body.
#[test]
fn the_bypass_strike_spans_the_card_corners() {
    let body = ph2d_editor_core::zones::Rect::new(10.0, 20.0, 190.0, 60.0);
    let [start, end] = crate::paint::bypass_strike(body);
    assert_eq!(start, (body.x, body.y), "starts at the top-left corner");
    assert_eq!(end, (body.x + body.w, body.y + body.h), "ends at the bottom-right corner");
    // It genuinely crosses the card, both axes.
    assert!((end.0 - start.0).abs() >= body.w && (end.1 - start.1).abs() >= body.h);
}

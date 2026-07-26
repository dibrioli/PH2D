//! Seam gates for **Color Harmonies** in the shared BlenderColorPicker.
//!
//! The engine math is proven in `widget::blender_color_picker::harmony::tests`.
//! These drive the REAL pointer dispatch through the four UI conditions — the
//! hit slot EXISTS (registered), is HIT-mapped, the click reaches the bus, and
//! the SEQUENCE lands somewhere (scheme selected / partner adopted / palette
//! grown). A partner the artist sees is picked through the SAME `harmony_partners`
//! door the painter draws with, so what is shown is what the click grabs.

use super::*;
use crate::interaction::BlenderHitKind;
use crate::widget::{Harmony, harmony_partners};

/// The shared blender setup + the Color Harmonies hit slots on top: the 7 scheme
/// segments (NodeId 240+i), the 4 partner swatches (250+i), and the "add all"
/// button (260). Mirrors `pre_populate` + `paint_harmony_section` in the product.
fn harmony_setup() -> (WidgetStore, HitIndex) {
    let (mut store, mut hits) = blender_picker_setup();
    for i in 0u8..7 {
        store.register(
            NodeId(240 + i as u64),
            InteractiveState::BlenderHit {
                parent: NodeId(100),
                kind: BlenderHitKind::HarmonyScheme(i),
            },
        );
        hits.register(
            NodeId(240 + i as u64),
            Rect::new(i as f32 * 20.0, 400.0, 18.0, 22.0),
        );
    }
    for i in 0u8..4 {
        store.register(
            NodeId(250 + i as u64),
            InteractiveState::BlenderHit {
                parent: NodeId(100),
                kind: BlenderHitKind::HarmonySwatch(i),
            },
        );
        hits.register(
            NodeId(250 + i as u64),
            Rect::new(i as f32 * 30.0, 430.0, 24.0, 22.0),
        );
    }
    store.register(
        NodeId(260),
        InteractiveState::BlenderHit {
            parent: NodeId(100),
            kind: BlenderHitKind::HarmonyAdd,
        },
    );
    hits.register(NodeId(260), Rect::new(240.0, 430.0, 22.0, 22.0));
    (store, hits)
}

/// Did the pointer-down at (x, y) emit `ValueChanged(100)`? Drives the REAL
/// dispatch; the arena-borrowed slice is consumed here so it never escapes.
fn click_changes_picker(store: &mut WidgetStore, hits: &HitIndex, x: f32, y: f32) -> bool {
    let arena = Bump::new();
    dispatch_pointer(store, hits, pointer(PointerKind::Down, x, y), &arena)
        .iter()
        .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(100)))
}

#[test]
fn clicking_a_scheme_segment_selects_it() {
    let (mut store, hits) = harmony_setup();
    // The picker starts on None; click the Complementary segment (index 1).
    assert_eq!(store.blender_harmony(NodeId(100)), Harmony::None);
    assert!(
        click_changes_picker(&mut store, &hits, 20.0 + 9.0, 411.0),
        "expected ValueChanged(100) from harmony scheme hit"
    );
    assert_eq!(store.blender_harmony(NodeId(100)), Harmony::Complementary);
}

#[test]
fn clicking_a_partner_swatch_adopts_that_partner() {
    let (mut store, hits) = harmony_setup();
    // Select Triad first (index 3), so there are 3 partners past the base.
    assert!(click_changes_picker(&mut store, &hits, 60.0 + 9.0, 411.0));
    assert_eq!(store.blender_harmony(NodeId(100)), Harmony::Triad);

    // What the SECOND partner (index 1: base is 0) should be, via the same door.
    let (base, _, _, _) = store.blender_picker(NodeId(100)).unwrap();
    let expected = harmony_partners(base, Harmony::Triad)[1];

    // Click partner swatch index 1 (NodeId 251).
    assert!(
        click_changes_picker(&mut store, &hits, 30.0 + 12.0, 441.0),
        "expected ValueChanged(100) from partner swatch hit"
    );
    let (got, _, _, _) = store.blender_picker(NodeId(100)).unwrap();
    assert_eq!(
        got.rgba, expected.rgba,
        "the partner the click grabbed must equal the one the section derives"
    );
}

#[test]
fn add_all_grows_the_palette_by_the_partner_count() {
    let (mut store, hits) = harmony_setup();
    // Select Triad (index 3): base + 3 partners = 4 colors.
    assert!(click_changes_picker(&mut store, &hits, 60.0 + 9.0, 411.0));
    let before = store.blender_palette(NodeId(100)).unwrap().len();
    let (base, _, _, _) = store.blender_picker(NodeId(100)).unwrap();
    let n = harmony_partners(base, Harmony::Triad).len();

    assert!(
        click_changes_picker(&mut store, &hits, 240.0 + 11.0, 441.0),
        "expected ValueChanged(100) from the add-all button"
    );
    let after = store.blender_palette(NodeId(100)).unwrap().len();
    assert_eq!(
        after,
        before + n,
        "add-all should append every derived color"
    );
}

//! Layout tests for the Audio Editor's collapsible sections.
//!
//! A child module so `paint_sections.rs` stays under the panel 600-LOC cap.

use super::*;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::widget::TextInputState;

fn body(open: [bool; 7]) -> Body {
    Body {
        open,
        loaded: true,
        undo_ok: true,
        redo_ok: true,
        has_sel: true,
        transport: Transport {
            loaded: true,
            playing: false,
            looping: false,
            pos: 0.0,
            dur: 6.0,
        },
        name: NameBox {
            state: TextInputState::Normal,
            text: "clip.wav".to_string(),
            caret: 0,
            anchor: None,
        },
    }
}

/// How tall the body comes out with that fold state.
fn height(open: [bool; 7]) -> f32 {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let mut hits = HitIndex::default();
    let clip = Rect::new(0.0, 0.0, 220.0, 40_000.0);
    let mut ch = ClippedHits::new(&mut hits, clip);
    paint_body(
        0.0,
        0.0,
        220.0,
        &body(open),
        &mut scene,
        &mut text,
        Theme::default(),
        &mut ch,
    )
}

/// **Folding a section has to actually fold it.** The chevron is the whole point of
/// the chrome: if the block below it still painted, the panel would be exactly the
/// wall of controls the sections exist to break up — and the user would be clicking
/// a decoration.
///
/// Every section folds on its own, and each one it folds makes the body shorter.
#[test]
fn folding_a_section_shortens_the_panel() {
    let all_open = height([true; 7]);
    let all_shut = height([false; 7]);
    assert!(
        all_shut < all_open * 0.5,
        "folding everything barely helped: {all_open} open, {all_shut} shut"
    );
    for i in 0..7 {
        let mut open = [true; 7];
        open[i] = false;
        assert!(
            height(open) < all_open,
            "section {i} ({:?}) painted its block while folded",
            SECTIONS[i]
        );
    }
}

/// ...and a folded section still shows its header, so the panel never loses the way
/// back in.
#[test]
fn a_folded_section_still_paints_its_header() {
    // Seven headers + seven dividers, even with every block folded away.
    assert!(
        height([false; 7]) > section_h() * 7.0,
        "the headers vanished along with their blocks"
    );
}

/// The order the user reads down the panel, pinned.
///
/// **Loop sits with the transport, not with the asset-prep half** — it is a *playback*
/// concept (Loop-on + Play is how you audition it), and Enio moved it there on sight
/// (2026-07-12). Pinned because the fold state is looked up by ID, so a reorder is
/// *safe* — which is exactly what makes it easy to do by accident.
#[test]
fn the_section_order_is_pinned() {
    assert_eq!(
        SECTIONS,
        [
            AEDIT_SEC_TRANSPORT,
            AEDIT_SEC_LOOP,
            AEDIT_SEC_EDIT,
            AEDIT_SEC_FX,
            AEDIT_SEC_MARKERS,
            AEDIT_SEC_VARIATIONS,
            AEDIT_SEC_DELIVERY,
        ]
    );
}

/// Fold state is resolved **by id**, never by position. Hand the body a fold array in
/// which exactly one section is shut and ask that section whether it is open: if the
/// lookup were positional, a reorder would quietly hand each section its neighbour's
/// state, and the panel would paint perfectly while folding the wrong block.
#[test]
fn the_fold_state_follows_the_section_not_the_slot() {
    for (i, id) in SECTIONS.iter().enumerate() {
        let mut open = [true; 7];
        open[i] = false;
        let b = body(open);
        assert!(!b.open(*id), "{id:?} lost its own fold state");
        for other in SECTIONS.iter().filter(|s| *s != id) {
            assert!(b.open(*other), "{other:?} was handed {id:?}'s fold state");
        }
    }
}

/// **No control may be painted twice.** The rack was being drawn once inside the Edit
/// section (which still delegated to it, from before the panel had sections) and again
/// as its own Effects section. It looks like a duplicated block, and it is worse than it
/// looks: `HitIndex::hit` walks back-to-front, so the LAST registration of an id wins
/// and the first copy becomes a ghost — painted, and unclickable.
///
/// A height test cannot see this (folding either copy still shortens the panel), so the
/// invariant has to be stated directly: paint the whole body, and no id may register a
/// hit rect more than once.
#[test]
fn no_control_is_painted_twice() {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let mut hits = HitIndex::default();
    {
        let clip = Rect::new(0.0, 0.0, 220.0, 40_000.0);
        let mut ch = ClippedHits::new(&mut hits, clip);
        paint_body(
            0.0,
            0.0,
            220.0,
            &body([true; 7]),
            &mut scene,
            &mut text,
            Theme::default(),
            &mut ch,
        );
    }
    let mut seen: std::collections::BTreeMap<ph2d_a11y::NodeId, usize> = Default::default();
    for (id, _) in hits.iter_registrations() {
        *seen.entry(id).or_default() += 1;
    }
    let dupes: Vec<_> = seen.iter().filter(|(_, n)| **n > 1).collect();
    assert!(
        dupes.is_empty(),
        "these controls are painted more than once (the first copy is a ghost): {dupes:?}"
    );
}

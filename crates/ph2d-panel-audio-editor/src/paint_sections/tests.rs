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

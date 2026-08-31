//! A lei das metades, e a que impede os encaixes de espelharem em silêncio.

use super::*;

fn layout(mirrored: bool) -> HeroLayout {
    HeroLayout::for_viewport_mirrored(Rect::new(0.0, 0.0, 1366.0, 1024.0), mirrored)
}

#[test]
fn a_column_with_one_occupant_is_not_halved() {
    let l = layout(false);
    let only_top = l.slot_rects(SlotSet::of(Slot::RightTop));
    let (_, right) = l.side_columns();
    assert_eq!(
        only_top.get(Slot::RightTop).h,
        right.h,
        "um painel sozinho na coluna perdeu metade da altura por uma divisão que ninguém pediu"
    );
}

#[test]
fn a_column_with_both_halves_occupied_is_halved_and_the_halves_do_not_overlap() {
    let l = layout(false);
    let both = l.slot_rects(SlotSet::RIGHT);
    let top = both.get(Slot::RightTop);
    let bottom = both.get(Slot::RightBottom);
    assert!(
        (top.y + top.h - bottom.y).abs() < 0.001,
        "as duas metades não se tocam: {top:?} / {bottom:?}"
    );
    let (_, band) = l.side_columns();
    assert!(
        (top.h + bottom.h - band.h).abs() < 0.001,
        "as metades não somam a coluna ({} + {} ≠ {})",
        top.h,
        bottom.h,
        band.h
    );
    assert!(top.h > 0.0 && bottom.h > 0.0);
}

/// ⛔ **O controlo do espelho.** Ler `layout.hierarchy` pelo nome daria a coluna da DIREITA sob
/// `ui_mirrored`, e os encaixes ficariam trocados sem uma linha de erro.
#[test]
fn the_left_slot_is_on_the_left_in_both_mirror_states() {
    for mirrored in [false, true] {
        let l = layout(mirrored);
        let r = l.slot_rects(SlotSet::SIDES);
        assert!(
            r.get(Slot::LeftTop).x < r.get(Slot::RightTop).x,
            "espelhado={mirrored}: o encaixe da esquerda ficou à direita ({:?} / {:?})",
            r.get(Slot::LeftTop),
            r.get(Slot::RightTop)
        );
    }
}

#[test]
fn the_center_is_the_drawing_area_and_the_bottom_is_the_strip() {
    let l = layout(false);
    let r = l.slot_rects(SlotSet::ANY_DOCK);
    assert_eq!(r.get(Slot::Center), l.draw_area);
    assert_eq!(r.get(Slot::Bottom), l.timeline);
}

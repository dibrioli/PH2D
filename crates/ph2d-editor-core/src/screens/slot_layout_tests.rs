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

/// ⛔⛔⛔ **A FAIXA DE ABAS DE UM ENCAIXE NÃO PODE MOVER O RECT DE OUTRO** — o report de
/// 2026-09-11 do dono, com duas fotos: *«Inspector e Hierarchy saíram da lateral e foram para uma
/// posição estranha na altura da timeline do Flip. Quando fecho o Flip e reabro, tudo volta para
/// posição normal.»*
///
/// # O mecanismo, e por que ele PRENDE
///
/// A regra do [`HeroLayout::reserve_slot_tabs`] é *«todo rect docado que TOCA a faixa começa onde
/// ela acaba»* — derivada de propósito, para não caducar quando um campo novo aparecer. Mas ela é
/// **larga demais**: quem toca a faixa do encaixe de BAIXO pode ser um rect que não é de lá.
///
/// As colunas são desenhadas com a altura inteira do chrome **sempre** — o `docks` só decide a
/// largura da ÁREA. Enquanto o `docks` diz que as duas colunas estão ocupadas, a faixa de baixo
/// nasce *entre* elas e nada se toca. Mas com `DockSides::NONE` ela alarga-se até `6 px` da borda
/// e passa a **correr por baixo das duas colunas** ⇒ a faixa de abas dela apanha a `hierarchy` e o
/// `inspector` e **empurra-os para dentro da banda de baixo**.
///
/// ⭐⭐ **E aí fecha o ciclo:** o `DockSides::from_published` do quadro seguinte mede esses rects
/// contra a coluna INTEIRA, lê ~20 % de cobertura (a barra é 50 %), conclui *«coluna vazia»*, e
/// devolve `NONE` outra vez. *São dois pontos fixos, e o primeiro quadro decide em qual se cai* —
/// é exactamente por isso que fechar e reabrir o Flip cura: o toggle muda a população e parte o
/// ciclo.
#[test]
fn the_bottom_tab_bar_never_moves_a_column_that_it_merely_runs_under() {
    // A configuração do report: colunas desenhadas, `docks` ainda a dizer NONE (o 1.º quadro,
    // antes de qualquer painel ter publicado rect), e a tira do Flip aberta em baixo.
    let mut l = HeroLayout::for_viewport_bands(
        Rect::new(0.0, 0.0, 1920.0, 1080.0),
        false,
        crate::screens::layout::ChromeBands {
            left_dock_w: 220.0,
            right_dock_w: 369.0,
            bottom_dock_h: 240.0,
            ..crate::screens::layout::ChromeBands::DEFAULT
        },
        crate::screens::layout::CenterSplit::None,
        crate::screens::layout::DockSides::NONE,
    );
    let (hier_antes, insp_antes) = (l.hierarchy, l.inspector);
    assert!(
        l.timeline.x < hier_antes.x + hier_antes.w,
        "controlo: esta fixtura só afirma alguma coisa se a banda de baixo de facto correr por \
         baixo da coluna — timeline={:?} hierarchy={hier_antes:?}",
        l.timeline
    );
    // um ocupante em cada uma das três: as duas colunas e a banda de baixo
    l.reserve_slot_tabs([1, 0, 1, 0, 1, 0], 22.0);
    assert_eq!(
        (l.hierarchy.y, l.hierarchy.h + 22.0),
        (hier_antes.y + 22.0, hier_antes.h),
        "a Hierarchy foi empurrada pela faixa de um encaixe que não é o dela — ela desceu para a \
         banda do Flip (report do dono, 2026-09-11)"
    );
    assert_eq!(
        (l.inspector.y, l.inspector.h + 22.0),
        (insp_antes.y + 22.0, insp_antes.h),
        "o Inspector foi empurrado pela faixa de um encaixe que não é o dele"
    );
}

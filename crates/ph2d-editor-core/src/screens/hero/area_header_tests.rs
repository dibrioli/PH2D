//! Os gates do cabeçalho da área (D2, metade 2).

use super::*;
use crate::screens::layout::{ChromeBands, DockSides, HeroLayout};

fn band() -> Rect {
    Rect::new(300.0, 60.0, 700.0, area_header_h())
}

/// ⭐ **As opções cabem na faixa e não se sobrepõem**, encostadas à direita.
#[test]
fn the_display_options_sit_inside_the_band_without_overlapping() {
    let mut ts = ph2d_text::TextSystem::new();
    let b = band();
    let rects = option_rects(b, &mut ts);
    assert_eq!(
        rects.len(),
        DISPLAY_OPTIONS.len(),
        "controlo: a faixa de referência não coube as opções e o gate mediria o vazio"
    );
    let mut prev_right = b.x;
    for (_, label, r) in &rects {
        assert!(r.x >= prev_right, "{label} sobrepõe-se ao anterior");
        assert!(
            r.y >= b.y && r.y + r.h <= b.y + b.h + 0.01,
            "{label} sai da faixa na vertical"
        );
        prev_right = r.x + r.w;
    }
    assert!(
        prev_right <= b.x + b.w + 0.01,
        "as opções passam da borda direita da faixa"
    );
}

/// ⛔ **Numa área estreita elas não nascem** — melhor ausentes do que por cima do que vem à
/// esquerda.
#[test]
fn a_narrow_area_gets_no_options_instead_of_overlapping_ones() {
    let mut ts = ph2d_text::TextSystem::new();
    let narrow = Rect::new(0.0, 0.0, 40.0, area_header_h());
    assert!(option_rects(narrow, &mut ts).is_empty());
}

/// ⭐⭐⭐ **O cabeçalho SUBTRAI** — a fila e o desenho começam por baixo dele.
///
/// ⛔ É a lei do modelo de áreas (`spec/01 §4`): *«uma faixa que continue a flutuar reproduz o
/// defeito, num modelo novo»*. Um cabeçalho que não subtraísse taparia a régua de cima, que é
/// exactamente o defeito de 86,8 % que a `D` curou.
#[test]
fn the_header_subtracts_from_the_area_it_heads() {
    let vp = Rect::new(0.0, 0.0, 1366.0, 1024.0);
    let bands = |h: f32| ChromeBands {
        area_header_h: h,
        tool_bar_h: 40.0,
        ..ChromeBands::DEFAULT
    };
    let without = HeroLayout::for_viewport_bands(
        vp,
        false,
        bands(0.0),
        crate::screens::layout::CenterSplit::None,
        DockSides::BOTH,
    );
    let with = HeroLayout::for_viewport_bands(
        vp,
        false,
        bands(32.0),
        crate::screens::layout::CenterSplit::None,
        DockSides::BOTH,
    );
    assert_eq!(with.area_header.h, 32.0);
    assert!(
        (with.tool_bar.y - without.tool_bar.y - 32.0).abs() < 0.01,
        "a fila não desceu com o cabeçalho"
    );
    assert!(
        (with.draw_area.y - without.draw_area.y - 32.0).abs() < 0.01,
        "a área de desenho não desceu com o cabeçalho"
    );
    assert!(
        (without.draw_area.h - with.draw_area.h - 32.0).abs() < 0.01,
        "o cabeçalho não SUBTRAIU altura — ele está a flutuar sobre o desenho"
    );
    // …e horizontalmente ele é a área, entre as colunas.
    assert!((with.area_header.x - with.draw_area.x).abs() < 0.01);
    assert!((with.area_header.w - with.draw_area.w).abs() < 0.01);
}

/// ⛔⛔ **O `ButtonState` destes ids significa «LIGADO», não «sob o rato»** — e é por isso que a
/// faixa não tem realce de hover.
///
/// Achado por uma **mutação que sobreviveu** (2026-08-31): apagar o `populate` da faixa deixava
/// tudo verde, porque os dois ids já eram registados pelo `pre_populate`. A seguir à pista veio o
/// resto: o [`crate::screens::hero::menu_bar::publish_toggle_state`] reescreve o estado deles **em todo quadro**
/// a partir da tabela de verdade, e escreve **depois** de qualquer hover.
///
/// > *Dois significados no mesmo campo, e ganha quem escreve por último.*
///
/// ⇒ um realce de hover na faixa seria pintado a partir de um valor que **nunca** é `Hovered`.
/// Este gate reprova no dia em que alguém devolver o campo ao hover — e nesse dia a faixa pode
/// voltar a ter realce.
#[test]
fn the_button_state_of_a_display_option_is_the_truth_and_never_the_hover() {
    let mut h = HeroScreen::new(crate::NodeId(1));
    // ⚠️ Registados à mão: num hero por pintar o `pre_populate` ainda não correu, e o que este gate
    // mede é o que o publish faz a um estado que EXISTE. Que eles chegam a existir no produto é o
    // que o gate de gesto prova (`ph2d-panel-registry-init`).
    for (id, _) in DISPLAY_OPTIONS {
        h.store.register(
            id,
            crate::interaction::InteractiveState::Button {
                state: crate::widget::ButtonState::Normal,
            },
        );
    }
    for on in [true, false, true] {
        h.view.rulers_visible = on;
        h.view.stats_visible = !on;
        // …e alguém põe a mão por cima, como o despacho de ponteiro faria.
        for (id, _) in DISPLAY_OPTIONS {
            if let Some(crate::interaction::InteractiveState::Button { state }) =
                h.store.get_mut(id)
            {
                *state = crate::widget::ButtonState::Hovered;
            }
        }
        crate::screens::hero::menu_bar::publish_toggle_state(&mut h);
        for ((id, label), expected) in DISPLAY_OPTIONS.iter().zip([on, !on]) {
            let st = h.store.button_state(*id);
            assert_eq!(
                st,
                Some(if expected {
                    crate::widget::ButtonState::Pressed
                } else {
                    crate::widget::ButtonState::Normal
                }),
                "{label}: o hover sobreviveu ao publish — o campo voltou a ter dois donos"
            );
        }
    }
}

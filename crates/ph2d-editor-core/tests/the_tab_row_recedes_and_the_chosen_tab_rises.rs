//! ⭐⭐⭐ **UMA ABA É UMA ABA: a faixa RECUA e a escolhida SOBE ATÉ AO PAINEL.**
//!
//! > *«Outra coisa: Belas abas de painéis.»* — Enio, 2026-09-07, com a captura das abas do
//! > Inspector do Godot ao lado.
//!
//! Portado de `theme_modern.cpp` (Godot 4.6, **MIT** — lemos e portamos): a faixa e a aba inactiva
//! levam `surface_lowest_color`; a **escolhida é um duplicado do `base_style`**, isto é, o corpo do
//! container; e os cantos são `set_corner_radius_individual(r, r, 0, 0)`.
//!
//! # ⛔⛔ O que estava invertido, e nenhum gate o via
//!
//! A faixa era pintada em `Bg1` — o tom do **CARTÃO**, que o `derive.rs` põe **12/255 acima** do
//! painel. Uma faixa mais clara do que a superfície em que assenta **não recua**: ela salta à
//! frente, e a aba escolhida não tem de onde subir. É a mesma inversão que a wave 31 curou um nível
//! acima (o chão da janela), e a razão de ninguém a ver é que **nenhum gate media a ORDEM** dos
//! tons que a fila escolhe — só os valores de cada token, um a um.
//!
//! ⇒ este gate mede a **relação**, e o controlo dele é o valor antigo: se `Bg1` voltasse, ele
//! reprova com o número.

use ph2d_editor_core::screens::hero::slot_tabs;
use ph2d_editor_core::widget::ButtonState;
use ph2d_tokens::{Color, ColorToken as C, Theme};

/// Os três temas modernos de base **não-preta**.
///
/// ⛔ O **OLED** fica de fora com mecanismo, e é o mesmo do gate `the_ground_stands_under_every_panel`:
/// a base dele é preta, a família multiplicativa colapsa lá, e quem separa duas superfícies é a
/// *Draw Extra Borders*.
const MODERN: [Theme; 3] = [Theme::Dark, Theme::Gray, Theme::Light];

fn depth(c: Color) -> u32 {
    u32::from(c.r) + u32::from(c.g) + u32::from(c.b)
}

/// ⭐⭐⭐ **A faixa está ATRÁS da aba escolhida** — a lei que estava invertida.
#[test]
fn the_row_sits_behind_the_tab_that_is_chosen() {
    for t in MODERN {
        let row = slot_tabs::tab_row_bg().resolve(t);
        let chosen = slot_tabs::tab_bg(true, ButtonState::Normal)
            .expect("a aba escolhida tem de ter tom próprio")
            .resolve(t);
        assert!(
            depth(row) < depth(chosen),
            "{t:?}: a faixa ({}) não está atrás da aba escolhida ({}) — uma faixa que não recua \
             salta à frente, e a aba deixa de ter de onde subir",
            depth(row),
            depth(chosen)
        );
    }
}

/// ⚠️⚠️ **O CONTROLO — e ele revelou que a inversão NÃO existia nos três temas.**
///
/// Medido em 2026-09-07 (soma dos três canais, 0..765):
///
/// | tema | chão | painel | `Bg1` (o tom que a faixa TINHA) |
/// |---|---:|---:|---:|
/// | Dark | 27 | 57 | **93** ⛔ acima do painel |
/// | Gray | 54 | 84 | **141** ⛔ acima do painel |
/// | Light | 732 | 762 | 723 ✅ já estava abaixo |
///
/// ⭐ **No tema claro a escada INVERTE-SE** — ali a «elevação» escurece (`derive.rs`: o contraste é
/// negativo num tema claro), logo o cartão é mais fundo que o painel e a faixa antiga já recuava.
/// ⇒ o defeito que o dono viu vivia no **Dark** (o tema dele) e no Gray, e a família clara era
/// imune por construção.
///
/// Sem este controlo o gate acima passaria sobre qualquer par em que um tom seja mais escuro —
/// inclusive um par que ninguém escolheu.
#[test]
fn the_row_that_did_not_recede_is_still_measurable() {
    for t in [Theme::Dark, Theme::Gray] {
        let chosen = slot_tabs::tab_bg(true, ButtonState::Normal)
            .expect("a aba escolhida tem de ter tom próprio")
            .resolve(t);
        assert!(
            depth(C::Bg1.resolve(t)) > depth(chosen),
            "{t:?}: controlo partido — o `Bg1` já não está acima do painel ({} contra {}), logo \
             este ficheiro deixou de medir a inversão que produziu o report",
            depth(C::Bg1.resolve(t)),
            depth(chosen)
        );
    }
}

/// ⭐⭐⭐ **Ela é quadrada EM BAIXO** — é isso que a solda ao corpo do painel.
///
/// Com os quatro cantos redondos ela lê-se como um botão a flutuar sobre a fila; com os de baixo
/// quadrados, ela e o painel passam a ser uma superfície só.
#[test]
fn a_tab_is_square_at_the_bottom_so_it_welds_to_the_panel() {
    for t in MODERN {
        let (tl, tr, br, bl) = slot_tabs::tab_radii(t);
        assert!(
            tl > 0.0 && tr > 0.0,
            "{t:?}: a aba perdeu a quina de cima ({tl}, {tr}) — sem ela não há forma de aba nenhuma"
        );
        assert!(
            br == 0.0 && bl == 0.0,
            "{t:?}: a aba arredondou em BAIXO ({br}, {bl}) — ela descolou do painel e voltou a ser \
             um botão pousado na fila"
        );
    }
}

/// ⚠️ **A inactiva não tem tom próprio, e isso É o modelo** — lá ela leva o `surface_lowest_color`,
/// que é exactamente a cor da faixa; pintá-la seria pintar o que já lá está.
#[test]
fn an_unchosen_tab_paints_nothing_of_its_own() {
    assert_eq!(
        slot_tabs::tab_bg(false, ButtonState::Normal),
        None,
        "a aba inactiva ganhou preenchimento — ela passa a competir com a escolhida"
    );
    // …mas sob o dedo ela responde, senão a fila fica morta ao toque.
    assert!(
        slot_tabs::tab_bg(false, ButtonState::Hovered).is_some(),
        "a aba não responde ao ponteiro"
    );
}

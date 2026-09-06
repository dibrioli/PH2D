//! Os gates do cartão de secção. ⚠️ **A régua é a CENA** (quantos caminhos foram emitidos e por
//! que ordem), não a aritmética do `Rect` — um cartão que se calcula certo e não se pinta lê-se
//! exactamente como um que se pinta.

use super::*;
use ph2d_tokens::Theme;

/// Quantos caminhos a cena emitiu.
fn path_count(scene: &VectorScene) -> usize {
    scene.inner().encoding().n_paths as usize
}

/// ⭐⭐⭐ **O corpo NÃO se perde ao ser estacionado.**
///
/// Esta é a metade que paga o mecanismo: se o `append` deixasse cair o que o corpo pintou, todo
/// painel convertido ficaria em branco — e nenhum gate de geometria o veria, porque os `Rect`
/// continuariam certos. *Uma faixa reservada não é uma faixa pintada.*
#[test]
fn the_parked_body_comes_back_whole_and_the_cards_come_under_it() {
    let theme = Theme::MODERN[0];

    // Quanto custa só o corpo, sem cartão nenhum.
    let mut plain = VectorScene::new();
    let mut y = 0.0;
    for _ in 0..3 {
        fill_rounded_rect(
            &mut plain,
            Rect::new(0.0, y, 100.0, 20.0),
            0.0,
            resolve(ColorToken::Text1, theme),
        );
        y += 30.0;
    }
    let body_paths = path_count(&plain);
    assert!(body_paths >= 3, "o corpo de controlo tem de pintar algo");

    // O mesmo corpo, dentro de cartões.
    let mut scene = VectorScene::new();
    let n = with_section_cards(&mut scene, theme, 0.0, |scene, cards| {
        let mut y = 0.0;
        let mut n = 0;
        for _ in 0..3 {
            fill_rounded_rect(
                scene,
                Rect::new(0.0, y, 100.0, 20.0),
                0.0,
                resolve(ColorToken::Text1, theme),
            );
            y += 30.0;
            y = cards.close(scene, 0.0, 100.0, y);
            n += 1;
        }
        n
    });

    assert_eq!(n, 3, "o corpo devolve o que devolveria fora do cartão");
    assert_eq!(
        path_count(&scene),
        body_paths + 3,
        "a cena tem de ter o corpo INTEIRO mais um cartão por secção — se o `append` perdesse o \
         corpo isto leria o número dos cartões sozinhos"
    );
}

/// ⭐ **O cartão envolve o conteúdo, e por FORA** — nenhuma linha muda de sítio.
#[test]
fn the_card_is_an_outset_so_no_row_moves() {
    let theme = Theme::MODERN[0];
    let mut cards = SectionCards::new(theme, 10.0);
    let mut scene = VectorScene::new();
    let next = cards.close(&mut scene, 20.0, 100.0, 70.0);

    let (rect, depth) = cards.rects[0];
    assert_eq!(depth, CardDepth::Section);
    assert!(
        rect.y < 10.0 && rect.y + rect.h > 70.0 && rect.x < 20.0 && rect.x + rect.w > 120.0,
        "o cartão {rect:?} tem de conter o conteúdo (10..70 em y, 20..120 em x) com folga"
    );
    assert!(next > 70.0, "o cursor avança para lá do conteúdo fechado");
}

/// ⭐⭐ **Uma subsecção é um cartão de OUTRA cor** — é isso, e só isso, que a distingue.
#[test]
fn a_subsection_is_a_lighter_card_on_top_of_its_parent() {
    let theme = Theme::MODERN[0];
    let section = resolve(CardDepth::Section.token(), theme);
    let sub = resolve(CardDepth::Subsection.token(), theme);
    assert_ne!(
        section, sub,
        "o conteúdo de uma subsecção vive num container de cor DIFERENTE (pedido do dono)"
    );
    let lum = |c: ph2d_vector::Color| {
        let [r, g, b, _] = c.components;
        r + g + b
    };
    assert!(
        lum(sub) > lum(section),
        "o degrau é para CIMA: a subsecção assenta sobre o cartão do pai"
    );
}

/// ⛔ **O clássico continua a ser o clássico** — risco, nenhum cartão.
#[test]
fn the_classic_family_still_draws_the_rule_and_no_card() {
    for theme in Theme::CLASSIC {
        let mut scene = VectorScene::new();
        with_section_cards(&mut scene, theme, 0.0, |scene, cards| {
            let y = cards.close(scene, 0.0, 100.0, 40.0);
            assert!(y > 40.0);
        });
        assert!(
            path_count(&scene) > 0,
            "o {theme:?} tem de continuar a desenhar o risco entre secções"
        );
    }
}

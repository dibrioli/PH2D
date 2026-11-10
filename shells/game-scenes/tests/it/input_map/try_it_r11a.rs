//! **R11a · C · Try it, os últimos detalhes**: C2 com as setas encostadas à DIREITA do cartão (as letras ficam à
//! esquerda); C3 com o disco Dead num cinza claramente mais claro e menos azul que o círculo. A prévia da C4 escalada
//! inteira mora nos testes da `ph2d-player-controls` (`r11a_tests`).

use super::try_it_censo::cena;
use ph2d_app_input_map::view_keyboard::state::KbHit;
use ph2d_app_input_map::view_try::stick::{DEAD_INK, OUTER_INK};
use ph2d_i18n::tr;
use ph2d_input::Key;
use ph2d_tokens::{ColorToken, Theme};

fn key(name: &str) -> Key {
    Key::named_in_script(name).expect("tecla com nome")[0]
}

#[test]
fn c2_as_setas_encostam_a_direita_do_cartao_e_as_letras_a_esquerda() {
    let m = cena("C2");
    let doc = &m.tab().doc;
    let at = |k: &str| {
        doc.rect_of(KbHit::Key(key(k)).id())
            .expect("a tecla pintada")
    };
    let (q, p, space) = (at("q"), at("p"), at("space"));
    let (up, left, down, right) = (at("up"), at("left"), at("down"), at("right"));
    // W2: o bloco da DIREITA do cartão é agora o numérico (arranjo Wide); a sua última coluna acaba no recuo.
    let edge = at("pagedown");
    let status = (doc.census.texts.iter())
        .find(|t| t.1 == tr("input_map_try.stopped"))
        .expect("«The game is not running»")
        .0;
    let pad = 10.0; // LITERAL-PX-OK: CARD_PAD_X, o recuo do desenho
    let card_right = status.x + status.w;
    assert!(
        (edge.x + edge.w - (card_right - pad)).abs() < 1.0,
        "o numérico no recuo DIREITO do desenho: {edge:?} × {status:?}"
    );
    assert!(
        left.x > space.x + space.w + p.w && left.x > p.x + p.w,
        "o bloco das setas sai de junto da barra de espaço: {left:?} × {space:?}"
    );
    assert!((up.x - down.x).abs() < 0.5, "a ↑ por cima da ↓");
    assert!(left.x < down.x && down.x < right.x, "← ↓ → em fila");
    assert!(q.x < left.x, "as letras à esquerda");
}

#[test]
fn c3_o_disco_dead_e_um_cinza_claramente_mais_claro_que_o_circulo() {
    let lum = |t: ColorToken, th: Theme| {
        let c = t.resolve(th);
        i32::from(c.r) * 299 + i32::from(c.g) * 587 + i32::from(c.b) * 114
    };
    let blue = |t: ColorToken, th: Theme| {
        let c = t.resolve(th);
        i32::from(c.b) - i32::from(c.r)
    };
    for th in [Theme::default(), Theme::Dark, Theme::Gray, Theme::Oled] {
        let old = lum(ColorToken::KanbanCardRaised, th) - lum(OUTER_INK, th);
        let new = lum(DEAD_INK, th) - lum(OUTER_INK, th);
        assert!(
            new > old * 2,
            "{th:?}: o disco Dead claramente mais claro ({new} contra o degrau antigo {old})"
        );
        assert!(
            blue(DEAD_INK, th) < blue(ColorToken::KanbanCardRaised, th),
            "{th:?}: um cinza, menos azul que o cartão aceso"
        );
    }
}

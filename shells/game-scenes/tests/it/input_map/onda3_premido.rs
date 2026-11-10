//! Onda 3, item 5: o × premido é mais fundo que o hover nos dois temas.
//!
//! O premido pintava `danger-soft`, que no tema CLARO é um rosa quase branco (Y 0,76 contra 0,15 do `danger`): o
//! dedo carregava e o emblema CLAREAVA. A lei, medida pela luminância relativa (WCAG) em todos os temas: premido
//! mais fundo que pairado, e o ícone premido lê-se a ≥ 4,5:1 sobre a face.

use ph2d_app_input_map::View;
use ph2d_app_input_map::census::ControlLook;
use ph2d_app_input_map::ids::ListHit;
use ph2d_input::{Binding, InputMap, Key};
use ph2d_tokens::{Theme, contrast::TEXT_MIN_RATIO};

use super::Mesa;

/// O olhar do × pairado e premido, com o chip «J» do «jump».
fn hover_e_premido() -> (ControlLook, ControlLook) {
    let mut m = Mesa::aberta(View::List);
    let mut map = InputMap::new();
    let k = map.create("jump");
    if let Some(a) = map.get_mut(k) {
        a.bindings.push(Binding::Key(Key(u32::from('J'))));
    }
    m.hero.input_map = map;
    m.frame();
    let chip = m
        .tab()
        .doc
        .rect_of(ListHit::Chip(k, 0).id())
        .expect("o chip");
    m.hover((chip.x + chip.w * 0.05, chip.y + chip.h * 0.5));
    let x = ListHit::RemoveBinding(k, 0).id();
    let at = m.at(x);
    m.hover(at);
    let hover = m.tab().doc.census.look(x).expect("pairado");
    m.down(at);
    let press = m.tab().doc.census.look(x).expect("premido");
    (hover, press)
}

#[test]
fn o_x_premido_e_mais_fundo_que_o_pairado_em_todo_tema() {
    let (hover, press) = hover_e_premido();
    let hf = hover.fill.expect("o pairado enche");
    let pf = press.fill.expect("o premido enche");
    let mut falhas = Vec::new();
    for theme in Theme::ALL {
        let yh = hf.resolve(theme).relative_luminance();
        let yp = pf.resolve(theme).relative_luminance();
        let cr = pf.resolve(theme).contrast_ratio(&press.ink.resolve(theme));
        let dark = if theme.is_dark() { "escuro" } else { "claro" };
        let ch = hf.resolve(theme).contrast_ratio(&hover.ink.resolve(theme));
        eprintln!(
            "{theme:?} ({dark}): Y pairado {yh:.4} · premido {yp:.4} · ícone {ch:.2}:1 → {cr:.2}:1"
        );
        if yp >= yh {
            falhas.push(format!(
                "{theme:?} ({dark}): premido {pf:?} Y {yp:.4} ≥ pairado {hf:?} Y {yh:.4}"
            ));
        }
        if cr < TEXT_MIN_RATIO {
            falhas.push(format!(
                "{theme:?} ({dark}): ícone {:?} sobre {pf:?} a {cr:.2}:1",
                press.ink
            ));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

//! O acabamento da vista A (plano §11 «Aberto da A»): a frase do pé do estado 4, a barra do cartão quando a lista
//! rola (as linhas encolhem para ela não tapar o lixo) e o chip da manche inteiro com a seta.

use super::lista::onze;
use super::{AREA, Mesa};
use ph2d_app_input_map::View;
use ph2d_app_input_map::ids::ListHit;
use ph2d_editor_core::widget::SCROLLBAR_W;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;

fn painted(m: &Mesa, s: &str) -> bool {
    m.tab().doc.census.texts.iter().any(|t| t.1 == s)
}

fn lista(map: ph2d_input::InputMap) -> Mesa {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = map;
    m.frame();
    m
}

#[test]
fn a_frase_das_zonas_aparece_so_com_uma_linha_de_manche() {
    let lead = tr("input_map_list.zones_lead");
    let m = lista(ph2d_input::InputMap::with_player_defaults());
    assert!(!painted(&m, lead), "sem manche nem gatilho, nada a explicar");
    let m = lista(onze());
    assert!(painted(&m, lead), "o move_left tem a manche: o pé explica");
    assert!(painted(&m, tr("input_map_list.zones_rest")));
}

#[test]
fn a_lista_que_rola_desenha_a_barra_ao_lado_das_linhas() {
    let mut m = lista(onze());
    let jump = m.hero.input_map.id("jump").expect("fábrica");
    let row = ListHit::Row(jump).id();
    let wide = m.tab().doc.rect_of(row).expect("a linha pintada").w;
    assert!(
        m.tab().doc.list.scroll_max <= 0.0,
        "a 1600×1000 as 11 cabem"
    );
    let short = Rect {
        h: 420.0, // LITERAL-PX-OK: uma aba baixa, para a lista rolar
        ..AREA
    };
    m.paint_in(short);
    assert!(m.tab().doc.list.scroll_max > 0.0, "a aba baixa rola");
    let narrow = m.tab().doc.rect_of(row).expect("a linha pintada").w;
    assert!(
        wide - narrow >= SCROLLBAR_W,
        "a linha encolhe pela barra: {wide} → {narrow}"
    );
}

#[test]
fn o_chip_da_manche_leva_o_nome_inteiro_ao_lado_da_seta() {
    let m = lista(onze());
    let stick = tr(ph2d_input::GamepadAxis::LeftStickX.label_key());
    let t = m.tab().doc.census.texts.iter().find(|t| t.1 == stick);
    let t = t.expect("o chip da manche pintado");
    assert!(!t.2, "«{stick}» cabe inteiro no chip, com a seta");
}

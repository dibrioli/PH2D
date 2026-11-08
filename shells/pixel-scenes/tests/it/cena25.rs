//! ⭐ **A cena 25 abre como o smoke diz** (P12, `docs/PixelArt/P12_plano.md` §8): o boneco de três
//! camadas e quatro quadros, a Undo History com os nomes dos caminhos reais, o `Fill` em `Shift+F`
//! num ficheiro DA cena, a grade 8 × 8, o onion 1 + 1 e o user data da camada Color.

use ph2d_editor_core::pixel_panel::{PixelEditAction as E, PixelRequest};
use ph2d_panel_pixel_history::canal;
use ph2d_pixel_cenas::historia as h;
use ph2d_pixel_core::{Owner, PropValue};
use ph2d_pixel_keys::{Chord, Context, PixelAction};
use winit::keyboard::KeyCode;

use crate::selecao::Palco;

fn rotulos() -> Vec<String> {
    canal::view()
        .rows
        .iter()
        .map(|r| ph2d_i18n::tr(r.label).to_string())
        .collect()
}

#[test]
fn a_cena_25_abre_com_a_historia_as_teclas_e_as_preferencias() {
    let mut p = Palco::cena(25);
    p.quadro();
    let s = p.sprite();
    let nomes: Vec<_> = s.layers().iter().map(|l| l.name.as_str()).collect();
    assert_eq!(nomes, ["Color", "Shadow", "Outline", "Layer 4"]);
    assert_eq!(s.frames().len(), h::FRAMES as usize);
    let cor = s.layers()[0].id;
    let u = s.user_of(Owner::Layer(cor)).expect("o user data da Color");
    assert_eq!((u.color, u.text.as_str()), (h::USER_COLOR, h::USER_TEXT));
    assert!(matches!(u.props[""]["hp"], PropValue::I32(10)));
    let g = s.prefs().grid;
    assert_eq!((g.w, g.h, g.visible, g.color), (8, 8, true, h::GRID_COLOR));
    let o = s.prefs().onion;
    assert_eq!((o.prev, o.next, p.app.onion_on), (1, 1, true));

    assert_eq!(
        rotulos(),
        [
            "Initial State",
            "Pencil Tool",
            "Line Tool",
            "Paint Bucket Tool",
            "New Layer",
            "Pencil Tool",
            "Flip Horizontal",
            "Hue/Saturation",
        ]
    );
    assert_eq!(canal::view().current, h::PASSOS.len());

    // As teclas: o Fill saiu do F e está no Shift+F, no mapa da app e no ficheiro da cena.
    let f = Chord::new(KeyCode::KeyF);
    assert_eq!(p.app.keys.resolve(f, Context::Normal), None);
    assert_eq!(
        p.app.keys.resolve(h::FILL_KEY, Context::Normal),
        Some(PixelAction::Fill)
    );
    let (do_ficheiro, ficheiro) = h::teclas().expect("o ficheiro da cena");
    assert!(ficheiro.starts_with(std::env::temp_dir()), "nunca a HOME");
    assert_eq!(do_ficheiro.to_text(), p.app.keys.to_text());
}

#[test]
fn a_cena_25_o_shift_f_enche_e_o_passo_do_menu_tem_o_nome_do_caminho_real() {
    let mut p = Palco::cena(25);
    // O Flip Horizontal do menu Edit grava a mesma chave que a cena pôs na história.
    p.pede(PixelRequest::Edit(E::FlipH));
    p.quadro();
    let v = canal::view();
    assert_eq!(v.rows.last().map(|r| r.label), Some(h::PASSOS[5]));
    p.tecla(KeyCode::KeyA, true, false);
    let antes = p.app.doc(p.key).expect("doc").history.len();
    let _ = p.tecla_com(KeyCode::KeyF, false, false);
    assert_eq!(
        p.app.doc(p.key).expect("doc").history.len(),
        antes,
        "o F sozinho já não enche"
    );
    p.tecla(KeyCode::KeyF, false, true);
    assert_eq!(
        p.app.doc(p.key).expect("doc").history.len(),
        antes + 1,
        "o Shift+F enche"
    );
}

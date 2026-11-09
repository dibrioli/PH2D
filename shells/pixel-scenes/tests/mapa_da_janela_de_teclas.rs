//! **«A tecla escolhida aparece uma atrasada»** (smoke do dono 08/11, `P12_plano.md` §9.8): o mapa
//! que a janela KEYBOARD SHORTCUTS publica só entrava em `PixelApp::keys` na TECLA SEGUINTE
//! (`input_teclas::key_with` e a paleta eram os únicos leitores de `toma_keymap`). Agora entra no
//! quadro do painel PIXEL. Binário à parte: o canal é do processo, e um mapa publicado no `it`
//! seria tomado pelos palcos dos outros testes em paralelo (a cena 25 tem o seu `Shift+F`).

#[allow(dead_code)]
#[path = "it/comum.rs"]
mod comum;

use ph2d_app_pixel::comando;
use ph2d_pixel_keys::{Chord, Context, Keymap, PixelAction as A};
use winit::keyboard::KeyCode as K;

#[test]
fn o_mapa_novo_da_janela_entra_no_quadro_sem_outra_tecla() {
    let mut p = comum::Palco::limpo();
    let f7 = Chord::new(K::F7);
    let mut k = Keymap::factory();
    k.assign_replacing(A::ToolPencil, f7, Context::Normal)
        .expect("F7 não é tecla do app");
    ph2d_panel_pixel_keys::canal::publica_keymap(k.clone());
    p.quadro();
    assert!(p.app.keys == k, "o quadro seguinte já tem o mapa novo");
    assert!(
        comando::label(&p.app, A::ToolPencil).contains("F7"),
        "{}",
        comando::label(&p.app, A::ToolPencil)
    );
}

/// ⭐ **A tecla que a coluna e o aviso dizem é a do mapa VIVO** (fecho P12 U5): o Pencil trocado de
/// `B` para `F7` diz «Pencil (F7)» na dica da coluna e no aviso da troca — nunca o `B` de fábrica.
#[test]
fn a_dica_e_o_aviso_da_ferramenta_dizem_a_tecla_do_mapa_vivo() {
    let mut p = comum::Palco::limpo();
    let f7 = Chord::new(K::F7);
    let mut k = Keymap::factory();
    k.remove(A::ToolPencil, Chord::new(K::KeyB));
    k.assign_replacing(A::ToolPencil, f7, Context::Normal)
        .expect("F7 não é tecla do app");
    ph2d_panel_pixel_keys::canal::publica_keymap(k);
    p.pede(ph2d_editor_core::pixel_panel::PixelRequest::Tool(
        ph2d_app_pixel::Tool::Eraser,
    ));
    let info = p.quadro();
    let pencil = ph2d_panel_pixel_tools::column::bars(&info)
        .column
        .into_iter()
        .flatten()
        .find(|b| b.label.starts_with("Pencil"))
        .expect("o Pencil está na coluna");
    assert_eq!(pencil.label, "Pencil (F7)", "a dica da coluna");
    let a =
        ph2d_app_pixel::input::key(&mut p.app, p.sim.world(), Some(p.bits), K::F7, false, false);
    assert!(
        matches!(&a, Some(ph2d_app_pixel::input::KeyAnswer::Toast(t)) if t == "Pencil (F7)"),
        "o aviso da troca: {a:?}"
    );
}

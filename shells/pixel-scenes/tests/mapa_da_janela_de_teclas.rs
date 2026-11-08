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

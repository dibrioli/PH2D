//! P13 L — **a *Light* do cartão Pixel Sprite, a SEQUÊNCIA**: o pedido `LuzNaPaleta` grava no
//! documento do objecto (UM passo do Pixel, Ctrl+Z volta), o cartão publica-o, e a faixa que a
//! família dá à shell passa a levar a banda de rampa. O clique real no cartão prova-se no
//! `ph2d-panel-inspector` (`a_light_do_cartao_pede_a_rampa_ao_documento`); o quadro aceso, em
//! `pixel_na_luz::a_luz_na_rampa_no_quadro_fica_na_rampa`.

use ph2d_app_pixel::PixelApp;
use ph2d_app_pixel::input;
use ph2d_editor_core::pixel_panel::PixelRequest;
use ph2d_vector::Affine;

use crate::edit::{PPM, born};

const RAMPA: [[u8; 4]; 3] = [[20, 40, 90, 255], [70, 120, 200, 255], [150, 190, 240, 255]];

#[test]
fn a_light_do_cartao_grava_a_rampa_num_passo() {
    let (mut app, sim, _camera, bits) = born();
    let key = input::key_of(sim.world(), bits).expect("marcador");
    let s = &app.doc(key).expect("doc").sprite;
    let op = ph2d_pixel_rampa::op_rampa_nova(s, "r", &RAMPA).expect("rampa nova");
    app.commit(key, op).expect("a rampa");
    let rampa_na_faixa = |app: &PixelApp| {
        let cam = (Affine::IDENTITY, PPM);
        let f = ph2d_app_pixel::luz::faixa_de_luz(app, sim.world(), cam, &[bits], Some(&[]));
        f.expect("a luz acesa dá as bandas").1.is_some()
    };
    let luz = |app: &PixelApp| app.doc(key).expect("doc").sprite.ramps().luz_na_paleta;
    let passos = |app: &PixelApp| app.doc(key).expect("doc").history.undo_len();
    assert!(
        !luz(&app) && !rampa_na_faixa(&app),
        "Smooth: sem banda de rampa"
    );
    let n = passos(&app);
    let pede = |app: &mut PixelApp, on| app.aplica(PixelRequest::LuzNaPaleta { doc: key, on });
    pede(&mut app, true);
    assert!(luz(&app), "Palette no documento");
    assert_eq!(passos(&app), n + 1, "UM passo");
    assert!(
        rampa_na_faixa(&app),
        "Palette: a banda de rampa chega à shell"
    );
    pede(&mut app, true);
    assert_eq!(passos(&app), n + 1, "o mesmo valor não é passo");
    let mut hero = ph2d_editor_core::screens::hero::HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.replace_selection(Some(bits));
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    let _ = ph2d_app_pixel::panel::frame(&mut app, sim.world(), &mut hero, &mut toasts);
    let c = hero.store.pixel_sprite_card(bits).expect("o cartão");
    assert!(c.luz_na_paleta, "o cartão acende Palette");
    assert!(app.undo(key), "Ctrl+Z");
    assert!(
        !luz(&app) && !rampa_na_faixa(&app),
        "o desfazer devolve Smooth"
    );
}

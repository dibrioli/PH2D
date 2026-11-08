//! **Sprite ▸ Properties… e o User Data do sprite e do tileset, do pedido ao documento** (P12 U,
//! O17): o botão do menu abre a secção; cada gesto do cartão é UM passo «User Data»; o desfazer
//! devolve o de antes; o `.ase` guarda o do sprite (o JSON do Trial não tem campo do sprite: a
//! verificação é o nosso leitor).

use ph2d_editor_core::panel::cards::user_data::CardEdit;
use ph2d_editor_core::pixel_panel::{
    PixelOwner, PixelRequest, PixelSpriteAction as Sp, sprite_props,
};
use ph2d_pixel_core::{Label, Owner, PropValue};

use crate::comum::Palco;

fn passo(p: &Palco) -> (usize, Label) {
    let d = p.app.doc(p.key).expect("doc");
    let ultimo = d.history.entries().last().expect("passo").0.clone();
    (d.history.undo_len(), ultimo)
}

#[test]
fn o_botao_properties_abre_a_seccao_e_o_user_data_do_sprite_e_um_passo() {
    let mut p = Palco::limpo();
    sprite_props::set_open(false);
    let antes = p.app.doc(p.key).expect("doc").history.undo_len();
    let info = p.pede(PixelRequest::Sprite(Sp::Properties));
    assert!(sprite_props::is_open(), "o botão do menu abre a secção");
    assert_eq!(
        p.app.doc(p.key).expect("doc").history.undo_len(),
        antes,
        "abrir não é passo"
    );
    assert!(
        info.user.sprite.is_some(),
        "a secção vê o user data do sprite"
    );
    let original = p.sprite().user_of(Owner::Sprite).expect("sprite").clone();
    p.pede(PixelRequest::UserData(
        PixelOwner::Sprite,
        CardEdit::Text("hero sheet".into()),
    ));
    assert_eq!(
        passo(&p),
        (antes + 1, Label::new("pixel.history.user_data"))
    );
    p.pede(PixelRequest::UserData(PixelOwner::Sprite, CardEdit::Add));
    p.pede(PixelRequest::UserData(
        PixelOwner::Sprite,
        CardEdit::Kind("property".into()),
    ));
    let info = p.quadro();
    assert_eq!(info.user.sprite.expect("vista").text, "hero sheet");
    let u = p.sprite().user_of(Owner::Sprite).expect("sprite");
    assert!(
        matches!(u.props[""]["property"], PropValue::Bool(_)),
        "o chip roda o tipo"
    );
    let (bytes, _) = ph2d_pixel_io::ase::escrever(p.sprite()).expect("grava o .ase");
    let (lido, _) = ph2d_pixel_io::ase::ler(&bytes).expect("lê o .ase");
    assert_eq!(
        lido.user_of(Owner::Sprite),
        p.sprite().user_of(Owner::Sprite),
        "o .ase guarda o user data do sprite"
    );
    for _ in 0..3 {
        assert!(p.app.undo(p.key));
    }
    assert_eq!(p.sprite().user_of(Owner::Sprite), Some(&original));
    p.pede(PixelRequest::Sprite(Sp::Properties));
    assert!(!sprite_props::is_open(), "o mesmo botão fecha-a");
}

#[test]
fn o_user_data_do_tileset_pelo_cartao_tileset_e_um_passo_e_desfaz() {
    use ph2d_app_pixel::{PixelApp, mode, panel};
    use ph2d_panel_pixel_tiles::canal::{self, Pedido};
    let _ = canal::take_pedidos();
    let (mut app, mut sim) = (PixelApp::default(), ph2d_ecs::SimWorld::new());
    assert!(ph2d_app_pixel::smoke::stage(
        &mut app,
        &mut sim,
        21,
        crate::edit::spawn_root
    ));
    let (bits, key) = mode::sprites(sim.world())[0];
    let mut hero = ph2d_editor_core::screens::hero::HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo
        .mode
        .enter(bits, ph2d_editor_core::object_mode::ObjectMode::Draw);
    let mut cam = ph2d_render::Camera2d::default();
    let (w, ppm) = (crate::edit::WINDOW, crate::edit::PPM);
    app.frame(
        &mut sim,
        &mut cam,
        w,
        ppm,
        Some(bits),
        crate::edit::spawn_root,
    );
    let mut toasts = ph2d_editor_core::ToastQueue::new();
    panel::frame(&mut app, sim.world(), &mut hero, &mut toasts);
    let d = app.doc(key).expect("doc");
    let set = d
        .sprite
        .layer(d.layer)
        .and_then(|l| l.tileset())
        .expect("tileset activo");
    let dono = Owner::Tileset(set);
    let original = d.sprite.user_of(dono).expect("tileset").clone();
    let antes = d.history.undo_len();
    assert!(
        canal::contexto().user.is_some(),
        "o cartão Tileset vê o user data"
    );
    canal::pede(Pedido::UserData(CardEdit::Text("grass".into())));
    panel::frame(&mut app, sim.world(), &mut hero, &mut toasts);
    let d = app.doc(key).expect("doc");
    assert_eq!(d.history.undo_len(), antes + 1, "um passo");
    assert_eq!(
        d.history.entries().last().expect("passo").0,
        Label::new("pixel.history.user_data")
    );
    assert_eq!(d.sprite.user_of(dono).expect("tileset").text, "grass");
    panel::frame(&mut app, sim.world(), &mut hero, &mut toasts);
    assert_eq!(canal::contexto().user.expect("vista").text, "grass");
    assert!(app.undo(key));
    let d = app.doc(key).expect("doc");
    assert_eq!(d.sprite.user_of(dono), Some(&original));
}

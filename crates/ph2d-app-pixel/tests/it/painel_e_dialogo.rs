//! **O painel PIXEL e o diálogo New Sprite do lado da família** (ADR-0192): o OK do diálogo nasce o
//! objecto com AS escolhas, sem OK não nasce nada; os pedidos do painel mudam a ferramenta e as
//! cores; o painel abre no flanco do Edit e fecha ao sair; o instantâneo leva a paleta do sprite.

use ph2d_app_pixel::{PixelApp, Tool, input, object_add, panel};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{
    NEW_SPRITE_MAX_SIDE, NewSpriteBackground, NewSpriteMode, PixelRequest, PixelTool,
};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_pixel_core::ColorMode;

fn spawn_root(sim: &mut SimWorld, name: &str) -> u64 {
    let bits = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(name)))
        .id()
        .to_bits();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    bits
}

fn sprites(sim: &SimWorld) -> usize {
    ph2d_app_pixel::mode::sprites(sim.world()).len()
}

#[test]
fn o_tecto_do_dialogo_e_o_do_documento() {
    assert_eq!(NEW_SPRITE_MAX_SIDE, ph2d_pixel_core::MAX_SIDE);
}

#[test]
fn o_pick_abre_o_dialogo_e_nao_cria_nada_ate_ao_ok() {
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    let (mut sim, mut app) = (SimWorld::new(), PixelApp::default());
    assert!(object_add::open_dialog(
        object_add::PIXEL_SPRITE,
        &mut hero.store
    ));
    assert!(hero.store.context_menu().is_some(), "o diálogo abriu");
    assert!(
        object_add::born_from_dialog(&mut hero.store, &mut sim, &mut app, spawn_root).is_none()
    );
    // Cancel: fecha sem pedido ⇒ nenhum objecto.
    hero.store.close_context_menu();
    assert!(
        object_add::born_from_dialog(&mut hero.store, &mut sim, &mut app, spawn_root).is_none()
    );
    assert_eq!(sprites(&sim), 0, "o Cancel criou um objecto");
}

#[test]
fn o_ok_nasce_o_sprite_com_as_escolhas_e_pede_o_edit() {
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    let (mut sim, mut app) = (SimWorld::new(), PixelApp::default());
    hero.store.open_new_sprite_dialog();
    hero.store.set_new_sprite_mode(NewSpriteMode::Indexed);
    hero.store.set_new_sprite_background(NewSpriteBackground::White);
    hero.store
        .set_number_value(ph2d_editor_core::ids::NEW_SPRITE_WIDTH, 64.0);
    hero.store
        .set_number_value(ph2d_editor_core::ids::NEW_SPRITE_HEIGHT, 48.0);
    hero.store.request_new_sprite();
    let bits = object_add::born_from_dialog(&mut hero.store, &mut sim, &mut app, spawn_root)
        .expect("há pedido")
        .expect("nasce");
    let key = input::key_of(sim.world(), bits).expect("o marcador");
    let s = &app.doc(key).expect("o documento").sprite;
    assert_eq!((s.width(), s.height()), (64, 48));
    assert_eq!(s.mode(), ColorMode::Indexed);
    assert_eq!(sprites(&sim), 1);
}

#[test]
fn os_pedidos_do_painel_mudam_a_ferramenta_e_as_cores() {
    let mut app = PixelApp::default();
    let (fg, bg) = (app.fg, app.bg);
    app.aplica(PixelRequest::Tool(PixelTool::Eyedropper));
    assert_eq!(app.tool, Tool::Eyedropper);
    app.aplica(PixelRequest::Swap);
    assert_eq!((app.fg, app.bg), (bg, fg));
    app.aplica(PixelRequest::Fg([1, 2, 3, 255]));
    app.aplica(PixelRequest::Bg([4, 5, 6, 255]));
    assert_eq!((app.fg, app.bg), ([1, 2, 3, 255], [4, 5, 6, 255]));
}

/// O painel abre ao ENTRAR em Edit e fecha ao SAIR — no flanco: entre os dois, quem o fechou à mão
/// não o vê reaberto a cada quadro.
#[test]
fn o_painel_abre_no_edit_fecha_ao_sair_e_leva_a_paleta() {
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    let (mut sim, mut app) = (SimWorld::new(), PixelApp::default());
    let bits = object_add::add(object_add::PIXEL_SPRITE, &mut sim, &mut app, spawn_root)
        .expect("é desta família")
        .expect("nasce");
    let info = panel::frame(&mut app, sim.world(), &mut hero);
    assert!(!info.editing && info.palette.is_empty());
    assert!(!hero.is_panel_visible(panel::PANEL_ID));

    hero.gizmo.mode.enter(bits, ObjectMode::Edit);
    app.pede(PixelRequest::Tool(PixelTool::Eraser));
    let info = panel::frame(&mut app, sim.world(), &mut hero);
    assert!(hero.is_panel_visible(panel::PANEL_ID), "entrar em Edit abre o painel");
    assert!(info.editing);
    assert_eq!(info.tool, PixelTool::Eraser, "o pedido do quadro aplicou-se");
    assert_eq!(info.palette.len(), 32, "a DB32");

    // Fechado à mão no meio do Edit: o quadro seguinte não o reabre.
    hero.panel_visibility.insert(panel::PANEL_ID, false);
    let _ = panel::frame(&mut app, sim.world(), &mut hero);
    assert!(!hero.is_panel_visible(panel::PANEL_ID));

    hero.panel_visibility.insert(panel::PANEL_ID, true);
    let _ = hero.gizmo.mode.leave();
    let _ = panel::frame(&mut app, sim.world(), &mut hero);
    assert!(!hero.is_panel_visible(panel::PANEL_ID), "sair do Edit fecha");
}

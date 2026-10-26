//! **O painel PIXEL e o diálogo New Sprite do lado da família** (ADR-0192): o OK do diálogo nasce o
//! objecto com AS escolhas, sem OK não nasce nada; os pedidos do painel mudam a ferramenta e as
//! cores; o painel abre no flanco do Draw e fecha ao sair; o instantâneo leva a paleta do sprite.

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

fn avisos() -> ph2d_editor_core::ToastQueue {
    ph2d_editor_core::ToastQueue::new()
}

fn sprites(sim: &SimWorld) -> usize {
    ph2d_app_pixel::mode::sprites(sim.world()).len()
}

#[test]
fn o_tecto_do_dialogo_e_o_do_documento() {
    assert_eq!(NEW_SPRITE_MAX_SIDE, ph2d_pixel_core::MAX_SIDE);
    assert_eq!(
        ph2d_editor_core::pixel_panel::PIXEL_MAX_COLORS as usize,
        ph2d_pixel_core::palette::MAX_COLORS
    );
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
    hero.store
        .set_new_sprite_background(NewSpriteBackground::White);
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
    let info = panel::frame(&mut app, sim.world(), &mut hero, &mut avisos());
    assert!(!info.editing && info.palette.is_empty());
    assert!(!hero.is_panel_visible(panel::PANEL_ID));

    hero.gizmo.mode.enter(bits, ObjectMode::Draw);
    app.pede(PixelRequest::Tool(PixelTool::Eraser));
    let info = panel::frame(&mut app, sim.world(), &mut hero, &mut avisos());
    assert!(
        hero.is_panel_visible(panel::PANEL_ID),
        "entrar em Edit abre o painel"
    );
    assert!(info.editing);
    assert_eq!(
        info.tool,
        PixelTool::Eraser,
        "o pedido do quadro aplicou-se"
    );
    assert_eq!(info.palette.len(), 32, "a DB32");

    // Fechado à mão no meio do Edit: o quadro seguinte não o reabre.
    hero.panel_visibility.insert(panel::PANEL_ID, false);
    let _ = panel::frame(&mut app, sim.world(), &mut hero, &mut avisos());
    assert!(!hero.is_panel_visible(panel::PANEL_ID));

    hero.panel_visibility.insert(panel::PANEL_ID, true);
    let _ = hero.gizmo.mode.leave();
    let _ = panel::frame(&mut app, sim.world(), &mut hero, &mut avisos());
    assert!(
        !hero.is_panel_visible(panel::PANEL_ID),
        "sair do Edit fecha"
    );
}

/// ⭐ **Report do dono 25/10, defeitos 1 e 2, pelo caminho do modo**: o seletor do topo oferece
/// Object · Draw ao Pixel Sprite (como a Imagem e o Vetor); em Draw a coluna da esquerda é a do
/// Pixel — com a marca que guarda o seletor na fila e que o vetor inactivo não apaga — e o rodapé lê
/// o zoom; ao sair, só as SUAS barras se limpam.
#[test]
fn o_seletor_oferece_object_e_draw_e_a_coluna_em_draw_e_a_do_pixel() {
    use ph2d_editor_core::interaction::{BarButton, BarFace, BarFamily, ToolBars};
    use ph2d_editor_core::screens::hero::mode_drive::{self, ModeFamily};
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    let (mut sim, mut app) = (SimWorld::new(), PixelApp::default());
    let bits = object_add::add(object_add::PIXEL_SPRITE, &mut sim, &mut app, spawn_root)
        .expect("é desta família")
        .expect("nasce");
    let mut tools = ph2d_editor_core::ToolRegistry::new();
    let mut toasts = ph2d_editor_core::ToastQueue::new();
    {
        let mut fam = ph2d_app_pixel::mode::Family::new(&mut app, sim.world());
        let mut fams: [&mut dyn ModeFamily; 1] = [&mut fam];
        mode_drive::drive(
            &mut fams,
            &|_| ph2d_component_desc::ObjectKind::PixelSprite,
            &|_| "Pixel Sprite".to_owned(),
            &mut tools,
            &mut hero,
            &mut toasts,
            None,
        );
    }
    assert_eq!(
        hero.gizmo.mode.available(),
        [ObjectMode::Object, ObjectMode::Draw]
    );
    assert!(hero.store.has_mode_selector(), "sem seletor no topo");
    assert_eq!(
        ph2d_app_pixel::mode::editing(sim.world(), &hero),
        Some(bits),
        "o sprite que nasceu entrou em Draw"
    );

    let mut cam = ph2d_render::Camera2d::default();
    let window = ph2d_host::WindowSize {
        width: 800,
        height: 600,
    };
    app.frame(&mut sim, &mut cam, window, 100.0, Some(bits), spawn_root);
    let _ = panel::frame(&mut app, sim.world(), &mut hero, &mut avisos());
    // ⭐ Report do dono 25/10 (2.ª volta): com o layout *Vector* gravado o vetor estava na mão e
    // cobria o sprite em Draw — o sinal diz-lhe que largue.
    assert!(
        hero.store.family_owns_canvas(),
        "em Draw o canvas é do Pixel"
    );
    let botao = BarButton {
        id: ph2d_editor_core::NodeId(7),
        face: BarFace::Icon(ph2d_editor_core::icons::IconId::Erase),
        label: String::new(),
        on: true,
        family: BarFamily::Draw,
    };
    let barras = ToolBars {
        column: vec![vec![botao.clone()]],
        hint: "dica".to_owned(),
        ..ToolBars::default()
    };
    panel::publish_bars(&app, &mut hero, barras);
    let bars = hero.store.tool_bars().expect("a coluna do Pixel");
    assert!(bars.pixel_rail && bars.is_family_column());
    assert_eq!(bars.column, vec![vec![botao]]);
    let level = app.zoom().expect("zoom em Draw");
    assert_eq!(
        bars.readout,
        [ph2d_i18n::tr_with(
            "pixel.zoom",
            &[("n", &level.to_string())]
        )]
    );

    let _ = hero.gizmo.mode.leave();
    hero.gizmo.selection = Some(bits);
    let _ = panel::frame(&mut app, sim.world(), &mut hero, &mut avisos());
    panel::publish_bars(&app, &mut hero, ToolBars::default());
    assert!(
        hero.store.tool_bars().is_none(),
        "sair do Draw deixou a coluna"
    );
    assert!(
        hero.store.family_owns_canvas(),
        "seleccionado em Object o vetor também larga (o seletor Object · Draw fica à vista)"
    );
    hero.gizmo.selection = None;
    let _ = panel::frame(&mut app, sim.world(), &mut hero, &mut avisos());
    assert!(
        !hero.store.family_owns_canvas(),
        "sem sprite activo o vetor volta"
    );
}

//! **P6g pelo caminho real**: as pontas soltas das formas e da selecção — inclinar pelo gizmo (a
//! moldura solta e a forma viva), as teclas com Alt, a amostra «camadas visíveis» da varinha, o
//! *Paste as New Sprite* e o *Auto Select Layer* do Transform.

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, KeyAnswer};
use ph2d_app_pixel::smoke_selecao as cena;
use ph2d_app_pixel::{Tool, mode, object_add};
use ph2d_editor_core::gizmo::{GizmoView, MODEL_IDENTITY, ids as g, sprite_gizmo_hit_at};
use ph2d_editor_core::pixel_panel::{
    PixelDialogKind, PixelEditAction as E, PixelLayerAction, PixelRequest, PixelSelectAction as S,
};
use ph2d_editor_core::zones::Rect as Zona;
use ph2d_pixel_core::fill::BucketSample;
use ph2d_pixel_core::selection::{self, Wand};
use winit::keyboard::KeyCode;

use crate::edit::spawn_root;
use crate::selecao::Palco;
use crate::viva::Mesa;

/// 12 px de ecrã para fora do meio de um lado: além da alça (6 px), dentro da zona de inclinar.
const FORA_PX: f32 = 12.0;

fn alt(p: &mut Palco, k: KeyCode, ctrl: bool, shift: bool) -> Option<KeyAnswer> {
    let m = Mods {
        alt: true,
        ctrl,
        shift,
    };
    let a = input::key_with(&mut p.app, p.sim.world(), Some(p.bits), k, m);
    p.quadro();
    a
}

fn caixa() -> GizmoView {
    GizmoView {
        bbox_min_world: [100.0, 100.0],
        bbox_max_world: [300.0, 200.0],
        rotation: 0.0,
        camera_center: [400.0, 300.0],
        camera_height_world: 600.0,
        window_w: 800.0,
        window_h: 600.0,
        canvas: Zona::new(0.0, 0.0, 800.0, 600.0),
        cursor_screen: None,
        pivot_world: [200.0, 150.0],
        pivot_tool_active: false,
        model: MODEL_IDENTITY,
    }
}

#[test]
fn the_skew_zones_sit_outside_each_edge_midpoint_and_only_for_who_asks() {
    let v = caixa();
    // Câmara de 600 de altura numa janela de 600: 1 de mundo = 1 px, Y para cima — o topo da caixa
    // (y = 200 de mundo) está em y = 400 de ecrã, a base em 500.
    let topo = (200.0, 400.0 - FORA_PX);
    assert_eq!(
        sprite_gizmo_hit_at(&v, true, true, topo.0, topo.1),
        Some(g::GIZMO_SKEW_T)
    );
    assert_eq!(
        sprite_gizmo_hit_at(&v, true, true, 300.0 + FORA_PX, 450.0),
        Some(g::GIZMO_SKEW_R)
    );
    assert_eq!(
        sprite_gizmo_hit_at(&v, true, true, 200.0, 500.0 + FORA_PX),
        Some(g::GIZMO_SKEW_B)
    );
    assert_eq!(
        sprite_gizmo_hit_at(&v, true, true, 100.0 - FORA_PX, 450.0),
        Some(g::GIZMO_SKEW_L)
    );
    assert_eq!(
        sprite_gizmo_hit_at(&v, true, true, 200.0, 400.0),
        Some(g::GIZMO_HANDLE_T),
        "a alça do lado ganha à zona"
    );
    assert_eq!(
        sprite_gizmo_hit_at(&v, true, false, topo.0, topo.1),
        None,
        "o gizmo dos sprites da casa não inclina"
    );
}

#[test]
fn dragging_outside_an_edge_skews_the_floating_frame_as_one_step() {
    let mut p = Palco::limpo();
    let r = cena::ORIGINAL_AT;
    p.ferramenta(Tool::RectMarquee);
    p.arrasta(r, (r.0 + 13, r.1 + 13), Mods::default());
    p.tecla(KeyCode::KeyT, true, false);
    let n = p.passos();
    let meio = (f64::from(r.0) + 7.0, f64::from(r.1));
    let k = p.canto(meio.0, meio.1);
    let a = (k.0, k.1 - FORA_PX);
    let d = p.canto(meio.0 + 5.0, meio.1);
    p.arrasta_ecra(a, (d.0, a.1), Mods::default());
    let t = p.quadro().transform.expect("solta");
    assert!(t.skew != 0, "o arrasto fora do meio do topo inclina: {t:?}");
    assert_eq!((t.w, t.h, t.angle), (14, 14, 0), "e só inclina: {t:?}");
    p.tecla(KeyCode::Enter, false, false);
    assert_eq!(p.passos(), n + 1, "assentar é UM passo");
}

#[test]
fn dragging_outside_an_edge_skews_the_live_shape() {
    let mut m = Mesa::nova();
    m.desenha(Tool::RECTANGLE, (4, 4), (13, 13));
    let k = m.ponto(9.0, 4.0);
    let d = m.ponto(14.0, 4.0);
    m.arrasta((k.0, k.1 - FORA_PX), (d.0, k.1 - FORA_PX));
    let v = m.app.live_shape().expect("continua viva");
    assert!(v.info().skew != 0, "a forma viva inclina: {:?}", v.info());
    assert_eq!(v.info().angle, 0);
}

#[test]
fn alt_reaches_the_family_and_ctrl_alt_i_opens_sprite_size() {
    let mut p = Palco::novo();
    assert!(alt(&mut p, KeyCode::KeyI, true, false).is_some());
    assert_eq!(
        p.hero.store.pixel_edit_dialog().kind,
        PixelDialogKind::SpriteSize,
        "Ctrl+Alt+I = Sprite Size (o Inverse é Ctrl+Shift+I)"
    );
    p.hero.store.close_context_menu();
    let quadros = p.sprite().frames().len();
    assert!(alt(&mut p, KeyCode::KeyN, false, false).is_some());
    assert_eq!(p.sprite().frames().len(), quadros + 1, "Alt+N = New Frame");
    let camadas = p.sprite().layers().len();
    assert!(alt(&mut p, KeyCode::KeyN, false, true).is_some());
    let l = p.sprite().layers();
    assert_eq!(l.len(), camadas + 1, "Alt+Shift+N = New Group");
    assert!(l.iter().any(|l| l.is_group()));
    let tool = p.app.tool;
    assert!(
        alt(&mut p, KeyCode::KeyE, false, false).is_none(),
        "Alt+E não é o E"
    );
    assert_eq!(p.app.tool, tool);
}

#[test]
fn the_wand_samples_the_visible_layers_when_asked() {
    let mut p = Palco::limpo();
    p.tecla(KeyCode::KeyN, false, true);
    let s = p.sprite().clone();
    let d = p.app.doc(p.key).expect("doc");
    let chave = (d.layer, d.frame);
    p.ferramenta(Tool::MagicWand);
    p.press(cena::WAND.0, Mods::default());
    p.release();
    let camada = selection::wand(&s, chave, cena::WAND.0, Wand::default());
    assert_eq!(
        p.mascara(),
        Some(camada.clone()),
        "por omissão, a camada activa (vazia)"
    );
    p.opcoes(|o| o.wand_sample_visible = true);
    p.pede(PixelRequest::Select(S::Deselect));
    p.press(cena::WAND.0, Mods::default());
    p.release();
    let opt = Wand {
        sample: BucketSample::Visible,
        ..Wand::default()
    };
    let visivel = selection::wand(&s, chave, cena::WAND.0, opt);
    assert_ne!(visivel, camada, "a figura verde só se vê nas visíveis");
    assert_eq!(p.mascara(), Some(visivel));
}

#[test]
fn paste_as_new_sprite_is_born_with_the_clip_and_leaves_this_document_alone() {
    let mut p = Palco::limpo();
    let r = cena::ORIGINAL_AT;
    p.ferramenta(Tool::RectMarquee);
    p.arrasta(r, (r.0 + 13, r.1 + 13), Mods::default());
    p.pede(PixelRequest::Edit(E::Copy));
    let (antes, n) = (p.sprite().clone(), p.passos());
    p.pede(PixelRequest::LayerAction(PixelLayerAction::PasteAsSprite));
    assert_eq!((*p.sprite() == antes, p.passos()), (true, n));
    let bits = object_add::born_from_paste(&mut p.sim, &mut p.app, spawn_root)
        .expect("pedido")
        .expect("nasce");
    let key = mode::sprites(p.sim.world())
        .into_iter()
        .find(|(b, _)| *b == bits)
        .map(|(_, k)| k)
        .expect("o objecto novo tem documento");
    let novo = &p.app.doc(key).expect("doc").sprite;
    assert_eq!((novo.width(), novo.height()), (14, 14));
    let (l0, f0) = (novo.layers()[0].id, novo.frames()[0].id);
    for y in 0..14 {
        for x in 0..14 {
            let quer = p.rgba(r.0 + x, r.1 + y);
            let tem = novo.rgba_of_value(novo.value_at((l0, f0), x, y));
            assert_eq!(tem, quer, "({x}, {y})");
        }
    }
    assert!(
        object_add::born_from_paste(&mut p.sim, &mut p.app, spawn_root).is_none(),
        "um pedido, um objecto"
    );
}

#[test]
fn auto_select_layer_picks_the_top_painted_layer_under_the_press() {
    let mut p = Palco::limpo();
    let original = p.app.doc(p.key).expect("doc").layer;
    p.tecla(KeyCode::KeyN, false, true);
    let vazia = p.app.doc(p.key).expect("doc").layer;
    assert_ne!(vazia, original);
    p.ferramenta(Tool::Transform);
    p.tecla(KeyCode::Escape, false, false);
    p.opcoes(|o| o.auto_select_layer = true);
    let r = cena::ORIGINAL_AT;
    p.press((r.0 + 7, r.1 + 7), Mods::default());
    p.release();
    assert_eq!(
        p.app.doc(p.key).expect("doc").layer,
        original,
        "premir na figura escolhe a camada dela"
    );
}

#[test]
fn scene_15_shows_a_skewed_frame_two_layers_a_clip_and_the_flyout_in_one_row() {
    use ph2d_app_pixel::smoke_pontas as c15;
    let mut app = ph2d_app_pixel::PixelApp::default();
    let mut sim = ph2d_ecs::SimWorld::new();
    assert!(ph2d_app_pixel::smoke::stage(
        &mut app, &mut sim, 15, spawn_root
    ));
    let (bits, key) = mode::sprites(sim.world())[0];
    let mut hero = ph2d_editor_core::screens::hero::HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo
        .mode
        .enter(bits, ph2d_editor_core::object_mode::ObjectMode::Draw);
    let mut cam = ph2d_render::Camera2d::default();
    let (w, ppm) = (crate::edit::WINDOW, crate::edit::PPM);
    app.frame(&mut sim, &mut cam, w, ppm, Some(bits), spawn_root);
    let mut toasts = ph2d_editor_core::ToastQueue::new();
    let info = ph2d_app_pixel::panel::frame(&mut app, sim.world(), &mut hero, &mut toasts);
    let t = info.transform.expect("a moldura solta");
    assert_eq!((t.skew, t.angle), (c15::SKEW, 0), "{t:?}");
    assert!(info.has_clip, "o «F» copiado acende o Paste as New Sprite");
    assert_eq!(
        info.tool,
        ph2d_editor_core::pixel_panel::PixelTool::Transform
    );
    assert!(info.options.auto_select_layer && info.options.wand_sample_visible);
    let s = &app.doc(key).expect("doc").sprite;
    assert_eq!(s.layers().len(), 2);
    assert_eq!(s.layers()[0].name, c15::NAME);
    assert_eq!(
        app.doc(key).expect("doc").layer,
        s.layers()[0].id,
        "Figures activa"
    );
    // As 4 numa linha: a lei é `float_bar::flyout_cols` (o teste dela, no editor-core).
    let aberto = ph2d_editor_core::pixel_panel::PixelToolGroup::Select;
    assert_eq!(
        info.groups.open,
        Some(aberto),
        "o flyout das selecções aberto"
    );
}

#[test]
fn save_to_msk_and_load_it_back_restores_the_selection_as_one_step() {
    let mut p = Palco::limpo();
    assert!(p.app.msk_de(p.key).is_none(), "sem selecção não há ficheiro");
    p.ferramenta(Tool::EllipseMarquee);
    p.arrasta((5, 7), (30, 20), Mods::default());
    let antes = p.mascara().expect("selecção");
    let bytes = p.app.msk_de(p.key).expect("o .msk");
    assert_eq!(
        ph2d_pixel_io::msk::read_msk(&bytes)
            .expect("lê-se")
            .to_selection(p.sprite().width(), p.sprite().height()),
        antes
    );
    p.pede(PixelRequest::Select(S::Deselect));
    let n = p.passos();
    p.app.carrega_msk(p.key, &bytes);
    p.quadro();
    assert_eq!(p.mascara(), Some(antes), "o Load devolve a selecção");
    assert_eq!(p.passos(), n + 1, "UM passo");
    p.app.carrega_msk(p.key, b"lixo");
    p.quadro();
    assert_eq!(p.passos(), n + 1, "um ficheiro estragado não mexe");
    assert!(!p.toasts.is_empty(), "e diz porquê");
}

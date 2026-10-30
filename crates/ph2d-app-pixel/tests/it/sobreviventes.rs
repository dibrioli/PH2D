//! **P6h — os sobreviventes das mutações da linha** (P6d–P6g): cada teste aqui fecha uma mutação
//! que passava verde. As equivalentes e as que ficaram por fechar estão no handoff da P6h.

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::formas::cells;
use ph2d_app_pixel::input::{self, KeyAnswer};
use ph2d_app_pixel::solta::{gizmo_of, hit_of};
use ph2d_app_pixel::{PixelApp, Tool};
use ph2d_editor_core::gizmo::{
    GizmoView, MODEL_IDENTITY, ids as g, paint_sprite_gizmo_skewing, sprite_gizmo_hit_at,
};
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::pixel_panel::{PixelRequest, PixelSelectAction as S, PixelShape};
use ph2d_editor_core::zones::Rect as Zona;
use ph2d_pixel_core::affine::{Affine, Handle, Pivot};
use ph2d_tokens::Theme;
use ph2d_tool_vector::shape_handles_face::Glyph;
use ph2d_vector::VectorScene;
use winit::keyboard::KeyCode;

use crate::edit::{PPM, WINDOW, spawn_root};
use crate::selecao::Palco;
use crate::viva::Mesa;

const CLEAR: [u8; 4] = [0, 0, 0, 0];
const WHITE: [u8; 4] = [255, 255, 255, 255];

/// Uma caixa de mundo `[x0, x0+w] × [y0, y0+h]` com 1 de mundo = 1 px de ecrã (ecrã y = 600 − y).
fn caixa(x0: f32, y0: f32, w: f32, h: f32) -> GizmoView {
    GizmoView {
        bbox_min_world: [x0, y0],
        bbox_max_world: [x0 + w, y0 + h],
        rotation: 0.0,
        camera_center: [400.0, 300.0],
        camera_height_world: 600.0,
        window_w: 800.0,
        window_h: 600.0,
        canvas: Zona::new(0.0, 0.0, 800.0, 600.0),
        cursor_screen: None,
        pivot_world: [x0 + w * 0.5, y0 + h * 0.5],
        pivot_tool_active: false,
        model: MODEL_IDENTITY,
    }
}

fn tecla(m: &mut Mesa, k: KeyCode, ctrl: bool, shift: bool) -> Option<KeyAnswer> {
    input::key(&mut m.app, m.sim.world(), Some(m.bits), k, ctrl, shift)
}

fn raio(m: &Mesa) -> f64 {
    m.app.options.shapes.get(PixelShape::RECTANGLE).get(0)
}

#[test]
fn on_a_tiny_box_where_the_skew_and_rotate_zones_touch_the_rotate_wins() {
    // Caixa de 4 px: o TL está em (100, 496) de ecrã. A zona de rodar do TL é [74, 94] × [470, 490];
    // a de inclinar do topo é [92, 112] × [470, 490] — tocam-se em x ∈ [92, 94].
    let v = caixa(100.0, 100.0, 4.0, 4.0);
    assert_eq!(
        sprite_gizmo_hit_at(&v, true, true, 93.0, 480.0),
        Some(g::GIZMO_ROTATE_TL),
        "onde as zonas se tocam, roda-se"
    );
    assert_eq!(
        sprite_gizmo_hit_at(&v, true, true, 102.0, 480.0),
        Some(g::GIZMO_SKEW_T),
        "fora do toque, inclina"
    );
}

#[test]
fn the_edge_handle_paints_as_a_diamond_with_the_cursor_in_its_skew_zone() {
    let pinta = |cursor: (f32, f32)| {
        let mut v = caixa(100.0, 100.0, 200.0, 100.0);
        v.cursor_screen = Some(cursor);
        let mut scene = VectorScene::new();
        let mut hits = HitIndex::new();
        paint_sprite_gizmo_skewing(&mut scene, &v, Theme::default(), &mut hits, true);
        scene.inner().encoding().path_data.clone()
    };
    // O topo da caixa está em y = 400 de ecrã; 12 px acima, a zona de inclinar do topo.
    let fora = pinta((700.0, 50.0));
    let na_zona = pinta((200.0, 388.0));
    assert_ne!(na_zona, fora, "a alça do topo vira losango");
    assert_eq!(pinta((710.0, 60.0)), fora, "longe de tudo, a mesma pintura");
}

#[test]
fn each_skew_zone_of_the_frame_maps_to_its_own_edge() {
    // A moldura de 10 × 10 ao ecrã com escala 20 e Y para baixo (imagem → ecrã).
    let to_screen = ph2d_vector::Affine::new([20.0, 0.0, 0.0, 20.0, 200.0, 100.0]);
    let pivot = Pivot { u: 5.0, v: 5.0 };
    let view = gizmo_of(
        Affine::at(0.0, 0.0),
        (10, 10),
        pivot,
        to_screen,
        (800.0, 600.0),
        None,
    );
    // A imagem ocupa [200, 400] × [100, 300] no ecrã. Cada zona de inclinar (12 px para fora do meio
    // de um lado) dá a alça do MESMO lado que a alça de escala sobre esse meio.
    let casos = [
        ((300.0, 100.0), (300.0, 88.0)),
        ((300.0, 300.0), (300.0, 312.0)),
        ((200.0, 200.0), (188.0, 200.0)),
        ((400.0, 200.0), (412.0, 200.0)),
    ];
    let mut lados = Vec::new();
    for (meio, fora) in casos {
        let Some(Handle::Scale(a, b)) = hit_of(&view, meio) else {
            panic!("a alça de escala em {meio:?}: {:?}", hit_of(&view, meio));
        };
        assert_eq!(hit_of(&view, fora), Some(Handle::Skew(a, b)), "{fora:?}");
        lados.push((a, b));
    }
    lados.sort_unstable();
    assert_eq!(lados, [(0, 1), (1, 0), (1, 2), (2, 1)], "os quatro lados");
}

/// Um Palco com uma segunda camada (de cima) pintada na figura, Transform activo e Auto Select.
fn duas_pintadas() -> (
    Palco,
    ph2d_pixel_core::LayerId,
    ph2d_pixel_core::LayerId,
    (i32, i32),
) {
    let mut p = Palco::limpo();
    let original = p.app.doc(p.key).expect("doc").layer;
    p.tecla(KeyCode::KeyN, false, true);
    let cima = p.app.doc(p.key).expect("doc").layer;
    assert_ne!(cima, original);
    let r = ph2d_app_pixel::smoke_selecao::ORIGINAL_AT;
    let c = (r.0 + 7, r.1 + 7);
    p.ferramenta(Tool::Pencil);
    p.press(c, Mods::default());
    p.release();
    assert_ne!(p.rgba(c.0, c.1)[3], 0, "a de cima pintada em {c:?}");
    p.ferramenta(Tool::Transform);
    p.tecla(KeyCode::Escape, false, false);
    p.opcoes(|o| o.auto_select_layer = true);
    (p, original, cima, c)
}

#[test]
fn auto_select_layer_picks_the_upper_of_two_painted_layers() {
    let (mut p, _, cima, c) = duas_pintadas();
    p.press(c, Mods::default());
    p.release();
    assert_eq!(p.app.doc(p.key).expect("doc").layer, cima, "a de cima");
}

#[test]
fn auto_select_layer_skips_a_hidden_layer() {
    let (mut p, original, cima, c) = duas_pintadas();
    p.pede(PixelRequest::LayerVisible(cima.0));
    assert!(!p.sprite().is_shown(cima), "escondida");
    p.press(c, Mods::default());
    p.release();
    assert_eq!(
        p.app.doc(p.key).expect("doc").layer,
        original,
        "a escondida não se escolhe"
    );
}

#[test]
fn ctrl_z_mid_handle_drag_is_swallowed() {
    let mut m = Mesa::nova();
    m.desenha(Tool::RECTANGLE, (4, 4), (24, 20));
    let alca = m.alca(Glyph::Corner);
    m.press(alca);
    let c = crate::edit::canvas(&m.sim, &m.cam, m.bits);
    let dentro = m.celula((12, 12));
    assert!(input::moved(&mut m.app, &c, dentro, Mods::default()));
    let r = raio(&m);
    assert!(r > 0.0);
    assert!(tecla(&mut m, KeyCode::KeyZ, true, false).is_some());
    assert!(m.app.live_shape().is_some(), "a forma continua viva");
    assert_eq!(raio(&m), r, "o Ctrl+Z a meio do arrasto não mexe");
}

#[test]
fn ctrl_shift_z_with_a_live_shape_redoes_its_adjustment_before_the_document() {
    let mut m = Mesa::nova();
    m.desenha(Tool::RECTANGLE, (4, 4), (12, 12));
    m.tecla(KeyCode::Enter);
    assert_eq!(m.v((4, 4)), WHITE);
    assert!(tecla(&mut m, KeyCode::KeyZ, true, false).is_some());
    assert_eq!(m.v((4, 4)), CLEAR, "o documento tem um refazer");
    m.desenha(Tool::RECTANGLE, (14, 14), (30, 28));
    let alca = m.alca(Glyph::Corner);
    let dentro = m.celula((20, 20));
    m.arrasta(alca, dentro);
    let r = raio(&m);
    assert!(r > 0.0);
    assert!(tecla(&mut m, KeyCode::KeyZ, true, false).is_some());
    assert_eq!(raio(&m), 0.0, "o Ctrl+Z desfez o ajuste da viva");
    assert!(tecla(&mut m, KeyCode::KeyZ, true, true).is_some());
    assert_eq!(raio(&m), r, "o Ctrl+Shift+Z refaz o ajuste da viva");
    assert_eq!(m.v((4, 4)), CLEAR, "e não o documento");
    assert!(m.app.live_shape().is_some());
}

#[test]
fn an_open_shape_stays_open_in_cells() {
    let app = PixelApp::default();
    let abertas: Vec<_> = PixelShape::ALL
        .into_iter()
        // A linha fechada refaz o próprio traço: só as curvas abertas mostram o fecho.
        .filter(|s| !s.kind().is_closed() && s.kind() != ph2d_vec_scene::ShapeKind::Line)
        .collect();
    assert!(!abertas.is_empty());
    for s in abertas {
        let v = app.options.shapes.get(s);
        let (contorno, dentro) = cells(s, &v, (2, 2), (40, 30), true);
        assert!(dentro.is_empty(), "{s:?}: aberta não tem interior");
        let set: std::collections::BTreeSet<_> = contorno.iter().copied().collect();
        // Uma curva aberta tem pontas: células com UM vizinho só. Fechá-la tira-as.
        let pontas = set
            .iter()
            .filter(|&&(x, y)| {
                let viz = (-1..=1)
                    .flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
                    .filter(|&(dx, dy)| (dx, dy) != (0, 0) && set.contains(&(x + dx, y + dy)))
                    .count();
                viz == 1
            })
            .count();
        assert!(pontas > 0, "{:?}: a curva aberta tem pontas", s.kind());
    }
}

#[test]
fn esc_mid_box_drag_puts_the_box_back() {
    let mut m = Mesa::nova();
    m.desenha(Tool::RECTANGLE, (4, 4), (13, 13));
    let antes = m.app.live_shape().expect("viva").info();
    let (de, para) = (m.ponto(14.0, 14.0), m.ponto(25.0, 25.0));
    m.press(de);
    let c = crate::edit::canvas(&m.sim, &m.cam, m.bits);
    assert!(input::moved(&mut m.app, &c, para, Mods::default()));
    assert_ne!(
        m.app.live_shape().expect("viva").info(),
        antes,
        "a caixa cresceu"
    );
    m.tecla(KeyCode::Escape);
    let v = m.app.live_shape().expect("o Esc a meio não larga");
    assert_eq!(v.info(), antes, "a caixa volta à do premir");
}

#[test]
fn a_document_request_settles_the_live_shape_first() {
    let mut m = Mesa::nova();
    m.desenha(Tool::RECTANGLE, (4, 4), (12, 12));
    m.app.aplica(PixelRequest::Select(S::All));
    assert!(m.app.live_shape().is_none(), "o pedido assentou-a");
    assert_eq!(m.v((4, 4)), WHITE);
}

#[test]
fn leaving_draw_settles_the_live_shape() {
    let mut m = Mesa::nova();
    m.desenha(Tool::RECTANGLE, (4, 4), (12, 12));
    m.app
        .frame(&mut m.sim, &mut m.cam, WINDOW, PPM, None, spawn_root);
    assert!(m.app.live_shape().is_none());
    assert_eq!(m.v((4, 4)), WHITE, "sair do Draw assenta");
}

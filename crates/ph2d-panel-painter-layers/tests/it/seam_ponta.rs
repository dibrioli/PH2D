//! **As linhas da PONTA pelo ponteiro** (L9 · L18, `docs/Painter/Pinceis/03_MOTORES.md` §4): a ponta de píxel no
//! Shape (só no Digital) e a lei do grão no Grain (com um Grain armado). Cada linha é accionada como o artista a
//! acciona (o chip e a opção, dígitos na caixa) e o efeito lê-se no TOOL (`BrushSettings::ponta`).

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::{LINHAS_DA_PONTA, LinhaDaPonta, PaintMedia, PainterTool, ids};
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 8000.0)
}

fn painted(
    tool: &PainterTool,
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
) -> Vec<(NodeId, Rect)> {
    set_current_brush(Some(tool.brush_settings()));
    host.paint::<PainterLayersPanel>(st, viewport())
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

fn clica(
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
    tool: &mut PainterTool,
    r: Rect,
) {
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<PainterLayersPanel>(st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
}

/// O painel pintado com as secções Shape e Grain ABERTAS (se nascem recolhidas, o clique no cabeçalho abre-as).
fn aberto(t: &mut PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let mut rects = painted(t, &mut host, &mut st);
    for (cab, dentro) in [
        (ids::PAINTER_SHAPE_SECTION, ids::PAINTER_SHAPE_KIND),
        (
            ids::PAINTER_BRUSH_TEXTURE_SECTION,
            ids::PAINTER_BRUSH_TEXTURE_KIND,
        ),
    ] {
        if rect_of(&rects, dentro).is_none()
            && let Some(r) = rect_of(&rects, cab)
        {
            clica(&mut host, &mut st, &mut PainterTool::default(), r);
            host.settle_section_folds();
            rects = painted(t, &mut host, &mut st);
        }
    }
    assert!(
        rect_of(&rects, ids::PAINTER_SHAPE_SECTION).is_some(),
        "a secção Shape"
    );
    (host, st, rects)
}

fn indice(l: LinhaDaPonta) -> usize {
    LINHAS_DA_PONTA.iter().position(|x| *x == l).unwrap()
}

/// Abre o menu da linha `l` e escolhe a opção `k`.
fn escolhe(t: &mut PainterTool, l: LinhaDaPonta, k: u8) {
    let i = indice(l);
    let (mut host, mut st, rects) = aberto(t);
    let chip = rect_of(&rects, ids::PAINTER_PONTA_FIELDS[i])
        .unwrap_or_else(|| panic!("o chip {}", l.chave()));
    clica(&mut host, &mut st, t, chip);
    let rects = painted(t, &mut host, &mut st);
    let op = rect_of(&rects, ids::painter_ponta_option_id(i, k)).expect("a opção do menu");
    clica(&mut host, &mut st, t, op);
}

/// ⭐ A ponta de píxel está na secção Shape do Digital — e só dele; escolher «Pixel-perfect» chega ao pincel.
#[test]
fn a_ponta_de_pixel_escolhe_se_no_menu_do_digital() {
    let i = indice(LinhaDaPonta::Pixel);
    for m in [
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        let mut t = PainterTool::default();
        t.set_paint_media(m);
        let (_h, _s, rects) = aberto(&mut t);
        assert!(
            rect_of(&rects, ids::PAINTER_PONTA_FIELDS[i]).is_none(),
            "{m:?} pinta a ponta de píxel"
        );
    }
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::Digital);
    escolhe(&mut t, LinhaDaPonta::Pixel, 2);
    assert_eq!(t.brush_settings().ponta[i], 2.0);
}

/// ⭐ A lei do grão só aparece com um Grain armado; o menu do modo e a caixa do jitter chegam ao pincel.
#[test]
fn a_lei_do_grao_chega_ao_pincel() {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::Digital);
    let (g, j) = (
        indice(LinhaDaPonta::GrainMode),
        indice(LinhaDaPonta::DepthJitter),
    );
    {
        let (_h, _s, rects) = aberto(&mut t);
        assert!(
            rect_of(&rects, ids::PAINTER_PONTA_FIELDS[g]).is_none(),
            "sem Grain não há lei do grão"
        );
    }
    t.set_brush_texture_kind(1);
    escolhe(&mut t, LinhaDaPonta::GrainMode, 4);
    assert_eq!(t.brush_settings().ponta[g], 4.0);
    escolhe(&mut t, LinhaDaPonta::DepthPressure, 1);
    assert_eq!(
        t.brush_settings().ponta[indice(LinhaDaPonta::DepthPressure)],
        1.0
    );
    let (mut host, mut st, _) = aberto(&mut t);
    for ev in host.type_into_number(ids::PAINTER_PONTA_FIELDS[j], "0.75") {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            t.handle_panel_event(pe);
        }
    }
    assert_eq!(t.brush_settings().ponta[j], 0.75);
}

/// Uma imagem 8×8 opaca de cor `c`.
fn img(c: u8) -> Vec<u8> {
    (0..64).flat_map(|_| [c, c, c, 255]).collect()
}

/// ⭐ **A ponta de N imagens na secção Shape** (L19): com uma imagem, «Add image» pede o seletor à shell; com três,
/// o menu da ordem e um peso por imagem (no sorteio) chegam ao pincel, e «Clear images» volta à imagem única.
#[test]
fn a_ponta_de_n_imagens_pelo_ponteiro() {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_shape_image_rgba(&img(200), 8, 8, None);
    let (mut host, mut st, rects) = aberto(&mut t);
    let add = rect_of(&rects, ids::PAINTER_SHAPE_HOSE_ADD).expect("o botão Add image");
    assert!(
        rect_of(
            &rects,
            ids::PAINTER_PONTA_FIELDS[indice(LinhaDaPonta::HoseMode)]
        )
        .is_none()
    );
    clica(&mut host, &mut st, &mut t, add);
    assert!(
        t.take_brush_shape_hose_request(),
        "o Add image não pediu o seletor"
    );
    t.add_brush_shape_hose_image_rgba(&img(120), 8, 8);
    t.add_brush_shape_hose_image_rgba(&img(60), 8, 8);
    assert_eq!(t.brush_shape_hose_count(), 3);
    escolhe(&mut t, LinhaDaPonta::HoseMode, 2);
    assert_eq!(
        t.brush_settings().ponta[indice(LinhaDaPonta::HoseMode)],
        2.0
    );
    escolhe(&mut t, LinhaDaPonta::HoseMode, 0);
    let (mut host, mut st, rects) = aberto(&mut t);
    let p3 = ids::PAINTER_PONTA_FIELDS[indice(LinhaDaPonta::Peso(2))];
    assert!(rect_of(&rects, p3).is_some(), "o peso da 3.ª imagem");
    assert!(
        rect_of(
            &rects,
            ids::PAINTER_PONTA_FIELDS[indice(LinhaDaPonta::Peso(3))]
        )
        .is_none()
    );
    for ev in host.type_into_number(p3, "6") {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            t.handle_panel_event(pe);
        }
    }
    assert_eq!(t.brush_settings().ponta[indice(LinhaDaPonta::Peso(2))], 6.0);
    let (mut host, mut st, rects) = aberto(&mut t);
    let limpa = rect_of(&rects, ids::PAINTER_SHAPE_HOSE_CLEAR).expect("o botão Clear images");
    clica(&mut host, &mut st, &mut t, limpa);
    assert_eq!(t.brush_shape_hose_count(), 1);
}

/// ⭐ **O dual brush pelo ponteiro** (L6): o menu «Dual brush» na secção Shape arma a 2.ª ponta (as linhas dela só
/// aparecem armada), a forma `Image` mostra «Dual image», que pede o seletor à shell, e a caixa do tamanho chega ao
/// pincel; «Off» desliga-a (o pincel volta a `None`).
#[test]
fn o_dual_brush_pelo_ponteiro() {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::Digital);
    let tamanho = ids::PAINTER_PONTA_FIELDS[indice(LinhaDaPonta::DualSize)];
    {
        let (_h, _s, rects) = aberto(&mut t);
        assert!(
            rect_of(
                &rects,
                ids::PAINTER_PONTA_FIELDS[indice(LinhaDaPonta::DualMode)]
            )
            .is_some()
        );
        assert!(
            rect_of(&rects, tamanho).is_none(),
            "a 2.ª ponta desligada não mostra as linhas dela"
        );
    }
    escolhe(&mut t, LinhaDaPonta::DualMode, 4);
    assert_eq!(
        t.brush_settings().ponta[indice(LinhaDaPonta::DualMode)],
        4.0
    );
    let imagem = ph2d_tool_painter::TextureKind::Image.to_u8();
    escolhe(&mut t, LinhaDaPonta::DualTip, imagem);
    let (mut host, mut st, rects) = aberto(&mut t);
    let botao = rect_of(&rects, ids::PAINTER_DUAL_IMAGE).expect("o botão Dual image");
    clica(&mut host, &mut st, &mut t, botao);
    assert!(
        t.take_brush_dual_image_request(),
        "o Dual image não pediu o seletor"
    );
    let (mut host, mut st, _) = aberto(&mut t);
    for ev in host.type_into_number(tamanho, "0.3") {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            t.handle_panel_event(pe);
        }
    }
    assert_eq!(
        t.brush_settings().ponta[indice(LinhaDaPonta::DualSize)],
        0.3
    );
    escolhe(&mut t, LinhaDaPonta::DualMode, 0);
    assert_eq!(
        t.brush_settings().ponta[indice(LinhaDaPonta::DualMode)],
        0.0
    );
}

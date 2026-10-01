//! ⭐⭐⭐ **As secções do corpo do pincel abrem o menu de TEMA e ARRASTAM pela pega** — o gesto REAL,
//! pelo despachante, com os rects que a pintura registou (ordem do dono, 2026-09-30: *«siga com os
//! outros painéis»*). O irmão deste gate vive no Vector; aqui ele mede as três espécies que o
//! Painter tem e o Vector não: a secção que se ARRASTA, a FIXA (muda de tema e não tem pega) e a
//! ANINHADA (dentro de outra — nem uma coisa nem outra).

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{PaintMedia, PainterTool};
use ph2d_ui_testkit::MockPanelHost;

const SEC: u128 = 1_000_000_000;

/// Alto de propósito: o corpo do pincel transborda depressa, e o gate quer as secções à vista.
fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 700.0, 4000.0)
}

/// As secções do corpo que se arrastam, pela ordem natural.
const MOVEIS: [NodeId; 8] = [
    pids::PAINTER_BRUSH_RANDOMIZE_SECTION,
    pids::PAINTER_SHAPE_SECTION,
    pids::PAINTER_SHAPE_RAMP_SECTION,
    pids::PAINTER_WATERCOLOR_PAPER_SECTION,
    pids::PAINTER_BRUSH_TEXTURE_SECTION,
    pids::PAINTER_BRUSH_STROKE_SECTION,
    pids::PAINTER_BRUSH_SYMMETRY_SECTION,
    pids::PAINTER_BRUSH_TILING_SECTION,
];

fn pointer(kind: PointerKind, button: PointerButton, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

fn pintado(tool: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(tool.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    (host, st, rects)
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .rev()
        .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

/// As móveis à vista, pela ordem pintada.
fn moveis_a_vista(rects: &[(NodeId, Rect)]) -> Vec<(NodeId, Rect)> {
    let mut v: Vec<(NodeId, Rect)> = MOVEIS
        .iter()
        .filter_map(|&id| rect_of(rects, id).map(|r| (id, r)))
        .collect();
    v.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));
    v
}

fn botao_direito(host: &mut MockPanelHost, cab: Rect) -> Option<ContextMenuKind> {
    // ⚠️ Num ponto do título à ESQUERDA da pega e do botão de repor — os dois são outros alvos.
    let (x, y) = (cab.x + cab.w * 0.25, cab.y + cab.h * 0.5);
    let _ = host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Secondary,
        x,
        y,
        SEC,
    ));
    host.store().context_menu().map(|m| m.kind)
}

/// ⭐⭐ **O botão direito no título de uma secção que se ARRASTA abre o menu de tema dela, e ela tem
/// pega.** *Mutação: o `paint_collapsible_section` a registar pelo `hit.register` de antes ⇒ o
/// menu aberto é o de notas e a pega não existe no índice.*
#[test]
fn uma_seccao_movel_tem_menu_de_tema_e_pega() {
    let (mut host, _, rects) = pintado(&PainterTool::default());
    let vista = moveis_a_vista(&rects);
    assert!(
        vista.len() >= 3,
        "fixtura: o corpo tem de pintar três secções móveis ou mais (pintou {})",
        vista.len()
    );
    let (alvo, cab) = vista[1];
    assert!(
        rect_of(&rects, ph2d_editor_core::ids::grip_de(alvo)).is_some(),
        "a secção móvel {alvo:?} não registou a pega"
    );
    let menu = botao_direito(&mut host, cab);
    assert!(
        matches!(menu, Some(ContextMenuKind::SectionOutline { section }) if section == alvo),
        "o botão direito no título abriu {menu:?} em vez do menu de tema da secção"
    );
}

/// ⭐⭐ **A secção do MEIO é FIXA: muda de tema e não tem pega.** O controlo é a móvel ao lado, que
/// tem as duas coisas. *Mutação: a Watercolor nas `SECCOES_MOVEIS` ⇒ ganha pega; fora das duas
/// listas ⇒ perde o menu.*
#[test]
fn a_seccao_do_meio_e_fixa_tem_tema_e_nao_tem_pega() {
    let mut tool = PainterTool::default();
    tool.set_paint_media(PaintMedia::Watercolor);
    let (mut host, _, rects) = pintado(&tool);
    let cab = rect_of(&rects, pids::PAINTER_WATERCOLOR_SECTION)
        .expect("fixtura: com o meio Watercolor a secção dele é pintada");
    assert!(
        rect_of(
            &rects,
            ph2d_editor_core::ids::grip_de(pids::PAINTER_WATERCOLOR_SECTION)
        )
        .is_none(),
        "a secção do meio ganhou uma pega — ela é FIXA"
    );
    let menu = botao_direito(&mut host, cab);
    assert!(
        matches!(menu, Some(ContextMenuKind::SectionOutline { section })
            if section == pids::PAINTER_WATERCOLOR_SECTION),
        "o botão direito no título da secção do meio abriu {menu:?} em vez do menu de tema"
    );
}

/// ⭐ **Uma secção ANINHADA (a rampa de cor dentro do Grain) não é uma secção do corpo** — nem pega
/// nem menu de tema: o plano não lhe aplicaria nenhum dos dois. *Mutação: a rampa nas
/// `SECCOES_MOVEIS` ⇒ pega pintada e menu aberto sobre um controlo morto.*
#[test]
fn uma_seccao_aninhada_nao_entra_no_livro() {
    // ⚠️ A rampa só é pintada com um grão escolhido — sem ele o Grain fecha no chip do tipo.
    let mut brush = PainterTool::default().brush_settings();
    brush.texture_kind = ph2d_tool_painter::TextureKind::Noise.to_u8();
    set_current_brush(Some(brush));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let rects = host.paint::<PainterLayersPanel>(&mut PainterLayersPanelState, viewport());
    let Some(cab) = rect_of(&rects, pids::PAINTER_BRUSH_COLOR_RAMP_SECTION) else {
        panic!("fixtura: com um grão escolhido o Grain abre e a rampa dele é pintada");
    };
    assert!(
        rect_of(
            &rects,
            ph2d_editor_core::ids::grip_de(pids::PAINTER_BRUSH_COLOR_RAMP_SECTION)
        )
        .is_none(),
        "a rampa aninhada ganhou uma pega"
    );
    let menu = botao_direito(&mut host, cab);
    assert!(
        !matches!(menu, Some(ContextMenuKind::SectionOutline { .. })),
        "o botão direito na rampa aninhada abriu o menu de tema de secção ({menu:?})"
    );
}

/// ⭐⭐ **Arrastar a pega da segunda móvel para cima da primeira põe-na à frente** — e a pintura
/// seguinte segue a ordem gravada. *Mutação: o `PlanoCtx::corre` pela ordem natural ⇒ a segunda
/// continua atrás.*
#[test]
fn arrastar_a_pega_poe_a_seccao_onde_se_larga() {
    let tool = PainterTool::default();
    let (mut host, mut st, rects) = pintado(&tool);
    let vista = moveis_a_vista(&rects);
    assert!(vista.len() >= 2, "fixtura: duas secções móveis à vista");
    let (primeira, cab1) = vista[0];
    let (segunda, _) = vista[1];
    let pega =
        rect_of(&rects, ph2d_editor_core::ids::grip_de(segunda)).expect("a segunda móvel tem pega");
    let (px, py) = (pega.x + pega.w * 0.5, pega.y + pega.h * 0.5);
    let alvo_y = cab1.y + 1.0;
    for (kind, y, t) in [
        (PointerKind::Down, py, SEC),
        (PointerKind::Move, alvo_y, SEC + 1),
        (PointerKind::Up, alvo_y, SEC + 2),
    ] {
        let _ = host.dispatch_pointer_event(pointer(kind, PointerButton::Primary, px, y, t));
    }
    assert_eq!(
        host.store().section_order().first(),
        Some(&segunda),
        "a ordem gravada não pôs a secção arrastada à frente"
    );
    set_current_brush(Some(tool.brush_settings()));
    let depois = moveis_a_vista(&host.paint::<PainterLayersPanel>(&mut st, viewport()));
    let pos = |id: NodeId| depois.iter().position(|(n, _)| *n == id);
    assert!(
        pos(segunda) < pos(primeira),
        "a pintura seguinte não seguiu a ordem gravada ({depois:?})"
    );
}

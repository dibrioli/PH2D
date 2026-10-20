//! **As linhas do FIM e do TRAÇO pelo ponteiro** (L5 · L7 · L8, `docs/Painter/Pinceis/03_MOTORES.md` §4): a cauda,
//! a ponta dela, o fall-off e o patamar no cartão Taper; o modo sobre o vazio debaixo do Blend; o Glaze, as orlas e
//! a largura debaixo do Accumulate — cada uma pintada onde o meio a lê, e digitar nela muda o pincel do tool.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::ids::{
    PAINTER_TAPER_END, PAINTER_TAPER_FALL_OFF, PAINTER_TAPER_FALL_OFF_START, PAINTER_TAPER_TIP_END,
    PAINTER_TRACO_FIELDS,
};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 4000.0)
}

fn painted(tool: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(tool.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    host.open_all_sections();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    (host, st, rects)
}

fn pintada(rects: &[(NodeId, Rect)], id: NodeId) -> bool {
    rects
        .iter()
        .any(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
}

const FIM_DO_DIGITAL: [NodeId; 2] = [PAINTER_TAPER_END, PAINTER_TAPER_TIP_END];
const FALL_OFF: [NodeId; 2] = [PAINTER_TAPER_FALL_OFF, PAINTER_TAPER_FALL_OFF_START];

/// ⭐ Cada linha pinta-se onde o meio a lê: a cauda e o traço como camada só no Digital (o plano do traço); o
/// fall-off com o cartão Taper (não na aguada); o modo sobre o vazio onde o Blend se oferece (não na aguada).
#[test]
fn as_linhas_do_fim_e_do_traco_estao_onde_o_meio_as_le() {
    for m in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        let mut t = PainterTool::default();
        t.set_paint_media(m);
        let (_h, _s, rects) = painted(&t);
        let digital = m == PaintMedia::Digital;
        for id in FIM_DO_DIGITAL
            .into_iter()
            .chain(PAINTER_TRACO_FIELDS[..4].iter().copied())
        {
            assert_eq!(
                pintada(&rects, id),
                digital,
                "{m:?}: a linha do Digital {id:?}"
            );
        }
        // O fall-off vive com o cartão Taper (a aguada não o mostra: ela não lê a cobertura do dab).
        let taper = pintada(&rects, ph2d_tool_painter::ids::PAINTER_TAPER_TIP_START);
        assert_eq!(taper, m != PaintMedia::Watercolor, "{m:?}: o cartão Taper");
        for id in FALL_OFF {
            assert_eq!(pintada(&rects, id), taper, "{m:?}: o fall-off {id:?}");
        }
        assert_eq!(
            pintada(&rects, PAINTER_TRACO_FIELDS[4]),
            m != PaintMedia::Watercolor,
            "{m:?}: o modo sobre o vazio"
        );
    }
}

/// ⭐ Digitar numa linha muda o pincel do tool (o leitor decide: o traço lê `taper`, `traco`, `blend_sobre_vazio`).
#[test]
fn digitar_uma_linha_do_traco_muda_o_pincel() {
    let mut t = PainterTool::default();
    let casos = [
        (PAINTER_TAPER_END, "2.5"),
        (PAINTER_TAPER_TIP_END, "0.3"),
        (PAINTER_TAPER_FALL_OFF, "9"),
        (PAINTER_TAPER_FALL_OFF_START, "6"),
        (PAINTER_TRACO_FIELDS[0], "0.4"),
        (PAINTER_TRACO_FIELDS[1], "0.6"),
        (PAINTER_TRACO_FIELDS[2], "0.7"),
        (PAINTER_TRACO_FIELDS[3], "12"),
        (PAINTER_TRACO_FIELDS[4], "1"),
    ];
    for (id, v) in casos {
        let (mut host, mut st, rects) = painted(&t);
        assert!(pintada(&rects, id), "{id:?} não pintada");
        for ev in host.type_into_number(id, v) {
            host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
        }
        for action in host.drained_actions() {
            if let EditorAction::ToolPanelEvent(pe) = action {
                t.handle_panel_event(pe);
            }
        }
    }
    let s = t.brush_settings();
    let tp = s.taper;
    assert_eq!(
        (tp.end, tp.tip_end, tp.fall_off, tp.fall_off_start),
        (2.5, 0.3, 9.0, 6.0)
    );
    let tr = s.traco;
    assert_eq!(
        (tr.tecto, tr.molhada, tr.queimada, tr.largura),
        (0.4, 0.6, 0.7, 12.0)
    );
    assert!(s.blend_sobre_vazio);
}

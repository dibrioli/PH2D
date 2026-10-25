//! ⭐ **O Wash** (dono 25/10: *«quero, mas como uma opção a mais num checkbox; não mude o comportamento atual»*): com o
//! Accumulate desligado, o texel do traço pára no MAIOR dab que o cobre (`stroke_cover::cover_max`) — o corte do
//! traço é o de um dab só e o traço que volta por cima de si não escurece. Desligado, a lei de sempre.

use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool, Tool,
};
use ph2d_tool_painter::{PaintMedia, PainterTool, ids};
use std::path::PathBuf;

const W: usize = 400;
const H: usize = 120;

fn tool(wash: bool, flow: f32) -> PainterTool {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let p = Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == "Soft Round")
        .unwrap()
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_color_srgb8([0, 0, 0]);
    t.preset_aplicar(&p).expect("aplica");
    t.set_brush_size_px(40.0);
    t.set_brush_strength(0.7);
    t.set_brush_flow(flow);
    if t.brush_settings().accumulate {
        t.handle_panel_event(PanelEvent::Click(ids::PAINTER_BRUSH_ACCUMULATE));
    }
    if t.brush_settings().wash != wash {
        t.handle_panel_event(PanelEvent::Click(ids::PAINTER_BRUSH_WASH));
    }
    assert!(!t.brush_settings().accumulate && t.brush_settings().wash == wash);
    t
}

fn traco(t: &mut PainterTool, xs: &[f32]) {
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, 60.0],
        pressure: 1.0,
        tilt: [0.0; 2],
        phase,
    };
    t.on_canvas_pointer(cp(xs[0], PointerPhase::Down));
    for &x in &xs[1..] {
        t.on_canvas_pointer(cp(x, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(*xs.last().unwrap(), PointerPhase::Up));
}

fn corte(t: &mut PainterTool, x: usize) -> Vec<i32> {
    let px = t.run_full().0;
    (0..=24)
        .map(|d| i32::from(px[((60 + d) * W + x) * 4 + 3]))
        .collect()
}

fn ida(a: f32, b: f32) -> Vec<f32> {
    let n = ((b - a).abs() / 4.0) as usize;
    (0..=n).map(|i| a + (b - a) * i as f32 / n as f32).collect()
}

/// O corte do traço é o de um toque (o Falloff de um dab), a Flow 1 e a 0,5 (que escala, não soma).
#[test]
fn a_borda_do_wash_e_a_de_um_dab() {
    for flow in [1.0, 0.5] {
        let mut toque = tool(true, flow);
        traco(&mut toque, &[200.0, 200.0]);
        let mut t = tool(true, flow);
        traco(&mut t, &ida(60.0, 340.0));
        let (a, b) = (corte(&mut toque, 200), corte(&mut t, 200));
        let pior = a.iter().zip(&b).map(|(x, y)| (x - y).abs()).max().unwrap();
        assert!(
            pior <= 2,
            "Flow {flow}: o corte do traço {b:?} × o de um dab {a:?}"
        );
    }
}

/// O traço que volta por cima de si não escurece — e sem o Wash escurece (o controlo: a lei da taxa).
#[test]
fn o_wash_nao_escurece_a_si_mesmo() {
    let mut vai_e_volta = ida(60.0, 340.0);
    vai_e_volta.extend(ida(340.0, 60.0));
    let medir = |wash: bool, xs: &[f32]| {
        let mut t = tool(wash, 0.3);
        traco(&mut t, xs);
        corte(&mut t, 200)[0]
    };
    let (uma, duas) = (medir(true, &ida(60.0, 340.0)), medir(true, &vai_e_volta));
    assert!(
        (uma - duas).abs() <= 1,
        "com o Wash a volta escureceu: {uma} → {duas}"
    );
    let (uma, duas) = (medir(false, &ida(60.0, 340.0)), medir(false, &vai_e_volta));
    assert!(
        duas > uma + 5,
        "o controlo: sem o Wash a volta escurece ({uma} → {duas})"
    );
}

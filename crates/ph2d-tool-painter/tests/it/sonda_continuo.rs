//! **Sonda: o Continuous deixa o traço em contas?** (dono 25/10, foto: Spacing 0,401 + Continuous ligado, o traço
//! macio com bandas a cada passo). Mede a ondulação do alfa ao longo de um recto, no eixo e perto da borda.

use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool, Tool,
};
use ph2d_tool_painter::{PaintMedia, PainterTool, ids};
use std::path::PathBuf;

const W: usize = 400;
const H: usize = 120;

fn tool(nome: &str) -> PainterTool {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let p = Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == nome)
        .unwrap()
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_color_srgb8([0, 0, 0]);
    t.preset_aplicar(&p).expect("aplica");
    t.set_brush_size_px(40.0);
    t
}

fn alterna(
    t: &mut PainterTool,
    id: ph2d_a11y::NodeId,
    on: bool,
    le: impl Fn(&PainterTool) -> bool,
) {
    if le(t) != on {
        t.handle_panel_event(PanelEvent::Click(id));
    }
    assert_eq!(le(t), on);
}

fn ondulacao(t: &mut PainterTool) -> (f32, f32) {
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, 60.0],
        pressure: 1.0,
        tilt: [0.0; 2],
        phase,
    };
    t.on_canvas_pointer(cp(40.0, PointerPhase::Down));
    let mut x = 40.0;
    while x < 360.0 {
        x += 6.0;
        t.on_canvas_pointer(cp(x, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(360.0, PointerPhase::Up));
    let px = t.run_full().0;
    if std::env::var("PH2D_PERFIL").is_ok() {
        let v: Vec<u8> = (120..200)
            .step_by(2)
            .map(|x| px[(60 * W + x) * 4 + 3])
            .collect();
        println!("[perfil] {v:?}");
    }
    let r = |y: usize| {
        let v: Vec<f32> = (120..280)
            .map(|x| f32::from(px[(y * W + x) * 4 + 3]))
            .collect();
        let m = v.iter().sum::<f32>() / v.len() as f32;
        let (lo, hi) = v
            .iter()
            .fold((f32::MAX, 0f32), |(a, b), &c| (a.min(c), b.max(c)));
        if m < 1.0 { 0.0 } else { (hi - lo) / m }
    };
    (r(60), r(60 + 13))
}

#[test]
#[ignore = "sonda: o Continuous em contas (dono 25/10)"]
fn sonda_continuo() {
    for nome in ["Soft Round", "Glaze Round", "Hard Round 1", "Airbrush"] {
        for (cont, acc, s, f) in [
            (false, true, 1.0, 1.0),
            (true, true, 1.0, 1.0),
            (true, false, 1.0, 1.0),
            (true, false, 0.5, 1.0),
            (true, false, 0.5, 0.3),
            (true, true, 0.5, 0.3),
        ] {
            let Some(mut t) = std::panic::catch_unwind(|| tool(nome)).ok() else {
                continue;
            };
            t.set_brush_spacing(0.401);
            alterna(&mut t, ids::PAINTER_BRUSH_CONTINUOUS_SPACING, cont, |t| {
                t.brush_settings().continuous_spacing
            });
            alterna(&mut t, ids::PAINTER_BRUSH_ACCUMULATE, acc, |t| {
                t.brush_settings().accumulate
            });
            t.set_brush_strength(s);
            t.set_brush_flow(f);
            let (eixo, borda) = ondulacao(&mut t);
            println!(
                "[continuo] {nome:12} cont {cont:5} acc {acc:5} S {s} F {f}: eixo {eixo:.3} borda {borda:.3}"
            );
        }
    }
}

/// ⭐ **O Continuous com o Accumulate desligado e o Flow baixo é LISO** (dono 25/10, a foto do traço em contas): a
/// cápsula aplicava o Flow uma vez por passo sobre a união dos sub-dabs, e a costura entre dois passos levava-o
/// duas vezes — 15 % de ondulação no Glaze Round a Flow 0,3. Com a taxa repartida pelos sub-dabs
/// (`dab/bands.rs`, `c_cap`) a ondulação é a do arredondamento (≤ 3 %), no eixo e na borda.
#[test]
fn o_continuo_com_flow_baixo_e_liso() {
    let mut t = tool("Glaze Round");
    t.set_brush_spacing(0.401);
    alterna(&mut t, ids::PAINTER_BRUSH_CONTINUOUS_SPACING, true, |t| {
        t.brush_settings().continuous_spacing
    });
    alterna(&mut t, ids::PAINTER_BRUSH_ACCUMULATE, false, |t| {
        t.brush_settings().accumulate
    });
    t.set_brush_strength(0.5);
    t.set_brush_flow(0.3);
    let (eixo, borda) = ondulacao(&mut t);
    assert!(
        eixo <= 0.03 && borda <= 0.03,
        "o traço em contas: eixo {eixo:.3} borda {borda:.3}"
    );
}

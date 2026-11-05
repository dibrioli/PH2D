//! **O «Same as Paper» do Grain não muda a QUANTIDADE de tinta** (Bug #61, smoke do dono 04/11): o mesmo traço pelo
//! pincel de fábrica, sem e com a opção. Antes da cura: Wet Drip 9,87× · Wet Round 13,76× · Wet Dry Brush 20,31×.
use crate::tool::PainterTool;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: usize = 400;
const H: usize = 240;
const PINCEIS: [&str; 8] = [
    "Wet Drip",
    "Wet Round",
    "Wet Wash",
    "Wet Liner",
    "Wet Dry Brush",
    "Gouache Opaque",
    "Filbert",
    "Dabber",
];

fn ponto(x: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, 120.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// `(massa, água, células com tinta, escurecimento médio do ecrã)`.
fn le(t: &PainterTool) -> [f64; 4] {
    let g = t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão do Wet Paint")
        .engine
        .active_grid();
    let (mut massa, mut agua, mut n) = (0.0, 0.0, 0.0);
    for y in 1..g.h {
        for x in 1..g.w {
            let i = x + y * g.s;
            let m = f64::from(g.sett[i]) + f64::from(g.susp[i]);
            massa += m;
            agua += f64::from(g.film[i]);
            if m > 1.0 {
                n += 1.0;
            }
        }
    }
    let px: f64 = t
        .canvas_rgba
        .chunks(4)
        .map(|p| 255.0 - f64::from(p[1]))
        .sum();
    [massa, agua, n, px / (W * H) as f64]
}

fn traco(p: &ph2d_brush_presets::Preset, papel: bool) -> ([f64; 4], PainterTool) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.wet_preset_aplicar_volumes(&p.parametros.volumes);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([30, 60, 200]);
    if papel {
        t.toggle_brush_grain_same_as_paper();
    }
    t.on_canvas_pointer(ponto(60.0, PointerPhase::Down));
    for k in 1..=70 {
        t.on_canvas_pointer(ponto(60.0 + 4.0 * k as f32, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponto(340.0, PointerPhase::Up));
    for _ in 0..8 {
        t.on_tick(1000.0 / 60.0);
    }
    (le(&t), t)
}

#[test]
fn o_same_as_paper_nao_muda_a_quantidade_de_tinta() {
    for nome in PINCEIS {
        let e = crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato(nome);
        let (s, _) = traco(&e, false);
        let (c, _) = traco(&e, true);
        let (massa, tela) = (c[0] / s[0], c[3] / s[3]);
        println!("{nome}: massa {massa:.3} · tela {tela:.3}");
        assert!((0.9..1.1).contains(&massa), "{nome}: massa {massa:.3}×");
        assert!((0.9..1.1).contains(&tela), "{nome}: tela {tela:.3}×");
    }
}

/// E o papel continua a desenhar: a opção muda onde a tinta pousa, célula a célula, sem mudar o total.
#[test]
fn o_same_as_paper_ainda_desenha_o_papel() {
    let e = crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato("Wet Round");
    let (_, s) = traco(&e, false);
    let (_, c) = traco(&e, true);
    let dif = s
        .canvas_rgba
        .iter()
        .zip(c.canvas_rgba.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        dif > W * H / 20,
        "só {dif} bytes mudaram: o papel não pintou"
    );
}

//! **Bug #49 (dono 11/10): azul escuro sobre azul claro ficava VERDE no soltar do rato**, com o *Pigment
//! mixing* desligado. Mecanismo: no soltar a sim volta a andar e o filme escuro ASSENTA sobre o claro
//! seco; o `assenta::resolve` acertava cada canal com a derivada do ecrã do INÍCIO da volta, mas o tapar
//! extra pesa as três cores juntas — mexer no vermelho mudava o verde, e o assentado ia a verde-água.
//! Depois do Undo o claro é a BASE congelada (não há assentado no motor): por isso «sumia».
//!
//! A sonda (`--ignored sonda_verde --nocapture`) corre os 12 pincéis de fábrica com três forças do escuro.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_wet_paint::tuning::Knob;

const W: usize = 240;
const H: usize = 160;
const PARES: [([u8; 3], [u8; 3]); 2] = [
    ([120, 180, 235], [20, 40, 140]),
    ([90, 190, 240], [10, 30, 110]),
];

fn p(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A cor média do cruzamento dos dois traços.
fn cruzamento(c: &[u8]) -> [f64; 3] {
    let mut s = [0.0; 3];
    for y in 72..88 {
        for x in 112..128 {
            let o = (y * W + x) * 4;
            for k in 0..3 {
                s[k] += f64::from(c[o + k]);
            }
        }
    }
    s.map(|v| v / 256.0)
}

fn ticks(t: &mut PainterTool, n: usize) {
    for _ in 0..n {
        t.on_tick(1000.0 / 60.0);
    }
}

/// O claro seco (botão Dry), o escuro a cruzá-lo com `forca` × o pigmento do preset. Devolve o
/// cruzamento antes do `pointer_up`, 1 s depois, e seco.
fn cena(
    e: &ph2d_brush_presets::Preset,
    km: bool,
    (claro, escuro): ([u8; 3], [u8; 3]),
    forca: f64,
) -> [[f64; 3]; 3] {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(e).is_empty());
    t.paint.wetpaint.km_mixing = km;
    t.set_brush_size_px(20.0);
    t.set_brush_color_srgb8(claro);
    t.on_canvas_pointer(p(30.0, 80.0, PointerPhase::Down));
    for k in 1..=60 {
        t.on_canvas_pointer(p(30.0 + 3.0 * k as f32, 80.0, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(p(210.0, 80.0, PointerPhase::Up));
    ticks(&mut t, 10);
    t.wetpaint_dry_canvas();
    ticks(&mut t, 4);
    t.set_brush_color_srgb8(escuro);
    let ppd = t.paint.wetpaint.knobs.get(Knob::PigmentPerDab);
    t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, ppd * forca);
    t.on_canvas_pointer(p(120.0, 20.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(p(120.0, 20.0 + 3.0 * k as f32, PointerPhase::Move));
        t.on_tick(8.0);
    }
    let antes = cruzamento(&t.canvas_rgba);
    t.on_canvas_pointer(p(120.0, 140.0, PointerPhase::Up));
    ticks(&mut t, 60);
    let depois = cruzamento(&t.canvas_rgba);
    t.wetpaint_dry_canvas();
    ticks(&mut t, 4);
    [antes, depois, cruzamento(&t.canvas_rgba)]
}

fn verde(c: [f64; 3]) -> f64 {
    c[1] - c[2]
}

/// O portão: no óleo (onde o dono o viu), o filme escuro ralo sobre o claro seco não sobe o verde no
/// soltar nem ao secar — caixa desligada e ligada. Antes da cura: Oil Bristle +17, Express Oil +10.
#[test]
fn o_azul_sobre_azul_nao_fica_verde_no_soltar() {
    let b = super::mistura_wet_drip_tests::biblioteca();
    for nome in ["Oil Bristle", "Express Oil"] {
        let e = &b
            .entradas
            .iter()
            .find(|e| e.preset.parametros.name == nome)
            .expect("o pincel de fábrica")
            .preset;
        for km in [false, true] {
            for par in PARES {
                let [antes, depois, seco] = cena(e, km, par, 0.1);
                let subiu = (verde(depois) - verde(antes)).max(verde(seco) - verde(antes));
                assert!(
                    subiu <= 3.0,
                    "{nome} km={km} {par:?}: o verde subiu {subiu:+.1} ({antes:.0?} → {depois:.0?} → {seco:.0?})"
                );
            }
        }
    }
}

#[test]
#[ignore = "sonda do Bug #49"]
fn sonda_verde() {
    let b = super::mistura_wet_drip_tests::biblioteca();
    for e in &b.entradas {
        for forca in [1.0, 0.3, 0.1] {
            for par in PARES {
                let [a, d, s] = cena(&e.preset, false, par, forca);
                println!(
                    "{:>16} forca {forca} {par:?}: {a:.0?} → {d:.0?} → seco {s:.0?} (verde {:+.1})",
                    e.preset.parametros.name,
                    verde(d).max(verde(s)) - verde(a)
                );
            }
        }
    }
}

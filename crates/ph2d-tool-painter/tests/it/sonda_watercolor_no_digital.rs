//! Sonda (07/11, pedido do dono): a mistura do Watercolor Basic trazida ao Digital no lugar do Color mixing?
//! A cena: mancha vermelha, depois um arrasto azul por cima dela até ao branco (a de `sonda_composite_vs_mixing`).
//! Corre à mão:
//! `PH2D_WCD_FOTO=<pasta> cargo test --profile smoke -p ph2d-tool-painter --test it sonda_wcd -- --ignored --nocapture`
//! Doc: `docs/Painter/ESTUDO_MISTURA_DO_WATERCOLOR_PARA_O_DIGITAL_2026-11-07.md`.

use crate::exame_bancada::pinceis;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{ModoMisturaOpaca, PaintMedia, PainterTool};
use std::time::Instant;

const W: usize = 400;
const H: usize = 240;
const Y: f32 = 120.0;

fn arrasta(t: &mut PainterTool, a: [f32; 2], b: [f32; 2]) {
    let cp = |pos: [f32; 2], phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(a, PointerPhase::Down));
    let n = 60;
    for k in 1..=n {
        let u = k as f32 / n as f32;
        t.on_canvas_pointer(cp(
            [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u],
            PointerPhase::Move,
        ));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(b, PointerPhase::Up));
    t.on_tick(8.0);
}

/// A mancha: 2 passadas de 9 traços horizontais (como a sonda irmã, menos passadas: a aguada já cobre).
fn mancha(t: &mut PainterTool, passadas: usize, cor: [u8; 3]) {
    t.set_brush_color_srgb8(cor);
    for _ in 0..passadas {
        for k in 0..9 {
            let y = Y - 40.0 + 10.0 * k as f32;
            arrasta(t, [140.0, y], [300.0, y]);
        }
    }
}

fn novo(preset: &str) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    pinceis(preset).remove(0).aplica(&mut t, true);
    t.sonda_raio(40.0);
    t
}

/// O meio Watercolor com a transparência DESLIGADA: corpo pleno (Opacity 1), densidade alta, sem orla, sem grão,
/// sem warp — e a mistura do Watercolor Basic (Charge 0,5 · Pull 0,6 · Rewet 0,5 · Smudge 0,5 · Dilution 0,2).
fn aguada_opaca(t: &mut PainterTool, hardness: f32) {
    t.set_paint_media(PaintMedia::Watercolor);
    t.set_brush_hardness(hardness);
    t.set_brush_spacing(0.1);
    t.set_brush_fill(1.0);
    t.set_brush_depth(4.0);
    t.set_brush_opacity(1.0);
    t.set_brush_edge_gain(0.0);
    t.set_brush_granulation(0.0);
    t.set_brush_warp(0.0);
    t.set_brush_wet_charge(0.5);
    t.set_brush_wet_pull(0.6);
    t.set_brush_wet_rewet(0.5);
    t.set_brush_wet_smudge(0.5);
    t.set_brush_wet_dilution(0.2);
}

#[derive(Clone, Copy)]
enum Var {
    /// O Watercolor Basic de fábrica, mancha e arrasto (a foto 1 do dono).
    WatercolorBasic,
    /// G-Pen, mancha Off, arrasto com o Color mixing Blend 0,5/1/0,5 (a foto 2).
    ColorMixing,
    /// G-Pen, arrasto com o Pigment do Digital (K–M por píxel contra a tela), mix 1.
    PigmentoDigital,
    /// Mancha G-Pen Digital opaca; arrasto no meio Watercolor sem transparência (hardness do G-Pen, 1).
    AguadaOpacaDura,
    /// Idem com hardness 0 (a do Watercolor Basic).
    AguadaOpacaMacia,
    /// Mancha G-Pen opaca; arrasto com o Watercolor Basic de fábrica por cima.
    BasicSobreOpaco,
}

const VERMELHO: [u8; 3] = [200, 0, 0];
const AMARELO: [u8; 3] = [245, 205, 0];

/// (rótulo, variante, cor da mancha, Pigment do arrasto e da mancha; `0` = desligado).
const VARS: [(&str, Var, [u8; 3], f32); 12] = [
    ("Watercolor Basic", Var::WatercolorBasic, VERMELHO, 0.0),
    ("G-Pen Color mixing", Var::ColorMixing, VERMELHO, 0.0),
    ("G-Pen Pigment 1", Var::PigmentoDigital, VERMELHO, 1.0),
    ("Aguada opaca dura", Var::AguadaOpacaDura, VERMELHO, 0.0),
    ("Aguada opaca macia", Var::AguadaOpacaMacia, VERMELHO, 0.0),
    ("Basic sobre opaco", Var::BasicSobreOpaco, VERMELHO, 0.0),
    ("Y Basic Pigment 1", Var::WatercolorBasic, AMARELO, 1.0),
    ("Y G-Pen Color mixing", Var::ColorMixing, AMARELO, 0.0),
    ("Y G-Pen Pigment 1", Var::PigmentoDigital, AMARELO, 1.0),
    ("Y Aguada op dura Pig1", Var::AguadaOpacaDura, AMARELO, 1.0),
    (
        "Y Aguada op macia Pig1",
        Var::AguadaOpacaMacia,
        AMARELO,
        1.0,
    ),
    ("Y Basic sem Pigment", Var::WatercolorBasic, AMARELO, 0.0),
];

fn corre(v: Var, cor: [u8; 3], pig: f32) -> (Vec<u8>, f64) {
    let mut t = match v {
        Var::WatercolorBasic => {
            let mut t = novo("Watercolor Basic");
            t.set_brush_pigment_mixing(pig);
            mancha(&mut t, 2, cor);
            t
        }
        _ => {
            let mut t = novo("G-Pen");
            t.set_brush_mistura_modo(ModoMisturaOpaca::Off);
            mancha(&mut t, 2, cor);
            t
        }
    };
    match v {
        Var::WatercolorBasic => {}
        Var::ColorMixing => {
            t.set_brush_mistura_modo(ModoMisturaOpaca::WetBlend);
            t.set_brush_mistura_numero(0, 0.5);
            t.set_brush_mistura_numero(1, 1.0);
            t.set_brush_mistura_numero(2, 0.5);
        }
        Var::PigmentoDigital => {}
        Var::AguadaOpacaDura => aguada_opaca(&mut t, 1.0),
        Var::AguadaOpacaMacia => aguada_opaca(&mut t, 0.0),
        Var::BasicSobreOpaco => {
            pinceis("Watercolor Basic").remove(0).aplica(&mut t, true);
            t.sonda_raio(40.0);
        }
    }
    t.set_brush_pigment_mixing(pig);
    t.set_brush_color_srgb8([0, 0, 220]);
    let ini = Instant::now();
    arrasta(&mut t, [60.0, Y], [340.0, Y]);
    let ms = ini.elapsed().as_secs_f64() * 1e3;
    (t.run_full().0, ms)
}

fn media(p: &[u8], x0: usize, x1: usize) -> [f32; 3] {
    let (mut s, mut n) = ([0.0f32; 3], 0.0);
    for y in Y as usize - 2..=Y as usize + 2 {
        for x in x0..=x1 {
            let i = (y * W + x) * 4;
            for c in 0..3 {
                s[c] += f32::from(p[i + c]);
            }
            n += 1.0;
        }
    }
    s.map(|v| v / n)
}

/// Rugosidade ao longo do eixo DENTRO da mancha (x 160..290): a 2.ª diferença média da luminância — os grumos.
fn grumos(p: &[u8]) -> f32 {
    let lum = |x: usize, y: usize| {
        let i = (y * W + x) * 4;
        0.299 * f32::from(p[i]) + 0.587 * f32::from(p[i + 1]) + 0.114 * f32::from(p[i + 2])
    };
    let (mut r, mut n) = (0.0, 0.0);
    for y in Y as usize - 15..=Y as usize + 15 {
        for x in 161..290 {
            r += (lum(x + 1, y) - 2.0 * lum(x, y) + lum(x - 1, y)).abs();
            n += 1.0;
        }
    }
    r / n
}

#[test]
#[ignore = "sonda"]
fn sonda_wcd_folha() {
    let pasta = std::env::var("PH2D_WCD_FOTO")
        .ok()
        .map(std::path::PathBuf::from);
    let mut melhor = [f64::INFINITY; VARS.len()];
    for _ in 0..3 {
        for (i, &(_, v, cor, pig)) in VARS.iter().enumerate() {
            melhor[i] = melhor[i].min(corre(v, cor, pig).1);
        }
    }
    for (i, &(rot, v, cor, pig)) in VARS.iter().enumerate() {
        let (p, _) = corre(v, cor, pig);
        let [a, b, c, d] = [
            media(&p, 165, 175),
            media(&p, 210, 230),
            media(&p, 285, 295),
            media(&p, 325, 335),
        ];
        eprintln!(
            "[wcd] {rot:<20} entra {:>3.0},{:>3.0},{:>3.0} | meio {:>3.0},{:>3.0},{:>3.0} | sai {:>3.0},{:>3.0},{:>3.0} | cauda {:>3.0},{:>3.0},{:>3.0} | grumos {:>5.2} | {:>6.1} ms",
            a[0],
            a[1],
            a[2],
            b[0],
            b[1],
            b[2],
            c[0],
            c[1],
            c[2],
            d[0],
            d[1],
            d[2],
            grumos(&p),
            melhor[i]
        );
        if let Some(dir) = &pasta {
            std::fs::write(dir.join(format!("wcd_{i:02}.rgba")), &p).expect("escreve");
        }
    }
}

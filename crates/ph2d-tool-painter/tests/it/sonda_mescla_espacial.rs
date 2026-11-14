//! Sonda (à mão, `#[ignore]`): a mescla ESPACIAL do Watercolor Basic, ingrediente a ingrediente (08/11).
//! Doc: `docs/Painter/ESTUDO_MESCLA_ESPACIAL_DO_WATERCOLOR_2026-11-08.md`. Nada de produto muda.
//! Cena: mancha azul SECA (os píxeis dela são a tela de entrada: sem mapa de molhado), depois 4 traços amarelos
//! paralelos que entram nela pela esquerda e um 5.º em diagonal com pressão a variar.

use crate::a_mistura_pigmento_digital::{AZUL, mancha};
use crate::exame_bancada::pinceis;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{ModoMisturaOpaca, PainterTool};

const W: usize = 400;
const H: usize = 240;
const AMARELO: [u8; 3] = [245, 205, 0];

fn arrasta_p(t: &mut PainterTool, a: [f32; 2], b: [f32; 2], p: impl Fn(f32) -> f32) {
    let cp = |pos: [f32; 2], pressure, phase| CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(a, p(0.0), PointerPhase::Down));
    let n = 60;
    for k in 1..=n {
        let u = k as f32 / n as f32;
        t.on_canvas_pointer(cp(
            [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u],
            p(u),
            PointerPhase::Move,
        ));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(b, p(1.0), PointerPhase::Up));
    t.on_tick(8.0);
}

/// A mancha azul seca: pintada com o Watercolor Basic de fábrica e achatada em píxeis.
fn azul_seco() -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    pinceis("Watercolor Basic").remove(0).aplica(&mut t, true);
    t.sonda_raio(40.0);
    mancha(&mut t, AZUL);
    t.run_full().0
}

#[derive(Clone, Copy)]
enum Var {
    Fabrica,
    SemSmudge,
    SemMixer,
    SemRewet,
    SemDilution,
    Spread1,
    SemOrla,
    SemGrao,
    PontaDura,
    Opaco,
    SemArrastoNenhum,
    DigitalPigmentMix(&'static str),
    WatercolorOpacoPigment,
}

const VARS: [(Var, &str); 15] = [
    (Var::Fabrica, "01 Watercolor Basic (fabrica)"),
    (Var::SemSmudge, "02 Smudge 0"),
    (Var::SemMixer, "03 Charge 1 (mixer off: pickup/pull 0)"),
    (Var::SemRewet, "04 Rewet 0"),
    (Var::SemDilution, "05 Dilution 0"),
    (Var::Spread1, "06 Spread 1 (dissolve blur off)"),
    (Var::SemOrla, "07 Edge 0"),
    (Var::SemGrao, "08 Granulation 0 + Warp 0"),
    (Var::PontaDura, "09 Shape auto off + Hardness 1"),
    (Var::Opaco, "10 Fill 1 + Opacity 1"),
    (Var::SemArrastoNenhum, "11 Smudge 0 + Rewet 0 + Charge 1"),
    (
        Var::DigitalPigmentMix("Hard Round"),
        "12 Digital Hard Round, Pigment mix",
    ),
    (
        Var::DigitalPigmentMix("Soft Round"),
        "13 Digital Soft Round, Pigment mix",
    ),
    (
        Var::DigitalPigmentMix("G-Pen"),
        "14 Digital G-Pen, Pigment mix",
    ),
    (
        Var::WatercolorOpacoPigment,
        "15 Wc Fill1 Op1 Pigment1 (proto)",
    ),
];

fn prepara(v: Var, base: &[u8]) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(base.to_vec(), W as u32, H as u32);
    let preset = match v {
        Var::DigitalPigmentMix(p) => p,
        _ => "Watercolor Basic",
    };
    pinceis(preset).remove(0).aplica(&mut t, true);
    t.sonda_raio(40.0);
    match v {
        Var::Fabrica => {}
        Var::SemSmudge => t.set_brush_wet_smudge(0.0),
        Var::SemMixer => t.set_brush_wet_charge(1.0),
        Var::SemRewet => t.set_brush_wet_rewet(0.0),
        Var::SemDilution => t.set_brush_wet_dilution(0.0),
        Var::Spread1 => t.set_brush_edge_spread(1.0),
        Var::SemOrla => t.set_brush_edge_gain(0.0),
        Var::SemGrao => {
            t.set_brush_granulation(0.0);
            t.set_brush_warp(0.0);
        }
        Var::PontaDura => {
            t.toggle_brush_watercolor_shape_auto();
            t.set_brush_hardness(1.0);
        }
        Var::Opaco => {
            t.set_brush_fill(1.0);
            t.set_brush_opacity(1.0);
        }
        Var::SemArrastoNenhum => {
            t.set_brush_wet_smudge(0.0);
            t.set_brush_wet_rewet(0.0);
            t.set_brush_wet_charge(1.0);
        }
        Var::DigitalPigmentMix(_) => t.set_brush_mistura_modo(ModoMisturaOpaca::WetBlend),
        Var::WatercolorOpacoPigment => {
            t.set_brush_fill(1.0);
            t.set_brush_opacity(1.0);
            t.set_brush_pigment_mixing(1.0);
        }
    }
    t.set_brush_color_srgb8(AMARELO);
    t
}

/// Os 4 traços amarelos paralelos (y 80..125, passo 15: sobrepõem-se) da esquerda branca para dentro do azul
/// (x 140..300), e o 5.º em diagonal de pressão 0,2 → 1 → 0,2 (`pressao`), ou de pressão 1.
fn amarelos(t: &mut PainterTool, pressao: bool) {
    for k in 0..4 {
        let y = 80.0 + 15.0 * k as f32;
        arrasta_p(t, [60.0, y], [250.0, y], |_| 1.0);
    }
    if pressao {
        arrasta_p(t, [200.0, 200.0], [330.0, 70.0], |u| {
            0.2 + 0.8 * (1.0 - (2.0 * u - 1.0).abs())
        });
    } else {
        arrasta_p(t, [200.0, 200.0], [330.0, 70.0], |_| 1.0);
    }
}

fn rgb(p: &[u8], x: usize, y: usize) -> [f32; 3] {
    let i = (y * W + x) * 4;
    let a = f32::from(p[i + 3]) / 255.0;
    [0, 1, 2].map(|c| f32::from(p[i + c]) * a + 255.0 * (1.0 - a))
}

/// (a) Largura 10 %→90 % (px) da passagem amarelo → mancha ao longo do traço (y 95 e 110, x 90..230, canal R:
/// amarelo alto, azul baixo), mediana; (b) maior degrau entre vizinhos no miolo (x 170..280, y 75..130, longe da
/// orla da mancha seca); (c) fracção de píxeis do miolo com degrau > 24 (as «placas»).
fn metricas(p: &[u8]) -> (f32, f32, f32) {
    let mut larg = Vec::new();
    for y in [95usize, 110] {
        let perfil: Vec<f32> = (90..230).map(|x| rgb(p, x, y)[0]).collect();
        let hi = perfil[0..6].iter().copied().fold(f32::MIN, f32::max);
        let lo = perfil.iter().copied().fold(f32::MAX, f32::min);
        let lim = |f: f32| {
            perfil
                .iter()
                .position(|&r| r < hi - f * (hi - lo))
                .unwrap_or(perfil.len()) as f32
        };
        larg.push(lim(0.9) - lim(0.1));
    }
    let mut gmax: f32 = 0.0;
    let (mut n, mut placas) = (0u32, 0u32);
    for y in 75..130 {
        for x in 170..280 {
            let c = rgb(p, x, y);
            for (dx, dy) in [(1, 0), (0, 1)] {
                let d = rgb(p, x + dx, y + dy);
                let g = (0..3).map(|k| (c[k] - d[k]).abs()).fold(0.0, f32::max);
                gmax = gmax.max(g);
                n += 1;
                placas += u32::from(g > 24.0);
            }
        }
    }
    (
        (larg[0] + larg[1]) * 0.5,
        gmax,
        100.0 * placas as f32 / n as f32,
    )
}

fn ppm(p: &[u8], destino: &str) {
    let (x0, y0, cw, ch) = (40usize, 50usize, 320usize, 170usize);
    let mut out = format!("P6\n{cw} {ch}\n255\n").into_bytes();
    for y in y0..y0 + ch {
        for x in x0..x0 + cw {
            out.extend(rgb(p, x, y).map(|v| v.round() as u8));
        }
    }
    std::fs::write(destino, out).expect("escreve");
}

/// `PH2D_MESCLA_DIR=<pasta>`: um `.ppm` por variante (mouse e pressão), as métricas em stderr, e o custo dos 5
/// traços amarelos por variante (intercalado, mínimo de 5 blocos).
#[test]
#[ignore = "sonda: corre à mão"]
fn sonda_mescla_espacial_do_watercolor() {
    let _ = ph2d_tool_painter::raio_da_sonda::perfil_da_sonda();
    let dir = std::env::var("PH2D_MESCLA_DIR").expect("PH2D_MESCLA_DIR");
    let base = azul_seco();
    ppm(&base, &format!("{dir}/00_base.ppm"));
    for pressao in [false, true] {
        for (i, (v, nome)) in VARS.iter().enumerate() {
            let mut t = prepara(*v, &base);
            amarelos(&mut t, pressao);
            let p = t.run_full().0;
            let (larg, gmax, p99) = metricas(&p);
            eprintln!(
                "[mescla] {} | {nome} | largura10-90 {larg:.0} px | degrau max miolo {gmax:.0} | placas {p99:.2} %",
                if pressao { "pressao" } else { "mouse" }
            );
            ppm(
                &p,
                &format!("{dir}/{}_{i:02}.ppm", if pressao { "p" } else { "m" }),
            );
        }
    }
    // O molhado: a mesma cena com o azul ainda MOLHADO (mesma ferramenta, sem secar).
    {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
        pinceis("Watercolor Basic").remove(0).aplica(&mut t, true);
        t.sonda_raio(40.0);
        mancha(&mut t, AZUL);
        t.set_brush_color_srgb8(AMARELO);
        amarelos(&mut t, false);
        let p = t.run_full().0;
        let (larg, gmax, p99) = metricas(&p);
        eprintln!(
            "[mescla] mouse | 16 Watercolor Basic, azul MOLHADO | largura10-90 {larg:.0} px | degrau max miolo {gmax:.0} | placas {p99:.2} %"
        );
        ppm(&p, &format!("{dir}/m_15.ppm"));
    }
    let n = VARS.len();
    let mut min = vec![f64::MAX; n];
    for bloco in 0..5 {
        for k in 0..n {
            let i = (k + bloco) % n;
            let mut t = prepara(VARS[i].0, &base);
            let t0 = std::time::Instant::now();
            amarelos(&mut t, false);
            let _ = std::hint::black_box(t.run_full());
            min[i] = min[i].min(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    for (i, (_, nome)) in VARS.iter().enumerate() {
        eprintln!("[custo] {nome}: {:.1} ms / 5 tracos (minimo de 5)", min[i]);
    }
}

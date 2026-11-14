//! Sonda (07/11, pergunta do dono): o Composite Brush (Brush + Smear [+ Blur]) leva a tinta melhor que o
//! Color mixing (mistura opaca)? A cena de `a_mistura_opaca_nos_presets`: mancha vermelha num traço próprio,
//! depois um arrasto azul por cima dela até ao branco. Corre à mão:
//! `PH2D_CMP_FOTO=<pasta> cargo test --profile smoke -p ph2d-tool-painter --test it sonda_cmp -- --ignored --nocapture`
//! Doc: `docs/Painter/COMPARACAO_COMPOSITE_VS_COLOR_MIXING_2026-11-07.md`.

use crate::exame_bancada::pinceis;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{ModoMisturaOpaca, PainterTool};
use std::time::Instant;

const W: usize = 400;
const H: usize = 240;
const Y: f32 = 120.0;
// Operações do `+` do cartão Composite: 0 Brush · 1 Smear · 2 Blur.
const BRUSH: u8 = 0;
const SMEAR: u8 = 1;
const BLUR: u8 = 2;

fn arrasta(t: &mut PainterTool, a: [f32; 2], b: [f32; 2], pressao: bool) {
    let cp = |pos: [f32; 2], phase, p: f32| CanvasPointer {
        pos,
        pressure: p,
        tilt: [0.0, 0.0],
        phase,
    };
    let pr = |u: f32| {
        if pressao {
            0.15 + 0.85 * (std::f32::consts::PI * u).sin()
        } else {
            1.0
        }
    };
    t.on_canvas_pointer(cp(a, PointerPhase::Down, pr(0.0)));
    let n = 60;
    for k in 1..=n {
        let u = k as f32 / n as f32;
        t.on_canvas_pointer(cp(
            [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u],
            PointerPhase::Move,
            pr(u),
        ));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(b, PointerPhase::Up, pr(1.0)));
}

#[derive(Clone, Copy, Debug)]
enum Var {
    Off,
    Mix(ModoMisturaOpaca, f32, f32, f32),
    /// Pilha de cima para baixo: (op, strength).
    Comp(&'static [(u8, f32)]),
}

fn com_a_mancha(nome: Option<&str>) -> PainterTool {
    let mut t = PainterTool::default();
    // `PH2D_CMP_FUNDO=vazio`: a camada TRANSPARENTE sobre o papel (o «branco» é a camada vazia, não tinta branca).
    let fundo = if std::env::var("PH2D_CMP_FUNDO").as_deref() == Ok("vazio") {
        0u8
    } else {
        255
    };
    t.set_source(vec![fundo; W * H * 4], W as u32, H as u32);
    if let Some(n) = nome {
        pinceis(n).remove(0).aplica(&mut t, true);
    }
    t.sonda_raio(40.0);
    t.set_brush_mistura_modo(ModoMisturaOpaca::Off);
    t.set_brush_color_srgb8([200, 0, 0]);
    for _ in 0..4 {
        for k in 0..9 {
            let y = Y - 40.0 + 10.0 * k as f32;
            arrasta(&mut t, [140.0, y], [300.0, y], false);
        }
    }
    t
}

fn configura(t: &mut PainterTool, v: Var) {
    match v {
        Var::Off => {}
        Var::Mix(m, a, d, s) => {
            t.set_brush_mistura_modo(m);
            t.set_brush_mistura_numero(0, a);
            t.set_brush_mistura_numero(1, d);
            t.set_brush_mistura_numero(2, s);
        }
        Var::Comp(pilha) => {
            if !t.composite_enabled() {
                t.toggle_composite();
            }
            for (i, &(op, s)) in pilha.iter().enumerate() {
                t.acrescenta_camada(op);
                t.set_composite_layer_strength(i, s);
            }
            assert_eq!(t.composite_len(), pilha.len(), "a pilha montada");
        }
    }
    t.set_brush_color_srgb8([0, 0, 220]);
}

struct Medida {
    dentro: [f32; 3],
    cauda_px: usize,
    branco: [f32; 3],
    ruido_branco: f32,
    borda_px: f32,
    grao: f32,
}

fn lum(p: &[u8], x: usize, y: usize) -> f32 {
    let i = (y * W + x) * 4;
    0.299 * f32::from(p[i]) + 0.587 * f32::from(p[i + 1]) + 0.114 * f32::from(p[i + 2])
}

fn media(p: &[u8], x0: usize, x1: usize, y0: usize, y1: usize) -> [f32; 3] {
    let mut s = [0.0f32; 3];
    let mut n = 0.0;
    for y in y0..=y1 {
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

fn mede(p: &[u8]) -> Medida {
    let yc = Y as usize;
    let dentro = media(p, 256, 264, yc - 2, yc + 2);
    // Cauda: px depois da borda (x=300+raio ≈ 320) com R−G > 25 na linha central (média de 5 linhas).
    let mut cauda_px = 0;
    for x in 322..W {
        let c = media(p, x, x, yc - 2, yc + 2);
        if c[0] - c[1] > 25.0 {
            cauda_px = x - 321;
        }
    }
    let branco = media(p, 80, 120, yc - 4, yc + 4);
    // Ruído ao longo do traço sobre o branco: 2.ª diferença da luminância (anéis / costuras).
    let mut r = 0.0;
    let mut n = 0.0;
    for y in yc - 4..=yc + 4 {
        for x in 71..129 {
            r += (lum(p, x + 1, y) - 2.0 * lum(p, x, y) + lum(p, x - 1, y)).abs();
            n += 1.0;
        }
    }
    // Borda: largura vertical (média x 80..120) da rampa 10 %..90 % da luminância entre branco e o centro.
    let mut borda = 0.0;
    let mut nb = 0.0;
    for x in 80..=120 {
        let centro = lum(p, x, yc);
        let fundo = 255.0;
        let mut k = 0;
        for y in yc..H.min(yc + 60) {
            let f = (fundo - lum(p, x, y)) / (fundo - centro).max(1.0);
            if f > 0.1 && f < 0.9 {
                k += 1;
            }
        }
        borda += k as f32;
        nb += 1.0;
    }
    // Grão: desvio-padrão da luminância no miolo sobre o branco.
    let mut v = Vec::new();
    for y in yc - 6..=yc + 6 {
        for x in 80..=120 {
            v.push(lum(p, x, y));
        }
    }
    let m = v.iter().sum::<f32>() / v.len() as f32;
    let grao = (v.iter().map(|a| (a - m) * (a - m)).sum::<f32>() / v.len() as f32).sqrt();
    Medida {
        dentro,
        cauda_px,
        branco,
        ruido_branco: r / n,
        borda_px: borda / nb,
        grao,
    }
}

fn corre(nome: Option<&str>, v: Var, pressao: bool) -> (Vec<u8>, f64) {
    let mut t = com_a_mancha(nome);
    configura(&mut t, v);
    let ini = Instant::now();
    arrasta(&mut t, [60.0, Y], [340.0, Y], pressao);
    let ms = ini.elapsed().as_secs_f64() * 1e3;
    (t.run_full().0, ms)
}

const C_SMEAR_EM_CIMA: &[(u8, f32)] = &[(SMEAR, 1.0), (BRUSH, 1.0)];
const C_SMEAR_EM_CIMA_06: &[(u8, f32)] = &[(SMEAR, 0.6), (BRUSH, 1.0)];
const C_BRUSH_MEIO_SOBRE_SMEAR: &[(u8, f32)] = &[(BRUSH, 0.5), (SMEAR, 1.0)];
const C_BLUR_SMEAR_BRUSH: &[(u8, f32)] = &[(BLUR, 0.4), (SMEAR, 0.8), (BRUSH, 1.0)];
const C_SMEAR_BRUSH_FRACO: &[(u8, f32)] = &[(SMEAR, 1.0), (BRUSH, 0.6)];

fn variantes() -> Vec<(&'static str, Var)> {
    use ModoMisturaOpaca::*;
    vec![
        ("Off", Var::Off),
        ("Mix Blend 0.5/1/0.5", Var::Mix(WetBlend, 0.5, 1.0, 0.5)),
        ("Mix Running 0.5/1/0.5", Var::Mix(WetBlend, 0.5, 1.0, 0.5)),
        (
            "Mix Blend afinado",
            Var::Mix(WetBlend, AFIN_B.0, AFIN_B.1, AFIN_B.2),
        ),
        (
            "Mix Running afinado",
            Var::Mix(WetBlend, AFIN_R.0, AFIN_R.1, AFIN_R.2),
        ),
        ("Comp Smear1 / Brush1", Var::Comp(C_SMEAR_EM_CIMA)),
        ("Comp Smear.6 / Brush1", Var::Comp(C_SMEAR_EM_CIMA_06)),
        ("Comp Smear1 / Brush.6", Var::Comp(C_SMEAR_BRUSH_FRACO)),
        ("Comp Brush.5 / Smear1", Var::Comp(C_BRUSH_MEIO_SOBRE_SMEAR)),
        ("Comp Blur.4/Smear.8/Brush1", Var::Comp(C_BLUR_SMEAR_BRUSH)),
    ]
}

// Afinados pela varredura `sonda_cmp_varre` (preencher depois da 1.ª corrida).
const AFIN_B: (f32, f32, f32) = (0.85, 1.0, 0.25);
const AFIN_R: (f32, f32, f32) = (0.5, 1.0, 0.25);

const PINCEIS: [(Option<&str>, &str); 3] = [
    (None, "redondo"),
    (Some("G-Pen"), "gpen"),
    (Some("Sketch Pencil"), "sketch_pencil"),
];

fn linha(rot: &str, m: &Medida, ms: f64) {
    eprintln!(
        "[cmp] {rot:<30} dentro {:>3.0},{:>3.0},{:>3.0} | cauda {:>3} px | branco {:>3.0},{:>3.0},{:>3.0} | ruido {:>5.2} | borda {:>4.1} px | grao {:>5.1} | {ms:>7.1} ms",
        m.dentro[0],
        m.dentro[1],
        m.dentro[2],
        m.cauda_px,
        m.branco[0],
        m.branco[1],
        m.branco[2],
        m.ruido_branco,
        m.borda_px,
        m.grao
    );
}

/// A varredura do Color mixing (sem relógio): amount × stretch, density 1, nos dois modos.
#[test]
#[ignore = "sonda"]
fn sonda_cmp_varre() {
    use ModoMisturaOpaca::*;
    for (nome, slug) in PINCEIS {
        for modo in [WetBlend] {
            for a in [0.25f32, 0.5, 0.85] {
                for s in [0.25f32, 0.5, 0.85] {
                    for d in [0.5f32, 1.0] {
                        let (p, _) = corre(nome, Var::Mix(modo, a, d, s), false);
                        linha(&format!("{slug} {modo:?} a{a} d{d} s{s}"), &mede(&p), 0.0);
                    }
                }
            }
        }
    }
}

/// A folha de contacto + a tabela + o relógio (mínimo de 5, intercalado).
#[test]
#[ignore = "sonda"]
fn sonda_cmp_folha() {
    let pasta = std::env::var("PH2D_CMP_FOTO")
        .ok()
        .map(std::path::PathBuf::from);
    let vs = variantes();
    for (nome, slug) in PINCEIS {
        let mut melhor = vec![f64::INFINITY; vs.len()];
        for _ronda in 0..5 {
            for (i, &(_, v)) in vs.iter().enumerate() {
                let (_, ms) = corre(nome, v, false);
                melhor[i] = melhor[i].min(ms);
            }
        }
        for pressao in [false, true] {
            for (i, &(rot, v)) in vs.iter().enumerate() {
                let (p, _) = corre(nome, v, pressao);
                let tag = if pressao { "P" } else { "-" };
                linha(&format!("{slug} {tag} {rot}"), &mede(&p), melhor[i]);
                if let Some(d) = &pasta {
                    let f = d.join(format!(
                        "{slug}_{}_{i:02}.rgba",
                        if pressao { "p" } else { "c" }
                    ));
                    std::fs::write(f, &p).expect("escreve");
                }
            }
        }
    }
    if let Some(d) = &pasta {
        let rot: Vec<&str> = vs.iter().map(|v| v.0).collect();
        std::fs::write(d.join("rotulos.txt"), rot.join("\n")).expect("rotulos");
    }
}

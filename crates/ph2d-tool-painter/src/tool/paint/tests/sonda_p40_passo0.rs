//! Prompt 40, passo 0 (15/10): o dono pintou com a caixa Continuous LIGADA e Follow Off e viu a grelha
//! por dab. Varre, num lote, as condições do pincel e do gesto e acusa onde a caixa está ligada e a
//! cápsula NÃO age (`Δ(com × sem) ≈ 0` enquanto `Δ(sem × denso) ≫ 0`).
//! `cargo test -p ph2d-tool-painter --profile smoke --lib sonda_p40 -- --ignored --nocapture`

use super::super::media::PaintMedia;
use super::capsula::verde_laranja;
use super::*;
use ph2d_painter_brush::{BrushSpec, StrokeMethod, TextureKind};

const WW: u32 = 720;
const HH: u32 = 280;
const XA: f32 = 80.0;
const XB: f32 = 640.0;

#[derive(Clone, Copy)]
struct Caso {
    nome: &'static str,
    tam: f32,
    eventos: usize,
    tiques: usize,
    esq: bool,
    pressao: Option<(f32, f32)>,
    extra: fn(&mut BrushSpec),
}

const BASE: Caso = Caso {
    nome: "base 40 px · 112 ev",
    tam: 40.0,
    eventos: 112,
    tiques: 1,
    esq: false,
    pressao: None,
    extra: |_| {},
};

fn corre(c: &Caso, sp: f32, cont: bool) -> (Vec<u8>, Vec<f32>) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (WW * HH * 4) as usize], WW, HH);
    t.set_paint_media(PaintMedia::Impasto);
    t.set_brush_color_srgb8([20, 30, 120]);
    t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
    t.set_brush_size_px(c.tam);
    t.set_shape_color_ramp(verde_laranja());
    t.set_shape_ramp_enabled(true);
    (c.extra)(&mut t.paint.brush);
    t.set_brush_spacing(sp);
    t.paint.brush.continuous_spacing = cont;
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    let ev = |pos: [f32; 2], pressure: f32, phase| CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    };
    let at = |k: usize| {
        let f = k as f32 / c.eventos as f32;
        let f = if c.esq { 1.0 - f } else { f };
        let p = c.pressao.map_or(1.0, |(a, b)| {
            a + (b - a) * (f * std::f32::consts::PI * 3.0).sin().abs()
        });
        ([XA + (XB - XA) * f, HH as f32 / 2.0], p)
    };
    let (p0, q0) = at(0);
    t.on_canvas_pointer(ev(p0, q0, PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=c.eventos {
        let (p, q) = at(k);
        t.on_canvas_pointer(ev(p, q, PointerPhase::Move));
        for _ in 0..c.tiques {
            t.on_tick(1000.0 / 60.0 / c.tiques as f32);
        }
    }
    let (p1, q1) = at(c.eventos);
    t.on_canvas_pointer(ev(p1, q1, PointerPhase::Up));
    for _ in 0..30 {
        t.on_tick(1000.0 / 60.0);
    }
    let mut acesa = t.canvas_rgba.to_vec();
    t.apply_impasto_light(
        &mut acesa,
        Region {
            x: 0,
            y: 0,
            w: WW,
            h: HH,
        },
    );
    let h = t
        .layers
        .active()
        .and_then(|id| t.layer_height_view(id))
        .unwrap_or_default();
    (acesa, h)
}

fn miolo(h: &[f32], r: f32) -> Vec<usize> {
    let (x0, x1) = (
        (XA + 2.0 * r + 10.0) as usize,
        (XB - 2.0 * r - 10.0) as usize,
    );
    (0..(WW * HH) as usize)
        .filter(|&i| {
            let x = i % WW as usize;
            x >= x0 && x < x1 && h.get(i).is_some_and(|v| *v > 1e-4)
        })
        .collect()
}

fn dif(a: &[u8], b: &[u8], m: &[usize]) -> f32 {
    let s: f32 = m
        .iter()
        .map(|&i| {
            (0..3)
                .map(|k| (f32::from(a[i * 4 + k]) - f32::from(b[i * 4 + k])).abs())
                .sum::<f32>()
                / 3.0
        })
        .sum();
    s / m.len().max(1) as f32 / 255.0
}

#[test]
#[ignore = "sonda do prompt 40 passo 0: onde a caixa está ligada e a cápsula não age"]
fn sonda_p40_onde_a_capsula_morre() {
    let casos = [
        BASE,
        Caso {
            nome: "lento 1 px/ev",
            eventos: 560,
            ..BASE
        },
        Caso {
            nome: "muito lento 0,5 px/ev",
            eventos: 1120,
            ..BASE
        },
        Caso {
            nome: "lento 1 px/ev, 4 tiques",
            eventos: 560,
            tiques: 4,
            ..BASE
        },
        Caso {
            nome: "rápido 14 ev",
            eventos: 14,
            ..BASE
        },
        Caso {
            nome: "direita→esquerda",
            esq: true,
            ..BASE
        },
        Caso {
            nome: "caneta 0,2–1",
            pressao: Some((0.2, 1.0)),
            ..BASE
        },
        Caso {
            nome: "caneta 0,5 fixa",
            pressao: Some((0.5, 0.5)),
            ..BASE
        },
        Caso {
            nome: "10 px",
            tam: 10.0,
            ..BASE
        },
        Caso {
            nome: "200 px",
            tam: 200.0,
            ..BASE
        },
        Caso {
            nome: "Spacing 0,1",
            extra: |b| b.spacing = 0.1,
            ..BASE
        },
        Caso {
            nome: "Spacing 0,5",
            extra: |b| b.spacing = 0.5,
            ..BASE
        },
        Caso {
            nome: "Spacing 1,0",
            extra: |b| b.spacing = 1.0,
            ..BASE
        },
        Caso {
            nome: "Dots",
            extra: |b| b.stroke_method = StrokeMethod::Dots,
            ..BASE
        },
        Caso {
            nome: "Airbrush",
            extra: |b| b.stroke_method = StrokeMethod::Airbrush,
            ..BASE
        },
        Caso {
            nome: "Line",
            extra: |b| b.stroke_method = StrokeMethod::Line,
            ..BASE
        },
        Caso {
            nome: "Stabilizer 0,5",
            extra: |b| b.stabilizer = 0.5,
            ..BASE
        },
        Caso {
            nome: "Stabilizer 0,9",
            extra: |b| b.stabilizer = 0.9,
            ..BASE
        },
        Caso {
            nome: "Input samples 4",
            extra: |b| b.input_samples = 4,
            ..BASE
        },
        Caso {
            nome: "Edge to edge",
            extra: |b| b.edge_to_edge = true,
            ..BASE
        },
        Caso {
            nome: "Jitter 0,3",
            extra: |b| b.jitter = 0.3,
            ..BASE
        },
        Caso {
            nome: "Jitter Spacing 0,5",
            extra: |b| b.jitter_spacing = 0.5,
            ..BASE
        },
        Caso {
            nome: "Spray 3",
            extra: |b| b.spray_count = 3,
            ..BASE
        },
        Caso {
            nome: "Simetria",
            extra: |b| b.symmetry.enabled = true,
            ..BASE
        },
        Caso {
            nome: "Strength 0,5",
            extra: |b| b.strength = 0.5,
            ..BASE
        },
        Caso {
            nome: "Accumulate off",
            extra: |b| b.accumulate = false,
            ..BASE
        },
        Caso {
            nome: "Space Attenuation",
            extra: |b| b.space_attenuation = true,
            ..BASE
        },
        Caso {
            nome: "Speed",
            eventos: 30,
            extra: |b| b.line_kind = ph2d_painter_brush::line_kind::LineKind::Speed,
            ..BASE
        },
        Caso {
            nome: "Sketchy",
            extra: |b| b.line_kind = ph2d_painter_brush::line_kind::LineKind::Sketchy,
            ..BASE
        },
        Caso {
            nome: "Rake",
            extra: |b| b.shape.rake = true,
            ..BASE
        },
        Caso {
            nome: "Flow",
            extra: |b| b.shape.flow = true,
            ..BASE
        },
        Caso {
            nome: "Flatten 0,5",
            extra: |b| b.dab_flatten = 0.5,
            ..BASE
        },
        Caso {
            nome: "Jitter Scale 0,3",
            extra: |b| b.jitter_scale = 0.3,
            ..BASE
        },
        Caso {
            nome: "Push 0,5",
            extra: |b| b.impasto_push = 0.5,
            ..BASE
        },
        Caso {
            nome: "Dash",
            extra: |b| {
                b.dash_samples = 4;
                b.dash_ratio = 0.5;
            },
            ..BASE
        },
        Caso {
            nome: "Rough",
            extra: |b| b.rough_amount = 0.5,
            ..BASE
        },
    ];
    println!("| caso | Δ(com × denso) | Δ(sem × denso) | Δ(com × sem) | veredito |");
    println!("|---|---:|---:|---:|---|");
    for c in casos {
        let (aa, ha) = corre(&c, 0.025, false);
        let spc = {
            let mut b = BrushSpec::default();
            b.spacing = 0.25;
            (c.extra)(&mut b);
            b.spacing
        };
        let (at, _) = corre(&c, spc, true);
        let (an, _) = corre(&c, spc, false);
        let m = miolo(&ha, c.tam / 2.0);
        let (dc, ds, dcs) = (dif(&at, &aa, &m), dif(&an, &aa, &m), dif(&at, &an, &m));
        let v = if ds > 0.01 && dcs < 0.1 * ds {
            "⛔ MORTA"
        } else if dc < 0.5 * ds {
            "age"
        } else {
            "⚠ age mal"
        };
        println!("| {} | {dc:.4} | {ds:.4} | {dcs:.4} | {v} |", c.nome);
    }
}

/// O Impasto com pressão: a culpa é da cor ou do relevo, do raio ou da cobertura?
#[test]
#[ignore = "sonda do prompt 40: o Impasto com pressão"]
fn sonda_p40_impasto_pressao() {
    use super::capsula::{acesa, dif_cor, dif_relevo, relevo};
    use super::capsula_estado::{Caso, Follow, traco_tool};
    let c = Caso {
        impasto: true,
        curva: false,
        follow: Follow::Off,
        pressao: true,
    };
    let vars: [(&str, fn(&mut PainterTool)); 4] = [
        ("os dois", |_| {}),
        ("só o raio", |t| t.paint.dynamics.strength_pressure = false),
        ("só a cobertura", |t| {
            t.paint.dynamics.size_pressure = false
        }),
        ("nenhum", |t| {
            t.paint.dynamics.size_pressure = false;
            t.paint.dynamics.strength_pressure = false;
        }),
    ];
    for linear in [false, true] {
        if linear {
            super::capsula_estado::PRESSAO.with(|p| p.set(|k| 0.4 + 0.6 * k as f32 / 72.0));
        }
        println!("pressão linear: {linear}");
        for (nome, f) in vars {
            let a = traco_tool(c, 0.025, false, f);
            let k = traco_tool(c, 0.25, true, f);
            let (ca, ck) = (a.canvas_rgba.to_vec(), k.canvas_rgba.to_vec());
            println!(
                "{nome}: Δacesa {:.4} · Δcru {:.4} · Δrelevo {:.4}",
                dif_cor(&acesa(&k, &ck), &acesa(&a, &ca), 20.0),
                dif_cor(&ck, &ca, 20.0),
                dif_relevo(&relevo(&k), &relevo(&a), 20.0).0
            );
        }
    }
}

/// Em que modo a caixa age: um traço de base no Impasto (listras de papel), depois o modo `m` a 0,25 com
/// e sem a caixa — quantos texels da tela e do relevo mudam.
#[test]
#[ignore = "sonda do prompt 40 #6: os modos que leem a cápsula"]
fn sonda_p40_modos() {
    let corre = |modo: &str, cont: bool| {
        let mut t = PainterTool::default();
        let mut src = vec![255u8; (WW * HH * 4) as usize];
        for (i, p) in src.chunks_mut(4).enumerate() {
            if (i % WW as usize / 12) % 2 == 0 {
                p[..3].copy_from_slice(&[200, 60, 40]);
            }
        }
        t.set_source(src, WW, HH);
        t.set_paint_media(PaintMedia::Impasto);
        t.set_brush_color_srgb8([20, 30, 120]);
        t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
        t.set_brush_size_px(40.0);
        t.set_brush_spacing(0.25);
        let seed = t.paint.brush;
        t.paint.brush_by_mode.fill(seed);
        let risca = |t: &mut PainterTool, y0: f32, y1: f32| {
            let ev = |k: usize, phase| CanvasPointer {
                pos: [
                    XA + (XB - XA) * k as f32 / 112.0,
                    y0 + (y1 - y0) * k as f32 / 112.0,
                ],
                pressure: 1.0,
                tilt: [0.0, 0.0],
                phase,
            };
            t.on_canvas_pointer(ev(0, PointerPhase::Down));
            t.on_tick(1000.0 / 60.0);
            for k in 1..=112 {
                t.on_canvas_pointer(ev(k, PointerPhase::Move));
                t.on_tick(1000.0 / 60.0);
            }
            t.on_canvas_pointer(ev(112, PointerPhase::Up));
            for _ in 0..10 {
                t.on_tick(1000.0 / 60.0);
            }
        };
        risca(&mut t, 140.0, 140.0);
        t.set_paint_tool_mode(modo);
        t.paint.brush.continuous_spacing = cont;
        t.paint.brush.spacing = 0.25;
        risca(&mut t, 100.0, 180.0);
        let h = t
            .layers
            .active()
            .and_then(|id| t.layer_height_view(id))
            .unwrap_or_default();
        let mut tela = t.canvas_rgba.to_vec();
        tela.extend_from_slice(&t.paint.mask_scratch_rgba);
        (tela, h)
    };
    for modo in [
        "paint", "eraser", "smear", "blur", "clone", "mask", "inpaint", "liquify", "sculpt",
        "knife",
    ] {
        let (ca, ha) = corre(modo, false);
        let (cb, hb) = corre(modo, true);
        let dc = ca
            .chunks(4)
            .zip(cb.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        let dh = ha
            .iter()
            .zip(&hb)
            .filter(|(a, b)| a.to_bits() != b.to_bits())
            .count();
        println!("{modo}: tela {dc} texels · relevo {dh} texels");
    }
}

//! AUDITORIA lente 1 (15/10): o portão `o_impasto_com_continuous_iguala_o_spacing_pequeno_na_tela_iluminada`
//! está verde e o dono ainda vê a grelha por dab. Varia o que a fixtura do portão fixa (tamanho, pressão,
//! eventos por quadro, curva, Shape, Push, Smooth Edges, mapeamento) e mede a tela ACESA contra o traço denso.
//! `PH2D_SONDA_OUT=<pasta> cargo test -p ph2d-tool-painter --profile smoke --lib auditoria_l1 -- --ignored --nocapture`

use super::super::media::PaintMedia;
use super::capsula::verde_laranja;
use super::*;
use ph2d_painter_brush::TextureKind;
use ph2d_painter_brush::texture::TextureMapping;

const WW: u32 = 720;
const HH: u32 = 280;
const XA: f32 = 80.0;
const XB: f32 = 640.0;

#[derive(Clone, Copy, Debug)]
struct Caso {
    nome: &'static str,
    tam: f32,
    pressao: Option<(f32, f32)>,
    eventos: usize,
    curva: f32,
    forma: f32,
    push: f32,
    smooth: Option<bool>,
    mapa: TextureMapping,
    rampa: bool,
    jitter_scale: f32,
    extra: fn(&mut ph2d_painter_brush::BrushSpec),
    doc: bool,
}

const BASE: Caso = Caso {
    nome: "portao (40 px, 72 ev)",
    tam: 40.0,
    pressao: None,
    eventos: 112,
    curva: 0.0,
    forma: 0.0,
    push: 0.0,
    smooth: None,
    mapa: TextureMapping::ViewPlane,
    rampa: true,
    jitter_scale: 0.0,
    extra: |_| {},
    doc: false,
};

fn caminho(c: &Caso, k: usize) -> ([f32; 2], f32) {
    let f = k as f32 / c.eventos as f32;
    let x = XA + (XB - XA) * f;
    let y = HH as f32 / 2.0 + c.curva * (f * std::f32::consts::PI * 2.0).sin();
    let p = c.pressao.map_or(1.0, |(a, b)| {
        a + (b - a) * (f * std::f32::consts::PI * 3.0).sin().abs()
    });
    ([x, y], p)
}

fn corre(c: &Caso, sp: f32, cont: bool) -> (PainterTool, Vec<u8>, Vec<f32>) {
    let mut t = PainterTool::default();
    if c.doc {
        t.bind_document(1, vec![255u8; (WW * HH * 4) as usize], WW, HH);
    } else {
        t.set_source(vec![255u8; (WW * HH * 4) as usize], WW, HH);
    }
    t.set_paint_media(PaintMedia::Impasto);
    t.set_brush_color_srgb8([20, 30, 120]);
    t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
    t.set_brush_size_px(c.tam);
    t.set_brush_spacing(sp);
    t.paint.brush.continuous_spacing = cont;
    t.paint.brush.shape.mapping = c.mapa;
    if c.forma > 0.0 {
        t.paint.brush.shape.size = [c.forma; 2];
    }
    if c.push > 0.0 {
        t.set_brush_impasto_push(c.push);
    }
    if let Some(s) = c.smooth {
        t.paint.brush.impasto_smooth_edges = s;
    }
    t.paint.brush.jitter_scale = c.jitter_scale;
    (c.extra)(&mut t.paint.brush);
    if c.rampa {
        t.set_shape_color_ramp(verde_laranja());
    }
    t.set_shape_ramp_enabled(c.rampa);
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    let ev = |pos: [f32; 2], pressure: f32, phase| CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    };
    let (p0, q0) = caminho(c, 0);
    t.on_canvas_pointer(ev(p0, q0, PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=c.eventos {
        let (p, q) = caminho(c, k);
        t.on_canvas_pointer(ev(p, q, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    let (p1, q1) = caminho(c, c.eventos);
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
    (t, acesa, h)
}

/// O miolo: texels com relevo no ALVO, longe das pontas.
fn miolo(h_alvo: &[f32], r: f32) -> Vec<usize> {
    let (x0, x1) = (
        (XA + 2.0 * r + 10.0) as usize,
        (XB - 2.0 * r - 10.0) as usize,
    );
    (0..(WW * HH) as usize)
        .filter(|&i| {
            let x = i % WW as usize;
            x >= x0 && x < x1 && h_alvo.get(i).is_some_and(|v| *v > 1e-4)
        })
        .collect()
}

fn dif_rgb(a: &[u8], b: &[u8], m: &[usize]) -> f32 {
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

fn dif_h(a: &[f32], t: &[f32], m: &[usize]) -> f32 {
    if a.is_empty() || t.is_empty() {
        return f32::NAN;
    }
    let st: f32 = m.iter().map(|&i| t[i].abs()).sum();
    let d: f32 = m.iter().map(|&i| (a[i] - t[i]).abs()).sum();
    d / st.max(1e-9)
}

fn grava(dir: &str, nome: &str, px: &[u8]) {
    let mut ppm = format!("P6 {WW} {HH} 255\n").into_bytes();
    for k in 0..(WW * HH) as usize {
        ppm.extend_from_slice(&px[k * 4..k * 4 + 3]);
    }
    std::fs::write(format!("{dir}/{nome}.ppm"), ppm).expect("grava");
}

#[test]
#[ignore = "auditoria lente 1: o Impasto com Continuous fora da fixtura do portão"]
fn auditoria_l1_impasto_continuous() {
    let out = std::env::var("PH2D_SONDA_OUT").ok();
    let casos = [
        BASE,
        Caso {
            nome: "120 px",
            tam: 120.0,
            ..BASE
        },
        Caso {
            nome: "120 px, 14 ev (rato rápido)",
            tam: 120.0,
            eventos: 14,
            ..BASE
        },
        Caso {
            nome: "40 px, 14 ev",
            eventos: 14,
            ..BASE
        },
        Caso {
            nome: "120 px, pressão 0,3–1",
            tam: 120.0,
            pressao: Some((0.3, 1.0)),
            ..BASE
        },
        Caso {
            nome: "120 px, curva 40",
            tam: 120.0,
            curva: 40.0,
            ..BASE
        },
        Caso {
            nome: "120 px, Shape size 3",
            tam: 120.0,
            forma: 3.0,
            ..BASE
        },
        Caso {
            nome: "120 px, Push 0,5",
            tam: 120.0,
            push: 0.5,
            ..BASE
        },
        Caso {
            nome: "120 px, Smooth off",
            tam: 120.0,
            smooth: Some(false),
            ..BASE
        },
        Caso {
            nome: "120 px, mapa Random",
            tam: 120.0,
            mapa: TextureMapping::Random,
            ..BASE
        },
        Caso {
            nome: "120 px, sem rampa",
            tam: 120.0,
            rampa: false,
            ..BASE
        },
        Caso {
            nome: "120 px, Jitter Scale 0,3",
            tam: 120.0,
            jitter_scale: 0.3,
            ..BASE
        },
        Caso {
            nome: "120 px, pressão 0,6 fixa",
            tam: 120.0,
            pressao: Some((0.6, 0.6)),
            ..BASE
        },
        Caso {
            nome: "120 px, Rake",
            tam: 120.0,
            extra: |b| b.shape.rake = true,
            ..BASE
        },
        Caso {
            nome: "120 px, Flow",
            tam: 120.0,
            extra: |b| b.shape.flow = true,
            ..BASE
        },
        Caso {
            nome: "120 px, Rake+Flow",
            tam: 120.0,
            extra: |b| {
                b.shape.rake = true;
                b.shape.flow = true;
            },
            ..BASE
        },
        Caso {
            nome: "120 px, Rake, curva 40",
            tam: 120.0,
            curva: 40.0,
            extra: |b| b.shape.rake = true,
            ..BASE
        },
        Caso {
            nome: "120 px, Flow, curva 40",
            tam: 120.0,
            curva: 40.0,
            extra: |b| b.shape.flow = true,
            ..BASE
        },
        Caso {
            nome: "120 px, ângulo 30",
            tam: 120.0,
            extra: |b| b.shape.angle_deg = 30,
            ..BASE
        },
        Caso {
            nome: "120 px, Jitter Rotate 0,3",
            tam: 120.0,
            extra: |b| b.jitter_rotate = 0.3,
            ..BASE
        },
        Caso {
            nome: "120 px, Stabilizer 0,5",
            tam: 120.0,
            extra: |b| b.stabilizer = 0.5,
            ..BASE
        },
        Caso {
            nome: "120 px, Airbrush",
            tam: 120.0,
            extra: |b| b.stroke_method = ph2d_painter_brush::StrokeMethod::Airbrush,
            ..BASE
        },
        Caso {
            nome: "120 px, spray 3",
            tam: 120.0,
            extra: |b| b.spray_count = 3,
            ..BASE
        },
        Caso {
            nome: "40 px, DOCUMENTO (papel + camada vazia)",
            doc: true,
            ..BASE
        },
        Caso {
            nome: "120 px, Line Speed, 14 ev",
            tam: 120.0,
            eventos: 14,
            extra: |b| b.line_kind = ph2d_painter_brush::line_kind::LineKind::Speed,
            ..BASE
        },
        Caso {
            nome: "120 px, DOCUMENTO (papel + camada vazia)",
            tam: 120.0,
            doc: true,
            ..BASE
        },
        Caso {
            nome: "120 px, DOCUMENTO, 14 ev",
            tam: 120.0,
            eventos: 14,
            doc: true,
            ..BASE
        },
    ];
    println!(
        "| caso | Δacesa (0,25+C × 0,025) | Δrelevo | Δacesa (0,25 sem C × 0,025) | Δ(sem C × com C) |"
    );
    println!("|---|---:|---:|---:|---:|");
    for c in casos {
        let (_, aa, ha) = corre(&c, 0.025, false);
        let (_, at, ht) = corre(&c, 0.25, true);
        let (_, an, _) = corre(&c, 0.25, false);
        let m = miolo(&ha, c.tam / 2.0);
        println!(
            "| {} | {:.4} | {:.4} | {:.4} | {:.4} |",
            c.nome,
            dif_rgb(&at, &aa, &m),
            dif_h(&ht, &ha, &m),
            dif_rgb(&an, &aa, &m),
            dif_rgb(&an, &at, &m)
        );
        if let Some(d) = &out {
            let n: String = c.nome.chars().filter(char::is_ascii_alphanumeric).collect();
            grava(d, &format!("{n}_denso"), &aa);
            grava(d, &format!("{n}_cont"), &at);
            grava(d, &format!("{n}_semc"), &an);
        }
    }
}

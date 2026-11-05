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
    traco_com(p, |t| {
        if papel {
            t.toggle_brush_grain_same_as_paper();
        }
    })
}

fn traco_com(
    p: &ph2d_brush_presets::Preset,
    muda: impl FnOnce(&mut PainterTool),
) -> ([f64; 4], PainterTool) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.wet_preset_aplicar_volumes(&p.parametros.volumes);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([30, 60, 200]);
    muda(&mut t);
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

/// Os pincéis com Grain próprio: o papel toma o lugar dele à média dele, nem mais tinta nem menos.
#[test]
fn o_same_as_paper_sobre_grain_proprio_nao_muda_a_quantidade() {
    for nome in [
        "Dabber 4",
        "Thick Impasto",
        "Knife Smooth 3",
        "Knife Textured",
        "Knife Dry",
        "Knife Grainy",
    ] {
        let e = crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato(nome);
        let (s, _) = traco(&e, false);
        let (c, _) = traco(&e, true);
        let massa = c[0] / s[0];
        println!("{nome}: massa com Grain próprio {massa:.3}");
        assert!((0.75..1.25).contains(&massa), "{nome}: massa {massa:.3}×");
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

/// As Grain Colors com o papel: o grão só dá a cor, a quantidade continua a das cerdas (smoke 04/11).
#[test]
fn o_same_as_paper_com_grain_colors_nao_muda_a_quantidade() {
    use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
    for nome in ["Wet Drip", "Wet Round", "Wet Dry Brush", "Filbert"] {
        let e = crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato(nome);
        let (s, _) = traco(&e, false);
        let (c, _) = traco_com(&e, |t| {
            t.toggle_brush_grain_same_as_paper();
            t.set_texture_ramp(ColorRamp::new(
                vec![
                    RampStop::new(0.0, [0.0, 1.0, 0.0, 1.0]),
                    RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
                ],
                RampColorMode::Rgb,
                RampInterp::Linear,
            ));
            t.set_texture_ramp_enabled(true);
        });
        let massa = c[0] / s[0];
        println!("{nome}: massa com as Grain Colors {massa:.3}");
        assert!((0.9..1.1).contains(&massa), "{nome}: massa {massa:.3}×");
    }
}

/// **Sonda do cordão** (smoke 04/11): a seção do traço simples. `pincel|variante|quando|` miolo e pico da borda, em
/// massa, altura de água e escurecimento do ecrã. PNG por variante em `PH2D_SONDA_OUT`.
#[test]
#[ignore = "sonda: o cordão na borda do traço simples"]
fn sonda_cordao_na_borda() {
    use ph2d_wet_paint::tuning::Knob;
    let dir = std::env::var("PH2D_SONDA_OUT").ok();
    println!(
        "pincel|variante|quando|massa_miolo|massa_borda|razao|tela_miolo|tela_borda|razao|largura_px"
    );
    for nome in ["Wet Liner", "Wet Wash", "Wet Round", "Gouache Opaque"] {
        let e = crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato(nome);
        for (v, f) in [
            ("fabrica", None),
            ("body0", Some((Knob::Body, 0.0))),
            ("edge0", Some((Knob::EdgeDarkening, 0.0))),
            ("capilar0", Some((Knob::Capillary, 0.0))),
        ] {
            let (_, mut t) = traco_com(&e, |t| {
                t.set_brush_color_srgb8([190, 30, 30]);
                if let Some((k, x)) = f {
                    t.paint.wetpaint.knobs.set(k, x);
                }
            });
            for (q, n) in [("fim", 0usize), ("4s", 240), ("seco", 0)] {
                for _ in 0..n {
                    t.on_tick(1000.0 / 60.0);
                }
                if q == "seco" {
                    t.wetpaint_dry_canvas();
                    for _ in 0..4 {
                        t.on_tick(1000.0 / 60.0);
                    }
                }
                let g = t
                    .paint
                    .wetpaint
                    .session
                    .as_ref()
                    .unwrap()
                    .engine
                    .active_grid();
                let r = t.paint.wetpaint.session.as_ref().unwrap().ratio as usize;
                let (mut ms, mut ts) = (vec![], vec![]);
                for y in 80..=160usize {
                    let c = 200 / r + 1 + (y / r + 1) * g.s;
                    ms.push(f64::from(g.sett[c]) + f64::from(g.susp[c]));
                    ts.push(255.0 - f64::from(t.canvas_rgba[(y * W + 200) * 4 + 1]));
                }
                let lim = |v: &[f64]| {
                    let top = v.iter().copied().fold(0.0, f64::max);
                    let i0 = v.iter().position(|x| *x > 0.05 * top).unwrap_or(0);
                    let i1 = v.iter().rposition(|x| *x > 0.05 * top).unwrap_or(0);
                    (top, i0, i1)
                };
                let (mt, i0, i1) = lim(&ts);
                let (mm, _, _) = lim(&ms);
                let meio = (i0 + i1) / 2;
                let med = |v: &[f64]| v[meio - 3..=meio + 3].iter().sum::<f64>() / 7.0;
                let (mmio, tmio) = (med(&ms), med(&ts));
                println!(
                    "{nome}|{v}|{q}|{mmio:.0}|{mm:.0}|{:.2}|{tmio:.1}|{mt:.1}|{:.2}|{}",
                    mm / mmio.max(1e-9),
                    mt / tmio.max(1e-9),
                    i1 - i0 + 1
                );
                if let (Some(d), "seco") = (&dir, q) {
                    let mut b = format!("P6 {W} {H} 255\n").into_bytes();
                    for p in t.canvas_rgba.chunks(4) {
                        let a = u32::from(p[3]);
                        for k in 0..3 {
                            b.push(((u32::from(p[k]) * a + 255 * (255 - a)) / 255) as u8);
                        }
                    }
                    std::fs::write(format!("{d}/{}_{v}.ppm", nome.replace(' ', "_")), b).unwrap();
                }
            }
        }
    }
}

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

/// Os knobs dos FIOS (W2, `ph2d_wet_paint::bristles_fios`): a cor e a pressão de cada fio são do pincel, não das
/// leis que estas réguas medem — quem usa um pincel de fábrica como VEÍCULO tira-os (o tapete de antes).
pub(super) const KNOBS_DOS_FIOS: [Knob; 12] = [
    Knob::BristlePressure,
    Knob::BristleNoiseLength,
    Knob::BristleLaneWidth,
    Knob::BristleWave,
    Knob::BristleWaveLength,
    Knob::BristleOscillation,
    Knob::BristleSpan,
    Knob::BristleSpanLength,
    Knob::BristleNoiseHue,
    Knob::BristleNoiseSat,
    Knob::BristleNoiseLight,
    Knob::BristleLift,
];

/// [`KNOBS_DOS_FIOS`] de volta ao neutro.
pub(super) fn sem_os_fios(t: &mut PainterTool) {
    // … e sem o RELEVO deles (W3): a luz que o acende também não é destas réguas.
    for k in KNOBS_DOS_FIOS.into_iter().chain([Knob::BristleHeight]) {
        t.paint
            .wetpaint
            .knobs
            .set(k, ph2d_wet_paint::tuning::KNOB_DEFS[k as usize].default);
    }
}

/// A tela do Oil Bristle SEM relevo nem fios (o realce do Emboss e a cor de cada fio não são desta régua) com o
/// claro seco e, se `escuro`, o escuro a `forca` × o pigmento por cima (soltado).
fn tela(claro: [u8; 3], escuro: Option<([u8; 3], f64)>) -> PainterTool {
    let b = super::mistura_wet_drip_tests::biblioteca();
    let e = &b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Oil Bristle")
        .expect("o pincel de fábrica")
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(e).is_empty());
    t.paint.wetpaint.knobs.set(Knob::Emboss, 0.0);
    sem_os_fios(&mut t);
    t.set_brush_size_px(40.0);
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
    if let Some((cor, forca)) = escuro {
        t.set_brush_color_srgb8(cor);
        let ppd = t.paint.wetpaint.knobs.get(Knob::PigmentPerDab);
        t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, ppd * forca);
        t.on_canvas_pointer(p(120.0, 10.0, PointerPhase::Down));
        for k in 1..=48 {
            t.on_canvas_pointer(p(120.0, 10.0 + 3.0 * k as f32, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(p(120.0, 154.0, PointerPhase::Up));
        ticks(&mut t, 60);
    }
    t
}

/// **Bug #50 (dono 12/10): a orla «esbranquiçada» do azul escuro fino sobre o azul claro seco** — (71,129,184)
/// entre (35,125,207) e (19,74,125); depois do Undo (o claro na base congelada) não havia. Onde o claro estava
/// CHEIO (o pixel de antes a ≤ 6 níveis dele) e a tinta não diminuiu, nenhum canal com o escuro por cima passa da mais clara das duas
/// tintas (± 4 do arredondamento: medido 3). Antes da caixa: 27 níveis. (Onde a água afina o claro, clarear para o papel é verdade — fora daqui.)
#[test]
fn o_escuro_fino_sobre_o_claro_seco_nao_deixa_orla_clara() {
    const CLARO: [u8; 3] = [35, 125, 207];
    const ESCURO: [u8; 3] = [19, 74, 125];
    // A massa por pixel: onde o traço escuro AFINOU o claro, clarear para o papel é verdade — fora da régua.
    let massa = |t: &PainterTool| -> Vec<f32> {
        let s = t
            .paint
            .wetpaint
            .session
            .as_ref()
            .expect("a sessão do Wet Paint");
        let (g, r) = (s.engine.active_grid(), s.ratio as usize);
        (0..W * H)
            .map(|i| {
                let c = (i % W) / r + 1 + ((i / W) / r + 1) * g.s;
                g.sett[c] + g.susp[c]
            })
            .collect()
    };
    let t0 = tela(CLARO, None);
    let (antes, m0) = (t0.canvas_rgba.to_vec(), massa(&t0));
    let cheio = |a: &[u8]| (0..3).all(|k| (i32::from(a[k]) - i32::from(CLARO[k])).abs() <= 6);
    for forca in [1.0, 0.3] {
        let t = tela(CLARO, Some((ESCURO, forca)));
        let depois = t.canvas_rgba.to_vec();
        let m1 = massa(&t);
        let (mut pior, mut onde, mut mudou) = (0i32, 0usize, 0usize);
        for (i, (a, d)) in antes.chunks(4).zip(depois.chunks(4)).enumerate() {
            if a == d || !cheio(a) || m1[i] < m0[i] * 0.98 {
                continue;
            }
            mudou += 1;
            for k in 0..3 {
                let fora = i32::from(d[k]) - i32::from(CLARO[k].max(ESCURO[k]));
                if fora > pior {
                    (pior, onde) = (fora, i);
                }
            }
        }
        assert!(
            mudou > 500,
            "forca {forca}: o escuro não pintou sobre o claro ({mudou} px)"
        );
        assert!(
            pior <= 4,
            "forca {forca}: a orla passou {pior} níveis da mais clara das tintas em ({}, {}) = {:?}",
            onde % W,
            onde / W,
            &depois[onde * 4..onde * 4 + 3]
        );
    }
}

//! **A queda da água na ladeira do óleo** (smoke do dono 30/10): um Wet Flat pintado POR CIMA da crista de um Dabber
//! assentado escorre ladeira abaixo — (a) devagar com pouca água, depressa muito diluído; (b) igual com o Real Oil
//! ligado no Wet Flat (o óleo de ANTES continua a ser relevo para a água de um traço novo).

use super::agua_sobre_oleo_tests::{com_oleo, ponteiro, preset, relato};
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_wet_paint::tuning::Knob;

/// O tique da tela.
const TIQUE: f32 = 1000.0 / 60.0;

fn massas(t: &mut PainterTool) -> Vec<f32> {
    t.wet_bring_home();
    let g = &t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão")
        .engine
        .layers[0]
        .grid;
    (0..g.cells).map(|i| g.sett[i] + g.susp[i]).collect()
}

/// A massa nova FORA do óleo (células com ≤ 1000 de massa antes) — o que desceu a ladeira — nos tiques `marcas`
/// depois de soltar um Wet Flat (água `escala` × a do preset, Real Oil `oleo`) ao longo da crista do Dabber.
pub(super) fn queda(escala: f64, oleo: bool, marcas: &[usize]) -> Vec<f64> {
    let mut t = com_oleo();
    let antes = massas(&mut t);
    let mut p = preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png");
    let w = p.parametros.volumes.water.unwrap_or(1.0);
    p.parametros.volumes.water = Some(w * escala);
    t.preset_aplicar(&p).expect("o meio");
    t.set_wet_oleo_real_ligado(oleo);
    t.set_brush_size_px(12.0);
    t.set_brush_color_srgb8([240, 130, 60]);
    t.on_canvas_pointer(ponteiro(60.0, 90.0, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(ponteiro(60.0 + 4.0 * k as f32, 90.0, PointerPhase::Move));
        t.on_tick(TIQUE);
    }
    t.on_canvas_pointer(ponteiro(180.0, 90.0, PointerPhase::Up));
    {
        t.wet_bring_home();
        let g = &t
            .paint
            .wetpaint
            .session
            .as_ref()
            .expect("a sessão")
            .engine
            .layers[0]
            .grid;
        let f: Vec<f32> = (0..g.cells)
            .filter(|&i| antes[i] > 1000.0 && g.film[i] > 0.0)
            .map(|i| g.film[i])
            .collect();
        let m = f.iter().sum::<f32>() / f.len().max(1) as f32;
        let mx = f.iter().copied().fold(0.0f32, f32::max);
        relato(format!(
            "  água ×{escala} óleo {oleo}: film no óleo ao soltar média {m:.3} máx {mx:.3} ({} células)",
            f.len()
        ));
    }
    let mut out = Vec::new();
    let mut feito = 0;
    for &m in marcas {
        for _ in feito..m {
            t.on_tick(TIQUE);
        }
        feito = m;
        let agora = massas(&mut t);
        let fora: f64 = antes
            .iter()
            .zip(&agora)
            .filter(|(a, _)| **a <= 1000.0)
            .map(|(a, b)| f64::from(b - a).max(0.0))
            .sum();
        out.push(fora);
    }
    out
}

const MARCAS: [usize; 5] = [0, 15, 30, 60, 240];

/// SONDA — a velocidade da queda por diluição e com o Real Oil (tabela do relatório).
#[test]
#[ignore = "diagnóstico"]
fn diag_a_queda_na_ladeira() {
    relato(format!("tiques após soltar: {MARCAS:?}"));
    for oleo in [false, true] {
        for escala in [1.0, 0.5, 0.15] {
            let q = queda(escala, oleo, &MARCAS);
            let q: Vec<String> = q.iter().map(|v| format!("{v:.0}")).collect();
            relato(format!(
                "água ×{escala} Real Oil {oleo}: massa nova fora do óleo {}",
                q.join(" · ")
            ));
        }
    }
}

/// Uma variante da sonda J11: a lei da [`ph2d_wet_paint::solver::sonda_da_queda`] (`0` hoje · `3` hoje contado ·
/// `1` tecto × `w` · `2` ladeira × `w` · `4` ladeira × `r`, linear) e um factor GLOBAL num knob (`None` = o de sempre).
#[derive(Clone, Copy)]
struct Variante {
    nome: &'static str,
    lei: u8,
    knob: Option<(&'static str, Knob, f64)>,
}

/// O que a sonda J11 lê de uma corrida: a massa nova fora do óleo nas `marcas`, a distância média dessa massa ao eixo
/// do óleo (células) em cada marca, o `film` médio no óleo ao soltar e a fracção de termos no tecto do nivelamento.
struct Leitura {
    fora: Vec<f64>,
    dist: Vec<f64>,
    film: f64,
    tecto: f64,
    imagem: Vec<u8>,
}

fn grade(t: &mut PainterTool) -> (usize, Vec<f32>) {
    t.wet_bring_home();
    let g = &t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão")
        .engine
        .layers[0]
        .grid;
    (g.s, (0..g.cells).map(|i| g.sett[i] + g.susp[i]).collect())
}

fn queda_v(escala: f64, v: Variante, fref: f32, marcas: &[usize]) -> Leitura {
    use ph2d_wet_paint::solver::sonda_da_queda as sd;
    use std::sync::atomic::Ordering::Relaxed;
    let mut t = com_oleo();
    let (s, antes) = grade(&mut t);
    // O eixo do óleo: a linha média das células com óleo.
    let (mut sy, mut n) = (0.0f64, 0.0f64);
    for (i, a) in antes.iter().enumerate() {
        if *a > 1000.0 {
            sy += (i / s) as f64;
            n += 1.0;
        }
    }
    let eixo = sy / n.max(1.0);
    let mut p = preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png");
    let w = p.parametros.volumes.water.unwrap_or(1.0);
    p.parametros.volumes.water = Some(w * escala);
    t.preset_aplicar(&p).expect("o meio");
    t.set_wet_oleo_real_ligado(true);
    t.set_brush_size_px(12.0);
    t.set_brush_color_srgb8([240, 130, 60]);
    if let Some((chave, k, f)) = v.knob {
        let alvo = t.paint.wetpaint.knobs.get(k) * f;
        t.paint.wetpaint.knobs.set(k, alvo);
        assert!(
            (t.paint.wetpaint.knobs.get(k) - alvo).abs() < 1e-9,
            "o knob {chave} chegou"
        );
    }
    sd::FREF.store(fref.to_bits(), Relaxed);
    sd::VARIANTE.store(v.lei, Relaxed);
    sd::N_LADEIRA.store(0, Relaxed);
    sd::N_TECTO.store(0, Relaxed);
    t.on_canvas_pointer(ponteiro(60.0, 90.0, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(ponteiro(60.0 + 4.0 * k as f32, 90.0, PointerPhase::Move));
        t.on_tick(TIQUE);
    }
    t.on_canvas_pointer(ponteiro(180.0, 90.0, PointerPhase::Up));
    let film = {
        t.wet_bring_home();
        if let Some((chave, k, _)) = v.knob {
            let vivo = t
                .paint
                .wetpaint
                .session
                .as_ref()
                .expect("a sessão")
                .engine
                .tuning
                .get(k);
            assert!(
                (vivo - t.paint.wetpaint.knobs.get(k)).abs() < 1e-9,
                "o knob {chave} chegou ao motor"
            );
        }
        let g = &t
            .paint
            .wetpaint
            .session
            .as_ref()
            .expect("a sessão")
            .engine
            .layers[0]
            .grid;
        let f: Vec<f32> = (0..g.cells)
            .filter(|&i| antes[i] > 1000.0 && g.film[i] > 0.0)
            .map(|i| g.film[i])
            .collect();
        f64::from(f.iter().sum::<f32>()) / f.len().max(1) as f64
    };
    let (mut fora, mut dist) = (Vec::new(), Vec::new());
    let mut feito = 0;
    for &m in marcas {
        for _ in feito..m {
            t.on_tick(TIQUE);
        }
        feito = m;
        let (_, agora) = grade(&mut t);
        let (mut mm, mut md) = (0.0f64, 0.0f64);
        for (i, (a, b)) in antes.iter().zip(&agora).enumerate() {
            if *a <= 1000.0 {
                let d = f64::from(b - a).max(0.0);
                mm += d;
                md += d * ((i / s) as f64 - eixo).abs();
            }
        }
        fora.push(mm);
        dist.push(md / mm.max(1e-9));
    }
    let nl = sd::N_LADEIRA.load(Relaxed);
    let tecto = sd::N_TECTO.load(Relaxed) as f64 / nl.max(1) as f64;
    sd::VARIANTE.store(0, Relaxed);
    let imagem = super::agua_sobre_oleo_tests::mostrada(&mut t);
    Leitura {
        fora,
        dist,
        film,
        tecto,
        imagem,
    }
}

/// SONDA J11 — a queda pela diluição (HANDOFF_JANELA_10 «Aberto 1»): quem manda na velocidade (o tecto do
/// nivelamento × o `MaxVelocity`) e ≤ 4 leis simuladas EM EXECUÇÃO, intercaladas num só processo. Grava a prancha
/// em `PH2D_SONDA_DIR` (`J11_<escala>_<variante>.ppm`).
#[test]
#[ignore = "diagnóstico J11 (sonda, não é lei): corre-se à mão com PH2D_SONDA_DIR"]
fn diag_a_queda_pela_diluicao() {
    const M: [usize; 4] = [15, 60, 120, 240];
    let hoje = Variante {
        nome: "hoje",
        lei: 3,
        knob: None,
    };
    let vs = [
        hoje,
        Variante {
            nome: "MaxVel×0,5",
            lei: 3,
            knob: Some(("maxVelocity", Knob::MaxVelocity, 0.5)),
        },
        Variante {
            nome: "LevelClamp×0,5",
            lei: 3,
            knob: Some(("levelClamp", Knob::LevelClamp, 0.5)),
        },
        Variante {
            nome: "V1 tecto×w",
            lei: 1,
            knob: None,
        },
        Variante {
            nome: "V2 ladeira×w",
            lei: 2,
            knob: None,
        },
        Variante {
            nome: "V3 ladeira×r",
            lei: 4,
            knob: None,
        },
    ];
    // O `film` de referência = o do pincel cheio (água ×1), lido HOJE neste mesmo processo.
    let fref = queda_v(1.0, hoje, 1.0, &[0]).film as f32;
    relato(format!(
        "J11 fref (film no óleo, água ×1) = {fref:.3}; marcas {M:?}"
    ));
    let escalas = [0.15, 0.5, 1.0];
    for (r, escala) in escalas.iter().enumerate() {
        for j in 0..vs.len() {
            let v = vs[(j + r) % vs.len()];
            let l = queda_v(*escala, v, fref, &M);
            let f: Vec<String> = l.fora.iter().map(|x| format!("{x:.0}")).collect();
            let d: Vec<String> = l.dist.iter().map(|x| format!("{x:.1}")).collect();
            relato(format!(
                "água ×{escala} {:<15} film {:.3} tecto {:.0}% | massa fora {} | dist {}",
                v.nome,
                l.film,
                100.0 * l.tecto,
                f.join(" · "),
                d.join(" · ")
            ));
            let nome = v
                .nome
                .split(' ')
                .next()
                .unwrap_or(v.nome)
                .replace(['×', ','], "_");
            super::agua_sobre_oleo_tests::grava(&format!("J11_{escala}_{nome}"), &l.imagem);
        }
    }
}

//! **A queda da água na ladeira do óleo** (smoke do dono 30/10): um Wet Flat pintado POR CIMA da crista de um Dabber
//! assentado escorre ladeira abaixo — (a) devagar com pouca água, depressa muito diluído; (b) igual com o Real Oil
//! ligado no Wet Flat (o óleo de ANTES continua a ser relevo para a água de um traço novo).

use super::agua_sobre_oleo_tests::{com_oleo, ponteiro, preset, relato};
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

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

/// O que uma corrida J11 lê: a massa nova fora do óleo nas `marcas`, a distância média dessa massa ao eixo do óleo
/// (células) em cada marca, o `film` médio no óleo ao soltar e a imagem mostrada no fim.
struct Leitura {
    fora: Vec<f64>,
    dist: Vec<f64>,
    film: f64,
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

/// Um Wet Flat (água `escala` × a do preset, Real Oil ligado) ao longo da crista do Dabber assentado.
fn queda_j11(escala: f64, marcas: &[usize]) -> Leitura {
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
    t.on_canvas_pointer(ponteiro(60.0, 90.0, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(ponteiro(60.0 + 4.0 * k as f32, 90.0, PointerPhase::Move));
        t.on_tick(TIQUE);
    }
    t.on_canvas_pointer(ponteiro(180.0, 90.0, PointerPhase::Up));
    let film = {
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
    let imagem = super::agua_sobre_oleo_tests::mostrada(&mut t);
    Leitura {
        fora,
        dist,
        film,
        imagem,
    }
}

/// A massa fora do óleo aos 240 tiques com água ×0,15 na lei ANTERIOR à J11 (a ladeira inteira, qualquer que fosse a
/// água): `23 654` (commit `6e466569d2`, prancha `docs/Painter/Pinceis/pranchas/J11_queda_diluicao.png`).
const POUCA_ANTES_J11: f64 = 23_654.0;

/// ⭐ **A queda cresce com a diluição** (dono 30/10, J11 — escolheu a V3 da prancha
/// `docs/Painter/Pinceis/pranchas/J11_queda_diluicao.png`): sobre a crista do óleo, o pincel quase seco não escorre
/// (≤ 10 % do que escorria) e a massa que desce cresce com a água: pouca < média < muita.
/// Vermelho visto na lei anterior: pouca 23 654 (e 12 618 na árvore de 30/10 22 h, com o `PINCEL_DE_OLEO` de volta);
/// com a lei: 1 438 · 70 270 · 120 564.
#[test]
fn a_queda_cresce_com_a_diluicao() {
    let q: Vec<f64> = [0.15, 0.5, 1.0]
        .iter()
        .map(|e| queda_j11(*e, &[240]).fora[0])
        .collect();
    let (pouca, media, muita) = (q[0], q[1], q[2]);
    assert!(
        pouca < media && media < muita,
        "pouca {pouca:.0} < média {media:.0} < muita {muita:.0}"
    );
    assert!(
        pouca <= 0.10 * POUCA_ANTES_J11,
        "o pincel quase seco ainda escorre: {pouca:.0} > 10 % de {POUCA_ANTES_J11:.0}"
    );
}

/// SONDA J11 — a queda pela diluição na lei do produto (tabela do relatório; grava `J11_<escala>.ppm` em
/// `PH2D_SONDA_DIR`).
#[test]
#[ignore = "diagnóstico J11: corre-se à mão com PH2D_SONDA_DIR"]
fn diag_a_queda_pela_diluicao() {
    const M: [usize; 4] = [15, 60, 120, 240];
    relato(format!("J11 marcas {M:?}"));
    for escala in [0.15, 0.5, 1.0] {
        let l = queda_j11(escala, &M);
        let f: Vec<String> = l.fora.iter().map(|x| format!("{x:.0}")).collect();
        let d: Vec<String> = l.dist.iter().map(|x| format!("{x:.1}")).collect();
        relato(format!(
            "água ×{escala} film {:.3} | massa fora {} | dist {}",
            l.film,
            f.join(" · "),
            d.join(" · ")
        ));
        super::agua_sobre_oleo_tests::grava(&format!("J11_{escala}"), &l.imagem);
    }
}

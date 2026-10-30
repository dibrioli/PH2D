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

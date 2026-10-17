//! **As formas** (prompt 44, doc [44_medicao](../../../../docs/Painter/NewFeaturesWetPaint/44_medicao_as_formas.md)):
//! a ponta RETA do pincel (`dab_quina`) e a pressão do pincel (`pressureSize` · `pressureOpacity`).

use super::midias_de_fabrica::de_fabrica;
use super::sonda_gestos::{pinta_cena_ate, tela};
use ph2d_editor_core::Tool;

/// O CUSTO da ponta reta (doc 44 §5): a cena da bancada com o Wet Round e com o mesmo de quina 1, no MESMO
/// processo, intercaladas em 9 rodadas; o resumo é o MÍNIMO (`docs/DevOps/MEDIR_VELOCIDADE.md`).
#[test]
#[ignore = "medição: o custo da ponta reta (prompt 44); corre-se à mão em --profile smoke"]
fn measure_o_custo_da_ponta_reta() {
    let com = |q: f64| {
        let mut p = de_fabrica("Wet Round");
        p.parametros
            .brush
            .insert("dab_quina".to_owned(), serde_json::json!(q));
        p
    };
    let (redondo, reto) = (com(0.0), com(1.0));
    let mut tempos = [Vec::new(), Vec::new()];
    for rodada in 0..9 {
        for k in 0..2 {
            let i = (k + rodada) % 2;
            let p = if i == 0 { &redondo } else { &reto };
            let mut t = tela();
            t.set_wet_relogio_fixo(true);
            assert!(t.wet_preset_aplicar(p).is_empty());
            t.set_brush_size_px(17.0);
            let t0 = std::time::Instant::now();
            let _ = pinta_cena_ate(&mut t, 60);
            tempos[i].push(t0.elapsed().as_secs_f64());
        }
    }
    let min = |v: &Vec<f64>| v.iter().copied().fold(f64::INFINITY, f64::min);
    let (a, b) = (min(&tempos[0]), min(&tempos[1]));
    println!(
        "[custo] redondo {:.1} ms · reto {:.1} ms · razão {:.3} (loadavg {})",
        a * 1e3,
        b * 1e3,
        b / a,
        std::fs::read_to_string("/proc/loadavg").unwrap_or_default().trim()
    );
}

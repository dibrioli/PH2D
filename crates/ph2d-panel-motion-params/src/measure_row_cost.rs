//! **QUANTO CUSTA UMA ROW DE PARAM, pintada** — a medição que o ciclo 1 exige antes de pôr
//! rows dentro dos cartões do grafo ([doc 103](../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md);
//! §0.0 do `CLAUDE.md`: medir ANTES de escrever um limite).
//!
//! ⚠️ Mesma porta e mesmo método do irmão [`super::measure_row_cap`] — `MockPanelHost`,
//! `--release`, várias leituras, a melhor (a mediana de um relógio ruidoso mente para cima).
//! O que muda é a variável: ali era o TETO de rows, aqui é o **custo marginal de UMA row**,
//! que é o número que decide o LOD por zoom e quantos cartões podem mostrar params.
//!
//! `cargo test -p ph2d-panel-motion-params --release -- --ignored --nocapture measure_row_cost`
use crate::snapshot::{ParamRow, ParamsSnapshot, RowDisplay, ScalarRow, set_current_params};
use crate::{MotionParamsPanel, MotionParamsPanelState};

/// Uma tela de MEDIÇÃO, não um desenho — o mesmo par do `measure_row_cap`, de propósito:
/// dois números diferentes mediriam duas telas e não se poderiam comparar.
const MEASURE_W: f32 = 1200.0; // LITERAL-PX-OK: tela de medicao, nao desenho
const MEASURE_H: f32 = 800.0; // LITERAL-PX-OK: tela de medicao, nao desenho

fn snapshot_with(rows: usize) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 1,
        title: "Measure".into(),
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
        rows: (0..rows)
            .map(|i| {
                ParamRow::Scalar(ScalarRow {
                    name: "rows",
                    // Rótulos de comprimento REALISTA e VARIÁVEL: o custo de uma row é
                    // dominado pelo texto, e medir sempre a mesma string mediria o cache.
                    label: format!("Parameter {i}"),
                    value: i as f64 * 0.5,
                    min: 0.0,
                    max: 20.0, // LITERAL-PX-OK: valor de FIXTURA de medicao, nao desenho
                    hard_min: 0.0,
                    hard_max: 20.0, // LITERAL-PX-OK: valor de FIXTURA de medicao, nao desenho
                    step: 0.1,      // LITERAL-PX-OK: valor de FIXTURA de medicao, nao desenho
                    integer: false,
                    driven_by: None,
                    display: RowDisplay::default(),
                })
            })
            .collect(),
    }
}

fn paint_ms(rows: usize, n: u32) -> f64 {
    set_current_params(Some(snapshot_with(rows)));
    let mut melhor = f64::MAX;
    for _ in 0..3 {
        let t0 = std::time::Instant::now();
        for _ in 0..n {
            let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
            let mut state = MotionParamsPanelState;
            let _ = host.paint::<MotionParamsPanel>(
                &mut state,
                ph2d_editor_core::zones::Rect {
                    x: 0.0,
                    y: 0.0,
                    w: MEASURE_W,
                    h: MEASURE_H,
                },
            );
        }
        melhor = melhor.min(t0.elapsed().as_secs_f64() * 1000.0 / f64::from(n)); // LITERAL-PX-OK: us por ms, conversao de unidade de RELOGIO
    }
    melhor
}

#[test]
#[ignore = "medicao"]
fn measure_row_cost() {
    const N: u32 = 200;
    eprintln!(
        "  load: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    eprintln!(
        "  {:>5} │ {:>10} │ {:>12}",
        "rows", "ms/pintura", "us/row (marg.)"
    );
    let mut anterior: Option<(usize, f64)> = None;
    for rows in [0usize, 1, 2, 4, 8, 16, 24, 33] {
        let ms = paint_ms(rows, N);
        let marg = anterior.map_or(f64::NAN, |(r0, m0)| {
            (ms - m0) * 1000.0 / (rows - r0) as f64 // LITERAL-PX-OK: us por ms, conversao de unidade de RELOGIO
        });
        eprintln!("  {rows:>5} │ {ms:>10.4} │ {marg:>12.2}");
        anterior = Some((rows, ms));
    }
    set_current_params(None);
    eprintln!(
        "  (o custo MARGINAL de uma row e' o que decide quantos cartoes podem mostrar params)"
    );
}

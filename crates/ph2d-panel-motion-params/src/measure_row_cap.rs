//! **A MEDIÇÃO do teto de linhas** — a mesma porta e o mesmo método das três vezes anteriores
//! (`MockPanelHost::with_panel`, 200 construções, `--release`, três leituras por lado).
//!
//! ⚠️ Ela existe como teste `#[ignore]` e não como bench para que o número que o doc do
//! [`super::snapshot_ids::MAX_PARAM_ROWS`] cita seja **reprodutível pelo próximo** com um
//! comando só, em vez de uma nota que envelhece.

use crate::{MotionParamsPanel, MotionParamsPanelState};

#[test]
#[ignore = "medicao"]
fn measure_screen_build() {
    const N: u32 = 500;
    let mut melhor = f64::MAX;
    for leitura in 0..5 {
        let t0 = std::time::Instant::now();
        for _ in 0..N {
            let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
            let mut state = MotionParamsPanelState;
            let _ = host.paint::<MotionParamsPanel>(
                &mut state,
                ph2d_editor_core::zones::Rect {
                    x: 0.0,
                    y: 0.0,
                    // A tela da MEDIÇÃO, e não um desenho: é o mesmo par que as outras
                    // fixturas deste painel usam, e mudá-lo mudaria o número que a tabela do
                    // `MAX_PARAM_ROWS` cita.
                    w: 1200.0, // LITERAL-PX-OK: tela de medicao, nao desenho
                    h: 800.0,  // LITERAL-PX-OK: tela de medicao, nao desenho
                },
            );
        }
        let us = t0.elapsed().as_secs_f64() * 1e6 / f64::from(N);
        melhor = melhor.min(us);
        println!(
            "MAX_PARAM_ROWS={} leitura {}: {us:.1} us/construcao",
            crate::MAX_PARAM_ROWS,
            leitura + 1
        );
    }
    // ⚠️ **O MÍNIMO, e não a média:** a carga só pode SOMAR tempo, então a leitura mais baixa
    // é a que menos ruído contém. Uma média de cinco leituras sob carga mede a carga.
    println!(
        "MAX_PARAM_ROWS={} MINIMO {melhor:.1}",
        crate::MAX_PARAM_ROWS
    );
}

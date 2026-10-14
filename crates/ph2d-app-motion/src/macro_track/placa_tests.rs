//! **O palco em cena recebe a PLACA** (ADR-0186 §Emenda W3, «A fronteira»): o produto coze na placa
//! por omissão, e a sessão do gate 3 gravada a cozer na placa toca no palco a cozer na placa — e o
//! modo fiel atravessa as duas rotas (placa ↔ CPU) sem acusar uma divergência que não houve.
//!
//! `#[ignore]`: precisa de adaptador real —
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --lib -- --ignored macro_track::placa`.

use super::gate_tests::{pelo_nucleo, retrato_visto, sessao_em, toca};
use super::tocar::Modo;
use ph2d_gpu::GpuContext;

fn placa() -> GpuContext {
    GpuContext::new(GpuContext::default_instance(), None).expect("adaptador de GPU")
}

#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn o_palco_na_placa_reproduz_cada_quadro_da_sessao_na_placa_ao_bit() {
    let gpu = placa();
    let g = sessao_em(Some(&gpu));
    assert!(
        g.na_placa > 0,
        "CONTROLO: a sessão tem de cozer na placa (0 de {} quadros)",
        g.retratos.len()
    );
    let doc = pelo_nucleo(&g);
    let mut vistos = 0;
    let t = toca(&doc, Modo::Fiel, Some(&gpu), |k, palco| {
        assert_eq!(
            retrato_visto(palco, Some(&gpu)),
            g.retratos[k],
            "o quadro {k} do palco na placa não é o que o artista viu"
        );
        vistos += 1;
    })
    .expect("a macro toca até ao fim");
    assert_eq!(vistos, g.retratos.len());
    assert_eq!(t.palco().doc, g.fim.doc);
}

#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn o_modo_fiel_atravessa_as_duas_rotas() {
    let gpu = placa();
    let na_placa = pelo_nucleo(&sessao_em(Some(&gpu)));
    toca(&na_placa, Modo::Fiel, None, |_, _| {})
        .expect("a sessão gravada na placa toca num palco sem placa");
    let na_cpu = pelo_nucleo(&sessao_em(None));
    toca(&na_cpu, Modo::Fiel, Some(&gpu), |_, _| {})
        .expect("a sessão gravada na CPU toca num palco com placa");
}

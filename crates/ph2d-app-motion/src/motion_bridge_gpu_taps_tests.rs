//! ⭐⭐⭐ **A COSTURA QUE NINGUÉM MEDIA — a rota totalmente-na-GPU cozinha as TOMADAS?**
//!
//! ⛔⛔ **O defeito, reportado pelo dono em 2026-09-08 e caçado em três rondas:** as tomadas são
//! cozidas dentro do [`ph2d_eval_motion::MotionCookPump::cook_target_into`], que corre **na
//! marcha**; a rota `FullyGpu` da ponte **retorna antes de qualquer marcha**. O device produzia o
//! quadro, `tap_streams` ficava vazio, e quem depende de uma tomada desaparecia **em silêncio**:
//! o gizmo de canvas dos deformadores de quadrilátero e os **sinais** de um documento inteiramente
//! no device.
//!
//! ⚠️ **As DUAS metades já tinham régua e a costura entre elas não:**
//! `cooking_the_taps_fills_them_without_marching_the_clock` (a porta funciona, em
//! `ph2d-eval-motion`) e `the_gizmo_paints_geometry_and_paints_none_when_inactive` (a tinta
//! aparece, ao lado). O defeito viveu exactamente **entre** as duas — *nenhum gate perguntava se
//! a rota que o app de facto corre chega a chamar a porta*.
//!
//! ⛔ **E a sonda que eu escrevi para o diagnosticar MENTIU** porque marchava na CPU antes de
//! perguntar: ***uma sonda que escolhe a rota mede a rota que ela escolheu.*** É por isso que a
//! montagem da cena vive em [`fx`](crate::warp_gizmo_fixtures) **sem marcha nenhuma**, e é este
//! ficheiro que entrega o quadro ao device.
//!
//! ⚠️ **`#[ignore]`, e a razão é o adapter** (CLAUDE.md §5.0: *skip gracioso não é verde*) — sem
//! GPU não há rota `FullyGpu` para medir, e um teste que passasse sem device estaria a afirmar
//! sobre um programa que não é este.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib -- --ignored --nocapture the_fully_gpu_route
//! ```

use super::{GpuOutcome, cook_gpu};
use crate::warp_gizmo;
use crate::warp_gizmo_fixtures as fx;
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::cook::TimeScopes;

/// A frase que a ponte diz quando ROTEOU pelo device inteiro. ⚠️ Lida do produto
/// (`MotionState::route_said`), nunca replanejada aqui: um segundo cálculo da rota seria uma
/// segunda opinião sobre qual rota este documento toma, e o gate deixaria de medir a do app.
const PELO_DEVICE: &str = "device: o plano inteiro (fully-GPU)";

fn adaptador() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

/// ⭐ **O PORTÃO:** com o quadro entregue ao device, a tomada do gizmo **está cozida** e o gizmo
/// existe. Vermelho antes da cura (provado por mutação: apagar a chamada a `cook_taps_only` na
/// ponte deixa `tap_streams` vazio e `resolve` em `None`, que é o report do dono à letra).
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_fully_gpu_route_cooks_the_taps_it_never_marched() {
    let Some(gpu) = adaptador() else {
        panic!("sem adapter — este gate mede a rota do device e nao tem versao de CPU");
    };
    let _trava = fx::trava();
    let scopes = TimeScopes::new();
    for (rotulo, depois) in [("ANTES do mirror", false), ("DEPOIS do mirror", true)] {
        let (mut m, bw) = fx::cena_do_dono(depois);
        let armadas = warp_gizmo::taps_for(&m);
        assert!(
            !armadas.is_empty(),
            "{rotulo}: a fixtura tem de ARMAR a tomada, senao o gate e' vacuo"
        );
        assert!(
            m.pump.tap_streams().is_empty(),
            "{rotulo}: montada e NAO marchada — se ja' vier cozida, a fixtura esconde o defeito"
        );

        let saida = cook_gpu(&mut m, &gpu, 0, 1.0 / 60.0, &scopes);

        assert!(
            matches!(saida, GpuOutcome::Handled),
            "{rotulo}: a ponte devolveu {saida:?} — motivo: {:?}",
            m.route_said
        );
        assert_eq!(
            m.route_said,
            Some(PELO_DEVICE),
            "{rotulo}: esta fixtura tem de cair na rota TOTALMENTE-NA-GPU, que e' onde o \
             defeito vivia — noutra rota o gate mede a marcha de outra pessoa"
        );
        let cozidas: Vec<_> = m.pump.tap_streams().iter().map(|(n, _)| *n).collect();
        for n in &armadas {
            assert!(
                cozidas.contains(n),
                "{rotulo}: a tomada {n:?} foi armada e NAO foi cozida — {cozidas:?}"
            );
        }
        assert!(
            warp_gizmo::resolve(&m, true).is_some(),
            "{rotulo}: sem gizmo no no' {bw:?} — e' o report de 08/09 de volta"
        );
    }
}

/// ⚠️ **O CONTROLO — sem tomada armada, a rota é o mundo anterior byte a byte.** Sem ele o gate
/// acima passaria mesmo que alguém cozinhasse tomadas incondicionalmente, e o preço nomeado no
/// doc da porta (*«só se paga enquanto há tomada armada»*) deixaria de ser verdade sem ninguém
/// ver.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_fully_gpu_route_cooks_nothing_when_no_tap_is_armed() {
    let Some(gpu) = adaptador() else {
        panic!("sem adapter — este gate mede a rota do device e nao tem versao de CPU");
    };
    let _trava = fx::trava();
    let scopes = TimeScopes::new();
    let (mut m, _bw) = fx::cena_do_dono(false);
    // Nenhum nó seleccionado ⇒ o gizmo não pede tomada nenhuma; a cena é a mesma.
    ph2d_panel_motion_graph::set_graph_selection(vec![]);
    m.pump.set_taps(&[]);

    let saida = cook_gpu(&mut m, &gpu, 0, 1.0 / 60.0, &scopes);

    assert!(matches!(saida, GpuOutcome::Handled), "{saida:?}");
    assert_eq!(m.route_said, Some(PELO_DEVICE));
    assert!(
        m.pump.tap_streams().is_empty(),
        "sem tomada armada nada e' cozido — {:?}",
        m.pump.tap_streams().len()
    );
}

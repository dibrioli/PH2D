//! **As TOMADAS dos gizmos e dos sinais na rota inteira na placa** (Bug #20, `de58233ef3`).
//!
//! A rota inteira retorna ANTES da marcha da bomba, e as tomadas (`pump.tap_streams`: a porta dos
//! pontos, warp, colisor, pivô, sinais) eram cozidas dentro dela. Este módulo é quem as pede à
//! placa antes do cozimento e as põe na bomba depois — só um nó que a placa não encenou volta à CPU.

use crate::motion_state::MotionState;
use ph2d_nodegraph::cook::TimeScopes;
use ph2d_nodegraph::graph::NodeId;

/// **Quantas linhas a placa devolve de cada tomada de gizmo** (Bug #20). O recurso é o DESENHO do
/// gizmo: nenhum desenha mais do que o [`crate::ponto_gizmo::MAX_PONTOS`], e o passo da leitura
/// (`i · count / n`) é o MESMO da [`crate::ponto_gizmo`] — ali os índices coincidem ao elemento.
/// ⚠️ Acima do tecto a caixa do warp sai da amostra, não da corrente inteira.
pub(crate) const TECTO_DA_TOMADA: u32 = crate::ponto_gizmo::MAX_PONTOS as u32;

/// As tomadas `gizmos` da última leitura da placa, na bomba — onde os gizmos as leem.
fn poe_as_tomadas_da_placa(motion: &mut MotionState, gizmos: &[NodeId]) {
    let lidas = motion.gpu_cook.tomadas().map_or_else(Vec::new, |t| {
        gizmos
            .iter()
            .filter_map(|n| t.get(n).map(|s| (*n, s.clone())))
            .collect()
    });
    motion.pump.poe_as_tomadas_lidas(lidas);
}

/// **Só os gates:** espera a placa acabar o último cozimento e põe as tomadas DELE na bomba — o que
/// o quadro seguinte do app recolheria. Para quem mede UM cozimento.
#[cfg(test)]
pub(crate) fn espera_as_tomadas(motion: &mut MotionState, gpu: &ph2d_gpu::GpuContext) {
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    motion.gpu_cook.recolhe(gpu);
    let gizmos: Vec<NodeId> = motion
        .gpu_cook
        .tomadas()
        .map(|t| t.keys().copied().collect())
        .unwrap_or_default();
    poe_as_tomadas_da_placa(motion, &gizmos);
}

/// ⭐ As tomadas dos GIZMOS lêem-se da PLACA — pedidas ANTES do cozimento, que as codifica no seu
/// `submit`; recolhidas por [`recolhe`]. As dos SINAIS ficam fora (precisam da marcha) — pelo
/// nome, não pela posição. Devolve os gizmos pedidos.
pub(super) fn pede(motion: &mut MotionState, gpu: &ph2d_gpu::GpuContext) -> Vec<NodeId> {
    let gizmos: Vec<NodeId> = motion
        .pump
        .tomadas_armadas()
        .iter()
        .copied()
        .filter(|n| !motion.signal_taps.contains(n))
        .collect();
    let pedidas: Vec<_> = gizmos.iter().map(|&n| (n, TECTO_DA_TOMADA)).collect();
    motion.gpu_cook.pede_as_tomadas(gpu, &pedidas);
    gizmos
}

/// ⭐⭐⭐ **AS TOMADAS, que a rota inteira deixava cair** (report do dono, 2026-09-08).
///
/// ⛔⛔ Elas são cozidas dentro da MARCHA da bomba, e a rota inteira **retorna antes da marcha**:
/// o device produziu o quadro, a bomba não corre, e `tap_streams` fica vazio. Quem depende de uma
/// tomada deixa de existir **em silêncio** — o gizmo dos deformadores de quadrilátero sumiu do
/// canvas, e os sinais de um documento inteiramente no device não gritam.
///
/// ⚠️ **A rota HÍBRIDA já estava coberta**, e o doc do `set_taps` conta essa cura: as tomadas
/// passaram a ser estado da bomba para cavalgarem *«a marcha que houver»*. O que ninguém viu é que
/// na rota inteira **não há marcha nenhuma**.
///
/// ⛔⛔ **Um SINAL precisa da história tique a tique** (doc 110 §14.2): cozer sem marchar deixa o
/// `pre` do cone parado e o livro-razão vazio. Com tomadas de sinal marcha-se o cone delas — a
/// marcha da rota híbrida, com zero fronteiras.
///
/// ⛔⛔ **E as dos GIZMOS lêem-se da PLACA, nunca do `cook_taps_only`** (Bug #20): ele coze na CPU
/// um estado que a bomba nunca marchou — o `motion.trail` sem ecos, o anel do `Slit Scan` parado —,
/// e os marcadores ficavam na pose de repouso. A leitura é a da placa, um quadro atrás como os
/// cartões. ⚠️ Só um nó que a placa NÃO encenou (sem corrente própria neste cozimento) volta à CPU.
pub(super) fn recolhe(
    motion: &mut MotionState,
    gizmos: &[NodeId],
    ultimo: f64,
    target: u64,
    fixed_dt: f64,
    scopes: &TimeScopes,
) {
    let fora: Vec<NodeId> = gizmos
        .iter()
        .copied()
        .filter(|&n| motion.gpu_cook.node_columns(n).is_none())
        .collect();
    if motion.signal_taps.is_empty() {
        motion.pump.esquece_as_tomadas();
        motion.pump.set_taps(&fora);
        let (g, r) = (&motion.doc.graph, &motion.registry);
        motion.pump.cook_taps_only(g, r, ultimo, scopes);
    } else {
        let mut sinais = motion.signal_taps.clone();
        sinais.extend(fora);
        motion.pump.set_taps(&sinais);
        motion.pump.set_taps_por_tique(motion.signal_taps.len());
        for tick in super::super::ticks_owed(motion.pump.last_cooked_tick(), target) {
            let (g, r) = (&motion.doc.graph, &motion.registry);
            let at = |t: u64| t as f64 * fixed_dt;
            motion
                .pump
                .advance_or_scrub_to_nodes_scoped(g, r, &[], tick, at, scopes);
        }
    }
    poe_as_tomadas_da_placa(motion, gizmos);
}

//! **A MARCHA da rota híbrida** — a bomba avança as fronteiras E os condutores com estado
//! ([`ph2d_gpu_cook::GpuPlan::condutores_na_marcha`], doc 120 §8.3.3), e devolve os relógios que a
//! placa deve e o mapa dirigido de cada um. Irmão do [`super`] pelo tecto de LOC: ali a rota e o
//! cozimento, aqui o que a CPU entrega.

use super::{substep_clocks, valores_dirigidos};
use crate::motion_state::MotionState;
use ph2d_nodegraph::cook::TimeScopes;
use ph2d_nodegraph::graph::NodeId;
use std::time::{Duration, Instant};

/// Os relógios que a placa deve neste quadro e, para cada um, o mapa dos params dirigidos.
pub(super) struct Entrega {
    pub relogios: Vec<(f64, Option<u64>)>,
    pub dirigidos: Vec<ph2d_gpu_cook::DrivenParams>,
}

/// Avança a bomba até `ate` nos `nos` (fronteiras + condutores), um tique devido de cada vez.
fn avanca(motion: &mut MotionState, nos: &[NodeId], ate: u64, fixed_dt: f64, scopes: &TimeScopes) {
    for tick in super::super::ticks_owed(motion.pump.last_cooked_tick(), ate) {
        motion.pump.advance_or_scrub_to_nodes_scoped(
            &motion.doc.graph,
            &motion.registry,
            nos,
            tick,
            |t| t as f64 * fixed_dt,
            scopes,
        );
    }
}

/// Marcha a bomba e deriva os relógios e os valores dirigidos do sufixo da placa.
///
/// - **Um plano com laço e condutores com estado anda EM PASSO com a placa:** cada tique que ela
///   deve (`rewind_for..=target`) marcha-se na bomba e o mapa dele lê-se logo a seguir. Lido depois
///   da marcha inteira, o condutor de um tique antigo seria recozido sobre o estado do último.
/// - Os outros marcham tudo e derivam depois, como sempre: as entradas de cada tique vêm da fita.
///
/// ⚠️ Sem entrega nada coze, e o `rewind_for` não pode correr sozinho (ele RESTAURA um ponto de
/// controlo na placa) — no passo-a-passo ele vem antes da marcha, então um quadro sem entrega deixa
/// a placa rebobinada, e o seguinte rebobina outra vez.
pub(super) fn marcha(
    motion: &mut MotionState,
    plan: &ph2d_gpu_cook::GpuPlan,
    target: u64,
    sub: u32,
    fixed_dt: f64,
    scopes: &TimeScopes,
) -> Entrega {
    let mut nos: Vec<NodeId> = plan.boundaries.iter().map(|(n, _)| *n).collect();
    for c in &plan.condutores_na_marcha {
        if !nos.contains(c) {
            nos.push(*c);
        }
    }
    if plan.drives_a_loop() && !plan.condutores_na_marcha.is_empty() {
        let ticks: Vec<u64> = (motion.gpu_cook.rewind_for(target)..=target).collect();
        let mut relogios = substep_clocks(&ticks, sub, fixed_dt, true);
        let (mut marcha, mut dirigiu) = (Duration::ZERO, Duration::ZERO);
        let mut dirigidos = Vec::with_capacity(relogios.len());
        for &(playhead, tick) in &relogios {
            let t = tick.expect("um plano com laço dá o tique a cada relógio");
            if motion.pump.last_cooked_tick() != Some(t) {
                let t0 = Instant::now();
                avanca(motion, &nos, t, fixed_dt, scopes);
                marcha += t0.elapsed();
            }
            let t0 = Instant::now();
            dirigidos.push(valores_dirigidos(motion, playhead));
            dirigiu += t0.elapsed();
        }
        (motion.fases.marcha, motion.fases.dirigidos) = (marcha, dirigiu);
        if motion.pump.boundary_streams().is_empty() {
            relogios.clear();
            dirigidos.clear();
        }
        return Entrega {
            relogios,
            dirigidos,
        };
    }
    let t = Instant::now();
    avanca(motion, &nos, target, fixed_dt, scopes);
    motion.fases.marcha = t.elapsed();
    // A hybrid plan CAN drive a loop since ADR-0136 §5 — but only when every boundary is STATIC (a
    // temporal one still retreats at plan time, because that would be two sims of one state). A
    // static boundary is a constant, so the SAME hand-off serves every marched tick, and the loop
    // keeps its sequence exactly like the FullyGpu arm: rewind if owed, then march.
    let relogios: Vec<(f64, Option<u64>)> = if motion.pump.boundary_streams().is_empty() {
        Vec::new()
    } else if plan.drives_a_loop() {
        let ticks: Vec<u64> = (motion.gpu_cook.rewind_for(target)..=target).collect();
        substep_clocks(&ticks, sub, fixed_dt, true)
    } else {
        vec![(target as f64 * fixed_dt, None)]
    };
    // ⛔⛔⛔ **OS VALORES DIRIGIDOS, POR TIQUE — a lei do FullyGpu, que este ramo não tinha**
    // (report do Enio, 2026-10-03: *«o Vortex continua com o valor do cartão»*). ⭐ E as ENTRADAS de
    // cada tique (doc 128 §3.3): a bomba já as gravou ao marchar; o condutor de cada tique lê as da
    // fita.
    let t = Instant::now();
    let mut replay = motion
        .pump
        .comeca_o_replay(relogios.first().and_then(|r| r.1).unwrap_or(target));
    let dirigidos = relogios
        .iter()
        .map(|&(playhead, tick)| {
            motion
                .pump
                .entradas_do_replay(&mut replay, tick.unwrap_or(target));
            valores_dirigidos(motion, playhead)
        })
        .collect();
    motion.fases.dirigidos = t.elapsed();
    Entrega {
        relogios,
        dirigidos,
    }
}

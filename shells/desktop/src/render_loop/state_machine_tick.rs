//! ⭐⭐⭐ **A PONTE do cérebro autorável** (TOP-20 #15) — a lei pura vive no
//! [`ph2d_ecs::state_machine`]; aqui ela encontra o mundo.
//!
//! # ⚠️ A ordem dentro do quadro, e porque ela é esta
//!
//! `ler os sinais → avançar TODAS as máquinas → publicar o que elas anunciaram → a tabela de
//! acções (#5) lê`.
//!
//! - **Antes da tabela de acções**, e é o que faz uma porta abrir no MESMO quadro em que o botão é
//!   tocado: o cursor daquela consumidora ainda não leu, então o que esta fase publica chega-lhe
//!   já. É a mesma janela que a fábrica escolheu, com a razão escrita lá (*«para que um
//!   `SignalActions` que arranque um timer não tenha de esperar um quadro»*).
//! - ⛔⛔ **E TODAS as máquinas leem a MESMA fotografia de sinais, tirada ANTES de qualquer uma
//!   avançar.** É isso que impede um cérebro de reagir ao que outro acabou de anunciar dentro do
//!   mesmo quadro — a classe dos laços que o doc do `SignalActions` nomeia. Uma emissão desta fase
//!   chega a quem a escuta no quadro **seguinte**, como todo sinal desta casa.
//!
//! # ⚠️ O vivo NASCE aqui, e antes de qualquer avanço
//!
//! É a lei do `ensure_runtime` da câmera, escrita pela mesma razão: uma máquina anexada pela paleta
//! num quadro sem sinal nenhum ficaria por armar, e o sintoma seria *«às vezes o objecto não
//! pensa»*. ⛔ O `StateMachineRuntime` **não é componente registado** — a cerca é o TIPO —, então
//! ele não vem do ficheiro e tem de nascer de alguém.

use ph2d_ecs::{Entity, SimWorld, StateMachine, StateMachineRuntime, World};

/// Um anúncio de uma máquina — o nome autorado, e quem pensou.
pub(crate) struct MachineSignal {
    /// A entidade que carrega a máquina.
    pub(crate) entity: Entity,
    /// O nome do sinal, como o artista o escreveu.
    pub(crate) name: String,
}

/// **Toda máquina tem um vivo, e ele nasce no estado inicial dela.**
fn ensure_runtime(world: &mut World) {
    let nascer: Vec<Entity> = world
        .query_filtered::<(Entity, &StateMachine), bevy_ecs::prelude::Without<StateMachineRuntime>>(
        )
        .iter(world)
        .map(|(e, _)| e)
        .collect();
    for e in nascer {
        let born = world
            .get::<StateMachine>(e)
            .map(ph2d_ecs::state_machine::born)
            .unwrap_or_default();
        if let Ok(mut ent) = world.get_entity_mut(e) {
            ent.insert(born);
        }
    }
}

/// **Um avanço de todos os cérebros da cena.** Devolve o que eles anunciaram, na ordem.
///
/// ⚠️ **A ordem é a da IDENTIDADE**, nunca a da query — a lei de determinismo desta casa (HR-5):
/// duas máquinas que anunciam no mesmo tique têm de o fazer sempre na mesma ordem, senão o replay
/// diverge entre máquinas.
pub(crate) fn advance_machines(sim: &mut SimWorld, fired: &[&str]) -> Vec<MachineSignal> {
    let world = sim.world_mut();
    ensure_runtime(world);

    let mut quem: Vec<(u64, Entity)> = world
        .query::<(Entity, &StateMachine, &ph2d_ecs::StableId)>()
        .iter(world)
        .map(|(e, _, s)| (s.0, e))
        .collect();
    if quem.is_empty() {
        return Vec::new();
    }
    quem.sort_unstable_by_key(|(id, _)| *id);

    let mut out = Vec::new();
    for (_, e) in quem {
        let Some(cfg) = world.get::<StateMachine>(e).cloned() else {
            continue;
        };
        let Some(mut rt) = world.get::<StateMachineRuntime>(e).copied() else {
            continue;
        };
        let a = ph2d_ecs::state_machine::advance(&cfg, &mut rt, fired);
        if let Some(mut vivo) = world.get_mut::<StateMachineRuntime>(e) {
            *vivo = rt;
        }
        for name in a.emitted {
            out.push(MachineSignal { entity: e, name });
        }
    }
    out
}

#[cfg(test)]
#[path = "state_machine_tick_tests.rs"]
mod tests;

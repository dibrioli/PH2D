//! **A secção PARTICLES: o instantâneo e o dreno** (TOP-20 #18, W3).
//!
//! ⚠️ **As duas metades vivem juntas de propósito** — o molde do `script_inspector`: quem lê o
//! mundo para o painel e quem escreve a edição de volta fazem a MESMA tradução, e uma delas
//! esquecer um campo é visível ao lado da outra. E vivem AQUI, não na shell, pelo tecto dela.
//!
//! ⚠️ **As DUAS grandezas da corrida** (o relógio a andar e quantas partículas vivem) chegam de
//! fora, da [`crate::particles_bridge`] — elas não estão no mundo, e é precisamente por isso que o
//! painel tem de as dizer: um emissor exactamente como o artista o pediu lê-se como partido quando
//! a única coisa que falta é o `Play`.

use ph2d_ecs::{Entity, ParticleEmitter, SimWorld};
use ph2d_editor_core::particles_edits::{
    InspectorParticlesInfo, ParticlesFieldEdit as E, ParticlesNumber as N, ParticlesText as T,
};

/// **O instantâneo.** `None` para quem não tem o componente (ADR-0166).
#[must_use]
pub fn build_info(
    sim: &SimWorld,
    bits: u64,
    selected_count: usize,
    clock_playing: bool,
    alive: usize,
) -> Option<InspectorParticlesInfo> {
    let cfg = sim
        .world()
        .get::<ParticleEmitter>(Entity::from_bits(bits))?;
    #[expect(
        clippy::cast_precision_loss,
        reason = "uma contagem de partículas e uma semente, as duas mostradas como número"
    )]
    let (amount, seed) = (cfg.amount as f32, cfg.seed as f32);
    Some(InspectorParticlesInfo {
        entity_bits: bits,
        emitting: cfg.emitting,
        one_shot: cfg.one_shot,
        amount,
        life: cfg.life,
        life_random: cfg.life_random,
        explosiveness: cfg.explosiveness,
        prewarm: cfg.prewarm,
        time_scale: cfg.time_scale,
        seed,
        shape: cfg.shape.index(),
        shape_size: cfg.shape_size,
        speed: cfg.speed,
        speed_random: cfg.speed_random,
        angle: cfg.angle,
        spread: cfg.spread,
        gravity: cfg.gravity,
        damping: cfg.damping,
        size: cfg.size,
        size_random: cfg.size_random,
        size_end: cfg.size_end,
        color: cfg.color,
        color_end: cfg.color_end,
        space: u8::from(cfg.space == ph2d_ecs::ParticleSpace::Local),
        start_on: cfg.start_on.clone(),
        stop_on: cfg.stop_on.clone(),
        restart_on: cfg.restart_on.clone(),
        finished_signal: cfg.finished_signal.clone(),
        clock_playing,
        alive,
        selected_count,
    })
}

/// Escreve um número no sítio dele. `true` = o valor MUDOU.
///
/// ⚠️ **Decide com uma LEITURA e só depois empresta em escrita** — a lei do `script_inspector`: o
/// `bevy` marca a alteração no `deref_mut`, e um componente tocado por nada faz a ponte deitar fora
/// o grafo e RENASCER o emissor (o `stale` dela compara o config). *Aqui isso seria o penacho a
/// piscar em cada quadro em que o rato passa por cima de um campo.*
fn poe_numero(cfg: &mut ParticleEmitter, which: N, v: f32) -> bool {
    let antes = cfg.clone();
    match which {
        N::Amount => cfg.amount = arredonda(v),
        N::Life => cfg.life = v,
        N::LifeRandom => cfg.life_random = v,
        N::Explosiveness => cfg.explosiveness = v,
        N::Prewarm => cfg.prewarm = v,
        N::TimeScale => cfg.time_scale = v,
        N::Seed => cfg.seed = arredonda(v),
        N::ShapeW => cfg.shape_size[0] = v,
        N::ShapeH => cfg.shape_size[1] = v,
        N::Speed => cfg.speed = v,
        N::SpeedRandom => cfg.speed_random = v,
        N::Angle => cfg.angle = v,
        N::Spread => cfg.spread = v,
        N::GravityX => cfg.gravity[0] = v,
        N::GravityY => cfg.gravity[1] = v,
        N::Damping => cfg.damping = v,
        N::Size => cfg.size = v,
        N::SizeRandom => cfg.size_random = v,
        N::SizeEnd => cfg.size_end = v,
    }
    *cfg != antes
}

/// Um campo inteiro lido de um campo de número — negativo é `0`, e o tecto é o do tipo.
///
/// ⚠️ **O `clamp` que estava aqui era INERTE, e foi uma prova de mutação que o disse:** apagá-lo
/// deixava o gate verde porque um `as` de vírgula flutuante para inteiro **satura** em Rust (desde
/// a 1.45) — `-3.0 as u32` é `0` e `1e30 as u32` é `u32::MAX`. *Uma cerca que repete o que a
/// linguagem já garante lê-se como a cerca que falta.* O que NÃO é de graça é o `round`: sem ele
/// `12.6` partículas viram `12`, e é isso que o gate mede.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "o `as` satura nas duas pontas — ver o doc acima"
)]
fn arredonda(v: f32) -> u32 {
    v.round() as u32
}

/// **Aplica uma edição.** `true` = o documento mudou.
pub fn apply(sim: &mut SimWorld, bits: u64, edit: &E) -> bool {
    let e = Entity::from_bits(bits);
    let Some(mut cfg) = sim.world_mut().get_mut::<ParticleEmitter>(e) else {
        return false;
    };
    // ⚠️ O `get_mut` do bevy já marcou a alteração; o que decide se ela CONTA é a comparação
    // abaixo, e é por isso que cada braço devolve *«mudou?»* em vez de escrever e calar.
    let antes = cfg.clone();
    match edit {
        E::Number(which, v) => {
            poe_numero(&mut cfg, *which, *v);
        }
        E::Emitting(on) => cfg.emitting = *on,
        E::OneShot(on) => cfg.one_shot = *on,
        E::Shape(i) => {
            let Some(&s) = ph2d_ecs::EmissionShape::ALL.get(*i as usize) else {
                return false;
            };
            cfg.shape = s;
        }
        E::Space(i) => {
            cfg.space = if *i == 0 {
                ph2d_ecs::ParticleSpace::World
            } else {
                ph2d_ecs::ParticleSpace::Local
            };
        }
        E::Color(fim, c) => {
            if *fim {
                cfg.color_end = *c;
            } else {
                cfg.color = *c;
            }
        }
        E::Text(which, t) => match which {
            T::StartOn => cfg.start_on.clone_from(t),
            T::StopOn => cfg.stop_on.clone_from(t),
            T::RestartOn => cfg.restart_on.clone_from(t),
            T::FinishedSignal => cfg.finished_signal.clone_from(t),
        },
    }
    *cfg != antes
}

/// **Aplica as edições de um quadro.** `true` = o documento mudou.
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, E)]) -> bool {
    let mut mexeu = false;
    for (bits, edit) in edits {
        mexeu |= apply(sim, *bits, edit);
    }
    mexeu
}

#[cfg(test)]
#[path = "particles_inspector_tests.rs"]
mod tests;

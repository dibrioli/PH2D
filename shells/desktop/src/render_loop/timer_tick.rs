//! ⭐⭐⭐ **A PONTE DO `Timer`** — o tique no passo fixo, e o sinal que sai dele (TOP-20 #2, W2).
//!
//! # O que esta ponte torna possível
//!
//! Até 2026-09-08 o **único** produtor de sinal autorável da cena era o contacto da física
//! (`SignalOnHit`): nada podia acontecer **por si**, todo evento de jogo precisava de dois corpos a
//! tocarem-se. O `Timer` é o primeiro relógio que o artista põe num objecto, e é ele que fecha o
//! circuito que o levantamento chama de *«sinais viram jogo»*.
//!
//! # ⚠️ As três leis que este ficheiro honra, e onde cada uma foi paga
//!
//! - **`ticks × dt` numa chamada só.** É a lição do irmão [`super::sprite_anim_tick`], onde ela foi
//!   uma MEDIÇÃO: a primeira versão dele corria um laço de `ticks` chamadas *«porque um passo
//!   grande atravessaria o fim de um ciclo sem o fechar»* — e **era falso**, porque a lei pura tem
//!   o próprio laço de recuperação. A [`ph2d_ecs::timer_advance`] tem-no também.
//! - **O relógio NÃO é documento.** O [`ph2d_ecs::Timers`] é CONFIG e está registado; o
//!   [`ph2d_ecs::TimerRuntime`] é o estado vivo e **não** está. ⇒ este tique escreve num componente
//!   que o undo não fotografa, e por isso **não precisa do ledger do `preview_drive`** — a
//!   separação faz o trabalho que lá seria feito por declaração.
//! - **Um tique atrasado colapsa num sinal só, com a contagem dentro** — a lei do `cycles`/`rows`.
//!
//! # ⚠️ A reconciliação de comprimento, e porque ela vive AQUI
//!
//! O `Timers` e o `TimerRuntime` são dois vectores ligados pelo **índice**. O artista acrescenta e
//! remove timers pelo painel, e o runtime tem de acompanhar. ⛔ **A alternativa — ligar por NOME —
//! foi recusada no desenho**: o nome é editável a meio, e o relógio saltaria de timer debaixo da
//! mão do artista.

use ph2d_ecs::{Entity, SimWorld, Timers};

/// Um sinal que um timer produziu neste tique.
pub(crate) struct TimerSignal {
    pub(crate) entity: Entity,
    pub(crate) name: String,
    /// Quantos períodos fecharam. **Sempre ≥ 1** — zero não chega a virar um destes.
    pub(crate) fires: u32,
}

/// Avança todo timer da cena em `ticks` passos fixos e devolve o que eles têm a dizer.
///
/// ⚠️ **Nada é publicado aqui**, e a razão é a mesma do irmão: este tique corre **antes** do dreno
/// da timeline, e um sinal publicado antes de o quadro virar chegaria ao consumidor um quadro
/// atrasado. Um atraso de um quadro é invisível num toast e deixa de o ser quando o consumidor for
/// **som**.
pub(crate) fn tick_timers(sim: &mut SimWorld, ticks: u32, fixed_dt: f64) -> Vec<TimerSignal> {
    let mut out = Vec::new();
    if ticks == 0 {
        return out;
    }
    let dt = super::sprite_anim_tick::step_ticks(fixed_dt) * u64::from(ticks);
    if dt == 0 {
        return out;
    }
    let world = sim.world_mut();
    let mut q = world.query::<(Entity, &Timers, &mut ph2d_ecs::TimerRuntime)>();
    for (entity, timers, mut rt) in q.iter_mut(world) {
        // ⚠️ **Só toca o vector quando o comprimento MUDOU.** O `bevy` marca a alteração no
        // `deref_mut`, e um `resize` incondicional marcaria o componente todo o quadro — ruído
        // para quem lê `Changed<…>`, e o hábito que o `SpriteGrid` já teve de corrigir.
        if rt.0.len() != timers.0.len() {
            rt.0.resize(timers.0.len(), ph2d_ecs::TimerState::default());
        }
        for (t, s) in timers.0.iter().zip(rt.0.iter_mut()) {
            let outcome = ph2d_ecs::timer_advance(t, s, dt);
            // ⚠️ **Vazio = calado**, e o teste é do NOME do sinal, não do disparo: um timer sem
            // nome cumpre o período e não fala. É a lei da §11 — *um produtor sem nome não fala,
            // em vez de falar com um nome vazio.*
            if outcome.fires > 0 && !t.signal.is_empty() {
                out.push(TimerSignal {
                    entity,
                    name: t.signal.clone(),
                    fires: outcome.fires,
                });
            }
        }
    }
    out
}

/// **O `autostart` a fazer o que promete**, no único momento em que ele tem significado no editor:
/// o projeto acabou de abrir.
///
/// ⚠️ **Ele não pode viver no tique**, e a razão é a que o `start_autoplay_animations` já escreve:
/// *«começar a correr» é uma ARESTA, e o tique só vê estados* — detectá-la ali pediria um bit a
/// mais que diz «já comecei», e um bit desses fica dessincronizado no primeiro `Ctrl+Z`.
///
/// ⚠️ **Ele também é quem CRIA o runtime**: o `Timers` viaja no ficheiro e o `TimerRuntime` não,
/// então uma entidade acabada de carregar tem a config e não tem relógio. Sem esta metade, um
/// projeto reaberto teria os timers todos mudos — e a query do tique nem os veria.
pub(crate) fn start_autostart_timers(sim: &mut SimWorld) {
    let world = sim.world_mut();
    let alvos: Vec<Entity> = {
        let mut q = world.query::<(Entity, &Timers)>();
        q.iter(world).map(|(e, _)| e).collect()
    };
    for e in alvos {
        let Some(timers) = world.get::<Timers>(e).cloned() else {
            continue;
        };
        let mut rt = world
            .get::<ph2d_ecs::TimerRuntime>(e)
            .cloned()
            .unwrap_or_default();
        ph2d_ecs::arm_autostart(&timers, &mut rt);
        world.entity_mut(e).insert(rt);
    }
}

/// Os gates desta ponte — módulo irmão, pelo teto de 600 LOC da shell.
#[cfg(test)]
#[path = "timer_tick_tests.rs"]
mod tests;

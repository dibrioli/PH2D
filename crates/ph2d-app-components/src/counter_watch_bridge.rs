//! ⭐⭐⭐ **A PONTE da vigia do contador** — onde a lei pura da [`ph2d_ecs::counter_watch`] encontra
//! o mundo, e onde uma travessia vira um sinal.
//!
//! # ⚠️ As três leis que esta ponte honra, e onde cada uma foi paga
//!
//! 1. **Ela lê pela PORTA** ([`ph2d_ecs::counter::soma`]) — a mesma que o `UiLabel` usa. ⛔ Uma
//!    segunda conta aqui faria o placar mostrar um número e a regra reagir a outro, e as duas
//!    concordariam em toda cena com **um** contador só, que é a cena de qualquer teste à pressa.
//! 2. **Ela corre no passo fixo e só com o relógio a ANDAR** — um editor parado não mata o herói.
//!    ⚠️ *Mas o `reconcile` corre sempre*, senão uma regra acabada de acrescentar no Inspector só
//!    ganharia estado no primeiro tique, e quem lesse o painel antes disso veria uma lista de
//!    regras sem lugar nenhum onde guardar a aresta.
//! 3. **Ela fala ANTES de a tabela de acções ler** — a janela dos motores (`fase_signal_outbox`).
//!    ⚠️ **A latência é a mesma nas duas margens e isso foi medido antes de escolher:** aqui a
//!    vigia vê o valor que o quadro anterior deixou e a reacção é no MESMO quadro; do outro lado
//!    veria o valor de agora e a reacção seria no seguinte. `1` quadro nas duas ⇒ escolhe-se a que
//!    fica junto dos irmãos, que é a que o censo de ordem do quadro já cobre.

use ph2d_ecs::{CounterWatch, CounterWatchRuntime, SimWorld, counter, counter_watch};

/// O que um quadro da vigia produziu.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VigiaFrame {
    /// As travessias deste quadro: `(bits da entidade, índice da regra, nome do sinal)`.
    pub disparos: Vec<(u64, u16, String)>,
    /// Quantas regras apontam a um contador que **não existe** — o número que o painel mostra.
    ///
    /// ⚠️ **Contado, não gritado:** apontar a um contador que ainda não foi criado é uma
    /// configuração a meio, não uma avaria (a lei do `inert` da tabela de acções).
    pub orfas: usize,
}

/// ⭐⭐ **Um quadro da vigia.**
///
/// ⚠️ **O `reconcile` corre com o relógio parado e o `advance` não** — ver a lei 2 do cabeçalho.
pub fn frame(sim: &mut SimWorld, playing: bool, ticks: u32) -> VigiaFrame {
    let mut out = VigiaFrame::default();
    let world = sim.world_mut();

    // ── Os slots primeiro, e sempre ──────────────────────────────────────────
    let alvos: Vec<bevy_ecs::entity::Entity> = world
        .query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::prelude::With<CounterWatch>>()
        .iter(world)
        .collect();
    for e in &alvos {
        let Some(cfg) = world.get::<CounterWatch>(*e).cloned() else {
            continue;
        };
        if world.get::<CounterWatchRuntime>(*e).is_none() {
            world.entity_mut(*e).insert(CounterWatchRuntime(Vec::new()));
        }
        if let Some(mut rt) = world.get_mut::<CounterWatchRuntime>(*e) {
            counter_watch::reconcile(&cfg, &mut rt);
        }
    }

    // ⚠️ **As órfãs contam-se em todo quadro**, mesmo parado: é o painel que as mostra, e um aviso
    // que só aparece com o jogo a correr é um aviso que o artista nunca vê enquanto autora.
    for e in &alvos {
        let Some(cfg) = world.get::<CounterWatch>(*e).cloned() else {
            continue;
        };
        for row in &cfg.0 {
            if counter::soma(world, &row.counter, row.scope.ambito(*e)).is_none() {
                out.orfas += 1;
            }
        }
    }

    if !playing || ticks == 0 {
        return out;
    }

    // ── A LEI ────────────────────────────────────────────────────────────────
    // ⚠️ **UMA travessia por QUADRO e não por tique**, e é deliberado: a soma de um contador só
    // muda quando a tabela de acções corre, o que acontece **uma vez por quadro**. Correr a lei N
    // vezes sobre o mesmo número daria N-1 avaliações que não podem mudar de resposta — e a aresta
    // já as ignoraria, mas o custo seria real.
    for e in alvos {
        let Some(cfg) = world.get::<CounterWatch>(e).cloned() else {
            continue;
        };
        // ⚠️ As somas colhem-se ANTES de emprestar o slot — a porta lê `&World` e o `get_mut`
        // seguinte quer `&mut`.
        let somas: Vec<Option<i64>> = cfg
            .0
            .iter()
            .map(|row| counter::soma(world, &row.counter, row.scope.ambito(e)))
            .collect();
        let Some(mut rt) = world.get_mut::<CounterWatchRuntime>(e) else {
            continue;
        };
        for (i, (row, soma)) in cfg.0.iter().zip(somas).enumerate() {
            let Some(state) = rt.0.get_mut(i) else {
                continue;
            };
            if counter_watch::advance(row, state, soma) {
                out.disparos.push((
                    e.to_bits(),
                    u16::try_from(i).unwrap_or(u16::MAX),
                    row.signal.trim().to_string(),
                ));
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "counter_watch_bridge_tests.rs"]
mod tests;

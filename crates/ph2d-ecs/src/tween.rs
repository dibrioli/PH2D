//! **O TWEEN de um objecto** (suplente #22) — que propriedade vai de A a B, e nada mais.
//!
//! # ⛔⛔ O que este módulo NÃO tem, e porquê
//!
//! Ele **não tem relógio** e **não tem estado vivo**.
//!
//! O relógio é o [`Timer`](crate::Timer) — a mesma medição que o `SequencePlayer` (#19) pagou antes
//! dele, e ela vale aqui letra por letra: duração · repetir · `autostart` · o sinal a cada disparo ·
//! um `TimerRuntime` cujo `progress()` é **derivado**, que um sinal já arranca (`StartTimer`) e que
//! o [`rewind_runtime`](crate::rewind_runtime) já faz **renascer**.
//!
//! ⭐⭐⭐ **E daí sai a propriedade mais forte desta wave: o tween é uma FUNÇÃO PURA do relógio.**
//! Ele não guarda nada, logo **rebobinar já funciona** — sem uma linha nova no `rewind_runtime` e
//! sem uma entrada nova no censo. É o mesmo que o emissor de partículas (#18) mediu para si
//! próprio: *as vivas são função pura do playhead ⇒ scrub e replay de graça.*
//!
//! # ⚠️ O RELÓGIO é o timer do MESMO ÍNDICE
//!
//! O tween `i` corre no timer `i`. É **a mesma lei** que liga o [`Timers`](crate::Timers) ao
//! [`TimerRuntime`](crate::TimerRuntime), e o doc dela já a escreve: *«o índice casa, e é isso que
//! os liga — não um nome. Um nome ligaria dois vectores por uma string que o artista pode editar a
//! meio, e o relógio saltaria de timer.»*
//!
//! ⛔ **E é por isso que o [`TWEENS_MAX`] é DERIVADO** e não escolhido: um tween depois do último
//! timer não tem relógio, logo é **inerte** — e um modelo que aceita o que não pode correr produz
//! estado inalcançável, que é a lei que o `ANIM_TAGS_MAX` desta casa já escreve.
//!
//! ⚠️ **Um tween no índice `0` PARTILHA o relógio com um [`SequencePlayer`](crate::SequencePlayer)**
//! no mesmo objecto, que também toma o primeiro. É **a leitura certa** — *a cutscene e o tween
//! correm juntos* — e não uma colisão: as duas coisas ficam sincronizadas por construção, que é o
//! que alguém que as ponha no mesmo objecto quer. Há gate a afirmá-lo.

use bevy_ecs::prelude::Component;
use ph2d_tween::{Relogio, Tween};
use serde::{Deserialize, Serialize};

use crate::SimComponent;
use crate::timer::{Timer, TimerState};

/// **Quantos tweens um objecto pode ter** — ⭐ **DERIVADO** do [`crate::TIMERS_MAX`], porque o
/// relógio de um tween é o timer do mesmo índice. Ver o cabeçalho do módulo.
pub const TWEENS_MAX: usize = crate::TIMERS_MAX;

/// **Os tweens de uma entidade** — o componente registado.
///
/// ⚠️ **Uma LISTA, como o [`crate::Timers`] e pelo mesmo motivo**: um objecto anima mais do que uma
/// propriedade (desvanecer **enquanto** encolhe), e um componente ECS é único por entidade.
///
/// ⚠️ **É CONFIG inteira** — o que ele guarda é o que o artista escreveu. O *«onde está agora»* é o
/// relógio, que não é registado; é isso que o mantém fora do `Ctrl+Z` sem uma declaração.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Tweens(pub Vec<Tween>);

impl SimComponent for Tweens {}

/// **O relógio, como a lei do [`ph2d_tween`] o vê** — a porta que traduz, e a ÚNICA.
///
/// ⚠️ **Ela existe porque a lei não conhece o ECS**, e é isso que a mantém testável sem um mundo.
/// Escrita duas vezes — uma na ponte e outra num gate —, o dia em que o `finished` mudasse de
/// sentido deixaria as duas a discordar em silêncio.
#[must_use]
pub fn relogio_de(timer: &Timer, estado: &TimerState) -> Relogio {
    Relogio {
        a_correr: estado.running,
        // ⭐ O FACTO da W0: sem ele um *one-shot* terminado e um que nunca começou seriam o mesmo
        // estado, bit a bit, e o `AoAcabar` não teria como escolher.
        acabou: estado.finished,
        progresso: timer.progress(estado),
    }
}

/// **Uma escrita que um tween pede NESTE quadro.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Escrita {
    /// Quem.
    pub entity: bevy_ecs::entity::Entity,
    /// Que propriedade — a ponte usa isto para escolher o campo **e** a entrada do ledger.
    pub canal: ph2d_tween::Canal,
    /// O valor, com as componentes acima da aridade do canal a zero.
    pub valor: [f32; 4],
}

/// ⭐⭐⭐ **O que os tweens da cena pedem neste quadro.**
///
/// ⚠️ **Ausente da lista = não escreve**, e é isso que faz o objecto voltar à pose e à cor da cena
/// sem uma linha de código a repô-las — a lei do `em_corrida` do #19, palavra por palavra.
///
/// ⚠️ **Um tween sem timer no mesmo índice sai da lista**, e ⛔ *não* cai no timer `0`: correr no
/// relógio errado lê-se como um defeito do motor, e não como uma lista curta.
///
/// ⚠️ `query` e não `try_query`: aqui o mundo é `&mut`, logo a consulta regista os componentes que
/// ainda não viu — a armadilha que a `tagged` pagou (um mundo que nunca viu um componente responde
/// *«ninguém»* à consulta INTEIRA) não existe neste caminho.
#[must_use]
pub fn a_escrever(world: &mut bevy_ecs::world::World) -> Vec<Escrita> {
    let mut q = world.query::<(
        bevy_ecs::entity::Entity,
        &Tweens,
        &crate::Timers,
        &crate::TimerRuntime,
    )>();
    let mut fora = Vec::new();
    for (entity, tweens, cfg, rt) in q.iter(world) {
        for (i, t) in tweens.0.iter().enumerate() {
            let (Some(timer), Some(estado)) = (cfg.0.get(i), rt.0.get(i)) else {
                continue;
            };
            if let Some(valor) = ph2d_tween::valor(t, relogio_de(timer, estado)) {
                fora.push(Escrita {
                    entity,
                    canal: t.canal,
                    valor,
                });
            }
        }
    }
    fora
}

#[cfg(test)]
#[path = "tween_tests.rs"]
mod tests;

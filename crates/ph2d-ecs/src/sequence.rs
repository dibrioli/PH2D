//! **A CUTSCENE de um objecto** (TOP-20 #19) — que sequência é a dele, e nada mais.
//!
//! # ⛔⛔ O que este módulo NÃO tem, e porquê
//!
//! Ele **não tem relógio**. A medição de 2026-09-17 (plano 16 §1-bis) mostrou que o relógio de
//! corrida já existe inteiro no [`Timer`](crate::timer::Timer): duração · repetir · `autostart` ·
//! o **sinal a cada disparo** · um `TimerRuntime` com o decorrido cujo `progress()` é **derivado**.
//! Um sinal já o arranca (`StartTimer`) e pára, e o `rewind_runtime` já o faz **renascer**.
//!
//! ⇒ um `SequenceRuntime` seria um **segundo relógio**, e um verbo `PlaySequence` uma segunda
//! maneira de dizer *«começa»*. *Duas respostas à mesma pergunta divergem no dia em que uma delas
//! mudar* — e aqui a divergência seria uma cutscene que corre num tempo e anuncia noutro.
//!
//! ⇒ e é por isso que o descritor dele **EXIGE** o `Timer` (`requires`): sem relógio ele não é meia
//! feature, é uma feature **inerte** — e a casa tem como o dizer na paleta em vez de deixar o
//! artista descobrir.
//!
//! # A sequência chama-se por NOME
//!
//! ⛔ **Nunca por índice.** Os containers vivem num `Vec` do documento da timeline, e apagar o
//! container de cima renumera todos os de baixo — um índice guardado no objecto passaria a tocar a
//! cutscene do vizinho, **em silêncio**. É a mesma lei do `Counter`/`Timer`/`Tag`, que a casa já
//! escreve em quatro sítios: *referência durável entre objectos é o NOME*.
//!
//! # ⚠️ Esta crate NÃO vê a timeline, e a lei fica pura à custa disso
//!
//! `ph2d-ecs` não depende de `ph2d-timeline` (medido). A resolução recebe então **os nomes**, e não
//! o documento — o que a torna testável sem uma timeline e mantém a fronteira onde ela está.

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **A sequência que este objecto toca** — o nome de um container da timeline.
///
/// Ausente = objecto sem cutscene, que é o default de toda cena que já existe.
///
/// ⚠️ **É CONFIG, nunca estado vivo** (a lei do módulo): o nome é autorado e não muda por conta
/// própria, logo o `canonicalize` do undo — que ordena pelos BYTES — não vê um passo por quadro.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequencePlayer {
    /// O nome do container. **Vazio = calado**, a mesma regra do `SignalOnHit` e do marcador da
    /// timeline: *um nome em branco não é um contrato que alguém possa casar*.
    pub container: String,
}

impl SequencePlayer {
    /// O nome, aparado, ou `None` se estiver em branco.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        let t = self.container.trim();
        (!t.is_empty()).then_some(t)
    }

    /// **O ÍNDICE do container deste objecto**, dada a lista de nomes do documento.
    ///
    /// ⚠️ **A comparação é do nome APARADO dos dois lados** — um container chamado `"Porta "` e um
    /// componente que diz `"Porta"` são a mesma cutscene para quem os escreveu, e fazê-los
    /// discordar por um espaço é o defeito que o `signal_name` desta casa já existe para impedir.
    ///
    /// ⚠️ **O PRIMEIRO que casa**, e a escolha é declarada: dois containers com o mesmo nome são um
    /// documento que o editor não devia produzir; escolher o primeiro é determinístico, e escolher
    /// *«nenhum»* faria o artista perder a cutscene por um nome duplicado que ele não vê.
    #[must_use]
    pub fn resolve<'a>(&self, nomes: impl IntoIterator<Item = &'a str>) -> Option<usize> {
        let meu = self.name()?;
        nomes.into_iter().position(|n| n.trim() == meu)
    }
}

impl SimComponent for SequencePlayer {}

/// **Uma cutscene a correr NESTE instante** — o que a ponte precisa de saber, e nada mais.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmCorrida {
    /// Quem a toca.
    pub entity: bevy_ecs::entity::Entity,
    /// O índice do container no documento da timeline.
    pub container: usize,
    /// O instante LOCAL dela, em segundos — o decorrido do relógio do objecto.
    pub t: f64,
}

/// ⭐⭐⭐ **As cutscenes a correr neste quadro, com o instante de cada uma.**
///
/// # ⚠️ O RELÓGIO é o timer `0`, e a escolha está declarada
///
/// Um objecto pode ter vários timers, e **um deles tem de ser o da sequência**. A regra é *o
/// PRIMEIRO*, por três razões que apontam ao mesmo lado:
///
/// * o descritor **exige** os `Timers` (`authored_requiring`), logo anexar a cutscene pela paleta
///   cria exactamente **um** — no caso normal não há ambiguidade nenhuma para resolver;
/// * o `StartTimer` com o argumento vazio arranca **todos**, que é o gesto que o artista faz;
/// * e as alternativas são piores: *«o primeiro a CORRER»* faz a cutscene trocar de relógio quando
///   o artista arranca outro timer para outra coisa, **em silêncio**; e *«o timer com o NOME do
///   container»* deixaria a coisa **inerte** acabada de anexar (o relógio do pacote nasce com o
///   nome de fábrica) — o defeito que esta casa já pagou muitas vezes: *uma ferramenta que não faz
///   nada e não diz porquê é indistinguível de uma partida*.
///
/// ⚠️ **Parado = ausente da lista.** Uma cutscene que não corre não escreve nada, e é isso que faz
/// o objecto voltar à pose da cena sem uma linha de código a repô-la.
///
/// ⚠️ **Um nome que não resolve também sai da lista** — e ⛔ *não* cai no container `0`: tocar a
/// cutscene errada lê-se como um defeito do motor, e não como um nome mal escrito.
#[must_use]
pub fn em_corrida(world: &mut bevy_ecs::world::World, nomes: &[&str]) -> Vec<EmCorrida> {
    // ⚠️ `query` e não `try_query`: aqui o mundo é `&mut`, logo a consulta regista os componentes
    // que ainda não viu — a armadilha que a `tagged` pagou (um mundo que nunca viu um componente
    // responde «ninguém» à consulta INTEIRA) não existe neste caminho.
    let mut q = world.query::<(
        bevy_ecs::entity::Entity,
        &SequencePlayer,
        &crate::timer::TimerRuntime,
    )>();
    let mut fora = Vec::new();
    for (entity, seq, relogios) in q.iter(world) {
        let Some(estado) = relogios.0.first() else {
            continue;
        };
        if !estado.running {
            continue;
        }
        let Some(container) = seq.resolve(nomes.iter().copied()) else {
            continue;
        };
        fora.push(EmCorrida {
            entity,
            container,
            // ⚠️ Microssegundos → segundos, que é a unidade do relógio da timeline. A divisão é em
            // `f64` de propósito: em `f32` um minuto de cutscene já perde o milissegundo.
            t: estado.elapsed_us as f64 / 1_000_000.0,
        });
    }
    fora
}

#[cfg(test)]
#[path = "sequence_tests.rs"]
mod tests;

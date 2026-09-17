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

#[cfg(test)]
#[path = "sequence_tests.rs"]
mod tests;

//! ⭐⭐⭐ **A PORTA de *«quanto vale o contador X?»*** — e a razão de ela viver num módulo só dela.
//!
//! O [`crate::Counter`] nasceu com o HUD (TOP-20 #20) e, enquanto teve **um** leitor — o rótulo —,
//! a soma dele podia viver dentro do `hud::valor`. Com a vigia ([`crate::counter_watch`]) ela
//! passou a ter **dois**, em dois assuntos que não se conhecem: o que **MOSTRA** o número e o que
//! **REAGE** a ele.
//!
//! ⛔⛔ **Escrita duas vezes, o placar mostraria um número e a regra reagiria a outro** — e o modo
//! de falha é mudo: as duas contas concordam em toda cena com um contador só, que é a cena de
//! qualquer teste escrito à pressa. É a mesma forma do `a_vista_deixa_correr` da wave do
//! `SequencePlayer`, e há censo a afirmar que os dois leitores entram por aqui.
//!
//! # ⚠️ As duas leis que esta porta carrega, e nenhuma delas é escolha desta wave
//!
//! 1. **SOMA, nunca «o primeiro».** A ordem de iteração entre arquétipos do bevy **não é
//!    prometida**, logo *«o primeiro `Counter` chamado vidas»* não é uma pergunta com resposta.
//! 2. ⭐⭐ **`None` e `Some(0)` são coisas diferentes.** O doc do `hud::valor` já o escreve para o
//!    rótulo — *«um zero inventado … leria-se como o jogo está a funcionar e a pontuação é zero»* —
//!    e a vigia herda-o com uma consequência mais dura: **uma vigia sobre um contador que não
//!    existe nunca dispara.** Sem isso, escrever `vidaas` num campo faria a regra `AtMost 0`
//!    anunciar *«morreste»* no arranque, e o artista não teria como saber porquê.

use bevy_ecs::world::World;

use crate::{Counter, CounterRuntime, Entity};

/// ⭐⭐⭐ **ONDE procurar o contador** — a terceira pergunta desta porta, e a que faltava.
///
/// # ⚠️ Porque ela entra na ASSINATURA
///
/// Até 2026-09-20 a porta respondia **sempre** pela cena inteira, e quem precisava do contador de
/// UM objecto escrevia a aritmética à mão: o [`crate::weapon`] lê o pente `mundo.get::<Counter>(e)`
/// com o doc dele a explicar porquê. ⛔ *Uma lei escrita em dois sítios ainda não é uma lei* — e a
/// segunda cópia chegou no dia em que a vigia precisou da mesma pergunta.
///
/// ⇒ o âmbito é **parâmetro**, e não um segundo método: um leitor novo é obrigado a dizer qual das
/// duas perguntas está a fazer, e *esquecê-lo é erro de compilação*.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ambito {
    /// **Toda a cena** — o placar. Dez moedas apanhadas por dez objectos somam dez.
    Mundo,
    /// **Só os contadores que vivem NESTE objecto** — a vida de UM inimigo.
    ///
    /// ⚠️ Uma entidade pode ter **um** `Counter`, logo aqui a soma tem no máximo uma parcela — mas
    /// ela continua a ser uma soma, para a lei ser a mesma dos dois lados.
    Objecto(Entity),
}

/// **A soma de todos os contadores com este nome**, ou `None` se não existir nenhum.
///
/// ⚠️ O nome é **aparado nos dois lados** antes de comparar — a mesma lei do
/// `SequencePlayer::resolve` e do alvo da tabela de acções. Um nome vazio não casa com nada (⛔ e
/// **não** casa com todos: um campo por preencher não é um curinga).
///
/// ⚠️ `saturating_add`: um placar não estoura em pânico.
/// ⚠️ **`&World` e não `&mut`**: é uma leitura, e ela corre em sítios onde só há um `&World` — o
/// instantâneo do Inspector entre eles.
///
/// ⛔⛔ **O `try_query` devolve `None` quando QUALQUER componente da consulta é desconhecido do
/// mundo** (a armadilha que o painel *Tags* pagou em 2026-09-14). Aqui isso é benigno **e é
/// deliberado**: um mundo sem `Counter` nenhum não tem contador com este nome, que é exactamente a
/// resposta. ⚠️ Mas ela tem uma consequência REAL no primeiro quadro de uma cena cujo `Counter`
/// ainda não ganhou `CounterRuntime`: a porta diz `None`, o placar não mostra nada e a vigia não
/// dispara. *É o comportamento que o rótulo já tinha* — esta porta não o inventou, herdou-o.
#[must_use]
pub fn soma(world: &World, nome: &str, ambito: Ambito) -> Option<i64> {
    let alvo = nome.trim();
    if alvo.is_empty() {
        return None;
    }
    // ⭐ O âmbito de OBJECTO é uma leitura directa e não uma varredura filtrada: sem isto o custo
    // de um mundo com mil inimigos seria `O(n²)` — cada vigia a varrer todos os contadores.
    if let Ambito::Objecto(e) = ambito {
        let cfg = world.get::<Counter>(e)?;
        if cfg.name.trim() != alvo {
            return None;
        }
        // ⚠️ Sem `CounterRuntime` a resposta é o `start` da config, e **não** `None`: um contador
        // no primeiro quadro da cena já vale o que o artista escreveu. ⛔ Aqui isto NÃO é o que a
        // varredura faz (ela exige os dois componentes), e a diferença é deliberada — ali a
        // pergunta é *«existe algum?»*, aqui o objecto está nomeado e a resposta é sobre ELE.
        return Some(
            world
                .get::<CounterRuntime>(e)
                .map_or(cfg.start, |r| r.value),
        );
    }
    let mut q = world.try_query::<(&Counter, &CounterRuntime)>()?;
    let mut achou = false;
    let mut total: i64 = 0;
    for (cfg, rt) in q.iter(world) {
        if cfg.name.trim() == alvo {
            achou = true;
            total = total.saturating_add(rt.value);
        }
    }
    achou.then_some(total)
}

#[cfg(test)]
#[path = "counter_tests.rs"]
mod tests;

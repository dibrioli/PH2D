//! **Quem é quem** — como o `motion.step` liga cada elemento de hoje à linha de
//! estado que era dele ontem, e como lê uma porta de pulso cujo comprimento não
//! é o do conjunto.
//!
//! Estas eram as duas metades da limitação que o próprio cabeçalho do nó
//! declarava (doc 89 folha 07): *"pareamento POSICIONAL — uma mudança de CONTAGEM
//! desalinha as linhas … uma stream que roda (o emitter) dessincroniza um beat
//! 'global'. Id-keyed pairing … é o follow-up v2"*.
//!
//! ⚠️ **São dois defeitos, e só um deles é o pareamento.** O diagnóstico separa-os
//! porque a cura é diferente:
//!
//! | metade | porquê | cura |
//! |---|---|---|
//! | o **estado** vem do tique ANTERIOR | o conjunto girou entre os dois tiques: nasceram e morreram peças, e a linha `j` de ontem é de outro elemento | [`pairing`] — casa por `id` |
//! | o **pulso** vem do MESMO tique | não gira; o que muda é o COMPRIMENTO, quando o batimento é global (uma linha só) | [`beat_at`] — a escada `0/1/n` |
//!
//! ⛔ Parear o pulso por `id` seria trabalho a mais a resolver nada: ele é cozido
//! no mesmo tique e a partir do mesmo gerador, então a linha `i` já é do elemento
//! `i`. *O que desalinha é o tempo, não a porta.*

use ph2d_nodegraph::attr::{Column, Stream};
use std::collections::BTreeMap;

/// Per-element **identity keys** of a stream: the `id` column when present (the
/// particle emitter stamps one, stable across a birth/death churn), else `None`
/// — the caller then knows identity is positional.
///
/// ⚠️ Cópia local dos ~15 linhas do `motion.integrate::columns::ids_of`, pela
/// convenção que a biblioteca já escolheu (drop-crate: cada nó é folha). A
/// extração para uma porta só é uma wave que toca nove crates.
fn ids_of(s: &Stream, n: usize) -> Option<Vec<u32>> {
    match s.get("id") {
        // Ids are authored as exact small integers in an f32 (`u32 as f32` is
        // lossless below 2^24), so the cast back is exact. A negative /
        // non-finite id is a corrupt stream: key it to 0.
        Some(Column::Scalar(v)) => {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "ids are small exact integers; the max() floors the corrupt case"
            )]
            let mut out: Vec<u32> = v.iter().map(|f| f.max(0.0) as u32).collect();
            out.resize(n, 0);
            Some(out)
        }
        _ => None,
    }
}

/// Para cada um dos `n` elementos de hoje, a linha de `state` que guarda o tique
/// dele — `None` para um elemento recém-nascido (a escada dele começa em 0).
///
/// O resultado INTEIRO é `None` (semear tudo) quando o estado ainda não tem a
/// coluna do tique (tique 0, `pre` = Empty) ou quando uma mudança de contagem
/// numa stream SEM `id` diz que o conjunto foi reconstruído.
///
/// ⚠️ **A lei é a do `motion.integrate::pairing`, arm por arm**, e de propósito:
/// os dois nós resolvem o mesmo problema (um estado de ontem contra um conjunto
/// de hoje), e dois nós irmãos com regras diferentes para *"a contagem mudou"*
/// seria a próxima pergunta sem resposta. `BTreeMap` mantém-no determinístico e
/// `O(n log n)`, em vez do `O(n·sn)` de um `position()` ingénuo.
///
/// ⚠️ **Sem `id` e com a contagem mudada, tudo re-semeia** — não se pareia por
/// índice. Uma grelha que passa de 2×2 a 2×3 renumera TODOS os pontos: a linha 1
/// de hoje não é a linha 1 de ontem, e herdar o tique dela é herdar o degrau de
/// outra peça. É a mesma escolha que o integrador fez, pela mesma razão.
pub(crate) fn pairing(input: &Stream, state: &Stream, n: usize) -> Option<Vec<Option<usize>>> {
    let sn = state.count();
    match (ids_of(input, n), ids_of(state, sn)) {
        // Identidade de elemento: casa por id. Um id sem linha anterior é uma
        // peça nova, e recebe `None` (semeia em 0) sem afectar as vizinhas.
        (Some(now), Some(before)) => {
            let index: BTreeMap<u32, usize> =
                before.iter().enumerate().map(|(j, id)| (*id, j)).collect();
            Some(now.iter().map(|id| index.get(id).copied()).collect())
        }
        // Identidade posicional: contagem estável pareia linha a linha.
        _ if sn == n => Some((0..n).map(Some).collect()),
        _ => None,
    }
}

/// O valor de uma porta de PULSO para o elemento `i` — a escada `0/1/n`:
/// **vazia** → `0.0` (nada ligado); **comprimento 1** → o mesmo valor para todos
/// (um batimento GLOBAL); **comprimento n** → um por elemento.
///
/// ⚠️ **O degrau do meio é a cura.** Até esta wave a coluna era esticada até `n`
/// com zeros, e um batimento de uma linha só chegava ao **elemento 0** — o resto
/// do conjunto ficava parado para sempre. Não é um caso de canto: é exactamente
/// o que se autora ao ligar um relógio ao conjunto inteiro, e a leitura na tela
/// era *"o Step não faz nada"*. É a mesma escada, e o mesmo bug, que o
/// `motion.color_ramp` corrigiu no `t`.
pub(crate) fn beat_at(col: &[f32], i: usize) -> f32 {
    match col.len() {
        0 => 0.0,
        1 => col[0],
        _ => col.get(i).copied().unwrap_or(0.0),
    }
}

/// Uma coluna `Scalar` inteira, sem esticar (ausente / mal-tipada → vazia, que é
/// o degrau *"nada ligado"* de [`beat_at`]).
pub(crate) fn raw_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
#[path = "pairing_tests.rs"]
mod tests;

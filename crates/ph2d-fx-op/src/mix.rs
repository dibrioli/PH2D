//! **Duas pilhas de FX, e o que está entre elas** — o alinhamento, que é onde mora a regra que o
//! Enio pediu.
//!
//! # A pergunta difícil não é interpolar, é ALINHAR
//!
//! Misturar dois degraus é aritmética ([`FxOp::mix`]). O que decide se a animação faz sentido é
//! *qual degrau de um lado corresponde a qual do outro* — e a resposta tem de servir ao caso real:
//! **a pilha CRESCE**. O artista grava o *Default*, grava o *Hover*, e só depois acrescenta um
//! blur. Nesse instante um dos estados conhece um degrau que o outro nunca viu.
//!
//! ⚠️ **Por ÍNDICE, e não por tipo**, porque a ordem é a feature: `Shadow → Blur` e
//! `Blur → Shadow` desenham coisas diferentes, e casar por tipo faria um degrau saltar de posição
//! no meio da animação — a pilha reordenar-se sozinha durante um hover.
//!
//! # E o que falta de um lado é o NEUTRO daquele lado
//!
//! É a lei, e ela já é o vocabulário desta casa: um lado sem o degrau é o lado com o degrau que
//! **não faz nada** ([`FxOp::neutral`], todo parâmetro em zero). Enio, 2026-08-21: *"se o filtro
//! só for acrescentado depois de algum estado já ter sido gravado, deve haver a adaptação com
//! aplicação do Filter ou Effect com valor 0 ao default"*.
//!
//! ⚠️ **É a MESMA lei que o `mix_width` da pose já aplica** (*"ausente é uniforme… um lado sem
//! perfil é um lado com o perfil que não faz nada"*) e que a pilha de deformadores já usa
//! (`ZigZagSpec::default()` tem `amplitude: 0.0`). Três canais, uma lei — e é por isso que não há
//! caso especial a escrever em lado nenhum: nada aqui pergunta *"este estado é antigo?"*.

use crate::FxOp;

/// **A pilha em `t`, entre `from` e `to`.**
///
/// O resultado tem o comprimento da MAIOR das duas: um degrau que só existe de um lado não pode
/// desaparecer do resultado, senão ele saltaria à chegada em vez de crescer.
#[must_use]
pub fn mix_stacks(from: &[FxOp], to: &[FxOp], t: f64) -> Vec<FxOp> {
    let n = from.len().max(to.len());
    (0..n)
        .map(|i| match (from.get(i), to.get(i)) {
            // O caso comum: o mesmo degrau dos dois lados, e o valor viaja.
            (Some(&a), Some(&b)) if a.kind == b.kind => a.mix(b, t),
            // ⚠️ **Tipos diferentes no mesmo índice** — o artista trocou um degrau por outro. Não
            // há meio-termo entre um Blur e um Glow, então o que chega ENTRA do próprio neutro;
            // é a mesma lei de quem entra, um nível abaixo (a pose já a aplica aos objetos).
            (Some(_), Some(&b)) | (None, Some(&b)) => FxOp::neutral(b.kind).mix(b, t),
            // Só a PARTIDA o tem: ele sai para o próprio neutro, e desaparece por valor.
            (Some(&a), None) => a.mix(FxOp::neutral(a.kind), t),
            (None, None) => unreachable!("i < max(from.len(), to.len())"),
        })
        .collect()
}

#[cfg(test)]
#[path = "mix_tests.rs"]
mod tests;

//! **A timeline é o TERCEIRO membro da família pré-visualização↔documento.**
//!
//! Enquanto o playhead toca, o `ph2d-timeline` escreve nos objetos a pose que as **curvas** dizem —
//! e um clique naquele quadro empilhava um passo de undo cujo conteúdo era só isso. É o mesmo
//! defeito que a §11 Animation e o solver tinham, e cura-se com o mesmo ledger
//! ([`crate::preview_drive`]).
//!
//! # ⚠️ A medição que desbloqueou isto, e a nota que ela corrigiu
//!
//! A auditoria deixou este caso de fora com duas razões, e **uma delas estava errada**: ela dizia
//! que a alternativa do lado do shell era *«um censo de TODAS as poses por quadro de reprodução, e
//! esse custo é um número que ninguém mediu»*.
//!
//! Não é. O `TimelineDoc` **nomeia** as entidades que anima (`doc.bindings()`), então o censo é
//! **`O(bindings)`** — a dúzia de objetos que o artista keyou — e não `O(mundo)`. *Uma ausência
//! afirmada sem olhar a API é um palpite com cara de medição*, e é a segunda vez no mesmo dia que
//! esta linha paga essa (a primeira foi *«este app não tem diálogo de ficheiro»*).
//!
//! ⚠️ A outra razão continua verdadeira e é o que decide o DESENHO: a escrita mora **dentro** da
//! crate da timeline, então declarar de lá inverteria a dependência. Por isso o shell mede
//! **antes e depois** — exactamente como faz com o solver, e pela mesma razão: é a única forma
//! exacta de saber o que aquele passe escreveu sem lhe mudar a API.
//!
//! # ⛔ O que fica de fora, e o motivo é uma COLISÃO
//!
//! O `apply` escreve quatro componentes: `Transform`, `VecMorph`, `PhysicsJoint` e **`Sprite`** (a
//! opacidade, que vive no `tint`). Os três primeiros entram; o `Sprite` **não**.
//!
//! ⚠️ A razão é que a §11 já conduz o `Sprite` — mas **por CAMPO** (o `frame`), e este censo é por
//! COMPONENTE. As duas entradas coexistiriam no ledger com chaves diferentes, e a substituição
//! escreveria as duas: a que corresse por último ganhava o `frame`. *Duas granularidades sobre o
//! mesmo componente é uma divergência à espera de acontecer*, e o preço de a evitar é uma linha
//! nesta lista em vez de um defeito que aparece «às vezes».
//!
//! ⇒ Uma animação de **opacidade** pura ainda suja a captura. A cura é dar ao ledger um facto
//! `SpriteTint` (por campo, como o `SpriteAnim`), e ela cabe numa wave própria.

use crate::preview_drive::{Driven, PreviewDrive};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::TimelineDoc;

/// A pose de cada entidade que o documento anima — **uma por entidade**, mesmo que ela tenha dez
/// props keyadas.
///
/// ⚠️ **Ordenado e sem repetidos** por construção: a mesma entidade aparece numa binding por prop
/// (`TranslationX` e `TranslationY` são duas), e declarar a mesma pose duas vezes é trabalho a
/// dobrar por nada.
#[must_use]
pub(crate) fn poses_of_bindings(world: &World, doc: &TimelineDoc) -> Vec<(Entity, Transform)> {
    let mut bits: Vec<u64> = doc.bindings().iter().map(|b| b.entity).collect();
    bits.sort_unstable();
    bits.dedup();
    bits.into_iter()
        .filter_map(|b| {
            let e = Entity::try_from_bits(b)?;
            // Uma binding pendurada (o objeto foi apagado) é MOSTRADA a vermelho de propósito —
            // ela não é um erro, é estado que o artista precisa de ver.
            Some((e, *world.get::<Transform>(e)?))
        })
        .collect()
}

/// **O que a timeline mexeu é pré-visualização, não autoria** (`crate::preview_drive`).
///
/// ⚠️ **Declara só quem de facto MUDOU** — a mesma regra do solver. Um objeto keyado cuja curva
/// está plana naquele instante não está a ser conduzido, e mantê-lo no ledger deixaria a `settle`
/// sem nada para esquecer: a reprodução nunca acabaria aos olhos do undo.
pub(crate) fn declare_timeline_writes(
    world: &World,
    before: &[(Entity, Transform)],
    drive: &mut PreviewDrive,
) {
    for &(entity, was) in before {
        let Some(now) = world.get::<Transform>(entity) else {
            continue; // o objeto saiu da cena neste quadro
        };
        if *now != was {
            drive.driven(entity, Driven::SolverPose(was), Driven::SolverPose(*now));
        }
    }
}

#[cfg(test)]
#[path = "timeline_preview_tests.rs"]
mod tests;

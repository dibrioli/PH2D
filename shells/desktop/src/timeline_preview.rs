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
//! # Os QUATRO componentes que o `apply` escreve, e porque nenhum colide
//!
//! Ele escreve `Transform`, `VecMorph`, `PhysicsJoint` e **`Sprite`** — e os **quatro** entram.
//!
//! ⚠️ **A 1.ª versão desta wave deixou o `Sprite` de fora**, e o motivo era real: a §11 já o conduz
//! **por CAMPO** (o `frame`), e um facto por COMPONENTE escreveria por cima dela — a entrada que
//! corresse por último ganhava. *Duas granularidades sobre o mesmo componente é uma divergência à
//! espera de acontecer.*
//!
//! ⭐ **A cura foi olhar o que a curva de facto escreve:** `sprite.tint[3] = f` — **um** número. Os
//! outros dois não-`Transform` são iguais (`morph.t = f`, e um campo do joint). ⇒ o ledger ganhou
//! factos **tão estreitos quanto a escrita** (`SpriteAlpha`, `MorphT`), e a colisão dissolveu-se em
//! vez de ser contornada. *Quando duas granularidades colidem, a pergunta é qual delas é grosseira
//! demais para o que o motor de facto faz.*
//!
//! ⚠️ O `PhysicsJoint` vai **inteiro**, e ali isso é correcto: nenhum outro motor o conduz por
//! campo, e a curva pode keyar qualquer um dos parâmetros dele.

use crate::preview_drive::{Driven, PreviewDrive};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::TimelineDoc;

/// O que uma entidade keyada tinha **antes** do `apply` — os quatro factos que as curvas escrevem.
///
/// ⚠️ Cada um é `Option` porque uma entidade keyada **não tem** necessariamente os quatro: uma
/// forma vetorial não é um joint, e um sprite não tem morph. Ausente = nada a comparar.
pub(crate) struct BoundBefore {
    pub(crate) entity: Entity,
    pub(crate) pose: Option<Transform>,
    pub(crate) alpha: Option<f32>,
    pub(crate) morph_t: Option<f32>,
    pub(crate) joint: Option<ph2d_physics_ecs::PhysicsJoint>,
}

/// O estado de cada entidade que o documento anima — **uma entrada por entidade**, mesmo que ela
/// tenha dez props keyadas.
///
/// ⚠️ Ela chamou-se `poses_of_bindings` enquanto só olhava o `Transform`; o nome deixou de dizer a
/// verdade quando ela passou a carregar os quatro factos, e um nome que mente é pior que um longo.
///
/// ⚠️ **Ordenado e sem repetidos** por construção: a mesma entidade aparece numa binding por prop
/// (`TranslationX` e `TranslationY` são duas), e ler a mesma pose duas vezes é trabalho a dobrar
/// por nada.
#[must_use]
pub(crate) fn state_of_bindings(world: &World, doc: &TimelineDoc) -> Vec<BoundBefore> {
    let mut bits: Vec<u64> = doc.bindings().iter().map(|b| b.entity).collect();
    bits.sort_unstable();
    bits.dedup();
    bits.into_iter()
        .filter_map(|b| {
            let entity = Entity::try_from_bits(b)?;
            // Uma binding pendurada (o objeto foi apagado) é MOSTRADA a vermelho de propósito —
            // ela não é um erro, é estado que o artista precisa de ver. Aqui ela some do censo.
            world.get_entity(entity).ok()?;
            Some(BoundBefore {
                entity,
                pose: world.get::<Transform>(entity).copied(),
                alpha: world.get::<ph2d_render::Sprite>(entity).map(|s| s.tint[3]),
                morph_t: world.get::<ph2d_ecs::VecMorph>(entity).map(|m| m.t),
                joint: world.get::<ph2d_physics_ecs::PhysicsJoint>(entity).copied(),
            })
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
    before: &[BoundBefore],
    drive: &mut PreviewDrive,
) {
    for b in before {
        let e = b.entity;
        // ⚠️ **Uma linha por facto, e o `zip` de `Option`s é o guarda**: um facto que a entidade
        // não tinha antes e não tem agora não se declara, e um que ela perdeu no meio do quadro
        // (o componente foi removido) também não — não há o que repor.
        if let (Some(was), Some(now)) = (b.pose, world.get::<Transform>(e).copied())
            && was != now
        {
            drive.driven(e, Driven::SolverPose(was), Driven::SolverPose(now));
        }
        if let (Some(was), Some(now)) = (
            b.alpha,
            world.get::<ph2d_render::Sprite>(e).map(|s| s.tint[3]),
        ) && was != now
        {
            drive.driven(e, Driven::SpriteAlpha(was), Driven::SpriteAlpha(now));
        }
        if let (Some(was), Some(now)) = (b.morph_t, world.get::<ph2d_ecs::VecMorph>(e).map(|m| m.t))
            && was != now
        {
            drive.driven(e, Driven::MorphT(was), Driven::MorphT(now));
        }
        if let (Some(was), Some(now)) = (
            b.joint,
            world.get::<ph2d_physics_ecs::PhysicsJoint>(e).copied(),
        ) && was != now
        {
            drive.driven(e, Driven::JointParams(was), Driven::JointParams(now));
        }
    }
}

#[cfg(test)]
#[path = "timeline_preview_tests.rs"]
mod tests;

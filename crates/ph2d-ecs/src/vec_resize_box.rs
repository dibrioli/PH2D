//! **RESIZE BOX** — o que a alça do gizmo significa para ESTE objeto (plano UI/UX, W3b).
//!
//! Marcado, arrastar a alça reescreve a **CAIXA** do objeto (a geometria dele). Desmarcado,
//! escala a **POSE** (`Transform.scale`) — que é o comportamento correto para um objeto de
//! **game**, e o que este editor sempre fez.
//!
//! # As duas coisas não são a mesma, e a diferença é herdada
//!
//! A pose de um pai é herdada por todo descendente: é isso que um grafo de cena É. Numa forma
//! solta a distinção é invisível (não há quem herde). Numa **moldura** ela é a feature inteira —
//! escalar estica os filhos, achata a tipografia, e a regra de âncora nunca corre, porque a
//! moldura não mudou de CAIXA, mudou de ESCALA.
//!
//! ⚠️ **E ela não é invisível nem numa forma-folha dentro de um fluxo:** o tamanho de um filho é
//! uma ENTRADA da disposição, e o passe mede a **caixa**. Escalar a pose deixa a caixa onde
//! estava, então o fluxo re-flui em volta de um número que já não descreve o que se vê.
//!
//! # O DEFAULT é derivado, e o componente só grava a DISCORDÂNCIA
//!
//! Molduras e os filhos delas nascem marcados; todo o resto nasce desmarcado. Esse fato é uma
//! função da hierarquia ([`super::vec_resize_box::default_for`]), não um valor a escrever em cada
//! entidade — então o componente é um **override** e existe apenas quando o artista discorda.
//!
//! É o idioma de override que este repo usa desde o `GravityScale` da física, e o que ele compra:
//! **zero churn** (nenhuma moldura precisa de ser tocada na criação), **arquivo limpo** (voltar ao
//! default DESTACA), e o default pode evoluir sem re-escrever arte já salva.
//!
//! # Componente NOVO não bumpa nada
//!
//! Ele cunha `stable_type_id = blake3(NOME)[..8]` próprio ⇒ **zero bump de `PROJECT_SCHEMA`** — o
//! precedente do [`crate::VecAnchors`] (W3), do [`crate::VecLayout`] (W2) e do `PhysicsJoint`.
//! Ausência do componente = o mundo é byte-idêntico ao de antes desta feature.

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::{ChildOf, Entity, VecFrame};

/// **O override** de *"a alça reescreve a caixa deste objeto?"*.
///
/// Ausente = o default derivado ([`default_for`]). Presente = o artista discordou dele.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecResizeBox(pub bool);

/// **O default: molduras e os filhos delas reescrevem a caixa; o resto escala.**
///
/// ⚠️ A pergunta é feita à HIERARQUIA e não a uma lista guardada — uma forma arrastada para dentro
/// de uma moldura passa a valer a regra de dentro no mesmo frame, sem que nada tenha de a
/// re-carimbar. Uma lista teria de ser mantida por quem move objetos, e é assim que ela apodrece.
///
/// A filiação conta **um nível**: quem decide é *ser moldura* ou *ter uma por pai*. Um neto de
/// moldura que não esteja dentro de outra moldura é geometria de dentro de um grupo, e um grupo
/// escala como qualquer objeto de game.
#[must_use]
pub fn default_for(world: &bevy_ecs::world::World, e: Entity) -> bool {
    // ⚠️ **Uma INSTÂNCIA escala a pose, mesmo dentro de uma moldura** (plano UI/UX W5). A caixa
    // guardada de uma instância é um SUPORTE — um retângulo do tamanho do mestre —, e o que se vê
    // é derivado dele; reescrever esse retângulo mudaria o número que ninguém olha e deixaria o
    // desenho exatamente onde estava. A regra da moldura (*"filho de moldura reescreve a caixa,
    // porque o tamanho é ENTRADA da disposição"*) vale para quem TEM caixa própria.
    if world.get::<crate::VecInstance>(e).is_some() {
        return false;
    }
    if world.get::<VecFrame>(e).is_some() {
        return true;
    }
    world
        .get::<ChildOf>(e)
        .is_some_and(|c| world.get::<VecFrame>(c.parent()).is_some())
}

/// **A resposta que vale**, com o override por cima do default. Porta única.
///
/// ⚠️ Perguntada por quem HONRA (o braço do gizmo) e por quem OFERECE (o checkbox do painel). Duas
/// respostas fariam o checkbox mostrar um estado e a alça fazer outro — e o artista descobriria a
/// divergência arrastando.
#[must_use]
pub fn resizes_box(world: &bevy_ecs::world::World, e: Entity) -> bool {
    world
        .get::<VecResizeBox>(e)
        .map_or_else(|| default_for(world, e), |o| o.0)
}

#[cfg(test)]
#[path = "vec_resize_box_tests.rs"]
mod tests;

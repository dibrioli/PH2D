//! **De quem é esta forma?** — a pergunta do corpo composto, respondida UMA vez
//! (W-Compound / W-PartFace).
//!
//! Um filho que carrega [`Collider`] e **não** [`RigidBody`] é uma **peça**: mais
//! uma forma do corpo ancestral mais próximo. A regra é curta e aparecia escrita
//! **três** vezes — o contorno (para colorir), o Inspector (para NOMEAR o dono) e
//! agora a contagem de peças de um corpo —, cada uma podendo envelhecer por conta
//! própria. Um painel que nomeia um dono diferente daquele em que o solver
//! pendurou a forma é a pior versão desse drift: os dois lados parecem certos.
//!
//! ⚠️ **A PONTE não delega, e isso é deliberado — ela faz outra pergunta.**
//! `bridge::parts::owner_body` sobe até o primeiro ancestral que a ponte de fato
//! CONSTRUIU (`self.bodies`), não até o primeiro que carrega o componente: uma
//! entidade com `RigidBody` e sem `Collider` não é um corpo para o solver (a
//! `BodyQuery` exige os dois), e pendurar uma peça nela seria pendurá-la em nada.
//! ⚠️ **O preço dessa diferença é nomeável:** com um ancestral assim no meio, o
//! §11 diz *"Shape of X"* enquanto a ponte pendura a forma no AVÔ. É estreito (o
//! `Add` do §11 sempre cria os dois componentes juntos, e o `Remove` os tira
//! juntos), precede esta wave, e **não** foi colapsado à força: fazer a ponte
//! delegar trocaria um rótulo impreciso por uma peça SILENCIOSAMENTE descartada.
//!
//! ⚠️ Esta nota nasceu de uma MUTAÇÃO: com o walk reduzido ao pai literal, a
//! suíte da ponte ficou **verde** — porque ela nunca chamou esta função. Foi
//! assim que a minha afirmação de *"quatro lugares"* caiu para três.
//!
//! ⚠️ **A varredura é para CIMA porque `ChildOf` é a única aresta que existe** —
//! não há índice de filhos no ECS (o `subtree_parts` do rig diz o mesmo, pela
//! mesma razão). É por isso que [`count_parts`] recebe os candidatos de uma query
//! do chamador em vez de os descobrir: quem tem a query é quem pode iterar.
//!
//! ⚠️ **Um GRUPO no meio é transparente**, e isso não é conveniência: pôr as
//! formas de uma peça dentro de uma pasta de organização não pode desligá-las do
//! corpo. O walk pula todo ancestral que não seja corpo.

use ph2d_ecs::{ChildOf, Entity, World};

use crate::{Collider, RigidBody};

/// O corpo ancestral mais próximo de `e`, ou `None` se não há nenhum acima.
///
/// ⚠️ **Não considera o próprio `e`**: um corpo não é peça de ninguém, e chamar
/// isto sobre um corpo pergunta *de quem ELE seria peça* — o que a face vazia do
/// §11 de fato quer saber quando oferece *Add Shape to X*.
#[must_use]
pub fn owner_body(world: &World, e: Entity) -> Option<Entity> {
    let mut cur = world.get::<ChildOf>(e).map(ChildOf::parent);
    while let Some(p) = cur {
        if world.get::<RigidBody>(p).is_some() {
            return Some(p);
        }
        cur = world.get::<ChildOf>(p).map(ChildOf::parent);
    }
    None
}

/// O `BodyKind` que governa esta forma: o dela mesma se `e` for corpo, senão o do
/// dono. `None` quando não há corpo nenhum acima — aquela forma **não é
/// simulada**, e quem desenha ou descreve tem de dizer isso em vez de inventar
/// um dono.
#[must_use]
pub fn governing_kind(world: &World, e: Entity) -> Option<crate::BodyKind> {
    if let Some(rb) = world.get::<RigidBody>(e) {
        return Some(rb.kind);
    }
    owner_body(world, e).and_then(|p| world.get::<RigidBody>(p).map(|rb| rb.kind))
}

/// Esta entidade é uma PEÇA — carrega collider, não carrega corpo, e tem um dono?
#[must_use]
pub fn is_part(world: &World, e: Entity) -> bool {
    world.get::<Collider>(e).is_some()
        && world.get::<RigidBody>(e).is_none()
        && owner_body(world, e).is_some()
}

/// Quantas peças estão penduradas em `body`.
///
/// `candidates` vem de uma query do chamador (`&Collider` sem `&RigidBody`) — não
/// há índice de filhos para descer, então a única direção disponível é subir a
/// partir de cada candidato.
///
/// ⚠️ Existe porque uma peça é **invisível dos dois lados**: com o contorno
/// desligado, nada na tela nem no §11 dizia que um corpo era composto.
#[must_use]
pub fn count_parts(
    world: &World,
    body: Entity,
    candidates: impl IntoIterator<Item = Entity>,
) -> usize {
    candidates
        .into_iter()
        .filter(|&e| owner_body(world, e) == Some(body))
        .count()
}

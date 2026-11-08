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

/// **Add ▸ Collision Shape** (W3): a forma `e` nasce FILHA de `pai` (se ele existe), e só ganha o seu
/// `RigidBody{Static}` quando não há corpo acima — com corpo acima é mais uma peça dele.
pub fn pendura_forma(world: &mut World, e: Entity, pai: Option<Entity>) {
    if let Some(p) = pai.filter(|&p| world.get_entity(p).is_ok()) {
        world.entity_mut(e).insert(ChildOf(p));
    }
    if owner_body(world, e).is_none() {
        world.entity_mut(e).insert(RigidBody {
            kind: crate::BodyKind::Static,
        });
    }
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

/// **A massa AUTOMÁTICA de `body`, contando as peças dele** (W-PartMass).
///
/// O seed do toggle *Mass: Auto → Manual* existe, pelas palavras do próprio
/// comentário dele, *"para a massa não saltar quando o toggle vira"* — e num
/// corpo composto ele fazia exatamente isso, porque lia `Collider::auto_mass()`
/// da forma PRÓPRIA e ignorava as peças. Medido numa jangada de duas metades
/// iguais: **seed `0,600` contra massa real `1,200`.** Metade, em silêncio, num
/// gesto cuja razão de existir é não mexer no número.
///
/// ⚠️ **É `Collider::auto_mass` SOMADO, nunca uma segunda fórmula de área.** Uma
/// re-derivação aqui divergiria de `auto_mass` no dia em que uma forma nova
/// entrar no enum, e o sintoma seria uma massa semeada que não bate com a que o
/// corpo tinha — o defeito que esta função existe para fechar, de volta por
/// outra porta.
///
/// `candidates` vem de uma query do chamador, pela mesma razão que
/// [`count_parts`]: `ChildOf` é a única aresta, então só se sobe.
#[must_use]
pub fn auto_mass_with_parts(
    world: &World,
    body: Entity,
    own: &crate::Collider,
    candidates: impl IntoIterator<Item = Entity>,
) -> f32 {
    let parts: f32 = candidates
        .into_iter()
        .filter(|&e| owner_body(world, e) == Some(body))
        .filter_map(|e| world.get::<crate::Collider>(e))
        // ⚠️ Um SENSOR não tem peso que o artista queira semear? **Tem** — e a
        // diferença com o empuxo (`world::shapes::displaces`) é deliberada: um
        // sensor não desloca fluido, mas o rapier lhe dá massa, e a massa que o
        // seed tem de reproduzir é a que o solver de fato usa.
        .map(crate::Collider::auto_mass)
        .sum();
    own.auto_mass() + parts
}

/// `part` é uma peça cujo corpo ancestral mais próximo é `body`?
#[must_use]
pub fn is_part_of(world: &World, part: Entity, body: Entity) -> bool {
    is_part(world, part) && owner_body(world, part) == Some(body)
}

/// **As formas de `body`, na ordem da lista «Shapes»** (W3-B): a dele primeiro (se tem `Collider`),
/// depois cada peça na ordem da árvore (pai antes dos filhos, irmãos por `SiblingOrder`).
///
/// Cada linha é `(bits, nome, tag)` com tag `0` Ball · `1` Box · `2` Capsule — a do Inspector. A
/// descida pára num corpo filho (as peças dele são dele) e atravessa grupos e peças, a mesma regra
/// do [`owner_body`] vista de cima.
#[must_use]
pub fn shapes_in_order(world: &World, body: Entity) -> Vec<(u64, String, u8)> {
    fn linha(world: &World, e: Entity, c: &Collider) -> (u64, String, u8) {
        let tag = match c.shape {
            crate::ColliderShape::Ball { .. } => 0,
            crate::ColliderShape::Cuboid { .. } => 1,
            crate::ColliderShape::Capsule { .. } => 2,
        };
        let nome = world
            .get::<ph2d_ecs::Name>(e)
            .map_or_else(String::new, |n| n.0.clone());
        (e.to_bits(), nome, tag)
    }
    fn desce(world: &World, pai: Entity, out: &mut Vec<(u64, String, u8)>) {
        for f in ph2d_ecs::ordered_children(world, pai) {
            if world.get::<RigidBody>(f).is_some() {
                continue;
            }
            if let Some(c) = world.get::<Collider>(f) {
                out.push(linha(world, f, c));
            }
            desce(world, f, out);
        }
    }
    let mut out = Vec::new();
    if let Some(c) = world.get::<Collider>(body) {
        out.push(linha(world, body, c));
    }
    desce(world, body, &mut out);
    out
}

/// **A forma PRINCIPAL de `body`** (opção B, 08/11): a 1.ª de [`shapes_in_order`] — o `Collider` do
/// próprio corpo se ele o carrega, senão a primeira forma descendente em ordem de árvore (pára num
/// corpo filho, atravessa grupos). É ela que viaja no `BodyDesc`; as outras são peças.
#[must_use]
pub fn main_shape(world: &World, body: Entity) -> Option<Entity> {
    fn desce(world: &World, pai: Entity) -> Option<Entity> {
        world.get::<ph2d_ecs::Children>(pai)?;
        for f in ph2d_ecs::ordered_children(world, pai) {
            if world.get::<RigidBody>(f).is_some() {
                continue;
            }
            if world.get::<Collider>(f).is_some() {
                return Some(f);
            }
            if let Some(m) = desce(world, f) {
                return Some(m);
            }
        }
        None
    }
    if world.get::<Collider>(body).is_some() {
        return Some(body);
    }
    desce(world, body)
}

#[cfg(test)]
mod shapes_tests {
    use super::*;
    use crate::{BodyKind, ColliderShape};
    use ph2d_ecs::Name;

    fn forma(world: &mut World, nome: &str, pai: Entity, shape: ColliderShape) -> Entity {
        let c = Collider {
            shape,
            ..Collider::default()
        };
        world.spawn((Name::new(nome), c, ChildOf(pai))).id()
    }

    /// O «Floor» da cena 120 com mais formas: um grupo no meio, um corpo filho (que leva as dele) e uma
    /// ordem de irmãos que não é a de inserção.
    #[test]
    fn as_formas_saem_na_ordem_da_arvore_e_param_num_corpo_filho() {
        let mut w = World::new();
        let caixa = ColliderShape::Cuboid {
            half_x: 1.0,
            half_y: 1.0,
        };
        let rb = RigidBody {
            kind: BodyKind::Static,
        };
        let chao = w.spawn((Name::new("Floor"), Collider::default(), rb)).id();
        let parede = forma(&mut w, "Invisible Wall", chao, caixa);
        let grupo = w.spawn((Name::new("Group"), ChildOf(chao))).id();
        let teto = forma(&mut w, "Roof", grupo, ColliderShape::Ball { radius: 1.0 });
        let outro = w
            .spawn((Name::new("Crate"), Collider::default(), rb, ChildOf(chao)))
            .id();
        let _dele = forma(&mut w, "Crate lid", outro, caixa);
        ph2d_ecs::set_sibling_order(&mut w, chao, &[grupo, outro, parede]);
        let nomes: Vec<(String, u8)> = shapes_in_order(&w, chao)
            .into_iter()
            .map(|(_, n, t)| (n, t))
            .collect();
        let esperado = [("Floor", 0), ("Roof", 0), ("Invisible Wall", 1)];
        let esperado: Vec<(String, u8)> = esperado.iter().map(|(n, t)| ((*n).into(), *t)).collect();
        assert_eq!(nomes, esperado);
        assert!(is_part_of(&w, teto, chao) && is_part_of(&w, parede, chao));
        assert!(!is_part_of(&w, chao, chao), "o corpo não é peça dele mesmo");
        assert!(!is_part_of(&w, outro, chao), "um corpo filho não é peça");
    }

    /// A forma principal é a 1.ª de `shapes_in_order`: a do corpo, senão a 1.ª filha na ordem dos
    /// irmãos (reordenar muda-a), atravessando grupos e nunca descendo num corpo filho.
    #[test]
    fn a_forma_principal_e_a_primeira_da_ordem_da_arvore() {
        let mut w = World::new();
        let rb = RigidBody {
            kind: BodyKind::Dynamic,
        };
        let corpo = w.spawn((Name::new("Box"), rb)).id();
        assert_eq!(main_shape(&w, corpo), None, "corpo sem forma");
        let filho = w
            .spawn((Name::new("Crate"), Collider::default(), rb, ChildOf(corpo)))
            .id();
        assert_eq!(
            main_shape(&w, corpo),
            None,
            "a forma de um corpo filho é dele"
        );
        let grupo = w.spawn((Name::new("Group"), ChildOf(corpo))).id();
        let a = forma(&mut w, "A", grupo, ColliderShape::Ball { radius: 1.0 });
        let b = forma(&mut w, "B", corpo, ColliderShape::Ball { radius: 1.0 });
        ph2d_ecs::set_sibling_order(&mut w, corpo, &[filho, grupo, b]);
        assert_eq!(main_shape(&w, corpo), Some(a));
        ph2d_ecs::set_sibling_order(&mut w, corpo, &[b, filho, grupo]);
        assert_eq!(main_shape(&w, corpo), Some(b), "reordenar muda a principal");
        w.entity_mut(corpo).insert(Collider::default());
        assert_eq!(
            main_shape(&w, corpo),
            Some(corpo),
            "a do corpo vem primeiro"
        );
    }
}

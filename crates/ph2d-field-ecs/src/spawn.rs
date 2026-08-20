//! **O caminho inverso do [`cook`](crate::cook)**: um [`FieldDoc`] explode numa cena de objetos.
//!
//! Corre uma vez — quando uma peça entra no projeto (uma cena de smoke, um preset, um arquivo de
//! versão antiga). Depois disso a **cena é a fonte** e ninguém volta a chamar isto.

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_field::{FieldDoc, NodeId, NodeKind, NodeShape, Op, Primitive};

use crate::{FieldNode, FieldObject, FieldPose};

/// **O nome que a Hierarquia mostra**, derivado do que o nó é.
///
/// ⚠️ É **conteúdo**, não chrome: entra num `Name`, que é dado do documento e o artista renomeia.
/// Por isso não passa pelo i18n — traduzir o nome de um objeto renomeável mudaria o nome dele ao
/// trocar de idioma. (O painel é o contrário: lá o rótulo é chrome e vai por chave, HR-15.)
#[must_use]
pub fn shape_name(shape: &NodeShape) -> &'static str {
    match shape {
        NodeShape::Combine(Op::Union(_)) => "Union",
        NodeShape::Combine(Op::Intersection(_)) => "Intersect",
        NodeShape::Combine(Op::Difference(_)) => "Difference",
        NodeShape::Leaf(p) => match p {
            Primitive::Box { .. } => "Box",
            Primitive::Sphere { .. } => "Sphere",
            Primitive::Cylinder { .. } => "Cylinder",
            Primitive::Torus { .. } => "Torus",
            Primitive::Extrude { .. } => "Extrude",
            Primitive::Revolve { .. } => "Revolve",
        },
    }
}

/// **Explode `doc` em entidades** e devolve a raiz.
///
/// A raiz recebe [`FieldObject`], `Transform` e `RootOrder` — é ela que a Hierarquia enumera como
/// objeto de topo. Os filhos recebem só o que é deles: nome, forma e pose.
///
/// ⚠️ **A ordem de `Children` é a ordem dos filhos no documento**, e isso é load-bearing: a
/// subtração é `children[0]` menos todos os seguintes. Arrastar uma linha na Hierarquia troca o que
/// é subtraído de quê — o que é a resposta certa, e não um efeito colateral.
///
/// ⚠️ Nós órfãos da arena (que ninguém referencia e que não são a raiz) **não são criados**: eles
/// não teriam pai onde aparecer, e um objeto solto na Hierarquia que não faz parte da peça seria
/// exatamente o tipo de fantasma que ninguém sabe apagar.
pub fn spawn_doc(world: &mut World, doc: &FieldDoc, root_name: &str) -> Entity {
    // Quantos de cada nome já saíram — "Cylinder", "Cylinder 2", "Cylinder 3". Sem isto a
    // Hierarquia mostra três linhas idênticas e o artista não tem como dizer qual é qual.
    let mut used: std::collections::BTreeMap<&'static str, u32> = std::collections::BTreeMap::new();
    let mut spawned: Vec<Option<Entity>> = vec![None; doc.nodes().len()];

    // A arena já vem ordenada filho-antes-de-pai, então uma passagem em frente basta: quando se
    // chega a um pai, os filhos dele já existem.
    for (i, node) in doc.nodes().iter().enumerate() {
        let shape = node.kind.shape();
        let base = shape_name(&shape);
        let n = used.entry(base).or_insert(0);
        *n += 1;
        let name = if *n == 1 {
            base.to_string()
        } else {
            format!("{base} {n}")
        };
        let e = world
            .spawn((
                ph2d_ecs::Name::new(name),
                FieldNode { shape },
                FieldPose { xform: node.xform },
            ))
            .id();
        spawned[i] = Some(e);
    }

    for (i, node) in doc.nodes().iter().enumerate() {
        if let NodeKind::Combine { children, .. } = &node.kind
            && let Some(parent) = spawned[i]
        {
            for c in children {
                if let Some(child) = spawned[c.0 as usize] {
                    world.entity_mut(parent).add_child(child);
                }
            }
        }
    }

    let root = spawned[doc.root().0 as usize].expect("a raiz da arena existe — o doc é válido");
    prune_orphans(world, doc, &spawned, root);

    // ⚠️ A raiz troca o nome derivado pelo nome da PEÇA. Um objeto chamado "Union" no topo da
    // Hierarquia descreve a operação e não a coisa; o artista guardou um suporte, não uma união.
    world.entity_mut(root).insert((
        ph2d_ecs::Name::new(root_name),
        ph2d_ecs::Transform::default(),
        ph2d_ecs::RootOrder(0),
        FieldObject,
    ));
    root
}

/// Remove o que a arena continha e a árvore não alcança. Ver a nota de [`spawn_doc`].
fn prune_orphans(world: &mut World, doc: &FieldDoc, spawned: &[Option<Entity>], root: Entity) {
    let mut reachable = vec![false; doc.nodes().len()];
    let mut stack = vec![doc.root()];
    while let Some(NodeId(i)) = stack.pop() {
        let i = i as usize;
        if reachable[i] {
            continue;
        }
        reachable[i] = true;
        if let NodeKind::Combine { children, .. } = &doc.nodes()[i].kind {
            stack.extend(children.iter().copied());
        }
    }
    for (i, ok) in reachable.iter().enumerate() {
        if !ok
            && let Some(e) = spawned[i]
            && e != root
        {
            world.entity_mut(e).despawn();
        }
    }
}

//! Os gates da porta de ordem dos filhos.

use super::*;
use crate::Transform;

fn scene(n: usize) -> (World, Entity, Vec<Entity>) {
    let mut w = World::new();
    let parent = w.spawn(Transform::IDENTITY).id();
    let kids: Vec<Entity> = (0..n)
        .map(|_| w.spawn((Transform::IDENTITY, ChildOf(parent))).id())
        .collect();
    (w, parent, kids)
}

fn order(w: &World, parent: Entity) -> Vec<Entity> {
    w.get::<Children>(parent)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default()
}

/// **A lista fica exactamente na sequência pedida.**
#[test]
fn the_children_end_up_in_the_requested_sequence() {
    let (mut w, parent, k) = scene(4);
    assert_eq!(order(&w, parent), k);
    let desired = vec![k[2], k[0], k[3], k[1]];
    assert!(reinsert_children_in_order(&mut w, parent, &desired));
    assert_eq!(order(&w, parent), desired);
}

/// **A ordem já pedida é um NO-OP** — e não é higiene: o undo deste editor regista por DIFF, e
/// remover-e-inserir todo o `ChildOf` marcaria cada filho como mudado, num arrasto que não moveu
/// nada.
#[test]
fn asking_for_the_order_it_already_has_changes_nothing() {
    let (mut w, parent, k) = scene(3);
    assert!(!reinsert_children_in_order(&mut w, parent, &k));
    assert_eq!(order(&w, parent), k);
}

/// **Uma entidade estranha é IGNORADA** — ela não é roubada do pai dela.
///
/// ⚠️ Sem esta recusa a porta seria também um *reparent*, e um índice mal resolvido arrastaria
/// para dentro da moldura uma forma que estava noutro sítio da cena.
#[test]
fn an_entity_that_is_not_a_child_is_not_adopted() {
    let (mut w, parent, k) = scene(2);
    let other_parent = w.spawn(Transform::IDENTITY).id();
    let stranger = w.spawn((Transform::IDENTITY, ChildOf(other_parent))).id();
    reinsert_children_in_order(&mut w, parent, &[stranger, k[1], k[0]]);
    assert_eq!(order(&w, parent), vec![k[1], k[0]]);
    assert_eq!(order(&w, other_parent), vec![stranger]);
}

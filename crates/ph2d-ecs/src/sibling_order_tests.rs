//! Testes do [`super`] — irmão pelo idioma de `#[path]` que a crate já usa.

use super::*;
use crate::Transform;

/// Um pai com `n` filhos, criados em sequência.
fn family(n: usize) -> (World, Entity, Vec<Entity>) {
    let mut w = World::new();
    let parent = w.spawn(Transform::IDENTITY).id();
    let kids: Vec<Entity> = (0..n)
        .map(|_| w.spawn((Transform::IDENTITY, ChildOf(parent))).id())
        .collect();
    (w, parent, kids)
}

/// **Todo filho recebe um número, e a ordem congelada é a que a árvore já mostrava.**
#[test]
fn every_child_gets_a_number_in_the_order_the_tree_showed() {
    let (mut w, parent, kids) = family(3);
    assert!(assign_missing_sibling_order(&mut w));
    let orders: Vec<u32> = kids
        .iter()
        .map(|&k| w.get::<SiblingOrder>(k).unwrap().0)
        .collect();
    assert_eq!(orders, vec![0, 1, 2], "a ordem de insercao foi congelada");
    assert_eq!(ordered_children(&w, parent), kids);
}

/// **Idempotente** — e isto não é higiene: sem ela a varredura reescreveria o número de todo
/// filho por quadro, e **toda captura de undo viraria um passo espúrio** (classe BUGS #15).
#[test]
fn running_twice_changes_nothing() {
    let (mut w, _, kids) = family(3);
    assert!(assign_missing_sibling_order(&mut w));
    let before: Vec<u32> = kids
        .iter()
        .map(|&k| w.get::<SiblingOrder>(k).unwrap().0)
        .collect();
    assert!(!assign_missing_sibling_order(&mut w));
    let after: Vec<u32> = kids
        .iter()
        .map(|&k| w.get::<SiblingOrder>(k).unwrap().0)
        .collect();
    assert_eq!(before, after);
}

/// **Um filho novo entra DEPOIS dos que já têm número** — a tela não pisca quando ele nasce.
#[test]
fn a_new_child_lands_after_the_numbered_ones() {
    let (mut w, parent, kids) = family(2);
    assign_missing_sibling_order(&mut w);
    let newcomer = w.spawn((Transform::IDENTITY, ChildOf(parent))).id();
    assert!(assign_missing_sibling_order(&mut w));
    assert_eq!(
        w.get::<SiblingOrder>(newcomer).unwrap().0,
        2,
        "o recem-chegado entra depois do maior (1)",
    );
    assert_eq!(
        ordered_children(&w, parent),
        vec![kids[0], kids[1], newcomer]
    );
}

/// ⭐ **A ordem escrita é DADO, e é o que a torna desfazível.**
///
/// O teste espelha o gesto: o artista arrasta o último para a frente, e a ordem lida volta
/// diferente. Antes desta wave não havia onde escrever — a lista `Children` do bevy é memória
/// de runtime e nunca entrava no snapshot.
#[test]
fn reordering_writes_data_that_the_reader_sees() {
    let (mut w, parent, kids) = family(3);
    assign_missing_sibling_order(&mut w);
    let desired = vec![kids[2], kids[0], kids[1]];
    assert!(set_sibling_order(&mut w, parent, &desired));
    assert_eq!(ordered_children(&w, parent), desired);
}

/// ⚠️ **Reescrever a MESMA ordem não muda nada** — a metade que impede o passo espúrio.
///
/// O undo regista por DIFF: se `set_sibling_order` inserisse o componente sempre, o arquétipo
/// de cada filho mudaria a cada quadro em que o gesto rodasse, e cada quadro viraria um passo.
#[test]
fn rewriting_the_same_order_is_a_no_op() {
    let (mut w, parent, kids) = family(3);
    assign_missing_sibling_order(&mut w);
    assert!(
        !set_sibling_order(&mut w, parent, &kids),
        "a ordem ja era essa — escrever de novo seria um passo de undo por quadro",
    );
}

/// **Quem não é filho deste pai é ignorado** — a porta não pode reparentar por engano.
#[test]
fn an_outsider_in_the_desired_list_is_ignored() {
    let (mut w, parent, kids) = family(2);
    assign_missing_sibling_order(&mut w);
    let outsider = w.spawn(Transform::IDENTITY).id();
    set_sibling_order(&mut w, parent, &[outsider, kids[1], kids[0]]);
    assert!(
        w.get::<SiblingOrder>(outsider).is_none(),
        "um estranho na lista nao pode receber ordem de irmao",
    );
    assert_eq!(
        ordered_children(&w, parent),
        vec![kids[1], kids[0]],
        "os filhos de verdade seguiram os indices que lhes tocaram",
    );
}

/// **Sem número, colate no FIM** — e o desempate entre os sem-número é a ordem de criação.
#[test]
fn an_unnumbered_child_collates_last() {
    let (mut w, parent, kids) = family(2);
    assign_missing_sibling_order(&mut w);
    let fresh = w.spawn((Transform::IDENTITY, ChildOf(parent))).id();
    // Ainda sem varredura: o `fresh` nao tem numero.
    assert_eq!(
        ordered_children(&w, parent),
        vec![kids[0], kids[1], fresh],
        "o sem-numero fica no fim ate a varredura lhe dar um",
    );
}

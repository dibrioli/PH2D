//! Gates do **Z-INDEX** e dos botões Arrange (Enio, 2026-08-04).
//!
//! ⚠️ O keystone é o primeiro: os quatro botões estavam **MORTOS** — escreviam na ordem do vetor
//! da cena, que a projeção da árvore reescreve a cada frame. Eles acendiam, mexiam, e o frame
//! seguinte desfazia. Nenhum gate via, porque o que existia media a CENA (onde a escrita de facto
//! pousava) e não a ÁRVORE (que é quem decide o desenho).
//!
//! A fixture monta a árvore e lê a pilha pelas portas do PRODUTO — o que se prova é uma RELAÇÃO
//! entre o gesto e a pilha, e uma lista escrita à mão afirmaria a relação em vez de a medir.

use super::*;
use ph2d_ecs::scene::{HierarchyWalkState, build_hierarchy_snapshot};
use ph2d_ecs::{ChildOf, Transform, VecPathRef};
use ph2d_vec_scene::ZOrder;

/// A pilha de z (FUNDO → topo) pela porta do produto.
fn stack(sim: &mut SimWorld) -> Vec<VecPathId> {
    let mut state = HierarchyWalkState::new(sim.world_mut());
    let mut scratch = Vec::new();
    let mut snap = HierarchySnapshot::default();
    build_hierarchy_snapshot(sim.world(), &mut state, &mut scratch, &mut snap);
    z_order(&snap)
}

/// Três RAÍZES (`10`, `20`, `30`), com `10` à frente. O mapa path→entidade vem junto.
fn three_roots() -> (SimWorld, VecEntityMap) {
    let mut sim = SimWorld::new();
    let mut map = VecEntityMap::new();
    for (i, id) in [10u64, 20, 30].into_iter().enumerate() {
        let e = sim
            .world_mut()
            .spawn((
                Transform::default(),
                VecPathRef(id),
                RootOrder(u32::try_from(i).unwrap()),
            ))
            .id();
        map.insert(id, e.to_bits());
    }
    (sim, map)
}

/// Um pai (`100`) com três FILHOS (`200`, `201`, `202`), na ordem de inserção.
fn parent_with_kids() -> (SimWorld, VecEntityMap) {
    let mut sim = SimWorld::new();
    let mut map = VecEntityMap::new();
    let p = sim
        .world_mut()
        .spawn((Transform::default(), VecPathRef(100), RootOrder(0)))
        .id();
    map.insert(100, p.to_bits());
    for id in [200u64, 201, 202] {
        let e = sim
            .world_mut()
            .spawn((Transform::default(), VecPathRef(id), ChildOf(p)))
            .id();
        map.insert(id, e.to_bits());
    }
    (sim, map)
}

/// **O KEYSTONE: o botão move a forma na PILHA.**
///
/// ⚠️ A mutação que o mata é voltar a escrever no `VecScene::reorder_path`: a cena muda, este gate
/// (que lê a ÁRVORE) fica vermelho, e o produto volta a ter quatro botões que não fazem nada.
#[test]
fn to_front_puts_the_shape_on_top_of_its_siblings() {
    let (mut sim, map) = three_roots();
    let before = stack(&mut sim);
    assert_eq!(*before.last().unwrap(), 10, "10 nasce na frente");

    assert!(reorder(&mut sim, &map, 30, ZOrder::ToFront));
    let after = stack(&mut sim);
    assert_eq!(
        *after.last().unwrap(),
        30,
        "o To Front nao poe a forma na frente: {after:?}"
    );
    assert_eq!(after.len(), before.len(), "alguem sumiu da pilha");
}

/// **Um passo é UM passo** — e os quatro verbos concordam sobre para que lado é a frente.
#[test]
fn raise_and_lower_move_exactly_one_place() {
    let (mut sim, map) = three_roots();
    // `20` está no meio: subir põe-no na frente de `10`.
    assert!(reorder(&mut sim, &map, 20, ZOrder::Raise));
    assert_eq!(stack(&mut sim), vec![30, 10, 20], "Raise andou errado");
    // E descer devolve-o ao meio.
    assert!(reorder(&mut sim, &map, 20, ZOrder::Lower));
    assert_eq!(stack(&mut sim), vec![30, 20, 10], "Lower nao e' o inverso");
}

/// **Quem já está no topo não se mexe** — e o gesto devolve `false`.
///
/// ⚠️ O `false` não é higiene: o undo global regista por DIFF, e um passo que não muda nada seria
/// um Ctrl+Z que o artista gasta sem ver nada acontecer.
#[test]
fn a_move_that_changes_nothing_is_refused() {
    let (mut sim, map) = three_roots();
    assert!(!reorder(&mut sim, &map, 10, ZOrder::ToFront));
    assert!(!reorder(&mut sim, &map, 10, ZOrder::Raise));
    assert!(!reorder(&mut sim, &map, 30, ZOrder::ToBack));
    assert!(!reorder(&mut sim, &map, 30, ZOrder::Lower));
}

/// **Um FILHO reordena entre os IRMÃOS dele** — e o pai não sai do lugar.
///
/// ⚠️ Esta é a metade que o `RootOrder` sozinho não alcança: um filho não tem `RootOrder`, e
/// escrever um nele não move nada. A ordem dele mora na sequência do `Children`.
#[test]
fn a_child_reorders_among_its_siblings_without_moving_the_parent() {
    let (mut sim, map) = parent_with_kids();
    let before = stack(&mut sim);
    assert_eq!(*before.last().unwrap(), 100, "o pai e' o ultimo da pilha");

    assert!(reorder(&mut sim, &map, 202, ZOrder::ToFront));
    let after = stack(&mut sim);
    assert_eq!(
        *after.last().unwrap(),
        100,
        "reordenar um filho mexeu no PAI: {after:?}"
    );
    // Entre os filhos, `202` passou a ser o da frente (o mais perto do fim, antes do pai).
    let kids: Vec<VecPathId> = after.iter().copied().filter(|id| *id != 100).collect();
    assert_eq!(*kids.last().unwrap(), 202, "o filho nao subiu: {after:?}");
    assert_eq!(kids.len(), 3, "um filho sumiu");
}

/// **Um filho ÚNICO não tem pilha em que andar.**
#[test]
fn an_only_child_has_nowhere_to_go() {
    let mut sim = SimWorld::new();
    let mut map = VecEntityMap::new();
    let p = sim
        .world_mut()
        .spawn((Transform::default(), VecPathRef(100), RootOrder(0)))
        .id();
    map.insert(100, p.to_bits());
    let c = sim
        .world_mut()
        .spawn((Transform::default(), VecPathRef(200), ChildOf(p)))
        .id();
    map.insert(200, c.to_bits());
    assert!(!reorder(&mut sim, &map, 200, ZOrder::ToFront));
}

/// **O Z-INDEX é o lugar na pilha dos irmãos, e MAIOR é a FRENTE** (a convenção do Godot).
///
/// ⚠️ Sem a segunda metade (`n`), o número seria incomparável: *"Z 2"* não diz se a forma está no
/// topo ou no meio, e o artista teria de contar as irmãs na Hierarquia para saber.
#[test]
fn the_z_index_counts_from_the_back_and_names_the_total() {
    let (mut sim, map) = three_roots();
    assert_eq!(
        z_index(&mut sim, &map, 10),
        Some((2, 3)),
        "10 esta' na frente"
    );
    assert_eq!(z_index(&mut sim, &map, 20), Some((1, 3)));
    assert_eq!(
        z_index(&mut sim, &map, 30),
        Some((0, 3)),
        "30 esta' no fundo"
    );
}

/// **O número que o painel mostra é o lugar que o botão move** — as duas portas concordam.
///
/// ⚠️ É o gate que impede o readout de virar decoração: uma `z_index` com a própria travessia
/// responderia hoje o mesmo e divergiria no dia em que o critério de desempate mudasse num lado só.
#[test]
fn the_readout_follows_the_button() {
    let (mut sim, map) = three_roots();
    assert_eq!(z_index(&mut sim, &map, 30), Some((0, 3)));
    reorder(&mut sim, &map, 30, ZOrder::Raise);
    assert_eq!(
        z_index(&mut sim, &map, 30),
        Some((1, 3)),
        "o readout nao acompanhou o botao"
    );
    reorder(&mut sim, &map, 30, ZOrder::ToFront);
    assert_eq!(z_index(&mut sim, &map, 30), Some((2, 3)));
}

/// **O Z-index de um filho conta os IRMÃOS, não o documento.**
#[test]
fn the_z_index_of_a_child_is_among_its_siblings() {
    let (mut sim, map) = parent_with_kids();
    assert_eq!(
        z_index(&mut sim, &map, 200),
        Some((2, 3)),
        "o filho da frente e' o 200, entre TRES irmaos"
    );
    // O pai é raiz única: um de um.
    assert_eq!(z_index(&mut sim, &map, 100), Some((0, 1)));
}

//! Gates do **Z-INDEX GLOBAL** e dos botões Arrange (Enio, 2026-08-04).
//!
//! ⚠️ **A lei mudou duas vezes, e a segunda é a que este arquivo mede.** Primeiro os quatro botões
//! estavam MORTOS (escreviam na ordem do vetor da cena, que a projeção reescreve a cada frame) e
//! passaram a escrever na ÁRVORE. Depois o Enio pediu o modelo do Godot — *"a ordem só conta se o
//! Z dos objetos for igual; o Z index deve ser global e sobrepõe a ordem na hierarquia"* — e a
//! pilha ganhou **duas chaves**: o Z efetivo manda, o DFS desempata.
//!
//! A fixture monta a árvore e lê a pilha pelas portas do PRODUTO — o que se prova é uma RELAÇÃO
//! entre o gesto e a pilha, e uma lista escrita à mão afirmaria a relação em vez de a medir.

use super::*;
use ph2d_ecs::scene::{HierarchyWalkState, build_hierarchy_snapshot};
use ph2d_ecs::{ChildOf, Transform, VecPathRef, ZIndexOverride};
use ph2d_vec_scene::ZOrder;

/// A pilha de z (FUNDO → topo) pela porta do produto.
fn stack(sim: &mut SimWorld) -> Vec<VecPathId> {
    let mut state = HierarchyWalkState::new(sim.world_mut());
    let mut scratch = Vec::new();
    let mut snap = HierarchySnapshot::default();
    build_hierarchy_snapshot(sim.world(), &mut state, &mut scratch, &mut snap);
    z_order(sim.world(), &snap)
}

/// Três RAÍZES (`10`, `20`, `30`) na ordem da Hierarquia — e desde a lei de Godot **`30` é a da
/// FRENTE**, porque a última linha da lista é a que desenha por cima.
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

/// A entidade de um caminho, para semear um Z à mão.
fn set_z(sim: &mut SimWorld, map: &VecEntityMap, id: VecPathId, z: i32) {
    let e = Entity::from_bits(*map.get(&id).unwrap());
    sim.world_mut().entity_mut(e).insert(ZIndexOverride(z));
}

/// **O KEYSTONE: o botão move a forma na PILHA.**
///
/// ⚠️ A mutação que o mata é voltar a escrever no `VecScene::reorder_path`: a cena muda, este gate
/// (que lê a ÁRVORE) fica vermelho, e o produto volta a ter quatro botões que não fazem nada.
#[test]
fn to_front_puts_the_shape_on_top_of_its_siblings() {
    let (mut sim, map) = three_roots();
    let before = stack(&mut sim);
    assert_eq!(*before.last().unwrap(), 30, "30 nasce na frente");

    assert!(reorder(&mut sim, &map, 10, ZOrder::ToFront));
    let after = stack(&mut sim);
    assert_eq!(
        *after.last().unwrap(),
        10,
        "o To Front nao poe a forma na frente: {after:?}"
    );
    assert_eq!(after.len(), before.len(), "alguem sumiu da pilha");
}

/// **Um passo é UM passo** — e os quatro verbos concordam sobre para que lado é a frente.
#[test]
fn raise_and_lower_move_exactly_one_place() {
    let (mut sim, map) = three_roots();
    // `20` está no meio: subir põe-no na frente de `30`.
    assert!(reorder(&mut sim, &map, 20, ZOrder::Raise));
    assert_eq!(stack(&mut sim), vec![10, 30, 20], "Raise andou errado");
    // E descer devolve-o ao meio.
    assert!(reorder(&mut sim, &map, 20, ZOrder::Lower));
    assert_eq!(stack(&mut sim), vec![10, 20, 30], "Lower nao e' o inverso");
}

/// **Quem já está no extremo não se mexe** — e o gesto devolve `false`.
///
/// ⚠️ O `false` não é higiene: o undo global regista por DIFF, e um passo que não muda nada seria
/// um Ctrl+Z que o artista gasta sem ver nada acontecer.
#[test]
fn a_move_that_changes_nothing_is_refused() {
    let (mut sim, map) = three_roots();
    assert!(!reorder(&mut sim, &map, 30, ZOrder::ToFront));
    assert!(!reorder(&mut sim, &map, 30, ZOrder::Raise));
    assert!(!reorder(&mut sim, &map, 10, ZOrder::ToBack));
    assert!(!reorder(&mut sim, &map, 10, ZOrder::Lower));
}

/// **Um FILHO reordena entre os IRMÃOS dele** — e o pai não sai do lugar.
///
/// ⚠️ Esta é a metade que o `RootOrder` sozinho não alcança: um filho não tem `RootOrder`, e
/// escrever um nele não move nada. A ordem dele mora na sequência do `Children`.
#[test]
fn a_child_reorders_among_its_siblings_without_moving_the_parent() {
    let (mut sim, map) = parent_with_kids();
    let before = stack(&mut sim);
    assert_eq!(before[0], 100, "o pai e' o PRIMEIRO da pilha");

    assert!(reorder(&mut sim, &map, 200, ZOrder::ToFront));
    let after = stack(&mut sim);
    assert_eq!(after[0], 100, "reordenar um filho mexeu no PAI: {after:?}");
    // Entre os filhos, `200` passou a ser o da frente (o último da pilha).
    assert_eq!(*after.last().unwrap(), 200, "o filho nao subiu: {after:?}");
    assert_eq!(after.len(), 4, "um filho sumiu");
}

/// **Um filho ÚNICO não tem pilha de irmãos em que andar** — e como também não há mais ninguém no
/// documento à frente dele, o gesto é recusado inteiro.
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

/// **O Z é GLOBAL e SOBREPÕE a árvore** — a lei que o Enio pediu, medida na pilha do produto.
///
/// ⚠️ A mutação que o mata é a `z_order` deixar de ordenar por `effective_z_index`: aí a forma do
/// FUNDO da hierarquia continua no fundo por mais alto que o número seja.
#[test]
fn the_z_index_is_global_and_overrides_the_hierarchy_order() {
    let (mut sim, map) = three_roots();
    // `10` é a de trás por hierarquia. Um Z alto passa-a à frente de TUDO.
    set_z(&mut sim, &map, 10, 5);
    assert_eq!(
        stack(&mut sim),
        vec![20, 30, 10],
        "o Z nao sobrepos a ordem da hierarquia"
    );
}

/// **A ordem da hierarquia só conta quando o Z EMPATA** — a outra metade da mesma frase.
///
/// ⚠️ Sem ela, um `sort` instável (ou por outra chave) passaria despercebido: com todos os Z
/// iguais a pilha tem de ser exatamente o DFS.
#[test]
fn the_tree_order_decides_only_among_equal_z() {
    let (mut sim, map) = three_roots();
    for id in [10u64, 20, 30] {
        set_z(&mut sim, &map, id, 7); // o MESMO Z para todos
    }
    assert_eq!(
        stack(&mut sim),
        vec![10, 20, 30],
        "com o Z empatado a pilha tem de ser a ordem da Hierarquia"
    );
}

/// **Um FILHO com Z alto passa à frente do vizinho do PAI** — o que a versão por-irmãos deste
/// módulo não conseguia exprimir, e é literalmente o que *"global"* significa.
#[test]
fn a_child_with_a_high_z_beats_its_parents_neighbour() {
    let (mut sim, map) = parent_with_kids();
    // Um vizinho raiz que nasce DEPOIS do pai, logo à frente de toda a sub-árvore dele.
    let e = sim
        .world_mut()
        .spawn((Transform::default(), VecPathRef(900), RootOrder(9)))
        .id();
    let mut map = map;
    map.insert(900, e.to_bits());
    assert_eq!(*stack(&mut sim).last().unwrap(), 900);

    set_z(&mut sim, &map, 201, 3);
    assert_eq!(
        *stack(&mut sim).last().unwrap(),
        201,
        "o filho com Z alto nao passou o vizinho do pai"
    );
}

/// **Quando a ÁRVORE não consegue entregar, o botão escreve o Z** — a regra única do `reorder`,
/// medida pelo RESULTADO e não por um `match` de casos.
///
/// ⚠️ A mutação que o mata é tirar o `bump_z`: o To Front vira um movimento de irmãos que não
/// muda nada na pilha, e o artista carrega no botão sem ver nada acontecer.
#[test]
fn the_button_writes_the_z_when_the_tree_cannot_deliver() {
    let (mut sim, map) = three_roots();
    set_z(&mut sim, &map, 30, 9); // `30` está travada na frente pelo Z
    assert_eq!(*stack(&mut sim).last().unwrap(), 30);

    assert!(reorder(&mut sim, &map, 10, ZOrder::ToFront));
    assert_eq!(
        *stack(&mut sim).last().unwrap(),
        10,
        "o To Front desistiu porque a arvore sozinha nao chegava la'"
    );
    assert!(
        authored_z(&sim, &map, 10).unwrap() > 9,
        "o Z nao foi escrito: {:?}",
        authored_z(&sim, &map, 10)
    );
}

/// **O campo escreve o número, e o ZERO destaca o componente** — a política de todo override deste
/// repo: um arquivo não guarda o neutro.
#[test]
fn writing_zero_detaches_the_override() {
    let (mut sim, map) = three_roots();
    assert_eq!(authored_z(&sim, &map, 10), Some(0), "nasce neutro");
    assert!(set_authored_z(&mut sim, &map, 10, 4));
    assert_eq!(authored_z(&sim, &map, 10), Some(4));

    let e = Entity::from_bits(*map.get(&10).unwrap());
    assert!(sim.world().get::<ZIndexOverride>(e).is_some());
    assert!(set_authored_z(&mut sim, &map, 10, 0));
    assert!(
        sim.world().get::<ZIndexOverride>(e).is_none(),
        "o zero deixou o componente pendurado no arquivo"
    );
    // E re-escrever o mesmo valor é recusado — um passo de undo vazio é ruído.
    assert!(!set_authored_z(&mut sim, &map, 10, 0));
}

/// **O número que o painel mostra é o que o botão move** — as duas portas concordam.
#[test]
fn the_readout_follows_the_button() {
    let (mut sim, map) = three_roots();
    set_z(&mut sim, &map, 30, 9);
    assert_eq!(authored_z(&sim, &map, 10), Some(0));
    reorder(&mut sim, &map, 10, ZOrder::ToFront);
    assert_eq!(
        authored_z(&sim, &map, 10),
        Some(10),
        "o readout nao acompanhou o botao"
    );
}

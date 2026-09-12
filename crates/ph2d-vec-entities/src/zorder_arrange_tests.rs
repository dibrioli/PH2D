//! Gates do **Z-INDEX GLOBAL** e dos botões Arrange (Enio, 2026-08-04).
//!
//! ⚠️ **A lei mudou três vezes, e a terceira é a que este arquivo mede.** Primeiro os quatro
//! botões estavam MORTOS (escreviam na ordem do vetor da cena, que a projeção reescreve a cada
//! frame). Depois passaram a escrever na ÁRVORE, com o Z como plano B. Agora o Enio fechou a
//! pergunta:
//!
//! > *"O objeto não deve ser movido na hierarquia, apenas o Z muda, e o Z determina na frente de
//! > quem ele é mostrado. A ordem na hierarquia só define a ordem se o Z for igual."*
//!
//! Logo os botões **escrevem o Z e mais nada**, e o oráculo desta wave é a LISTA DA HIERARQUIA:
//! ela é o que o artista vê e organizou, e tem de sair de todo gesto de Arrange **intacta**.
//!
//! A fixture monta a árvore e lê a pilha pelas portas do PRODUTO — o que se prova é uma RELAÇÃO
//! entre o gesto e a pilha, e uma lista escrita à mão afirmaria a relação em vez de a medir.

use super::*;
use ph2d_ecs::scene::{HierarchyWalkState, build_hierarchy_snapshot};
use ph2d_ecs::{ChildOf, Transform, VecPathRef, ZIndexOverride};
use ph2d_vec_scene::ZOrder;

/// O snapshot da árvore, por uma porta só.
fn snapshot(sim: &mut SimWorld) -> HierarchySnapshot {
    let mut state = HierarchyWalkState::new(sim.world_mut());
    let mut scratch = Vec::new();
    let mut snap = HierarchySnapshot::default();
    build_hierarchy_snapshot(sim.world(), &mut state, &mut scratch, &mut snap);
    snap
}

/// A pilha de z (FUNDO → topo) pela porta do produto.
fn stack(sim: &mut SimWorld) -> Vec<VecPathId> {
    let snap = snapshot(sim);
    z_order(sim.world(), &snap)
}

/// **A LISTA DA HIERARQUIA** — o que o painel mostra, na ordem em que o mostra.
///
/// É o oráculo de *"o objeto não foi movido na hierarquia"*, e é o certo por ser o que o artista
/// VÊ: comparar `RootOrder` cru daria vermelho numa renumeração que não move linha nenhuma.
fn hierarchy(sim: &mut SimWorld) -> Vec<VecPathId> {
    snapshot(sim)
        .entries
        .iter()
        .filter_map(|e| e.vec_path)
        .collect()
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

/// **O KEYSTONE DA LEI: o botão escreve o Z e NÃO TOCA na Hierarquia.**
///
/// ⚠️ A mutação que o mata é devolver a metade de árvore (`sibling_move`): a pilha continua a
/// mexer-se — todos os outros gates ficam VERDES —, e o que muda é a lista do artista por baixo
/// dele. Um botão de z-order que re-arruma a Hierarquia é trabalho destruído sem aviso.
#[test]
fn every_verb_writes_the_z_and_leaves_the_hierarchy_alone() {
    for order in [
        ZOrder::ToFront,
        ZOrder::Raise,
        ZOrder::Lower,
        ZOrder::ToBack,
    ] {
        let (mut sim, map) = three_roots();
        let tree = hierarchy(&mut sim);
        // `20` está no meio: os quatro verbos têm todos para onde ir.
        assert!(reorder(&mut sim, &map, 20, order), "{order:?} recusou");
        assert_eq!(
            hierarchy(&mut sim),
            tree,
            "{order:?} MOVEU o objeto na hierarquia — a lista do artista mudou por baixo dele"
        );
        assert_ne!(
            authored_z(&sim, &map, 20),
            Some(0),
            "{order:?} nao escreveu o Z: entao mexeu noutra coisa qualquer"
        );
    }
}

/// **Cada verbo entrega o que o nome dele diz.** To Front/To Back são EXACTOS por construção (a
/// referência é o extremo da pilha); Forward/Backward andam na direção pedida.
///
/// ⚠️ **A fixture dos extremos tem de conter o fenómeno, e a primeira versão não continha:** com
/// os três empatados em `z = 0`, *passar o vizinho* e *ir para a frente* dão o MESMO número, e a
/// mutação "To Front vira um passo só" — o defeito que a wave anterior já pagou — ficava VERDE
/// aqui e só sangrava num gate sobre o readout. Um Z alheio no meio separa as duas perguntas.
#[test]
fn each_verb_delivers_what_it_names() {
    let (mut sim, map) = three_roots();
    set_z(&mut sim, &map, 20, 5); // a pilha é [10, 30, 20]: o vizinho de `10` NÃO é o da frente
    assert!(reorder(&mut sim, &map, 10, ZOrder::ToFront));
    assert_eq!(
        *stack(&mut sim).last().unwrap(),
        10,
        "To Front parou no vizinho em vez de ir a' frente"
    );

    let (mut sim, map) = three_roots();
    set_z(&mut sim, &map, 20, -5); // a pilha é [20, 10, 30]: o vizinho de `30` NÃO é o do fundo
    assert!(reorder(&mut sim, &map, 30, ZOrder::ToBack));
    assert_eq!(
        stack(&mut sim)[0],
        30,
        "To Back parou no vizinho em vez de ir ao fundo"
    );

    let (mut sim, map) = three_roots();
    let before = stack(&mut sim).iter().position(|p| *p == 20).unwrap();
    assert!(reorder(&mut sim, &map, 20, ZOrder::Raise));
    let after = stack(&mut sim).iter().position(|p| *p == 20).unwrap();
    assert!(
        after > before,
        "Forward andou para tras: {before} -> {after}"
    );

    let (mut sim, map) = three_roots();
    assert!(reorder(&mut sim, &map, 20, ZOrder::Lower));
    assert_eq!(stack(&mut sim), vec![20, 10, 30], "Backward andou errado");
}

/// **O passo é o MENOR que entrega, e por isso Backward é o inverso exacto de Forward.**
///
/// ⚠️ A mutação que o mata é somar sempre `±1` em vez de EMPATAR quando a árvore já favorece: o Z
/// passa a inflar um número por clique, e voltar deixa de devolver a pilha ao lugar.
#[test]
fn the_step_is_the_smallest_that_delivers() {
    let (mut sim, map) = three_roots();
    set_z(&mut sim, &map, 20, 3); // `20` sobe pelo Z: a pilha é [10, 30, 20]
    assert_eq!(stack(&mut sim), vec![10, 30, 20]);

    // `30` está atrás de `20` só pelo Z; a árvore já a põe depois (dfs 2 > 1), então EMPATAR
    // basta — e o número escrito é 3, não 4.
    assert!(reorder(&mut sim, &map, 30, ZOrder::Raise));
    assert_eq!(
        stack(&mut sim),
        vec![10, 20, 30],
        "Forward nao andou um lugar"
    );
    assert_eq!(
        authored_z(&sim, &map, 30),
        Some(3),
        "o Forward inflou o Z em vez de empatar"
    );

    // E voltar devolve exactamente a pilha anterior.
    assert!(reorder(&mut sim, &map, 30, ZOrder::Lower));
    assert_eq!(
        stack(&mut sim),
        vec![10, 30, 20],
        "Backward nao e' o inverso de Forward"
    );
}

/// **O limite que é ARITMÉTICA, não desleixo** — e fica MEDIDO para ninguém o descobrir sozinho.
///
/// Com três formas empatadas em `z = 0` não existe inteiro que ponha a do fundo *entre* as outras
/// duas: `0` deixa-a atrás das duas (a árvore desempata contra ela) e `1` põe-na à frente das
/// duas. Nesse regime o Forward passa **dois** lugares.
///
/// ⚠️ As duas saídas foram recusadas pela própria lei: renumerar os vizinhos é escrever no número
/// de um objeto que o artista não selecionou, e mexer na árvore é o que ele proibiu.
#[test]
fn a_forward_out_of_a_tie_group_clears_the_whole_group() {
    let (mut sim, map) = three_roots(); // os três empatam em z = 0
    assert!(reorder(&mut sim, &map, 10, ZOrder::Raise));
    assert_eq!(
        stack(&mut sim),
        vec![20, 30, 10],
        "o numero medido deste limite mudou — releia a doc antes de re-calibrar"
    );
    // E a Hierarquia continua intacta: o preço é na PILHA, nunca na lista do artista.
    assert_eq!(hierarchy(&mut sim), vec![10, 20, 30]);
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

/// **Um FILHO vai à frente sem arrastar o PAI** — e a sub-árvore continua a ser a mesma.
///
/// ⚠️ Aqui a cascata do Godot faz o trabalho: o Z do filho soma-se ao do pai, então subir o filho
/// não move o pai. A versão por-irmãos precisava de escrever na sequência do `Children` para isto,
/// que é precisamente o que a lei nova proíbe.
#[test]
fn a_child_goes_to_the_front_without_dragging_its_parent() {
    let (mut sim, map) = parent_with_kids();
    let tree = hierarchy(&mut sim);
    let before = stack(&mut sim);
    assert_eq!(before[0], 100, "o pai e' o PRIMEIRO da pilha");

    assert!(reorder(&mut sim, &map, 200, ZOrder::ToFront));
    let after = stack(&mut sim);
    assert_eq!(after[0], 100, "reordenar um filho mexeu no PAI: {after:?}");
    assert_eq!(*after.last().unwrap(), 200, "o filho nao subiu: {after:?}");
    assert_eq!(after.len(), 4, "um filho sumiu");
    assert_eq!(hierarchy(&mut sim), tree, "a hierarquia foi mexida");
}

/// **Subir o PAI leva a sub-árvore inteira** — a cascata é o que torna um grupo um bloco.
#[test]
fn raising_the_parent_carries_the_whole_subtree() {
    let (mut sim, map) = parent_with_kids();
    // Um vizinho raiz à frente de toda a sub-árvore.
    let e = sim
        .world_mut()
        .spawn((Transform::default(), VecPathRef(900), RootOrder(9)))
        .id();
    let mut map = map;
    map.insert(900, e.to_bits());
    assert_eq!(*stack(&mut sim).last().unwrap(), 900);

    assert!(reorder(&mut sim, &map, 100, ZOrder::ToFront));
    assert_eq!(
        stack(&mut sim),
        vec![900, 100, 200, 201, 202],
        "a sub-arvore nao subiu como bloco"
    );
}

/// **Um DESCENDENTE é incruzável, e o botão recusa em vez de gastar um Ctrl+Z.**
///
/// ⚠️ O Z é uma CASCATA: subir o número do pai sobe o do filho pelo mesmo tanto, então a distância
/// entre os dois **não muda por número nenhum**. Um pai sozinho com um filho não tem para onde ir
/// à frente, e o filho já é o da frente — os dois gestos são recusas.
///
/// ⚠️ **Este gate nasceu de um defeito meu:** a primeira versão mirava no vizinho imediato sem
/// perguntar de quem ele era, e o `To Front` do pai escrevia `z = 1`, devolvia `true` e **não
/// movia um pixel** (a sub-árvore inteira subia junto). A mutação que o mata é tirar o filtro do
/// `descendant_count`.
#[test]
fn a_descendant_is_uncrossable_so_the_gesture_is_refused() {
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
    let before = stack(&mut sim);
    assert_eq!(before, vec![100, 200], "o filho desenha sobre o pai");

    assert!(
        !reorder(&mut sim, &map, 200, ZOrder::ToFront),
        "o filho ja' e' o da frente"
    );
    assert!(
        !reorder(&mut sim, &map, 100, ZOrder::ToFront),
        "o pai nao pode cruzar o proprio filho: a cascata leva-o junto"
    );
    assert_eq!(stack(&mut sim), before, "uma recusa escreveu alguma coisa");
    assert_eq!(
        authored_z(&sim, &map, 100),
        Some(0),
        "o pai ganhou um Z a' toa"
    );

    // ⚠️ **E o CONTROLE, que prova que o filtro é dos DESCENDENTES e não da família:** a cascata
    // só corre num sentido, então o filho PODE ir para trás do pai — com um Z próprio negativo.
    // Sem esta metade, um filtro largo demais (que excluísse também os ascendentes) passaria.
    assert!(
        reorder(&mut sim, &map, 200, ZOrder::Lower),
        "o filho nao consegue ir para tras do pai"
    );
    assert_eq!(stack(&mut sim), vec![200, 100]);
    assert_eq!(authored_z(&sim, &map, 200), Some(-1));
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
/// ⚠️ Sem ela, ordenar por outra chave passaria despercebido: com todos os Z iguais a pilha tem de
/// ser exatamente o DFS.
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
///
/// ⚠️ É esta concordância que torna os quatro botões legíveis: carregar em To Front e ver o campo
/// saltar para `10` diz ao artista, sem documentação, o que o botão acabou de fazer.
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

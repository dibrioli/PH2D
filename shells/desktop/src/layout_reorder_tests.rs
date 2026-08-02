//! Os gates do gesto de REORDENAR.
//!
//! O que só se pode afirmar aqui: que o slot sai dos CENTROS publicados, que o arrastado sai da
//! régua (senão ele competiria consigo mesmo), que a ordem do mundo ECS de facto muda — e que
//! **fora de um fluxo o gesto não existe**, que é o que impede uma forma solta de ser reordenada
//! por engano.

use super::*;
use ph2d_ecs::{LayoutDir, Transform, VecLayout};

/// Uma moldura em LINHA com `n` filhos, o `i`-ésimo em `x = i·10`.
fn flow(n: usize) -> (SimWorld, Entity, Vec<Entity>, LayoutLive) {
    let mut sim = SimWorld::default();
    let frame = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            VecLayout {
                dir: LayoutDir::Row,
                ..VecLayout::default()
            },
        ))
        .id();
    let kids: Vec<Entity> = (0..n)
        .map(|_| {
            sim.world_mut()
                .spawn((Transform::IDENTITY, ChildOf(frame)))
                .id()
        })
        .collect();
    let layout = LayoutLive::with_slots(
        frame,
        true,
        kids.iter()
            .enumerate()
            .map(|(i, e)| (*e, i as f64 * 10.0))
            .collect(),
    );
    (sim, frame, kids, layout)
}

fn order(sim: &SimWorld, frame: Entity) -> Vec<Entity> {
    sim.world()
        .get::<Children>(frame)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default()
}

/// **O slot é quantos centros ficam ANTES do cursor** — a aritmética inteira, sem cena.
#[test]
fn the_slot_counts_the_centres_the_cursor_has_passed() {
    let centres = [10.0, 20.0, 30.0];
    assert_eq!(slot_at(&centres, 5.0), 0, "antes de todos");
    assert_eq!(slot_at(&centres, 15.0), 1);
    assert_eq!(slot_at(&centres, 25.0), 2);
    assert_eq!(slot_at(&centres, 99.0), 3, "depois de todos");
    assert_eq!(slot_at(&[], 0.0), 0, "sem irmaos ha' um slot so'");
}

/// **Arrastar o primeiro para depois do último põe-no no fim.**
#[test]
fn dragging_the_first_past_the_last_puts_it_at_the_end() {
    let (mut sim, frame, k, layout) = flow(3);
    assert!(drop_at(&mut sim, &layout, k[0], [99.0, 0.0]));
    assert_eq!(order(&sim, frame), vec![k[1], k[2], k[0]]);
}

/// **E o caminho de volta** — a régua do gate é a MESMA (o passe publica-a uma vez por frame),
/// então este gate mede o gesto e não uma segunda medição das formas.
#[test]
fn dragging_the_last_before_the_first_puts_it_at_the_front() {
    let (mut sim, frame, k, layout) = flow(3);
    assert!(drop_at(&mut sim, &layout, k[2], [-5.0, 0.0]));
    assert_eq!(order(&sim, frame), vec![k[2], k[0], k[1]]);
}

/// **O arrastado sai da régua, e o gate mede onde isso IMPORTA.**
///
/// ⚠️ **Primeira versão deste gate era VERDE sobre a mutação.** Ela largava o filho do meio
/// exactamente sobre o próprio centro (x = 10), e ali as duas versões concordam por acidente do
/// `<`: com o centro dele na lista, `10 < 10` é falso, então ele não conta contra si e o slot sai
/// igual. O fenómeno só existe quando o próprio centro fica **estritamente antes** do cursor —
/// isto é, ao mexer o filho um pouco PARA A FRENTE sem chegar ao vizinho.
///
/// Com o centro dele na régua, esse empurrãozinho conta como *"passei um irmão"* e a forma salta
/// uma casa que o artista não pediu: o gesto move sozinho.
#[test]
fn nudging_a_child_within_its_own_slot_does_not_move_it() {
    let (mut sim, frame, k, layout) = flow(3);
    // O primeiro (centro 0) empurrado até x = 5 — ainda muito antes do centro do vizinho (10).
    assert!(
        !drop_at(&mut sim, &layout, k[0], [5.0, 0.0]),
        "um empurrao dentro do proprio slot nao pode reordenar"
    );
    assert_eq!(order(&sim, frame), k);
    // E o caso de fronteira: largado sobre o proprio centro, tambem nao.
    assert!(!drop_at(&mut sim, &layout, k[1], [10.0, 0.0]));
    assert_eq!(order(&sim, frame), k);
}

/// **O eixo TRANSVERSAL é ignorado** — num fluxo em linha o `y` do cursor não escolhe slot.
///
/// ⚠️ É o que mantém o gesto previsível quando a mão treme: no eixo em que o filho não tem para
/// onde ir, mexer não é informação.
#[test]
fn the_cross_axis_does_not_choose_the_slot() {
    let (mut sim, frame, k, layout) = flow(3);
    assert!(!drop_at(&mut sim, &layout, k[1], [10.0, 500.0]));
    assert_eq!(order(&sim, frame), k);
}

/// **Numa COLUNA quem manda é o `y`** — a régua diz qual eixo é o principal, e o gesto obedece.
#[test]
fn a_column_reads_the_other_axis() {
    let (mut sim, frame, k, _) = flow(3);
    if let Ok(mut e) = sim.world_mut().get_entity_mut(frame) {
        e.insert(VecLayout {
            dir: LayoutDir::Column,
            ..VecLayout::default()
        });
    }
    let layout = LayoutLive::with_slots(
        frame,
        false,
        k.iter()
            .enumerate()
            .map(|(i, e)| (*e, i as f64 * 10.0))
            .collect(),
    );
    assert!(drop_at(&mut sim, &layout, k[0], [0.0, 99.0]));
    assert_eq!(order(&sim, frame), vec![k[1], k[2], k[0]]);
}

/// **Fora de um fluxo o gesto NÃO existe** — uma forma solta continua a ser MOVIDA.
///
/// ⚠️ É a metade que impede a wave de mudar o significado do arrasto na cena inteira: o
/// `flow_parent` responde `None`, e quem chama volta ao caminho de sempre.
#[test]
fn a_shape_outside_a_flow_is_never_reordered() {
    let mut sim = SimWorld::default();
    let group = sim.world_mut().spawn(Transform::IDENTITY).id();
    let kid = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ChildOf(group)))
        .id();
    let loose = sim.world_mut().spawn(Transform::IDENTITY).id();
    assert!(flow_parent(&sim, kid).is_none(), "o pai nao empilha");
    assert!(flow_parent(&sim, loose).is_none(), "nem sequer tem pai");

    let layout = LayoutLive::with_slots(group, true, vec![(kid, 0.0)]);
    assert!(
        !drop_at(&mut sim, &layout, kid, [99.0, 0.0]),
        "sem fluxo no pai o arrasto tem de cair no caminho de MOVER"
    );
}

/// **A hierarquia VIVA é a autoridade sobre quem são os irmãos, e o gate mede o lado que DÓI.**
///
/// A régua do layout é do frame ANTERIOR, e pode estar desactualizada nos dois sentidos:
///
/// - **irmão APAGADO** — inofensivo, e o gate abaixo prova-o: a porta de ordem ignora quem já não
///   é filho, então tirar o `desired` da régua daria a mesma resposta. Um gate feito só disto fica
///   **VERDE sobre a mutação**, e a primeira versão deste ficou;
/// - **irmão NOVO** — este dói. Uma forma criada depois do último passe não está na régua, então
///   um `desired` tirado dela **não a menciona**; e quem não é mencionado nunca é removido, o que
///   o deixa no COMEÇO da lista enquanto todos os outros são re-inseridos depois dele. O artista
///   arrasta uma forma e vê **outra** saltar para a frente da fila.
#[test]
fn a_sibling_the_ruler_has_not_seen_yet_keeps_its_place() {
    // A régua conhece dois; um terceiro nasce depois dela.
    let (mut sim, frame, k, layout) = flow(2);
    let newborn = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ChildOf(frame)))
        .id();
    assert_eq!(order(&sim, frame), vec![k[0], k[1], newborn]);

    drop_at(&mut sim, &layout, k[0], [99.0, 0.0]);
    let after = order(&sim, frame);
    assert_eq!(
        after.first(),
        Some(&k[1]),
        "o recem-nascido saltou para a FRENTE da fila num arrasto que nao era dele: {after:?}"
    );
    assert!(after.contains(&newborn), "e ele tem de continuar na lista");
}

/// **E um irmão APAGADO não vira fantasma** — a outra metade, barata e real.
#[test]
fn a_despawned_sibling_never_reaches_the_order_door() {
    let (mut sim, frame, k, layout) = flow(3);
    sim.world_mut().entity_mut(k[1]).despawn();
    assert!(drop_at(&mut sim, &layout, k[0], [99.0, 0.0]));
    assert_eq!(order(&sim, frame), vec![k[2], k[0]]);
}

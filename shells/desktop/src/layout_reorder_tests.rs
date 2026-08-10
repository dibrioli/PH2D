//! Os gates do gesto de REORDENAR.
//!
//! O que só se pode afirmar aqui: que o slot sai dos CENTROS publicados, que o arrastado sai da
//! régua (senão ele competiria consigo mesmo), que a ordem do mundo ECS de facto muda — e que
//! **fora de um fluxo o gesto não existe**, que é o que impede uma forma solta de ser reordenada
//! por engano.

use super::*;
use crate::layout_live::Reading;
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
        Reading::RowX,
        kids.iter()
            .enumerate()
            .map(|(i, e)| (*e, cell(i as f64 * 10.0, 0.0)))
            .collect(),
    );
    (sim, frame, kids, layout)
}

/// Uma caixa de `10 × 10` cujo CENTRO cai em `(cx, cy)` — a régua publica caixas, não centros.
fn cell(cx: f64, cy: f64) -> crate::layout_live::Box2 {
    ([cx - 5.0, cy - 5.0], [cx + 5.0, cy + 5.0])
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
        Reading::ColumnY,
        k.iter()
            .enumerate()
            .map(|(i, e)| (*e, cell(0.0, i as f64 * 10.0)))
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

    let layout = LayoutLive::with_slots(group, Reading::RowX, vec![(kid, cell(0.0, 0.0))]);
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

/// ⭐ **A RÉGUA 1-D É ERRADA NUMA FILA EM LINHAS, e este gate é o defeito medido.**
///
/// Numa grade 3×3 de células de 10, as três da coluna 0 partilham o mesmo `x`. A régua antiga —
/// *"quantos centros estão antes do cursor"* — conta as três mesmo com o cursor na PRIMEIRA:
/// soltar na célula (0,0) devolvia o **slot 3**, o começo da segunda linha, e soltar na última
/// devolvia 6 em vez de 8.
///
/// ⚠️ O `RowWrap` shipa com este defeito desde que nasceu; ele ficou invisível ali porque uma
/// faixa de wrap raramente alinha duas linhas. Numa grade ele é visível em TODA a primeira linha.
#[test]
fn a_drop_in_a_grid_lands_in_the_cell_it_was_dropped_on() {
    // Três linhas de três: centros em x = 5/15/25 e y = 25/15/5 (o mundo é Y-up, a 1ª linha em
    // cima). A caixa mede 10, então a célula (r, c) tem centro (5 + 10c, 25 − 10r).
    let boxes: Vec<_> = (0..9)
        .map(|i| {
            cell(
                5.0 + 10.0 * f64::from(i % 3),
                25.0 - 10.0 * f64::from(i / 3),
            )
        })
        .collect();
    for (name, cursor, want) in [
        ("a 1a celula", [5.0, 25.0], 0),
        ("entre a 1a e a 2a", [10.0, 25.0], 1),
        ("a ultima celula", [25.0, 5.0], 8),
        ("depois da ultima", [30.0, 5.0], 9),
        ("a 1a da 2a linha", [5.0, 15.0], 3),
    ] {
        assert_eq!(
            slot_at_rows(&boxes, cursor),
            want,
            "soltar em {name} devia pedir o slot {want}"
        );
    }
    // ⚠️ **O CONTROLO que nomeia o defeito:** a régua 1-D no mesmo ponto responde outra coisa.
    let centres: Vec<f64> = boxes.iter().map(|(lo, hi)| (lo[0] + hi[0]) * 0.5).collect();
    assert_eq!(
        slot_at(&centres, 10.0),
        3,
        "se a regua 1-D ja' respondesse 1 aqui, este gate nao estaria a medir nada"
    );
}

/// **O vão entre duas linhas pertence à linha mais PRÓXIMA** — e não a nenhuma.
///
/// ⚠️ É o mesmo argumento que fez a régua 1-D medir centros e não fronteiras: num fluxo com `gap`
/// o artista passa a maior parte do arrasto exactamente no vão, e uma régua que ali não responda
/// tem um ponto morto do tamanho do vão.
#[test]
fn the_gap_between_two_rows_belongs_to_the_nearer_one() {
    // Duas linhas de dois, com 10 de vão entre elas: centros em y = 25 e y = 5.
    let boxes: Vec<_> = (0..4)
        .map(|i| {
            cell(
                5.0 + 10.0 * f64::from(i % 2),
                25.0 - 20.0 * f64::from(i / 2),
            )
        })
        .collect();
    assert_eq!(slot_at_rows(&boxes, [5.0, 18.0]), 0, "perto da 1a linha");
    assert_eq!(slot_at_rows(&boxes, [5.0, 12.0]), 2, "perto da 2a linha");
}

/// **As duas leituras de UMA fila continuam exactamente como estavam** — a grade é uma ADIÇÃO.
#[test]
fn the_one_dimensional_readings_are_untouched() {
    let (mut sim, frame, k, layout) = flow(3);
    // `flow` publica `Reading::RowX` com centros em 0/10/20.
    assert!(drop_at(&mut sim, &layout, k[0], [99.0, 0.0]));
    assert_eq!(order(&sim, frame), vec![k[1], k[2], k[0]]);
}

/// ⭐ **O GESTO REAL numa grade** — pela porta do produto (`drop_at`), e não pelo kernel.
///
/// ⚠️ Os gates acima afirmam a régua; este afirma que a régua CERTA é a que o gesto escolhe. Um
/// `Reading` mal atribuído passa nos outros três e cai só aqui — é a diferença entre *a aritmética
/// está certa* e *o produto usa a aritmética certa*.
#[test]
fn dropping_a_child_on_the_first_cell_of_a_grid_moves_it_to_the_front() {
    let (mut sim, frame, k, _) = flow(6);
    if let Ok(mut e) = sim.world_mut().get_entity_mut(frame) {
        e.insert(VecLayout {
            dir: LayoutDir::Grid,
            columns: 3,
            ..VecLayout::default()
        });
    }
    // Duas linhas de três, células de 10: a 1ª linha em y = 25, a 2ª em y = 15.
    let layout = LayoutLive::with_slots(
        frame,
        Reading::Rows,
        k.iter()
            .enumerate()
            .map(|(i, e)| {
                (
                    *e,
                    cell(5.0 + 10.0 * (i % 3) as f64, 25.0 - 10.0 * (i / 3) as f64),
                )
            })
            .collect(),
    );
    // O ÚLTIMO filho, solto na primeira célula, tem de ir para a frente da fila.
    assert!(drop_at(&mut sim, &layout, k[5], [5.0, 25.0]));
    assert_eq!(
        order(&sim, frame),
        vec![k[5], k[0], k[1], k[2], k[3], k[4]],
        "o filho solto na 1a celula nao foi para o 1o slot"
    );
}

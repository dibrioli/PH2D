//! Os gates da metade que a shell possui: *que intervalo da pilha de z uma moldura ocupa?*
//!
//! A fixture monta a árvore pela porta do PRODUTO (`build_hierarchy_snapshot`) e lê a pilha pela
//! porta do produto (`vec_entities::z_order`) — o que esta wave precisa provar é uma RELAÇÃO entre
//! as duas, e uma lista de entradas escrita à mão afirmaria a relação em vez de a medir.

use super::*;
use ph2d_ecs::scene::{HierarchyWalkState, build_hierarchy_snapshot};
use ph2d_ecs::{ChildOf, Transform, VecPathRef};

/// Os intervalos E a pilha de z do mesmo mundo, pelas duas portas do produto.
fn spans_and_z(sim: &mut SimWorld) -> (Vec<VecClipSpan>, Vec<u64>) {
    let mut state = HierarchyWalkState::new(sim.world_mut());
    let mut scratch = Vec::new();
    let mut snap = HierarchySnapshot::default();
    build_hierarchy_snapshot(sim.world(), &mut state, &mut scratch, &mut snap);
    let z = crate::vec_entities::z_order(&snap);
    (clip_spans(sim, &snap), z)
}

fn spans_of(sim: &mut SimWorld) -> Vec<VecClipSpan> {
    spans_and_z(sim).0
}

/// Uma cena com moldura: o retângulo `100` com `children` filhos vetoriais, mais um vizinho raiz
/// (`900`) que a moldura NÃO contém.
fn scene(children: usize, clip: bool) -> (SimWorld, u64, Vec<u64>, u64) {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    w.spawn((Transform::default(), VecPathRef(900)));
    let frame = w
        .spawn((Transform::default(), VecPathRef(100), VecFrame { clip }))
        .id();
    let mut kids = Vec::new();
    for k in 0..children {
        let id = 200 + k as u64;
        w.spawn((Transform::default(), VecPathRef(id), ChildOf(frame)));
        kids.push(id);
    }
    (sim, 100, kids, 900)
}

/// A pilha de z é o DFS INVERTIDO, então o intervalo abre no descendente que o DFS lista por
/// ÚLTIMO. Ler isto ao contrário abre o recorte no lugar errado e some com quase toda a arte.
///
/// O gate afirma a PROPRIEDADE (*o intervalo abre no filho mais ao fundo*), não uma posição
/// literal: quem responde qual é ele é a mesma `z_order` que o renderer vai percorrer.
#[test]
fn the_span_opens_at_the_descendant_that_draws_first() {
    let (mut sim, frame, kids, _) = scene(3, true);
    let (spans, z) = spans_and_z(&mut sim);
    assert_eq!(spans.len(), 1, "uma moldura que recorta, um intervalo");
    assert_eq!(spans[0].frame, frame);

    let bottom = *z
        .iter()
        .find(|id| kids.contains(id))
        .expect("algum filho na pilha");
    assert_eq!(
        spans[0].first, bottom,
        "o intervalo abre no filho mais ao FUNDO"
    );

    // E a moldura é a ÚLTIMA da própria sub-árvore — é este fato que faz um par (abre, fecha)
    // bastar para descrever o recorte.
    let pos_frame = z
        .iter()
        .position(|id| *id == frame)
        .expect("moldura na pilha");
    for k in &kids {
        let pk = z.iter().position(|id| id == k).expect("filho na pilha");
        assert!(pk < pos_frame, "o filho {k} desenha ANTES da moldura");
    }
}

/// Uma moldura com `clip` desligado continua sendo moldura, e não abre camada nenhuma.
#[test]
fn an_unclipped_frame_produces_no_span() {
    let (mut sim, _, _, _) = scene(3, false);
    assert!(spans_of(&mut sim).is_empty());
}

/// Sem descendente vetorial não há o que recortar — e um intervalo vazio faria a moldura abrir e
/// fechar sobre si mesma.
#[test]
fn an_empty_frame_produces_no_span() {
    let (mut sim, _, _, _) = scene(0, true);
    assert!(spans_of(&mut sim).is_empty());
}

/// Molduras aninhadas: a lista sai de FORA para DENTRO, porque as duas abrem no mesmo path e a
/// camada de clip é uma pilha.
#[test]
fn nested_frames_come_outermost_first() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let outer = w
        .spawn((
            Transform::default(),
            VecPathRef(10),
            VecFrame { clip: true },
        ))
        .id();
    let inner = w
        .spawn((
            Transform::default(),
            VecPathRef(20),
            VecFrame { clip: true },
            ChildOf(outer),
        ))
        .id();
    w.spawn((Transform::default(), VecPathRef(30), ChildOf(inner)));

    let spans = spans_of(&mut sim);
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].frame, 10, "a de FORA primeiro");
    assert_eq!(spans[1].frame, 20);
    // As duas abrem no MESMO path (o neto é o único descendente vetorial de ambas) — é exatamente
    // o caso em que a ordem decide se o LIFO fecha certo.
    assert_eq!(spans[0].first, 30);
    assert_eq!(spans[1].first, 30);
}

/// Um vizinho fora da moldura não entra no intervalo. Sem isto o recorte comeria a cena inteira.
#[test]
fn a_sibling_outside_the_frame_is_not_in_the_span() {
    let (mut sim, _, kids, outsider) = scene(2, true);
    let spans = spans_of(&mut sim);
    assert_eq!(spans.len(), 1);
    assert_ne!(spans[0].first, outsider);
    assert!(kids.contains(&spans[0].first));
}

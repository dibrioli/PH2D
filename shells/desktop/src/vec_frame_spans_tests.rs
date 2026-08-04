//! Os gates da metade que a shell possui: *que intervalo da pilha de z uma moldura ocupa?*
//!
//! A fixture monta a árvore pela porta do PRODUTO (`build_hierarchy_snapshot`) e lê a pilha pela
//! porta do produto (`vec_entities::z_order`) — o que esta wave precisa provar é uma RELAÇÃO entre
//! as duas, e uma lista de entradas escrita à mão afirmaria a relação em vez de a medir.

use super::*;
use ph2d_ecs::scene::{HierarchyWalkState, build_hierarchy_snapshot};
use ph2d_ecs::{ChildOf, Transform, VecPathRef};

/// Os intervalos E a pilha de z do mesmo mundo, pelas duas portas do produto.
fn spans_and_z(sim: &mut SimWorld) -> (Vec<VecParentSpan>, Vec<u64>) {
    let mut state = HierarchyWalkState::new(sim.world_mut());
    let mut scratch = Vec::new();
    let mut snap = HierarchySnapshot::default();
    build_hierarchy_snapshot(sim.world(), &mut state, &mut scratch, &mut snap);
    let z = crate::vec_entities::z_order(&snap);
    (parent_spans(sim, &snap), z)
}

fn spans_of(sim: &mut SimWorld) -> Vec<VecParentSpan> {
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
    assert_eq!(spans[0].parent, frame);

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

/// **Uma moldura com `clip` desligado TEM intervalo — ela só não abre camada.**
///
/// ⚠️ Este gate afirmava o contrário (*"não produz intervalo"*), e **consagrava o defeito**: sem
/// intervalo, o preenchimento dela não é antecipado, e como a pilha de z é o DFS invertido ela
/// pinta na FRENTE do próprio conteúdo. Foi o report do Enio de 2026-08-02 — *"os filhos estão
/// ficando atrás do pai"* —, e a mutação *"só quem recorta tem intervalo"* SOBREVIVEU a esta
/// suíte enquanto ele estava escrito assim.
///
/// O intervalo é *onde a moldura é desenhada*; recortar é a metade opcional que o `clip` decide.
#[test]
fn an_unclipped_frame_still_gets_a_span_it_just_does_not_clip() {
    let (mut sim, _, _, _) = scene(3, false);
    let spans = spans_of(&mut sim);
    assert_eq!(
        spans.len(),
        1,
        "a moldura precisa de intervalo para ser o FUNDO"
    );
    assert!(!spans[0].clip, "e ela nao recorta");
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
    assert_eq!(spans[0].parent, 10, "a de FORA primeiro");
    assert_eq!(spans[1].parent, 20);
    // As duas abrem no MESMO path (o neto é o único descendente vetorial de ambas) — é exatamente
    // o caso em que a ordem decide se o LIFO fecha certo.
    assert_eq!(spans[0].first, 30);
    assert_eq!(spans[1].first, 30);
}

/// Uma cena SEM moldura nenhuma: o retângulo `100` com um filho vetorial, e um vizinho raiz.
fn plain_parent() -> (SimWorld, u64, u64) {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    w.spawn((Transform::default(), VecPathRef(900)));
    let parent = w.spawn((Transform::default(), VecPathRef(100))).id();
    w.spawn((Transform::default(), VecPathRef(200), ChildOf(parent)));
    (sim, 100, 200)
}

/// **A LEI: o filho desenha SOBRE o pai** — e o pai não precisa de ser uma moldura.
///
/// ⚠️ Este gate nasceu de um relato do Enio (2026-08-04) e da constatação de que **nada o
/// pinava**: a suíte inteira ficou verde quando a antecipação deixou de ser gateada em `VecFrame`,
/// porque toda fixture de intervalo tinha uma moldura. O defeito viveu no vão entre duas
/// asserções, cada uma certa sozinha.
///
/// A mutação que o mata é voltar a exigir `VecFrame` para haver intervalo: aí uma forma com uma
/// forma filha pinta por cima dela, e o artista vê a filha desaparecer.
#[test]
fn a_plain_parent_is_a_backdrop_too_so_the_child_draws_over_it() {
    let (mut sim, parent, kid) = plain_parent();
    let (spans, z) = spans_and_z(&mut sim);
    assert_eq!(
        spans.len(),
        1,
        "um pai comum com um filho vetorial tem de ter intervalo: sem ele o desenho dele nao e' \
         antecipado e ele cobre o proprio filho"
    );
    assert_eq!(spans[0].parent, parent);
    assert_eq!(spans[0].first, kid, "o intervalo abre no filho");
    // E a pilha continua a listar o pai por ÚLTIMO — a lei mora na ANTECIPAÇÃO, não na ordem:
    // pôr o contêiner no fundo da própria sub-árvore desemparelharia o push/pop do recorte.
    let (pk, pp) = (
        z.iter().position(|id| *id == kid),
        z.iter().position(|id| *id == parent),
    );
    assert!(pk < pp, "a pilha deixou de pôr o pai por ultimo");
}

/// **Só uma MOLDURA recorta** — a metade que continua a ser pergunta de `VecFrame`.
///
/// ⚠️ Sem ela, generalizar a antecipação passaria a recortar em toda forma que tem filhos, e o
/// sintoma seria arte a sumir na borda de qualquer pai.
#[test]
fn a_plain_parent_backdrops_but_does_not_clip() {
    let (mut sim, _, _) = plain_parent();
    let spans = spans_of(&mut sim);
    assert!(
        !spans[0].clip,
        "um pai comum passou a RECORTAR: o `clip` deixou de ser pergunta de moldura"
    );
    // O controle: uma moldura que recorta continua a recortar.
    let (mut sim, _, _, _) = scene(1, true);
    assert!(spans_of(&mut sim)[0].clip, "a moldura deixou de recortar");
}

/// **Uma folha não tem intervalo** — nem uma raiz sem filhos, nem um filho.
///
/// ⚠️ É a metade da AUSÊNCIA: sem ela, *"todo path tem intervalo"* passaria neste arquivo, e cada
/// forma da cena abriria e fecharia em cima de si mesma.
#[test]
fn a_leaf_gets_no_span() {
    let (mut sim, _, _) = plain_parent();
    let spans = spans_of(&mut sim);
    assert_eq!(spans.len(), 1, "so' o PAI tem intervalo: {spans:?}");
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

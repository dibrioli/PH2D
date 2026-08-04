//! Os gates da metade que a shell possui: *que intervalo da pilha de z uma moldura RECORTA?*
//!
//! A fixture monta a árvore pela porta do PRODUTO (`build_hierarchy_snapshot`) e lê a pilha pela
//! porta do produto (`vec_entities::z_order`) — o que esta wave precisa provar é uma RELAÇÃO entre
//! as duas, e uma lista de entradas escrita à mão afirmaria a relação em vez de a medir.
//!
//! ⚠️ **Metade destes gates afirmava o CONTRÁRIO até 2026-08-04**, e a mudança não é de gosto: a
//! pilha de z deixou de ser o DFS invertido (a lei de Godot — *o filho desenha sobre o pai*), então
//! o pai passou a ser o PRIMEIRO membro da própria sub-árvore, a antecipação do desenho dele
//! MORREU, e com ela o intervalo de quem não recorta.

use super::*;
use ph2d_ecs::scene::{HierarchyWalkState, build_hierarchy_snapshot};
use ph2d_ecs::{ChildOf, Transform, VecPathRef, ZIndexOverride};

/// Os intervalos E a pilha de z do mesmo mundo, pelas duas portas do produto.
fn spans_and_z(sim: &mut SimWorld) -> (Vec<VecClipSpan>, Vec<u64>) {
    let mut state = HierarchyWalkState::new(sim.world_mut());
    let mut scratch = Vec::new();
    let mut snap = HierarchySnapshot::default();
    build_hierarchy_snapshot(sim.world(), &mut state, &mut scratch, &mut snap);
    let z = crate::vec_entities::z_order(sim.world(), &snap);
    let spans = clip_spans(sim, &snap, &z);
    (spans, z)
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

/// **O intervalo FECHA no descendente que desenha por ÚLTIMO** — e a moldura é o PRIMEIRO membro
/// da própria sub-árvore, que é o que faz o preenchimento dela ser o fundo do card sem ninguém
/// antecipar nada.
///
/// ⚠️ Ler isto ao contrário fecha o recorte no lugar errado e some com quase toda a arte.
#[test]
fn the_span_closes_at_the_descendant_that_draws_last() {
    let (mut sim, frame, kids, _) = scene(3, true);
    let (spans, z) = spans_and_z(&mut sim);
    assert_eq!(spans.len(), 1, "uma moldura que recorta, um intervalo");
    assert_eq!(spans[0].frame, frame);

    let top = *z
        .iter()
        .rev()
        .find(|id| kids.contains(id))
        .expect("algum filho na pilha");
    assert_eq!(spans[0].last, top, "o intervalo fecha no filho da FRENTE");

    // E a moldura é a PRIMEIRA da própria sub-árvore — é este fato que faz um par (abre, fecha)
    // bastar para descrever o recorte, e é ele que põe o filho SOBRE o pai.
    let pos_frame = z
        .iter()
        .position(|id| *id == frame)
        .expect("moldura na pilha");
    for k in &kids {
        let pk = z.iter().position(|id| id == k).expect("filho na pilha");
        assert!(pk > pos_frame, "o filho {k} desenha DEPOIS da moldura");
    }
}

/// **Uma moldura com `clip` desligado NÃO tem intervalo** — e isto é o oposto do que este arquivo
/// afirmava até 2026-08-04.
///
/// ⚠️ Naquele mundo o intervalo era *onde o desenho do pai é antecipado*, e por isso todo pai
/// precisava de um. Com a projeção invertida a antecipação não existe: o pai desenha primeiro
/// porque é o primeiro. Um intervalo para quem não recorta seria um `push`/`pop` de camada que
/// não recorta nada.
#[test]
fn an_unclipped_frame_gets_no_span_because_there_is_nothing_to_clip() {
    let (mut sim, _, _, _) = scene(3, false);
    assert!(
        spans_of(&mut sim).is_empty(),
        "quem nao recorta nao precisa de intervalo"
    );
}

/// Sem descendente vetorial não há o que recortar — e um intervalo vazio faria a moldura abrir e
/// fechar sobre si mesma.
#[test]
fn an_empty_frame_produces_no_span() {
    let (mut sim, _, _, _) = scene(0, true);
    assert!(spans_of(&mut sim).is_empty());
}

/// Molduras aninhadas: a lista sai de FORA para DENTRO, porque as duas fecham no mesmo path e a
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
    // As duas fecham no MESMO path (o neto é o único descendente vetorial de ambas) — é exatamente
    // o caso em que a ordem decide se o LIFO fecha certo.
    assert_eq!(spans[0].last, 30);
    assert_eq!(spans[1].last, 30);
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

/// **A LEI: o filho desenha SOBRE o pai** — e ela mora na PROJEÇÃO, não num intervalo.
///
/// ⚠️ Este gate nasceu de um relato do Enio (2026-08-04) e da constatação de que **nada o
/// pinava**. Ele foi escrito primeiro contra a *antecipação* (o pai continuava por último na pilha
/// e o renderer desenhava-o cedo) — e o report seguinte mostrou o preço daquele desenho: a
/// INSTÂNCIA de componente percorre a cena e **não tem renderer por trás**, então herdava a pilha
/// crua e cobria os próprios filhos. A cura foi inverter a projeção, e é ela que este gate mede.
///
/// A mutação que o mata é voltar a inverter a pilha (`entries … .rev()`).
#[test]
fn a_plain_parent_draws_before_its_child_so_the_child_is_on_top() {
    let (mut sim, parent, kid) = plain_parent();
    let (spans, z) = spans_and_z(&mut sim);
    assert!(
        spans.is_empty(),
        "um pai comum nao recorta, logo nao tem intervalo: {spans:?}"
    );
    let pp = z.iter().position(|id| *id == parent).expect("pai na pilha");
    let pk = z.iter().position(|id| *id == kid).expect("filho na pilha");
    assert!(
        pp < pk,
        "o pai deixou de desenhar ANTES do filho — ele volta a cobrir o proprio conteudo"
    );
}

/// Um vizinho fora da moldura não entra no intervalo. Sem isto o recorte comeria a cena inteira.
#[test]
fn a_sibling_outside_the_frame_is_not_in_the_span() {
    let (mut sim, _, kids, outsider) = scene(2, true);
    let spans = spans_of(&mut sim);
    assert_eq!(spans.len(), 1);
    assert_ne!(spans[0].last, outsider);
    assert!(kids.contains(&spans[0].last));
}

/// **O Z tira um filho de dentro do recorte, e isso é o que *"global"* significa.**
///
/// ⚠️ A consequência é honesta e é a razão de o intervalo ser resolvido contra a pilha FINAL, por
/// CONTIGUIDADE: assim que uma forma alheia pousa entre a moldura e um filho, o recorte fecha ali
/// — o filho que ficou do outro lado **deixa de ser recortado**, e a forma alheia **nunca** é
/// recortada por um card que não é dela.
#[test]
fn a_child_lifted_by_z_leaves_the_clip_span() {
    let (mut sim, _, kids, _) = scene(2, true);
    // Uma forma alheia, que nasce à FRENTE de toda a sub-árvore da moldura.
    let stranger = sim
        .world_mut()
        .spawn((Transform::default(), VecPathRef(902)))
        .id();
    let _ = stranger;

    // O filho da FRENTE é levado para além dela pelo Z.
    let lifted = kids[1];
    let e = {
        let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &VecPathRef)>();
        q.iter(sim.world())
            .find(|(_, v)| v.0 == lifted)
            .map(|(e, _)| e)
            .expect("o filho existe")
    };
    sim.world_mut().entity_mut(e).insert(ZIndexOverride(5));

    let (spans, z) = spans_and_z(&mut sim);
    assert_eq!(
        *z.last().unwrap(),
        lifted,
        "o Z nao levou o filho para a frente de tudo"
    );
    assert_eq!(spans.len(), 1, "a moldura ainda recorta o que sobrou");
    assert_ne!(
        spans[0].last, lifted,
        "o intervalo ainda alcanca o filho que o Z tirou de dentro dele — a forma alheia entre \
         os dois seria recortada por um card que nao e' dela"
    );
    assert_eq!(spans[0].last, kids[0], "ele fecha no filho que FICOU");
}

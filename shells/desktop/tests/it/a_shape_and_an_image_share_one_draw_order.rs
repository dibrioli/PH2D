//! ⭐⭐⭐ **A LEI DO REPORT DE 2026-08-30**, executável e sem GPU.
//!
//! Enio: *«desenhei um vector, depois uma imagem; no hierarchy ficou correto — a imagem abaixo do
//! vector —, mas no canvas o vector ficou acima da IMG. A regra neste app é que se ambos têm o
//! mesmo z index, quem está abaixo na hierarquia fica acima no canvas.»*
//!
//! # O que este gate afirma
//!
//! Que as duas famílias saem de **UM** ordenador. Até hoje elas saíam de dois: o
//! `sort_key::compute_sort_ranks_into` para os sprites, e o `vec_zorder::keyed_stack` para as
//! formas — dois totais independentes, colados por um `over` fixo no compositor. Nenhum valor de Z
//! podia atravessar essa fronteira, porque os dois Z nunca eram comparados um com o outro.
//!
//! ⚠️ **Ele NÃO precisa de `Sprite` para nada**, e essa é a descoberta que tornou a cura pequena: o
//! ordenador lê `ChildOf`, `ShowBehindParent`, `SortingLayer`, `OrderInLayer`, `ZIndexOverride` e
//! `YSort` — todos genéricos. Ele era sprite-only *porque só sprites lhe eram entregues*.

use ph2d_ecs::sort_key::{SortInput, compute_sort_ranks};
use ph2d_ecs::{Entity, Name, RootOrder, SimWorld, Transform, VecPathRef, ZIndexOverride};
use ph2d_render::Sprite;

/// Uma forma vetorial, como o `vec_entities::sync` a cunha.
fn shape(sim: &mut SimWorld, order: u32, id: u64) -> Entity {
    sim.world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Shape"),
            VecPathRef(id),
            RootOrder(order),
        ))
        .id()
}

/// Uma imagem importada.
fn image(sim: &mut SimWorld, order: u32) -> Entity {
    sim.world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Image"),
            Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            RootOrder(order),
        ))
        .id()
}

/// O rank de cada entidade, alimentando o ordenador em ordem de hierarquia (DFS pre-order) — que é
/// exactamente o que o `sim_extract` faz na travessia.
fn ranks(sim: &SimWorld, dfs: &[Entity]) -> Vec<(Entity, u32)> {
    let inputs: Vec<SortInput> = dfs
        .iter()
        .map(|&entity| SortInput {
            entity,
            world_pos: ph2d_core::Vec2::ZERO,
        })
        .collect();
    compute_sort_ranks(sim.world(), &inputs)
}

fn rank_of(r: &[(Entity, u32)], e: Entity) -> u32 {
    r.iter()
        .find(|(x, _)| *x == e)
        .map(|(_, k)| *k)
        .expect("a entidade nao recebeu rank — ela nao entrou na ordem")
}

/// ⭐⭐⭐ **O caso exacto do report.** Forma primeiro, imagem depois ⇒ a imagem está ABAIXO na
/// hierarquia ⇒ ela desenha DEPOIS ⇒ por cima.
///
/// **Mutação que deve sangrar:** tirar a forma do `inputs` (que é literalmente o estado em que o
/// app estava — o `sim_extract` só empurrava sprites).
#[test]
fn the_image_below_the_shape_in_the_hierarchy_draws_above_it() {
    let mut sim = SimWorld::new();
    let s = shape(&mut sim, 0, 1);
    let i = image(&mut sim, 1);
    let r = ranks(&sim, &[s, i]);
    assert_eq!(r.len(), 2, "uma das duas familias ficou de fora da ordem");
    assert!(
        rank_of(&r, s) < rank_of(&r, i),
        "a forma tem de sair ANTES (mais atras) — ela esta' acima na hierarquia"
    );
}

/// E o contrário também: importar a imagem antes de desenhar põe a forma por cima.
#[test]
fn the_shape_below_the_image_draws_above_it() {
    let mut sim = SimWorld::new();
    let i = image(&mut sim, 0);
    let s = shape(&mut sim, 1, 1);
    let r = ranks(&sim, &[i, s]);
    assert!(
        rank_of(&r, i) < rank_of(&r, s),
        "a imagem tem de sair ANTES — ela esta' acima na hierarquia"
    );
}

/// ⭐⭐ **E o Z-index ATRAVESSA as duas famílias** — que é a metade que a seção nova do Inspector
/// promete e que, com dois ordenadores, era impossível.
#[test]
fn a_z_index_on_the_image_lifts_it_above_the_shape() {
    let mut sim = SimWorld::new();
    // A forma fica ABAIXO na hierarquia ⇒ sem Z ela ganharia.
    let i = image(&mut sim, 0);
    let s = shape(&mut sim, 1, 1);
    assert!(rank_of(&ranks(&sim, &[i, s]), i) < rank_of(&ranks(&sim, &[i, s]), s));
    // Com um Z maior, a imagem sobe.
    sim.world_mut().entity_mut(i).insert(ZIndexOverride(5));
    let r = ranks(&sim, &[i, s]);
    assert!(
        rank_of(&r, s) < rank_of(&r, i),
        "o Z da imagem nao a levantou acima da forma — as duas familias nao partilham a ordem"
    );
}

/// ⚠️ **E o Z da FORMA também funciona**, no sentido oposto — a assimetria seria o sintoma de uma
/// família a ser lida por um caminho diferente.
#[test]
fn a_z_index_on_the_shape_lifts_it_above_the_image() {
    let mut sim = SimWorld::new();
    let s = shape(&mut sim, 0, 1);
    let i = image(&mut sim, 1);
    sim.world_mut().entity_mut(s).insert(ZIndexOverride(5));
    let r = ranks(&sim, &[s, i]);
    assert!(
        rank_of(&r, i) < rank_of(&r, s),
        "o Z da forma nao a levantou acima da imagem"
    );
}

/// A ordem é TOTAL: com três objectos alternados, os três ranks são distintos e seguem a árvore.
#[test]
fn three_alternating_objects_get_three_distinct_ranks_in_tree_order() {
    let mut sim = SimWorld::new();
    let a = shape(&mut sim, 0, 1);
    let b = image(&mut sim, 1);
    let c = shape(&mut sim, 2, 2);
    let r = ranks(&sim, &[a, b, c]);
    let (ra, rb, rc) = (rank_of(&r, a), rank_of(&r, b), rank_of(&r, c));
    assert!(
        ra < rb && rb < rc,
        "ordem nao segue a arvore: {ra} {rb} {rc}"
    );
}

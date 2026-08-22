//! Os gates dos GESTOS da booleana viva.
//!
//! O produtor já prova o que um grupo desenha. O que só se pode afirmar aqui é a decisão do
//! clique — e ela tem três destinos, com uma ordem que, invertida, DESTRÓI trabalho.

use super::*;
use ph2d_vec_scene::rectangle;

fn setup() -> (SimWorld, VecScene, VecEntityMap, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [2.0, 2.0]));
    let b = scene.push_path(rectangle([1.0, 1.0], [3.0, 3.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    (sim, scene, map, vec![a, b])
}

/// **Armar cria um grupo com o componente, e os caminhos continuam no documento.**
#[test]
fn arming_makes_a_group_and_keeps_the_operands() {
    let (mut sim, scene, map, ids) = setup();
    assert!(arm(&mut sim, &scene, &map, &ids, 0));

    let g = group_of_selection(&sim, &map, &ids).expect("a seleção passou a ter grupo booleano");
    assert_eq!(
        sim.world().get::<VecBoolGroup>(g).map(|c| c.op),
        Some(0),
        "o grupo tem de carregar a operação pedida"
    );
    assert_eq!(
        scene.paths().len(),
        2,
        "os operandos continuam no documento"
    );
    // O nome importa: é por ele que o artista acha o grupo na Hierarquia.
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::Name>(g)
            .map(|n| n.0.to_string()),
        Some("Boolean".to_string())
    );
}

/// **Clicar noutra operação com o grupo selecionado RE-MIRA, e não cria um segundo grupo.**
///
/// ⚠️ É a metade que a ordem dos três destinos protege: sem ela, o segundo clique empilharia um
/// grupo dentro do outro (ou, com o modo desligado, CONSUMIRIA os operandos).
#[test]
fn a_second_click_retargets_instead_of_nesting() {
    let (mut sim, scene, map, ids) = setup();
    assert!(arm(&mut sim, &scene, &map, &ids, 0));
    let first = group_of_selection(&sim, &map, &ids).unwrap();

    assert!(arm(&mut sim, &scene, &map, &ids, 2));
    let second = group_of_selection(&sim, &map, &ids).unwrap();
    assert_eq!(first, second, "o clique criou um SEGUNDO grupo");
    assert_eq!(
        sim.world().get::<VecBoolGroup>(second).map(|c| c.op),
        Some(2),
        "a operação tem de ter mudado"
    );
}

/// **Menos de duas regiões FECHADAS não é uma booleana** — a mesma triagem do caminho destrutivo.
#[test]
fn fewer_than_two_closed_shapes_refuses() {
    let (mut sim, mut scene, map, ids) = setup();
    scene.path_mut(ids[1]).unwrap().closed = false;
    assert!(!arm(&mut sim, &scene, &map, &ids, 0));
    assert!(group_of_selection(&sim, &map, &ids).is_none());
}

/// **O bake materializa o plano no z da BASE, consome os operandos e mata o grupo.**
#[test]
fn baking_puts_the_result_where_the_base_sat_and_kills_the_group() {
    let (mut sim, mut scene, mut map, ids) = setup();
    // Uma terceira forma NA FRENTE, fora do grupo: ela prova que o resultado ocupa a fatia da
    // base em vez de saltar para o topo.
    let front = scene.push_path(rectangle([9.0, 9.0], [10.0, 10.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert!(arm(&mut sim, &scene, &map, &ids, 0));
    let g = group_of_selection(&sim, &map, &ids).unwrap();

    let mut live = ph2d_vec_render::LiveGeometry::new();
    let mut bl = crate::bool_live::BoolLive::default();
    bl.recook(
        &scene,
        &sim,
        &map,
        &ph2d_vec_scene::VecXforms::default(),
        &mut live,
    );
    let plan = bl.plan(g).expect("o grupo cozinhou");
    let out_len: usize = plan.sinks.iter().map(|(_, v)| v.len()).sum();

    let mut pen = ph2d_vec_edit::PenTool::default();
    let made = bake(&mut sim, &mut scene, &mut pen, plan, g);

    assert_eq!(
        made, out_len,
        "o bake materializa o que o produtor cozinhou"
    );
    assert_eq!(
        scene.paths().len(),
        out_len + 1,
        "os dois operandos saíram; a forma da frente ficou"
    );
    assert_eq!(
        scene.paths().last().map(|p| p.id),
        Some(front),
        "o resultado tem de ficar ATRÁS da forma da frente"
    );
    assert!(
        sim.world().get_entity(g).is_err(),
        "o grupo tem de morrer — um grupo vazio na Hierarquia é lixo"
    );
}

/// **O que o bake escreve é EXATAMENTE o que estava na tela.**
///
/// ⚠️ O gate existe para pinar a porta única: se o Apply re-chamasse o motor, este teste passaria
/// hoje e falharia no dia em que qualquer coisa a montante mudasse entre o cozimento e o clique —
/// que é precisamente o modo de falha que ele previne.
#[test]
fn the_bake_writes_the_geometry_that_was_on_screen() {
    let (mut sim, mut scene, map, ids) = setup();
    assert!(arm(&mut sim, &scene, &map, &ids, 0));
    let g = group_of_selection(&sim, &map, &ids).unwrap();

    let mut live = ph2d_vec_render::LiveGeometry::new();
    let mut bl = crate::bool_live::BoolLive::default();
    bl.recook(
        &scene,
        &sim,
        &map,
        &ph2d_vec_scene::VecXforms::default(),
        &mut live,
    );
    let drawn = live.get(&ids[0]).expect("a base carrega o desenho").clone();
    let plan = bl.plan(g).unwrap();

    let mut pen = ph2d_vec_edit::PenTool::default();
    bake(&mut sim, &mut scene, &mut pen, plan, g);

    let baked: Vec<_> = scene.paths().iter().map(|p| p.verts.clone()).collect();
    let expect: Vec<_> = drawn.iter().map(|p| p.verts.clone()).collect();
    assert_eq!(baked, expect, "o bake não pode mover um vértice");
}

/// **O APPLY DE DOIS SUMIDOUROS PÕE CADA RESULTADO NA FATIA DE Z DELE.**
///
/// Um grafo pode produzir mais de uma forma: aqui `b` soma com `a` e é subtraída de `c`, e as duas
/// respostas ocupam lugares diferentes na pilha. O caminho do grupo único nunca precisou disto —
/// ele tem uma base e um resultado.
///
/// ⚠️ O oráculo é a ORDEM na pilha, não a contagem: a remoção dos operandos desloca todo índice à
/// frente dela, e um Apply que insira pelos índices de ANTES da remoção produz o número certo de
/// formas no lugar errado. É o modo de falha que uma contagem não vê.
#[test]
fn o_apply_de_dois_sumidouros_poe_cada_resultado_na_fatia_de_z_dele() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [2.0, 2.0]));
    let b = scene.push_path(rectangle([1.0, 0.0], [3.0, 2.0]));
    let c = scene.push_path(rectangle([2.0, 0.0], [4.0, 2.0]));
    // Uma forma FORA do grupo, no topo: ela prova que os dois resultados ficam ATRÁS dela.
    let front = scene.push_path(rectangle([9.0, 9.0], [10.0, 10.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let g = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&a], map[&b], map[&c]], "Bool".into())
            .unwrap(),
    );
    sim.world_mut()
        .entity_mut(g)
        .insert(ph2d_ecs::VecBoolGroup { op: 0 });
    sim.world_mut()
        .entity_mut(g)
        .insert(ph2d_ecs::VecBoolEdges::new(vec![
            ph2d_ecs::VecBoolEdge {
                from: b,
                to: a,
                op: 0,
            },
            ph2d_ecs::VecBoolEdge {
                from: b,
                to: c,
                op: 1,
            },
        ]));

    let mut live = ph2d_vec_render::LiveGeometry::new();
    let mut bl = crate::bool_live::BoolLive::default();
    bl.recook(
        &scene,
        &sim,
        &map,
        &ph2d_vec_scene::VecXforms::default(),
        &mut live,
    );
    let plan = bl.plan(g).expect("o grafo cozinhou");
    let esperado: Vec<Vec<ph2d_vec_scene::VecVertex>> = plan
        .sinks
        .iter()
        .flat_map(|(_, v)| v.iter().map(|p| p.verts.clone()))
        .collect();

    let mut pen = ph2d_vec_edit::PenTool::default();
    let made = bake(&mut sim, &mut scene, &mut pen, plan, g);

    assert_eq!(made, 2, "dois sumidouros, duas formas");
    assert_eq!(
        scene.paths().len(),
        3,
        "os três operandos saíram, dois resultados entraram, a forma da frente ficou"
    );
    assert_eq!(
        scene.paths().last().map(|p| p.id),
        Some(front),
        "os resultados têm de ficar ATRÁS da forma da frente"
    );
    let escrito: Vec<Vec<ph2d_vec_scene::VecVertex>> = scene
        .paths()
        .iter()
        .take(2)
        .map(|p| p.verts.clone())
        .collect();
    assert_eq!(escrito, esperado, "o bake não pode mover um vértice");
}

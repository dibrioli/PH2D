//! **OS GATES DO VERBO POR FORMA** (2026-08-22) — irmão dos gates-base pelo teto de 600 LOC do
//! HR-18, e o corte é por ASSUNTO: ali mora *o que um grupo booleano desenha*; aqui, *o que cada
//! forma DENTRO dele manda*.
//!
//! ⚠️ Fixture própria (`scene_with_three`), e ela é obrigatória: com DUAS formas a frase do pedido
//! — *"somo com esta, subtraio aquela"* — não tem a segunda metade, e um gate escrito sobre um par
//! ficaria verde sem nunca exercitar uma cadeia.

use super::tests::{area_of, run, scene_with_group};
use super::*;
use ph2d_vec_scene::rectangle;

/// Três retângulos num grupo booleano `op`, na ordem de z `a` (base) → `b` → `c`.
///
/// `a ∪ b` cobre `[0,30]×[0,20]` (área **600**); `c` é um 10×10 no meio dele. Logo, com `c` a
/// SUBTRAIR, a resposta é **500** — separada da de herança por 100, e não por ruído.
fn scene_with_three(op: u8) -> (SimWorld, VecScene, VecEntityMap, [VecPathId; 3], Entity) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [20.0, 20.0]));
    let b = scene.push_path(rectangle([10.0, 0.0], [30.0, 20.0]));
    let c = scene.push_path(rectangle([15.0, 5.0], [25.0, 15.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let g = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&a], map[&b], map[&c]], "Bool".into())
            .unwrap(),
    );
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op });
    (sim, scene, map, [a, b, c], g)
}

/// Põe o verbo `op` na forma `id` — o que o painel faz ao clicar num modo com a forma em mãos.
fn set_verb(sim: &mut SimWorld, map: &VecEntityMap, id: VecPathId, op: u8) {
    sim.world_mut()
        .entity_mut(Entity::from_bits(map[&id]))
        .insert(ph2d_ecs::VecBoolOp { op });
}

/// A área que o grupo desenha neste frame.
fn drawn_area(sim: &SimWorld, scene: &VecScene, map: &VecEntityMap, base: VecPathId) -> f64 {
    let mut live = LiveGeometry::new();
    run(sim, scene, map, &mut live);
    area_of(live.get(&base).expect("a base carrega o resultado"))
}

/// **A CAPACIDADE INTEIRA, numa afirmação:** *somo com esta, subtraio aquela.*
///
/// Grupo em Union; `c` traz Subtract. `(a ∪ b) − c` = 500, contra os 600 que a herança dá.
#[test]
fn a_per_shape_verb_folds_onto_the_result_of_the_ones_before_it() {
    let (mut sim, scene, map, ids, _g) = scene_with_three(0); // Union
    assert!(
        (drawn_area(&sim, &scene, &map, ids[0]) - 600.0).abs() < 1.0,
        "pré-condição: sem override, a união dos três cobre 600"
    );
    set_verb(&mut sim, &map, ids[2], 1); // Subtract
    let with = drawn_area(&sim, &scene, &map, ids[0]);
    assert!(
        (with - 500.0).abs() < 1.0,
        "o verbo da forma tinha de abrir o furo de 100: deu {with:.2}"
    );
}

/// **HERANÇA: toda forma a repetir o verbo do grupo desenha o que NENHUM override desenha.**
///
/// É o gate que garante que todo documento anterior a esta feature continua byte-idêntico — e o
/// que impede a cura de virar *"o grupo deixou de decidir"*.
#[test]
fn every_shape_repeating_the_groups_verb_draws_what_no_override_draws() {
    for op in 0..=3u8 {
        let (sim, scene, map, ids, _g) = scene_with_three(op);
        let mut bare = LiveGeometry::new();
        run(&sim, &scene, &map, &mut bare);
        let (mut sim2, scene2, map2, ids2, _g2) = scene_with_three(op);
        for id in ids2 {
            set_verb(&mut sim2, &map2, id, op);
        }
        let mut spelled = LiveGeometry::new();
        run(&sim2, &scene2, &map2, &mut spelled);
        assert_eq!(
            bare.get(&ids[0]),
            spelled.get(&ids2[0]),
            "op {op}: escrever o verbo do grupo em cada forma mudou o desenho"
        );
    }
}

/// **TROCAR O MODO DE UMA FORMA RE-COZINHA.** O memo compara a entrada e o `op` do grupo — e
/// nenhum dos dois muda quando o artista clica num modo por forma.
///
/// ⚠️ Sem os verbos na chave, o memo dá acerto, a resposta velha é re-servida, e **o clique não
/// faz nada na tela**: o defeito mais barato de escrever e o mais caro de diagnosticar, porque
/// não há erro nenhum — só uma UI que parece morta.
#[test]
fn changing_a_shapes_verb_recooks_instead_of_serving_the_memo() {
    let (mut sim, scene, map, ids, _g) = scene_with_three(0); // Union
    let mut bl = BoolLive::default();
    let xf = VecXforms::default();
    let mut live = LiveGeometry::new();
    bl.recook(&scene, &sim, &map, &xf, &[], &mut live);
    let before = area_of(live.get(&ids[0]).unwrap());

    set_verb(&mut sim, &map, ids[2], 1); // Subtract
    // ⚠️ O MESMO `BoolLive` — um novo teria memo vazio e o gate seria verde sem provar nada.
    let mut live2 = LiveGeometry::new();
    bl.recook(&scene, &sim, &map, &xf, &[], &mut live2);
    let after = area_of(live2.get(&ids[0]).unwrap());
    assert!(
        (before - 600.0).abs() < 1.0 && (after - 500.0).abs() < 1.0,
        "o memo re-serviu a resposta velha: {before:.2} -> {after:.2}"
    );
}

/// **A ORDEM DA HIERARQUIA É A ORDEM DA CADEIA** — e é por isso que ela precisa de ser visível.
///
/// As mesmas três formas e o mesmo verbo em `c`, com `c` antes ou depois de `b`: 500 contra 600.
#[test]
fn the_order_in_the_hierarchy_is_the_order_of_the_chain() {
    // a → b → c, com `c` a subtrair: o furo é aberto no fim e sobrevive.
    let (mut sim, scene, map, ids, _g) = scene_with_three(0);
    set_verb(&mut sim, &map, ids[2], 1);
    let late = drawn_area(&sim, &scene, &map, ids[0]);

    // a → c → b: `c` subtrai cedo, e `b` volta a cobrir o furo ao unir-se depois.
    let mut sim2 = SimWorld::default();
    let mut scene2 = VecScene::new();
    let mut map2 = VecEntityMap::new();
    let a = scene2.push_path(rectangle([0.0, 0.0], [20.0, 20.0]));
    let c = scene2.push_path(rectangle([15.0, 5.0], [25.0, 15.0]));
    let b = scene2.push_path(rectangle([10.0, 0.0], [30.0, 20.0]));
    crate::vec_entities::sync(&mut sim2, &mut scene2, &mut map2);
    let g2 = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim2, &[map2[&a], map2[&c], map2[&b]], "B".into())
            .unwrap(),
    );
    sim2.world_mut()
        .entity_mut(g2)
        .insert(VecBoolGroup { op: 0 });
    set_verb(&mut sim2, &map2, c, 1);
    let early = drawn_area(&sim2, &scene2, &map2, a);

    assert!(
        (late - 500.0).abs() < 1.0 && (early - 600.0).abs() < 1.0,
        "mover a forma na hierarquia tem de mudar o desenho: {late:.2} e {early:.2}"
    );
}

/// ⛔ **UMA RECEITA NO GRUPO IGNORA OS VERBOS POR FORMA**, e o gate existe para que a UI seja
/// obrigada a dizê-lo.
///
/// `Trim`/`Crop`/`Merge`/`MinusBack` são afirmações sobre a PILHA INTEIRA — *"cada forma menos a
/// união do que está acima dela"* não é uma relação entre duas. Um seletor por forma oferecido
/// sobre uma receita seria um controlo que não controla nada.
#[test]
fn a_recipe_on_the_group_ignores_the_per_shape_verbs() {
    let (sim, scene, map, ids, _g) = scene_with_three(5); // Trim
    let mut bare = LiveGeometry::new();
    run(&sim, &scene, &map, &mut bare);

    let (mut sim2, scene2, map2, ids2, _g2) = scene_with_three(5);
    set_verb(&mut sim2, &map2, ids2[2], 1); // Subtract — não pode mudar nada
    let mut over = LiveGeometry::new();
    run(&sim2, &scene2, &map2, &mut over);

    assert_eq!(
        bare.get(&ids[0]),
        over.get(&ids2[0]),
        "o override mexeu numa receita, que é verbo da pilha inteira"
    );
}

/// **UM CÓDIGO DE RECEITA NUMA FORMA DEGRADA PARA HERANÇA** — a leitura que não perde arte.
///
/// Um save vindo de um build futuro (ou um dedo escorregado) pode pôr `Trim` numa forma. Ela não
/// tem como o honrar; herdar o verbo do grupo desenha algo coerente, e recusar desenharia nada.
#[test]
fn a_recipe_code_on_a_shape_falls_back_to_the_groups_verb() {
    let (mut sim, scene, map, ids, _g) = scene_with_three(0); // Union
    set_verb(&mut sim, &map, ids[2], 5); // Trim — não é operação de conjunto
    let area = drawn_area(&sim, &scene, &map, ids[0]);
    assert!(
        (area - 600.0).abs() < 1.0,
        "devia ter herdado o Union do grupo e dado 600, deu {area:.2}"
    );
}

/// **O VERBO DA BASE É INERTE**, e não por um `if` — pela representação.
///
/// A forma mais ao fundo não dobra sobre nada: ela É o acumulador inicial. O Illustrator tem a
/// mesma inércia no componente de baixo de um compound shape. ⚠️ Este gate é o que obriga a UI a
/// não oferecer o seletor na linha da base: um controlo inerte pintado como vivo é pior que
/// controlo nenhum.
#[test]
fn the_verb_of_the_base_is_inert() {
    let (mut sim, scene, map, ids, _g) = scene_with_three(0); // Union
    set_verb(&mut sim, &map, ids[0], 1); // Subtract NA BASE
    let area = drawn_area(&sim, &scene, &map, ids[0]);
    assert!(
        (area - 600.0).abs() < 1.0,
        "o verbo da base mudou o desenho: deu {area:.2}"
    );
}

/// **O VERBO É DA FORMA, e alcança TODOS os caminhos que ela trouxe.**
///
/// Um operando não contribui necessariamente com um caminho: um offset vivo, um pattern ou um
/// composto entregam vários. O verbo é da FORMA, então tem de valer para todos eles.
///
/// ⚠️ Este gate existe porque uma prova de mutação o exigiu: trocar *"repete o verbo por caminho"*
/// por *"uma vez por forma"* **sobrevivia a todos os outros gates**, porque em todos eles cada
/// forma trazia exactamente um caminho. Com o verbo a mais na conta, o zip trunca e **o último
/// caminho cai fora da cadeia em silêncio** — a forma some da operação sem erro nenhum.
#[test]
fn the_verb_reaches_every_path_the_shape_contributed() {
    let (mut sim, scene, map, ids, _g) = scene_with_group(0); // Union
    set_verb(&mut sim, &map, ids[1], 1); // Subtract

    // O operando de cima chega ao mapa já DERIVADO, como dois quadrados disjuntos dentro da base.
    let mut live = LiveGeometry::new();
    live.insert(
        ids[1],
        vec![
            rectangle([0.2, 0.2], [0.7, 0.7]),
            rectangle([1.2, 1.2], [1.7, 1.7]),
        ],
    );
    run(&sim, &scene, &map, &mut live);

    let got = area_of(live.get(&ids[0]).expect("a base carrega o resultado"));
    assert!(
        (got - 3.5).abs() < 1e-6,
        "a base (4,0) menos DOIS quadrados de 0,25 mede {got:.4}, esperado 3,5 — \
         3,75 significa que o segundo caminho caiu fora da cadeia"
    );
}

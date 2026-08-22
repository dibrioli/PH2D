//! Os gates da BOOLEANA VIVA.
//!
//! O motor (`ph2d-vec-boolean`) já prova o que cada operação produz. O que só se pode afirmar
//! aqui é a costura: que o grupo desenha UMA forma no lugar de N, que os operandos continuam no
//! documento, que a operação re-cozinha, que a recusa do motor não pisca a arte — e que o mundo
//! de quem nunca criou um grupo booleano fica **byte-intocado**.

use super::*;
use ph2d_vec_scene::{Paint, Rgba8, VecVertex, rectangle};

/// Cena mínima: dois retângulos SOBREPOSTOS, sincados, agrupados, com a operação `op`.
/// Devolve `(sim, scene, map, ids em z, grupo)`.
fn scene_with_group(op: u8) -> (SimWorld, VecScene, VecEntityMap, Vec<VecPathId>, Entity) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [2.0, 2.0]));
    let b = scene.push_path(rectangle([1.0, 1.0], [3.0, 3.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let g = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&a], map[&b]], "Bool".into()).unwrap(),
    );
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op });
    (sim, scene, map, vec![a, b], g)
}

fn run(sim: &SimWorld, scene: &VecScene, map: &VecEntityMap, live: &mut LiveGeometry) -> BoolLive {
    let mut bl = BoolLive::default();
    bl.recook(scene, sim, map, &VecXforms::default(), live);
    bl
}

/// **Um grupo booleano desenha UMA forma, e os operandos desenham NADA.**
///
/// ⚠️ A segunda metade é a que a feature inteira depende: sem a lista vazia os retângulos
/// continuariam a ser desenhados POR CIMA do resultado, e a união pareceria não ter acontecido.
#[test]
fn a_live_boolean_draws_one_shape_and_its_operands_draw_nothing() {
    let (sim, scene, map, ids, _g) = scene_with_group(0); // Union
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);

    let base = live.get(&ids[0]).expect("a base carrega o resultado");
    assert_eq!(base.len(), 1, "a união de dois retângulos é uma forma");
    assert_eq!(
        live.get(&ids[1]).map(Vec::len),
        Some(0),
        "o outro operando tem de estar no mapa, VAZIO"
    );
    // E o documento não foi tocado: os dois operandos continuam lá, editáveis.
    assert_eq!(scene.paths().len(), 2);
}

/// **Sem o componente, o mapa nasce vazio** — é o que torna todo documento anterior a esta
/// feature byte-idêntico.
#[test]
fn a_plain_group_never_enters_the_live_map() {
    let (mut sim, scene, map, _ids, g) = scene_with_group(0);
    sim.world_mut().entity_mut(g).remove::<VecBoolGroup>();
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);
    assert!(
        live.is_empty(),
        "o mapa nasceu com {} entrada(s)",
        live.len()
    );
}

/// **Trocar a operação re-cozinha, e o documento não muda.**
#[test]
fn changing_the_op_recooks_without_touching_the_document() {
    let (mut sim, scene, map, ids, g) = scene_with_group(0); // Union
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);
    let union_area = area_of(live.get(&ids[0]).unwrap());

    let before = scene.clone();
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op: 2 }); // Intersect
    let mut live2 = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live2);
    let inter_area = area_of(live2.get(&ids[0]).unwrap());

    assert!(
        inter_area < union_area * 0.5,
        "a interseção ({inter_area:.3}) tem de ser bem menor que a união ({union_area:.3})"
    );
    assert_eq!(scene, before, "o documento não pode ter mudado");
}

/// **A booleana COMPÕE com a derivada de outro produtor em vez de a re-derivar da fonte.**
///
/// ⚠️ É o gate que justifica este módulo transformar o mapa em vez de o estender. A fixture põe
/// no mapa uma geometria que a FONTE não tem (um triângulo onde a cena guarda um retângulo): se a
/// booleana lesse a fonte, o resultado teria a área do retângulo.
#[test]
fn the_boolean_consumes_what_the_other_producers_drew() {
    let (sim, scene, map, ids, _g) = scene_with_group(2); // Intersect

    // "Já derivado": o operando de cima é, no mapa, um retângulo MINÚSCULO dentro do de baixo.
    let tiny = rectangle([0.2, 0.2], [0.4, 0.4]);
    let mut live = LiveGeometry::new();
    live.insert(ids[1], vec![tiny]);
    run(&sim, &scene, &map, &mut live);

    let got = area_of(live.get(&ids[0]).expect("a base carrega o resultado"));
    assert!(
        (got - 0.04).abs() < 1e-6,
        "a interseção com o retângulo derivado mede {got:.4}, esperado 0.04 \
         (a booleana re-derivou da FONTE?)"
    );
}

/// **Um caminho ABERTO dentro do grupo passa verbatim** — ele não é operando, e suprimi-lo
/// apagaria arte que a operação nunca consumiu.
#[test]
fn an_open_path_inside_the_group_is_not_an_operand() {
    let (mut sim, mut scene, mut map, _ids, g) = scene_with_group(0);
    let open = scene.push_path(VecPath {
        verts: vec![VecVertex::corner([5.0, 5.0]), VecVertex::corner([6.0, 6.0])],
        closed: false,
        ..VecPath::default()
    });
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let e = Entity::from_bits(map[&open]);
    sim.world_mut()
        .entity_mut(e)
        .remove::<ph2d_ecs::RootOrder>()
        .insert(ph2d_ecs::ChildOf(g));

    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);
    assert!(
        !live.contains_key(&open),
        "a linha aberta entrou no mapa — ela seria suprimida ou consumida"
    );
}

/// **Um grupo com menos de duas regiões fechadas não faz nada.**
#[test]
fn fewer_than_two_closed_operands_is_a_no_op() {
    let (mut sim, mut scene, mut map, ids, _g) = scene_with_group(0);
    // Abre um dos dois: sobra um operando só.
    scene.path_mut(ids[1]).unwrap().closed = false;
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);
    assert!(live.is_empty(), "um operando não é uma booleana");
}

/// **O memo esquece um grupo que deixou de ser booleano.**
///
/// ⚠️ Sem o `retain` a resposta velha ficaria guardada e seria re-servida se o grupo voltasse a
/// ser booleano com OUTRA geometria — a falha é silenciosa e desenha a forma de antes.
#[test]
fn the_memo_forgets_a_group_that_stopped_being_boolean() {
    let (mut sim, scene, map, _ids, g) = scene_with_group(0);
    let mut live = LiveGeometry::new();
    let mut bl = run(&sim, &scene, &map, &mut live);
    assert_eq!(bl.memo.len(), 1);

    sim.world_mut().entity_mut(g).remove::<VecBoolGroup>();
    let mut live2 = LiveGeometry::new();
    bl.recook(&scene, &sim, &map, &VecXforms::default(), &mut live2);
    assert!(
        bl.memo.is_empty(),
        "o memo guardou um grupo que não combina"
    );
    assert!(live2.is_empty());
}

/// **Um código de operação que este build não conhece degrada para GRUPO COMUM.**
///
/// A alternativa — desenhar nada — apagaria a arte de um projeto salvo por um build mais novo.
#[test]
fn an_unknown_op_code_draws_the_operands() {
    let (sim, scene, map, _ids, _g) = scene_with_group(200);
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);
    assert!(
        live.is_empty(),
        "código desconhecido não pode suprimir nada"
    );
}

/// **A tradução código↔operação é uma bijeção.** As duas metades moram lado a lado justamente
/// para que ninguém reescreva uma delas com os índices trocados.
#[test]
fn the_op_code_round_trips() {
    use ph2d_vec_boolean::PathfinderOp as P;
    for op in [
        P::Union,
        P::Subtract,
        P::Intersect,
        P::Exclude,
        P::MinusBack,
        P::Trim,
        P::Crop,
        P::Merge,
    ] {
        assert_eq!(op_of_code(code_of_op(op)), Some(op), "{op:?}");
    }
    assert_eq!(op_of_code(8), None, "8 ainda não existe");
}

/// **Aninhamento: o grupo de DENTRO cozinha antes, e o de fora consome o resultado dele.**
///
/// ⚠️ É o único gate que a ordem por profundidade pode falhar, e ela falha em silêncio: sem ela o
/// de fora leria a fonte crua do de dentro e a arte sairia diferente sem nada indicar por quê.
#[test]
fn a_nested_boolean_cooks_from_the_inside_out() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    // Dentro: dois retângulos que se unem numa barra. Fora: essa barra ∩ um terceiro.
    //
    // ⚠️ Eles se SOBREPÕEM (não apenas encostam): dois retângulos que partilham exatamente a
    // aresta `x=2` são um caso degenerado do sweep, e a primeira versão desta fixture media zero
    // por causa disso — testando a degeneração em vez do aninhamento que o nome promete.
    let a = scene.push_path(rectangle([0.0, 0.0], [2.5, 1.0]));
    let b = scene.push_path(rectangle([1.5, 0.0], [4.0, 1.0]));
    let c = scene.push_path(rectangle([1.0, -1.0], [3.0, 2.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let inner = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&a], map[&b]], "In".into()).unwrap(),
    );
    sim.world_mut()
        .entity_mut(inner)
        .insert(VecBoolGroup { op: 0 }); // Union
    let outer = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[inner.to_bits(), map[&c]], "Out".into())
            .unwrap(),
    );
    sim.world_mut()
        .entity_mut(outer)
        .insert(VecBoolGroup { op: 2 }); // Intersect

    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);

    // A união de `a` e `b` é a barra `[0,4]×[0,1]`; ∩ `c` = `[1,3]×[0,1]`, área 2.
    let got = area_of(live.get(&a).expect("o resultado pousa na base mais funda"));
    assert!(
        (got - 2.0).abs() < 1e-6,
        "a composição mediu {got:.4}, esperado 2.0 — o de fora leu a fonte crua do de dentro?"
    );
}

/// **O resultado veste o estilo que a booleana destrutiva daria** — é o `pathfinder` quem decide,
/// e este gate existe para que ninguém acrescente uma segunda regra de estilo aqui.
#[test]
fn the_result_wears_the_style_the_engine_gives_it() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut ra = rectangle([0.0, 0.0], [2.0, 2.0]);
    ra.fill = Some(Paint::Solid(Rgba8::new(10, 20, 30, 255)));
    let mut rb = rectangle([1.0, 1.0], [3.0, 3.0]);
    rb.fill = Some(Paint::Solid(Rgba8::new(200, 100, 50, 255)));
    let a = scene.push_path(ra);
    let b = scene.push_path(rb);
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let g = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&a], map[&b]], "Bool".into()).unwrap(),
    );
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op: 0 });

    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);

    // O motor tira o estilo do ÚLTIMO argumento (a da frente doa) — a mesma convenção do
    // `apply_many` e do Illustrator.
    let world = [
        {
            let mut p = scene.path(a).unwrap().clone();
            p.id = 0;
            p
        },
        {
            let mut p = scene.path(b).unwrap().clone();
            p.id = 0;
            p
        },
    ];
    let refs: Vec<&VecPath> = world.iter().collect();
    let expect =
        ph2d_vec_boolean::pathfinder(&refs, ph2d_vec_boolean::PathfinderOp::Union).unwrap();
    assert_eq!(
        live.get(&a).unwrap()[0].fill,
        expect[0].fill,
        "o estilo tem de ser o que o motor decide"
    );
}

/// Área absoluta de uma lista de caminhos — o oráculo de "quanta forma saiu".
fn area_of(items: &[VecPath]) -> f64 {
    items.iter().map(|p| ph2d_vec_boolean::area(p).abs()).sum()
}

/// **Quanto custa um frame de booleana viva** — o número que decide se ela é animável.
///
/// Rodar: `cargo test -p ph2d-host-desktop --bins measure_a_live_boolean_frame --release
/// -- --ignored --nocapture`
///
/// ⚠️ O oráculo é o `recook` INTEIRO (a caminhada da árvore, o assamento em mundo, o motor e o
/// mapa), não o `pathfinder` isolado: o que o artista paga por frame é a porta do produto.
#[test]
#[ignore = "sonda de custo — roda sob demanda"]
fn measure_a_live_boolean_frame() {
    use std::time::Instant;
    println!("\n--- custo de UM frame de booleana viva (o `recook` inteiro) ---");
    for (name, op) in [("Union", 0u8), ("Subtract", 1), ("Intersect", 2)] {
        for (shape, n) in [("par simples", 2usize), ("dez operandos", 10)] {
            let mut sim = SimWorld::default();
            let mut scene = VecScene::new();
            let mut map = VecEntityMap::new();
            let mut ids = Vec::new();
            for i in 0..n {
                let x = i as f64 * 0.7;
                ids.push(scene.push_path(ph2d_vec_scene::ellipse([x, 0.0], 1.0, 1.0)));
            }
            crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
            let members: Vec<u64> = ids.iter().map(|i| map[i]).collect();
            let g = Entity::from_bits(
                crate::vec_entities::group_entities(&mut sim, &members, "B".into()).unwrap(),
            );
            sim.world_mut().entity_mut(g).insert(VecBoolGroup { op });

            let mut bl = BoolLive::default();
            let xf = VecXforms::default();
            // Uma corrida a frio, e depois a MEDIÇÃO com o memo INVALIDADO a cada volta — é o
            // caso do arrasto, que é o único em que o custo importa. Um memo quente mede zero.
            let mut live = LiveGeometry::new();
            bl.recook(&scene, &sim, &map, &xf, &mut live);
            let t = Instant::now();
            const N: u32 = 20;
            for k in 0..N {
                // Move um operando: invalida o memo, como um arrasto faz.
                let dx = f64::from(k) * 1e-4;
                for v in &mut scene.path_mut(ids[0]).unwrap().verts {
                    v.anchor[0] += dx;
                }
                let mut live = LiveGeometry::new();
                bl.recook(&scene, &sim, &map, &xf, &mut live);
            }
            let ms = t.elapsed().as_secs_f64() * 1000.0 / f64::from(N);
            println!("  {op:>1} {name:<10} | {shape:<14} | {ms:>7.3} ms/frame");
        }
    }
    println!("\nOrcamento de um quadro a 60 fps: 16,6 ms.");
}

// ============================================================================
// **UM VERBO POR FORMA** (Enio, 2026-08-22) — os gates da cadeia dentro do grupo.
//
// O modelo: *as formas combinam-se na ordem da hierarquia, e cada uma traz o verbo com que dobra
// sobre o resultado das anteriores.* É o compound shape vivo do Illustrator.
//
// ⚠️ Estes gates vêm em pares CAPACIDADE/HERANÇA de propósito. A capacidade sozinha passaria com
// uma implementação que ignorasse o `op` do grupo; a herança sozinha passaria com uma que
// ignorasse o override. É a existência das duas que prende o desenho.
// ============================================================================

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
    bl.recook(&scene, &sim, &map, &xf, &mut live);
    let before = area_of(live.get(&ids[0]).unwrap());

    set_verb(&mut sim, &map, ids[2], 1); // Subtract
    // ⚠️ O MESMO `BoolLive` — um novo teria memo vazio e o gate seria verde sem provar nada.
    let mut live2 = LiveGeometry::new();
    bl.recook(&scene, &sim, &map, &xf, &mut live2);
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

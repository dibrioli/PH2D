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

/// Como [`scene_with_group`], mas com TRÊS retângulos em escada — o mínimo em que *"soma com uma e
/// subtrai de outra"* é dizível.
fn scene_with_three(op: u8) -> (SimWorld, VecScene, VecEntityMap, Vec<VecPathId>, Entity) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [2.0, 2.0]));
    let b = scene.push_path(rectangle([1.0, 0.0], [3.0, 2.0]));
    let c = scene.push_path(rectangle([2.0, 0.0], [4.0, 2.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let g = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&a], map[&b], map[&c]], "Bool".into())
            .unwrap(),
    );
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op });
    (sim, scene, map, vec![a, b, c], g)
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

/// O mapa inteiro, comparável — o que a tela mostra, e não uma entrada dele.
fn snapshot(live: &LiveGeometry, ids: &[VecPathId]) -> Vec<Option<Vec<VecPath>>> {
    ids.iter().map(|id| live.get(id).cloned()).collect()
}

/// **A ESTRELA MATERIALIZADA NO COMPONENTE NÃO MOVE A ARTE.**
///
/// O gate irmão de `a_estrela_derivada_desenha_o_que_o_grupo_de_hoje_desenha` (que prova a
/// igualdade no MOTOR); este prova que a **costura** da shell honra a mesma igualdade — o mesmo
/// mapa, para os mesmos ids, com e sem grafo.
///
/// ⚠️ É a licença da etapa 2: abrir a janela do diagrama sobre um grupo que já existe escreve as
/// ligações, e nada na tela pode mudar por causa disso. Sem esta prova, a feature de VISUALIZAR
/// alteraria o que se está a visualizar.
#[test]
fn a_estrela_materializada_no_componente_nao_move_a_arte() {
    for op in 0u8..4 {
        let (mut sim, scene, map, ids, g) = scene_with_three(op);
        let mut sem = LiveGeometry::new();
        run(&sim, &scene, &map, &mut sem);

        let edges = ph2d_vec_boolean::derive_star(&ids, super::op_of_code(op).unwrap());
        sim.world_mut().entity_mut(g).insert(VecBoolEdges::new(
            edges
                .iter()
                .map(|e| VecBoolEdge {
                    from: e.from,
                    to: e.to,
                    op: super::code_of_op(e.op),
                })
                .collect(),
        ));
        let mut com = LiveGeometry::new();
        run(&sim, &scene, &map, &mut com);

        assert_eq!(
            snapshot(&com, &ids),
            snapshot(&sem, &ids),
            "op {op}: materializar a estrela mudou o desenho"
        );
    }
}

/// **A MESMA FORMA SOMA COM UMA VIZINHA E SUBTRAI DE OUTRA** — o pedido do Enio, pela costura real.
///
/// Escada de três retângulos `2×2` em `x = 0/1/2`. `b` soma com `a` e é subtraída de `c`:
/// - `a ∪ b` = `[0,3] × [0,2]` ⇒ área 6.
/// - `c − b` = `[3,4] × [0,2]` ⇒ área 2.
///
/// ⚠️ A operação do GRUPO fica em `Merge` de propósito — uma receita que nem sequer é dizível numa
/// ligação. Se a shell caísse no caminho antigo, o resultado seria outro, e o gate diria.
#[test]
fn o_grafo_liga_a_mesma_forma_a_duas_com_operacoes_diferentes() {
    let (mut sim, scene, map, ids, g) = scene_with_three(7); // Merge
    sim.world_mut().entity_mut(g).insert(VecBoolEdges::new(vec![
        VecBoolEdge {
            from: ids[1],
            to: ids[0],
            op: 0,
        }, // Union
        VecBoolEdge {
            from: ids[1],
            to: ids[2],
            op: 1,
        }, // Subtract
    ]));
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);

    let uniao = area_of(live.get(&ids[0]).expect("o sumidouro a desenha"));
    assert!((uniao - 6.0).abs() < 1e-6, "a ∪ b deu {uniao}, esperado 6");
    let resto = area_of(live.get(&ids[2]).expect("o sumidouro c desenha"));
    assert!((resto - 2.0).abs() < 1e-6, "c − b deu {resto}, esperado 2");
    assert_eq!(
        live.get(&ids[1]).map(Vec::len),
        Some(0),
        "b foi consumido pelas duas ligações e tem de desenhar NADA"
    );
}

/// **O MEMO VÊ O GRAFO** — trocar só a operação de uma ligação re-cozinha.
///
/// ⚠️ Sem o grafo na chave, a geometria não mudou e o memo daria acerto: a resposta velha ficaria
/// na tela até que o artista MEXESSE numa das formas. É o modo de falha que ninguém atribui ao
/// cache, porque tudo o que se vê é *"o app ignorou o meu clique"*.
#[test]
fn o_memo_ve_o_grafo_e_trocar_so_a_ligacao_re_cozinha() {
    let (mut sim, scene, map, ids, g) = scene_with_three(0);
    let liga = |op: u8| {
        VecBoolEdges::new(vec![
            VecBoolEdge {
                from: ids[1],
                to: ids[0],
                op,
            },
            VecBoolEdge {
                from: ids[2],
                to: ids[0],
                op,
            },
        ])
    };
    sim.world_mut().entity_mut(g).insert(liga(0)); // Union
    let mut live = LiveGeometry::new();
    let mut bl = BoolLive::default();
    bl.recook(&scene, &sim, &map, &VecXforms::default(), &mut live);
    let uniao = area_of(live.get(&ids[0]).unwrap());

    sim.world_mut().entity_mut(g).insert(liga(2)); // Intersect
    // ⚠️ O MESMO `BoolLive`, de propósito: um objeto novo não teria memo, e o gate passaria sem
    // provar nada. E um mapa NOVO, também de propósito — o `render_loop` reconstrói o mapa a cada
    // frame, e reusá-lo aqui faria os operandos entrarem com o RESULTADO do frame anterior.
    let mut live2 = LiveGeometry::new();
    bl.recook(&scene, &sim, &map, &VecXforms::default(), &mut live2);
    let inter = area_of(live2.get(&ids[0]).unwrap());

    assert!(
        inter < uniao * 0.5,
        "a interseção ({inter:.3}) não é bem menor que a união ({uniao:.3}) -- o memo não viu a ligação mudar"
    );
}

/// **UMA LIGAÇÃO ÓRFÃ NÃO APAGA A BOOLEANA DO GRUPO.**
///
/// O resolvedor recusa o grafo inteiro quando uma ligação nomeia um nó ausente — e é a lei certa
/// lá dentro. Aqui a shell filtra pelos operandos vivos ANTES de perguntar, senão apagar uma forma
/// qualquer da cena apagaria a booleana de um grupo que o artista nem tocou.
#[test]
fn uma_ligacao_orfa_nao_apaga_a_booleana_do_grupo() {
    let (mut sim, scene, map, ids, g) = scene_with_three(0);
    sim.world_mut().entity_mut(g).insert(VecBoolEdges::new(vec![
        VecBoolEdge {
            from: ids[1],
            to: ids[0],
            op: 0,
        },
        VecBoolEdge {
            from: 9_999,
            to: ids[0],
            op: 0,
        }, // uma forma que não existe
    ]));
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);

    let uniao = area_of(live.get(&ids[0]).expect("a base continua a desenhar"));
    assert!((uniao - 6.0).abs() < 1e-6, "a ∪ b deu {uniao}, esperado 6");
    assert_eq!(
        live.get(&ids[2]).map(Vec::len),
        Some(1),
        "c ficou solto e tem de se desenhar a si próprio -- uma peça"
    );
}

/// **UM GRAFO VAZIO NÃO REINSTALA A OPERAÇÃO DO GRUPO** — cortar o último elo SEPARA as formas.
///
/// ⚠️ É a distinção entre *lista vazia* e *componente ausente*, e ela é load-bearing pelo gesto: no
/// diagrama, cortar a última ligação tem de deixar as formas soltas. Se a lista vazia caísse de
/// volta na operação única, cortar o último elo faria as formas **fundirem-se** — o oposto exato do
/// que o artista acabou de fazer, e um bug que nenhum teste de geometria apanha, porque a geometria
/// está certa: é a LEI que está errada.
#[test]
fn um_grafo_vazio_nao_reinstala_a_operacao_do_grupo() {
    let (mut sim, scene, map, ids, g) = scene_with_three(0); // Union
    sim.world_mut()
        .entity_mut(g)
        .insert(VecBoolEdges::default());
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);

    for (k, id) in ids.iter().enumerate() {
        assert_eq!(
            live.get(id).map(Vec::len),
            Some(1),
            "a forma {k} tem de se desenhar a si própria -- a união voltou pela porta dos fundos"
        );
    }
    let total: f64 = ids.iter().map(|id| area_of(live.get(id).unwrap())).sum();
    assert!(
        (total - 12.0).abs() < 1e-6,
        "as três formas somam {total}, esperado 12 (a união daria 8)"
    );
}

/// **UM CICLO DEIXA A ARTE COMO ESTAVA** — a recusa não pisca a tela.
///
/// ⚠️ O mapa fica **intocado**, e não vazio: vazio significaria *"desenhe nada"*, e a forma
/// desapareceria enquanto o artista arrasta a ligação que fechou o ciclo.
#[test]
fn um_ciclo_deixa_a_arte_exatamente_como_estava() {
    let (mut sim, scene, map, ids, g) = scene_with_three(0);
    let mut live = LiveGeometry::new();
    let mut bl = BoolLive::default();
    bl.recook(&scene, &sim, &map, &VecXforms::default(), &mut live);
    let antes = snapshot(&live, &ids);

    sim.world_mut().entity_mut(g).insert(VecBoolEdges::new(vec![
        VecBoolEdge {
            from: ids[0],
            to: ids[1],
            op: 0,
        },
        VecBoolEdge {
            from: ids[1],
            to: ids[0],
            op: 0,
        },
    ]));
    bl.recook(&scene, &sim, &map, &VecXforms::default(), &mut live);

    assert_eq!(snapshot(&live, &ids), antes, "o ciclo mexeu na arte");
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
            for (via, graph) in [("grupo", false), ("grafo", true)] {
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
                // ⚠️ A ESTRELA, para a mesma cena — o que se quer saber não é o custo do grafo em
                // abstrato, é se ele custa MAIS que o grupo que ele substitui. A estrela derivada
                // desenha o mesmo (há gate), então a diferença de relógio é só do caminho.
                if graph {
                    let edges = ph2d_vec_boolean::derive_star(&ids, super::op_of_code(op).unwrap());
                    sim.world_mut().entity_mut(g).insert(VecBoolEdges::new(
                        edges
                            .iter()
                            .map(|e| VecBoolEdge {
                                from: e.from,
                                to: e.to,
                                op: super::code_of_op(e.op),
                            })
                            .collect(),
                    ));
                }

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
                println!("  {op:>1} {name:<10} | {shape:<14} | {via:<5} | {ms:>7.3} ms/frame");
            }
        }
    }
    println!("\nOrcamento de um quadro a 60 fps: 16,6 ms.");
}

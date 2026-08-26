//! Os gates do CONJUNTO de estados (plano 32 W8) — a **lei** do grafo, e a **costura** que a
//! aplica ao mundo.

use super::{create, eligible, states_for, upkeep};
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform, VecMorph, VecMorphMachine, Visibility};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

use crate::vec_entities::{VecEntityMap, sync};

/// Três formas soltas, com nome, já sincronizadas com o mundo.
fn world(n: usize) -> (SimWorld, VecScene, VecEntityMap, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let ids: Vec<VecPathId> = (0..n)
        .map(|_| scene.push_path(VecPath::default()))
        .collect();
    sync(&mut sim, &mut scene, &mut map);
    for (i, id) in ids.iter().enumerate() {
        let e = Entity::from_bits(map[id]);
        sim.world_mut()
            .entity_mut(e)
            .insert(Name::new(format!("S{i}")));
    }
    (sim, scene, map, ids)
}

/// ⭐⭐⭐ **UMA ENTRADA POR FORMA, e a tecla pertence ao DESTINO** — a lei da W10.
///
/// Enio, 2026-08-25: *"em vez de um evento para cada transição, melhor seria um evento por shape
/// (…) assim reduzimos o número de transições no painel para o número de formas envolvidas."*
///
/// **Mutação que deve sangrar:** o `states_for` voltar a emitir um par ordenado por passagem — a
/// lista cresce para `n(n-1)` e o artista escreve a mesma tecla `n-1` vezes.
#[test]
fn the_list_has_one_entry_per_shape_and_nothing_more() {
    for n in 2..=6usize {
        let shapes: Vec<VecPathId> = (1..=n as u64).collect();
        let g = states_for(&shapes);
        assert_eq!(
            g.states.len(),
            n,
            "com {n} formas a lista tem {n} entradas -- uma por forma, nao uma por passagem"
        );
        assert_eq!(
            g.shapes(),
            shapes,
            "e na ordem em que o artista as escolheu"
        );
    }
    // ⭐ O CONTROLE que dá o número: com 9 formas (o tecto), a lista tem **9** linhas e nao 72.
    assert_eq!(states_for(&(1..=9).collect::<Vec<u64>>()).states.len(), 9);
}

/// **A primeira forma escolhida é onde a máquina nasce — e o `start` é DERIVADO.**
///
/// **Mutação que deve sangrar:** o `start()` devolver `states.last()` — o conjunto nasceria a
/// mostrar a última forma escolhida, e o artista que escolheu da esquerda para a direita veria a
/// da direita.
#[test]
fn the_first_shape_chosen_is_the_start() {
    let g = states_for(&[7, 3, 9]);
    assert_eq!(g.start(), Some(7));
    // O CONTROLE: sem formas nenhumas nao ha' `start` inventado nem panico.
    assert_eq!(states_for(&[]).start(), None);
}

/// ⭐ **A ORDEM da lista é determinística** — o painel indexa por POSIÇÃO.
///
/// ⚠️ Sem isto o menu «When» da linha 3 escreveria a tecla noutra forma depois de um undo, e o
/// artista não teria como saber. É a mesma razão do `BTreeMap` da física.
#[test]
fn the_state_order_is_stable_so_a_row_index_means_one_thing() {
    assert_eq!(states_for(&[10, 20, 30]).shapes(), vec![10, 20, 30]);
    assert_ne!(
        states_for(&[10, 20, 30]).shapes(),
        states_for(&[30, 20, 10]).shapes(),
        "a ordem e' a dos MEMBROS, nunca a dos ids"
    );
}

/// ⭐ **Toda forma nasce SEM tecla** — alcançável só pela pré-visualização.
///
/// ⚠️ É a metade que torna a lista segura: se cada forma nascesse com uma acção, um conjunto de 9
/// nasceria com 9 regras a disparar todas na primeira tecla.
#[test]
fn every_state_is_born_silent() {
    let g = states_for(&[1, 2, 3]);
    assert!(
        g.states.iter().all(|s| s.when.is_empty()),
        "uma forma com tecla de fabrica seria alcancada sem ninguem a ter pedido"
    );
    assert!(
        g.reached_by("jump").is_none(),
        "e nenhuma acao pode alcancar coisa nenhuma"
    );
}

/// ⛔ **Uma forma que JÁ é um Morph nunca vira estado.**
///
/// ⚠️ Um conjunto sobre um conjunto daria uma máquina cujos estados são re-escritos por baixo dela
/// a cada quadro (o `recook` do morph interior).
#[test]
fn a_morph_is_never_a_state_of_another_set() {
    let (mut sim, _scene, map, ids) = world(3);
    let e = Entity::from_bits(map[&ids[1]]);
    sim.world_mut().entity_mut(e).insert(VecMorph::new(1, 2));
    let ok = eligible(&sim, &map, &ids);
    assert_eq!(ok, vec![ids[0], ids[2]], "o morph do meio tem de sair");
    // O CONTROLE: sem morph nenhum, as três passam.
    let (sim2, _s2, map2, ids2) = world(3);
    assert_eq!(eligible(&sim2, &map2, &ids2), ids2);
}

/// ⭐⭐ **A COSTURA: o botão faz o objecto, ele fica com os filhos, e SÓ ELE aparece.**
///
/// **Mutação que deve sangrar (1):** tirar o `Visibility::hidden()` — as nove formas ficariam
/// empilhadas por cima do conjunto, que é a foto que o Enio nunca quer ver.
///
/// **Mutação que deve sangrar (2):** esconder o HOST em vez dos membros — o `visible_chain` lê os
/// ancestrais, então o conjunto inteiro sumiria do canvas.
///
/// **Mutação que deve sangrar (3):** `sources: [start, start]` virar `VecMorph::new(a, b)` (que
/// nasce em `t = 0,5`) — a primeira coisa na tela seria uma forma a meio caminho, que ninguém
/// desenhou.
#[test]
fn the_set_owns_the_shapes_hides_them_and_shows_the_start() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    assert!(pending.is_some(), "tres formas dao um conjunto");
    // O `sync` do quadro seguinte é que faz nascer a entidade do path novo.
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    assert!(pending.is_none(), "o pendente tem de ser consumido");

    let host = Entity::from_bits(map[&scene.paths().last().unwrap().id]);
    let m = sim
        .world()
        .get::<VecMorph>(host)
        .expect("o conjunto DESENHA");
    assert_eq!(
        (m.sources, m.t),
        ([ids[0], ids[0]], 0.0),
        "ele tem de mostrar EXACTAMENTE o estado inicial"
    );
    assert_eq!(
        sim.world()
            .get::<VecMorphMachine>(host)
            .expect("e tem maquina")
            .graph
            .shapes(),
        ids,
        "tres formas => TRES estados, na ordem em que foram escolhidas (W10)"
    );
    assert!(
        sim.world().get::<Visibility>(host).is_none(),
        "o CONTROLE: o conjunto NAO se esconde -- e' ele que se ve'"
    );
    for id in &ids {
        let e = Entity::from_bits(map[id]);
        assert_eq!(
            sim.world().get::<ChildOf>(e).map(ChildOf::parent),
            Some(host),
            "a forma {id} tem de ser FILHA do conjunto"
        );
        assert!(
            sim.world().get::<Visibility>(e).is_some_and(|v| v.hidden),
            "a forma {id} tem de ficar oculta -- so' o estado actual aparece"
        );
    }
}

/// **Uma forma só, ou formas a mais, RECUSAM** — e a recusa não põe lixo na cena.
///
/// ⚠️ A metade da cena é a que importa: um `push_path` antes da checagem deixaria um path órfão por
/// cada clique recusado, e eles acumulam sem nada na tela a dizê-lo.
#[test]
fn one_shape_or_too_many_refuses_without_littering_the_scene() {
    let (sim, mut scene, map, ids) = world(3);
    let before = scene.paths().len();
    assert!(create(&sim, &mut scene, &map, &ids[..1], 9).is_none());
    assert!(
        create(&sim, &mut scene, &map, &ids, 2).is_none(),
        "tres > 2"
    );
    assert_eq!(
        scene.paths().len(),
        before,
        "uma recusa nao pode deixar um path orfao na cena"
    );
    // O CONTROLE POSITIVO: dentro do tecto, ela aceita e o path nasce.
    assert!(create(&sim, &mut scene, &map, &ids, 3).is_some());
    assert_eq!(scene.paths().len(), before + 1);
}

/// ⭐⭐ **A PERGUNTA QUE OS OUTROS GATES NÃO FAZEM: o conjunto de facto DESENHA alguma coisa?**
///
/// ⚠️ Os gates acima medem os **componentes** — que o `VecMorph` existe, com que fontes e que `t`.
/// Nenhum deles mede o que sai no canvas, e a lei do repo é essa: *contar o trabalho FEITO não é
/// contar o trabalho ENTREGUE*. Aqui o par é **degenerado** (`[start, start]`), e um plano de
/// correspondência que recusasse um par igual deixaria o path **vazio**: o artista carregaria no
/// botão, as três formas desapareciam (ficam ocultas) e **nada** apareceria no lugar delas.
///
/// **Mutação que deve sangrar:** o `recook` deixar de correr sobre o par degenerado (por exemplo,
/// um `if a == b { continue }` no `morph_live`) — o conjunto nasceria invisível.
#[test]
fn the_new_set_actually_draws_the_start_shape() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    // Duas formas BEM diferentes, para o `at(0)` não poder passar por acaso.
    let a = scene.push_path(ph2d_vec_scene::rectangle([-2.0, -0.5], [2.0, 0.5]));
    let b = scene.push_path(ph2d_vec_scene::rectangle([-0.4, -2.0], [0.4, 2.0]));
    sync(&mut sim, &mut scene, &mut map);
    let want = scene
        .paths()
        .iter()
        .find(|p| p.id == a)
        .unwrap()
        .verts
        .clone();
    assert!(
        !want.is_empty(),
        "o CONTROLE: a forma de partida tem geometria"
    );

    let mut pending = create(&sim, &mut scene, &map, &[a, b], 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);

    let host_id = scene.paths().last().unwrap().id;
    let xf = crate::vec_transform::build(&sim, &map);
    let mut plans = crate::morph_live::MorphPlans::new();
    crate::morph_live::recook(&mut sim, &mut scene, &map, &xf, &mut plans);

    let drawn = &scene
        .paths()
        .iter()
        .find(|p| p.id == host_id)
        .unwrap()
        .verts;
    assert!(
        !drawn.is_empty(),
        "⛔ o conjunto nasceu INVISIVEL: as formas ficam ocultas e nada aparece no lugar delas"
    );
    // ⭐ E o que ele desenha é a forma de PARTIDA, não uma mistura: o par é `[start, start]`.
    assert_eq!(
        drawn.len(),
        want.len(),
        "o conjunto tem de desenhar a forma inicial, e nao uma mistura dela com a outra"
    );
}

/// ⭐⭐ **TODO ESTADO FICA NO MESMO SÍTIO — o morph acontece EM LUGAR, não viaja.**
///
/// Enio, 2026-08-25: *"aqui, diferente da tool morph, todas as peças participantes são alinhadas
/// numa mesma posição e o morph states faz o morph numa mesma posição, não desloca a peça de
/// lugar."*
///
/// **Mutação que deve sangrar:** o `align` devolver `Vec2::ZERO` sem escrever nada — as formas
/// ficariam onde o artista as desenhou (lado a lado) e a transição atravessaria o ecrã.
#[test]
fn every_state_is_centred_on_the_set_so_the_morph_never_travels() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    // Três formas LADO A LADO e de tamanhos diferentes — a fixtura tem de conter o fenómeno.
    let ids: Vec<VecPathId> = [
        ph2d_vec_scene::rectangle([-9.0, -1.0], [-7.0, 1.0]),
        ph2d_vec_scene::rectangle([-0.5, -2.0], [0.5, 2.0]),
        ph2d_vec_scene::rectangle([7.0, -0.2], [9.0, 0.2]),
    ]
    .into_iter()
    .map(|p| scene.push_path(p))
    .collect();
    sync(&mut sim, &mut scene, &mut map);
    // O CONTROLE: antes, os centros são BEM diferentes.
    let before = centres(&sim, &scene, &map, &ids);
    assert!(
        (before[0][0] - before[2][0]).abs() > 10.0,
        "a fixtura tem de comecar com as formas LONGE umas das outras"
    );

    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);

    let host = Entity::from_bits(map[&scene.paths().last().unwrap().id]);
    let origin = crate::vec_transform::world_transform(&sim, host).translation;
    for (i, c) in centres(&sim, &scene, &map, &ids).into_iter().enumerate() {
        assert!(
            (c[0] - f64::from(origin.x)).abs() < 1e-4 && (c[1] - f64::from(origin.y)).abs() < 1e-4,
            "o estado {i} ficou em {c:?} e a origem do conjunto e' ({}, {}) -- o morph ia VIAJAR",
            origin.x,
            origin.y
        );
    }
}

/// ⚠️ **Alinhar move a POSIÇÃO e mais nada** — rotação, escala e cisalhamento sobrevivem.
///
/// **Mutação que deve sangrar:** o `align` escrever `Transform::from_translation(..)` em vez de
/// `..world` — a forma girada endireitava-se ao entrar no conjunto, e o artista veria o desenho
/// dele mudar por ter carregado num botão.
#[test]
fn aligning_moves_the_position_and_nothing_else() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let ids: Vec<VecPathId> = (0..2)
        .map(|i| {
            let x = f64::from(i) * 6.0;
            scene.push_path(ph2d_vec_scene::rectangle([x - 1.0, -1.0], [x + 1.0, 1.0]))
        })
        .collect();
    sync(&mut sim, &mut scene, &mut map);
    // Uma pose com TUDO mexido, para nenhum campo passar por acaso.
    let posed = Transform {
        translation: ph2d_core::Vec2::new(3.0, -2.0),
        rotation: 0.7,
        scale: ph2d_core::Vec2::new(1.5, 0.5),
        skew_x: 0.2,
        skew_y: -0.1,
    };
    let e0 = Entity::from_bits(map[&ids[0]]);
    sim.world_mut().entity_mut(e0).insert(posed);

    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);

    let after = *sim.world().get::<Transform>(e0).expect("a pose fica");
    assert_eq!(
        (after.rotation, after.scale, after.skew_x, after.skew_y),
        (posed.rotation, posed.scale, posed.skew_x, posed.skew_y),
        "alinhar destruiu a FORMA do estado, e nao so' a posicao dele"
    );
    assert_ne!(
        after.translation, posed.translation,
        "o CONTROLE: a posicao TEM de ter mudado, senao o gate acima nao mede nada"
    );
}

/// ⭐⭐⭐ **ARRASTAR O CONJUNTO LEVA OS ESTADOS JUNTO — e o desenho vai com ele.**
///
/// Enio, 2026-08-25: *"o objeto criado como pai tem que ser arrastável no canvas como um objeto
/// qualquer (…) e deve arrastar os filhos junto."*
///
/// **Mutação que deve sangrar (1):** tirar o `!is_set` da guarda do `recook` — a pose do conjunto é
/// zerada no quadro seguinte ao arrasto, e a forma **volta ao sítio** debaixo do dedo. É, palavra
/// por palavra, o mecanismo que torna um Morph comum não-arrastável.
///
/// **Mutação que deve sangrar (2):** o `recook` voltar a cozer um conjunto em MUNDO (`is_set` a
/// escolher `world`) — o afim entraria **duas** vezes e a forma andaria o dobro do arrasto.
#[test]
fn dragging_the_set_carries_the_states_and_the_drawing() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let ids: Vec<VecPathId> = [
        ph2d_vec_scene::rectangle([-4.0, -1.0], [-2.0, 1.0]),
        ph2d_vec_scene::rectangle([2.0, -2.0], [4.0, 2.0]),
    ]
    .into_iter()
    .map(|p| scene.push_path(p))
    .collect();
    sync(&mut sim, &mut scene, &mut map);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host_id = scene.paths().last().unwrap().id;
    let host = Entity::from_bits(map[&host_id]);

    let mut plans = crate::morph_live::MorphPlans::new();
    let cook =
        |sim: &mut SimWorld, scene: &mut VecScene, plans: &mut crate::morph_live::MorphPlans| {
            let xf = crate::vec_transform::build(sim, &map);
            crate::morph_live::recook(sim, scene, &map, &xf, plans);
        };
    cook(&mut sim, &mut scene, &mut plans);
    let drawn0 = scene
        .paths()
        .iter()
        .find(|p| p.id == host_id)
        .unwrap()
        .verts
        .clone();
    let kid0 =
        crate::vec_transform::world_transform(&sim, Entity::from_bits(map[&ids[0]])).translation;
    assert!(
        !drawn0.is_empty(),
        "o CONTROLE: o conjunto desenha alguma coisa"
    );

    // O ARRASTO: o gizmo escreve o `Transform`, como em qualquer forma.
    let pose = *sim
        .world()
        .get::<Transform>(host)
        .expect("o conjunto tem pose");
    let delta = ph2d_core::Vec2::new(5.0, -7.0);
    sim.world_mut().entity_mut(host).insert(Transform {
        translation: pose.translation + delta,
        ..pose
    });
    cook(&mut sim, &mut scene, &mut plans);

    assert_eq!(
        sim.world().get::<Transform>(host).map(|t| t.translation),
        Some(pose.translation + delta),
        "⛔ o `recook` ZEROU a pose do conjunto -- a forma volta ao sitio debaixo do dedo"
    );
    // ⭐ A geometria guardada é LOCAL, então ela **não muda** com o arrasto — é o `Transform` que a
    // leva ao mundo, exactamente como em qualquer forma do documento.
    assert_eq!(
        &scene
            .paths()
            .iter()
            .find(|p| p.id == host_id)
            .unwrap()
            .verts,
        &drawn0,
        "a geometria guardada mudou com o arrasto -- ela devia ser LOCAL ao conjunto"
    );
    // ⭐⭐ E os ESTADOS foram junto: o filho anda exactamente o delta.
    let kid1 =
        crate::vec_transform::world_transform(&sim, Entity::from_bits(map[&ids[0]])).translation;
    assert!(
        (kid1.x - kid0.x - delta.x).abs() < 1e-4 && (kid1.y - kid0.y - delta.y).abs() < 1e-4,
        "o estado nao acompanhou o conjunto: andou ({}, {}) e o conjunto andou ({}, {})",
        kid1.x - kid0.x,
        kid1.y - kid0.y,
        delta.x,
        delta.y
    );
}

/// O centro de MUNDO do que cada forma desenha.
fn centres(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    ids: &[VecPathId],
) -> Vec<[f64; 2]> {
    ids.iter()
        .map(|&id| {
            let e = Entity::from_bits(map[&id]);
            let x = crate::vec_transform::xform_of_transform(
                crate::vec_transform::world_transform(sim, e),
            );
            let (bl, tr) = scene.path_curve_bbox(id).expect("a forma tem caixa");
            let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
            for c in [
                [bl[0], bl[1]],
                [tr[0], bl[1]],
                [tr[0], tr[1]],
                [bl[0], tr[1]],
            ] {
                let w = x.apply(c);
                for k in 0..2 {
                    lo[k] = lo[k].min(w[k]);
                    hi[k] = hi[k].max(w[k]);
                }
            }
            [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
        })
        .collect()
}

/// ⛔ **CONVERTER UM CONJUNTO EM CURVAS LEVA A MÁQUINA JUNTO.**
///
/// ⚠️ Sem isto ficaria um `VecMorphMachine` **órfão**: a seção continuaria a listar as `n(n-1)`
/// transições, o interruptor `Preview` a acender — e nenhuma tecla mudaria nada, porque já não há
/// morph a dirigir. *Um painel que oferece o que o mundo não pode fazer é pior que um painel
/// vazio.*
///
/// **Mutação que deve sangrar:** tirar o `em.remove::<VecMorphMachine>()` do
/// `vec_convert::drop_relation_hosts`.
#[test]
fn converting_the_set_to_curves_takes_the_machine_with_it() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let ids: Vec<VecPathId> = (0..2)
        .map(|i| {
            let x = f64::from(i) * 5.0;
            scene.push_path(ph2d_vec_scene::rectangle([x, -1.0], [x + 2.0, 1.0]))
        })
        .collect();
    sync(&mut sim, &mut scene, &mut map);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host_id = scene.paths().last().unwrap().id;
    let host = Entity::from_bits(map[&host_id]);
    // O CONTROLE: antes da conversão, as duas coisas existem.
    assert!(sim.world().get::<VecMorph>(host).is_some());
    assert!(sim.world().get::<VecMorphMachine>(host).is_some());

    // ⚠️ Pela porta REAL do produto (`to_curves`), e não pela função privada que ela chama: um
    // caminho próprio aqui provaria que o `remove` funciona, não que o produto o faz.
    let mut pen = ph2d_vec_edit::PenTool::default();
    let mut history = ph2d_vec_edit::History::default();
    let xf = crate::vec_transform::build(&sim, &map);
    crate::vec_convert::to_curves(
        &mut sim,
        &mut scene,
        &mut map,
        &mut pen,
        &mut history,
        &xf,
        &[host_id],
    );

    assert!(
        sim.world().get::<VecMorph>(host).is_none(),
        "o CONTROLE: converter tem de soltar o morph"
    );
    assert!(
        sim.world().get::<VecMorphMachine>(host).is_none(),
        "⛔ a maquina ficou ORFA: a seccao lista transicoes que nenhuma tecla pode disparar"
    );
}

/// ⛔⛔ **UMA FORMA QUE JÁ É ESTADO DE UM CONJUNTO NÃO ENTRA NOUTRO.**
///
/// ⚠️ **É alcançável, e por isso importa:** os membros ficam ocultos, mas a Hierarquia mostra-os e
/// clicar numa linha selecciona-a. Sem esta exclusão a seção ofereceria *"Make Morph States"* sobre
/// formas que já têm dono — e o segundo conjunto governaria formas que o primeiro esconde e move.
///
/// **Mutação que deve sangrar:** tirar a checagem do `ChildOf` + `VecMorphMachine` do `eligible`.
#[test]
fn a_shape_that_already_belongs_to_a_set_is_never_offered_to_another() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);

    assert!(
        eligible(&sim, &map, &ids).is_empty(),
        "as tres formas ja' tem dono -- oferecer um segundo conjunto daria dois donos a cada uma"
    );
    // O CONTROLE: uma forma SOLTA, desenhada depois, continua a ser elegível.
    let solta = scene.push_path(ph2d_vec_scene::rectangle([20.0, 0.0], [21.0, 1.0]));
    sync(&mut sim, &mut scene, &mut map);
    assert_eq!(
        eligible(&sim, &map, &[ids[0], solta]),
        vec![solta],
        "a forma solta tem de continuar elegivel -- senao a exclusao apanhou o mundo inteiro"
    );
}

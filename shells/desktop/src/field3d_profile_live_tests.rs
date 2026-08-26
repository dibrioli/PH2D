//! Os gates do **contorno vivo** — ver [`super`].
//!
//! ⚠️ Todos correm pelo caminho de produção (`sync_scene_and_birth`), e não pela função sozinha: a
//! metade que pode partir aqui é a **costura** — a ordem em que a reconciliação corre em relação ao
//! cozimento, e o vínculo a nascer com a forma. Uma chamada directa à `reconcile` provaria o corpo e
//! deixaria as duas de fora.

use ph2d_ecs::SimWorld;
use ph2d_field::{FieldDoc, NodeId, NodeShape, Primitive, Xform};
use ph2d_field_ecs::{FieldNode, FieldProfileSource};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex};

use crate::field3d_smoke::ProfileShape;

/// Uma peça mínima para a ponte ter raiz — o contorno entra por cima dela.
fn seed() -> FieldDoc {
    FieldDoc::new(
        vec![ph2d_field::Node {
            xform: Xform::IDENTITY,
            kind: ph2d_field::NodeKind::Leaf(Primitive::Box {
                half: [0.2; 3],
                round: 0.0,
            }),
            mods: Vec::new(),
        }],
        NodeId(0),
    )
    .expect("a semente")
}

/// Um quadrado de lado `side`, em quatro quinas — **fechado**, que é o que um perfil exige.
fn square(side: f64) -> VecPath {
    let h = side * 0.5;
    VecPath {
        verts: [[-h, -h], [h, -h], [h, h], [-h, h]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        closed: true,
        ..VecPath::default()
    }
}

/// Uma cena com um contorno, e o mundo com a peça a sair dele.
///
/// Devolve `(mundo, cena, id do contorno, entidade da forma)`.
fn a_drawn_extrusion() -> (SimWorld, VecScene, VecPathId, bevy_ecs::entity::Entity) {
    let _ = ph2d_panel_model3d::drain_intents();
    let mut scene = VecScene::default();
    let id = scene.push_path(square(1.0));
    let mut sim = SimWorld::new();
    // A peça nasce; o contorno é pedido no mesmo quadro, como o botão faz.
    let msg = crate::field3d_profile::from_selection(&scene, &[id], ProfileShape::Extrude);
    assert!(
        msg.starts_with("Extruded"),
        "o contorno tinha de cozer: {msg}"
    );
    crate::field3d_scene::sync_scene_and_birth(&mut sim, Some(&seed()), &[], 0.0, &scene);
    let e = the_profile_node(&mut sim);
    (sim, scene, id, e)
}

/// A entidade que carrega o vínculo — há exactamente uma.
fn the_profile_node(sim: &mut SimWorld) -> bevy_ecs::entity::Entity {
    let world = sim.world_mut();
    let mut q = world.query::<(bevy_ecs::entity::Entity, &FieldProfileSource)>();
    let found: Vec<bevy_ecs::entity::Entity> = q.iter(world).map(|(e, _)| e).collect();
    assert_eq!(found.len(), 1, "uma forma ligada ao desenho: {found:?}");
    found[0]
}

/// Quantas arestas o perfil deste nó tem agora.
fn edges(sim: &mut SimWorld, e: bevy_ecs::entity::Entity) -> usize {
    match sim.world().get::<FieldNode>(e).map(|n| &n.shape) {
        Some(NodeShape::Leaf(
            Primitive::Extrude { profile, .. } | Primitive::Revolve { profile },
        )) => profile.segment_count(),
        other => panic!("o nó tinha de ser uma forma de perfil: {other:?}"),
    }
}

/// Corre mais um quadro da ponte com esta cena.
fn tick(sim: &mut SimWorld, scene: &VecScene, sel: &[bevy_ecs::entity::Entity]) {
    crate::field3d_scene::sync_scene_and_birth(sim, None, sel, 0.0, scene);
}

/// ⭐ **A forma nasce LIGADA ao desenho** — sem isto, tudo o resto desta wave é inalcançável.
///
/// ⚠️ O gate mede o **id**, e não só a presença do componente: o pedido atravessa uma caixa de
/// correio (`ask_spawn_profile`), e um vínculo que nascesse a apontar para o contorno errado seria
/// uma peça a seguir o desenho do vizinho — que é pior do que não seguir nenhum.
#[test]
fn a_drawn_shape_is_born_linked_to_the_outline_it_came_from() {
    let (sim, _scene, id, e) = a_drawn_extrusion();
    let link = sim
        .world()
        .get::<FieldProfileSource>(e)
        .copied()
        .expect("o vínculo");
    assert_eq!(link.path, id, "o vínculo aponta para o contorno desenhado");
    assert_eq!(
        link.level,
        ph2d_field::DEFAULT_PROFILE_RESOLUTION,
        "e nasce no nível de omissão — o joelho que a W54 mediu"
    );
}

/// ⭐⭐ **EDITAR O DESENHO MUDA A PEÇA** — a promessa inteira desta wave, medida na forma.
///
/// ⚠️ A régua é a **caixa** do perfil e não a contagem de arestas: um quadrado com o dobro do lado
/// tem as mesmas quatro arestas, então uma gate que contasse arestas passaria com a reconciliação
/// arrancada. *Uma régua que não vê a mudança que o artista fez não mede a feature.*
#[test]
fn editing_the_outline_reshapes_the_piece() {
    let (mut sim, mut scene, id, e) = a_drawn_extrusion();
    let before = profile_span(&mut sim, e);

    // O artista mexe na curva: o mesmo contorno, com o dobro do tamanho.
    *scene.path_mut(id).expect("o contorno") = VecPath { id, ..square(2.0) };
    tick(&mut sim, &scene, &[e]);

    let after = profile_span(&mut sim, e);
    assert!(
        (after / before - 2.0).abs() < 1.0e-3,
        "a peça tinha de seguir o desenho (esperado 2x): {before} -> {after}"
    );
}

/// A maior dimensão da caixa do perfil deste nó.
fn profile_span(sim: &mut SimWorld, e: bevy_ecs::entity::Entity) -> f32 {
    match sim.world().get::<FieldNode>(e).map(|n| &n.shape) {
        Some(NodeShape::Leaf(
            Primitive::Extrude { profile, .. } | Primitive::Revolve { profile },
        )) => {
            let (lo, hi) = profile.bounds();
            (hi[0] - lo[0]).max(hi[1] - lo[1])
        }
        other => panic!("o nó tinha de ser uma forma de perfil: {other:?}"),
    }
}

/// ⭐⭐ **UM DESENHO QUE NÃO MUDOU NÃO ESCREVE NO NÓ** — e isto não é velocidade, é o undo.
///
/// ⚠️ O retrato do desfazer compara **bytes de componente**, e reescrever o mesmo perfil a cada
/// quadro é a receita conhecida de transformar todo quadro num passo de histórico — o defeito que o
/// `canonicalize()` do shell existe para matar, e que este repositório já pagou uma vez.
///
/// ⚠️ **A régua é a marca de escrita do ECS, não o conteúdo.** Um perfil reescrito com o mesmo valor
/// tem os mesmos bytes, então uma comparação de conteúdo passaria com a guarda arrancada — e é
/// exactamente a guarda que este gate defende.
#[test]
fn an_unchanged_outline_never_writes_the_node() {
    use bevy_ecs::query::Changed;
    let (mut sim, scene, _id, e) = a_drawn_extrusion();
    // Um quadro em branco primeiro: o nascimento escreve, de propósito.
    tick(&mut sim, &scene, &[e]);

    sim.world_mut().clear_trackers();
    tick(&mut sim, &scene, &[e]);
    let world = sim.world_mut();
    let mut q = world.query_filtered::<bevy_ecs::entity::Entity, Changed<FieldNode>>();
    let touched: Vec<bevy_ecs::entity::Entity> = q.iter(world).collect();
    assert!(
        !touched.contains(&e),
        "o mesmo desenho reescreveu a forma — todo quadro viraria um passo de desfazer: {touched:?}"
    );
}

/// ⭐⭐ **A RESOLUÇÃO É UM NÚMERO, e ele muda a peça** — o pedido literal do Enio (W54).
///
/// ⚠️ O gate escreve pela porta do painel (`set_param`) e lê a forma **depois do quadro seguinte**:
/// é a costura inteira — o número muda a *intenção*, e quem a converte é a reconciliação. Escrever e
/// ler o componente provaria só que um `u32` guarda um `u32`.
#[test]
fn the_resolution_level_buys_edges_through_the_panel_door() {
    let (mut sim, mut scene, id, e) = a_drawn_extrusion();
    // ⚠️ Um QUADRADO tem quatro arestas em qualquer nível — o achatamento não parte uma reta. A
    // finura só se vê numa **curva**, e é por isso que este gate desenha uma.
    *scene.path_mut(id).expect("o contorno") = VecPath { id, ..circle(0.5) };
    tick(&mut sim, &scene, &[e]);
    let coarse = edges(&mut sim, e);

    ph2d_field_ecs::set_param(sim.world_mut(), e, ph2d_field::Param::Resolution, 4.0)
        .expect("o nível 4 cabe");
    tick(&mut sim, &scene, &[e]);
    let fine = edges(&mut sim, e);

    assert!(
        fine > coarse,
        "subir a resolução tinha de comprar arestas: {coarse} -> {fine}"
    );
    // E o número tem de ser o que a lei da conversão diz — não «mais alguma».
    let want = ph2d_field_profile::cook_path_at(scene.path(id).expect("o contorno"), 4)
        .expect("coze")
        .segment_count();
    assert_eq!(fine, want, "a forma tem de ser a do nível pedido");
}

/// Um círculo de raio `r` em quatro segmentos de Bézier.
fn circle(r: f64) -> VecPath {
    let k = 0.552_284_75_f64 * r;
    let verts = [(r, 0.0), (0.0, r), (-r, 0.0), (0.0, -r)]
        .into_iter()
        .zip([(0.0, k), (-k, 0.0), (0.0, -k), (k, 0.0)])
        .map(|((ax, ay), (tx, ty))| VecVertex {
            in_handle: [ax - tx, ay - ty],
            out_handle: [ax + tx, ay + ty],
            ..VecVertex::corner([ax, ay])
        })
        .collect();
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// ⭐ **O teto do nível é imposto na ESCRITA**, e o piso recusa.
///
/// ⚠️ O campo numérico do painel aceita o que se digitar — a faixa do controle é uma sugestão de
/// arrasto, nunca uma garantia. Quem garante é a porta, e a lei é a da contagem da matriz: abaixo de
/// 1 **recusa**, acima do teto **limita**.
#[test]
fn the_resolution_is_bounded_where_it_is_written_not_only_where_it_is_dragged() {
    let (mut sim, _scene, _id, e) = a_drawn_extrusion();
    let level = |sim: &SimWorld| sim.world().get::<FieldProfileSource>(e).unwrap().level;

    assert!(
        ph2d_field_ecs::set_param(sim.world_mut(), e, ph2d_field::Param::Resolution, 0.0).is_err(),
        "meio nível não existe, e zero seria uma peça sem contorno"
    );
    assert_eq!(level(&sim), 1, "e a recusa deixa o nó como estava");

    ph2d_field_ecs::set_param(sim.world_mut(), e, ph2d_field::Param::Resolution, 1.0e6)
        .expect("acima do teto não é erro — é limitado");
    assert_eq!(
        level(&sim),
        ph2d_field::MAX_PROFILE_RESOLUTION,
        "o teto é do custo do traçado assente, e impõe-se aqui"
    );
}

/// ⭐⭐ **O DESENHO APAGADO FALA UMA VEZ, e a peça FICA.**
///
/// ⚠️ As duas metades são uma decisão cada, e as duas doem se faltarem:
///
/// - **a peça fica** porque um desenho apagado volta com um desfazer, e uma forma largada não volta
///   com nada;
/// - **fala uma vez** porque a peça continua a cozer bem sem o desenho, e a voz do módulo é limpa a
///   cada cozimento — sem memória, a mesma frase sairia sessenta vezes por segundo.
#[test]
fn a_deleted_outline_speaks_once_and_the_shape_keeps_its_form() {
    let (mut sim, mut scene, id, e) = a_drawn_extrusion();
    let before = edges(&mut sim, e);
    assert!(scene.remove_path(id), "o contorno existia");

    let first = crate::field3d_profile_live::reconcile(sim.world_mut(), &scene);
    assert_eq!(
        first.len(),
        1,
        "o desenho que sumiu tem de ser dito: {first:?}"
    );
    let again = crate::field3d_profile_live::reconcile(sim.world_mut(), &scene);
    assert!(again.is_empty(), "e não repetido a cada quadro: {again:?}");
    assert_eq!(
        edges(&mut sim, e),
        before,
        "a forma tinha de ficar — largá-la seria trabalho perdido sem desfazer possível"
    );
}

/// ⭐ **A linha «Resolution» é OFERECIDA exactamente onde tem onde escrever** (a lei da W34).
///
/// ⚠️ Lida do **retrato publicado**, e não de `params_of`: o que o artista alcança é o que foi
/// publicado ao painel. E as duas pontas: uma caixa (sem vínculo) não a mostra — mostrá-la seria a
/// affordance que mente, porque `set_param` não teria onde escrever.
#[test]
fn the_resolution_row_is_offered_exactly_where_the_link_is() {
    let has_row = |sim: &mut SimWorld, scene: &VecScene, sel: &[bevy_ecs::entity::Entity]| {
        let _ = ph2d_panel_model3d::drain_intents();
        tick(sim, scene, sel);
        ph2d_panel_model3d::state::current()
            .rows
            .iter()
            .any(|r| r.key == "field.dim.resolution")
    };

    let (mut sim, scene, _id, e) = a_drawn_extrusion();
    assert!(
        has_row(&mut sim, &scene, &[e]),
        "a forma que veio do desenho tem de oferecer a resolução"
    );

    // A caixa da semente — mesma peça, outro nó, sem vínculo nenhum.
    let plain = {
        let world = sim.world_mut();
        let mut q = world.query::<(bevy_ecs::entity::Entity, &FieldNode)>();
        let all: Vec<bevy_ecs::entity::Entity> = q.iter(world).map(|(x, _)| x).collect();
        all.into_iter()
            .find(|x| *x != e && world.get::<FieldProfileSource>(*x).is_none())
            .expect("a caixa da semente")
    };
    assert!(
        !has_row(&mut sim, &scene, &[plain]),
        "e um nó sem vínculo não pode oferecer um número que não tem onde cair"
    );
}

/// ⭐⭐⭐ **TODA forma escolhida vira peça, e cada uma fica ligada ao SEU desenho** (W74).
///
/// ⛔ **O defeito era duplo e mudo:** a função cozia só `closed.first()`, e a caixa de correio era um
/// **slot** — mesmo que ela cozesse as duas, a segunda apagaria a primeira. Um artista com dois
/// contornos escolhidos via **uma** peça e nenhuma palavra sobre a outra.
///
/// ⚠️ **As três metades são o gate:** que nascem `N`, que cada uma aponta para um desenho
/// **diferente**, e que a mensagem **conta**. Sem a segunda, duas peças a apontar para o mesmo
/// contorno passariam — e é justamente o que um laço mal escrito produz.
#[test]
fn every_selected_shape_becomes_its_own_linked_piece() {
    let _ = ph2d_panel_model3d::drain_intents();
    let mut scene = VecScene::default();
    let a = scene.push_path(square(1.0));
    let mut far = square(0.6);
    for v in &mut far.verts {
        v.anchor[0] += 3.0;
        v.in_handle[0] += 3.0;
        v.out_handle[0] += 3.0;
    }
    let b = scene.push_path(far);

    let msg = crate::field3d_profile::from_selection(&scene, &[a, b], ProfileShape::Extrude);
    assert!(
        msg.starts_with("Extruded 2 shapes"),
        "a mensagem tem de CONTAR o que nasceu: {msg}"
    );

    let mut sim = SimWorld::new();
    crate::field3d_scene::sync_scene_and_birth(&mut sim, Some(&seed()), &[], 0.0, &scene);
    let world = sim.world_mut();
    let mut q = world.query::<&FieldProfileSource>();
    let mut linked: Vec<u64> = q.iter(world).map(|s| s.path).collect();
    linked.sort_unstable();
    let mut want = vec![a, b];
    want.sort_unstable();
    assert_eq!(
        linked, want,
        "cada peça segue o SEU desenho — duas a apontar para o mesmo é um laço mal escrito"
    );
}

/// ⭐⭐ **E o que o documento recusa é DITO, não deitado fora** (W74).
///
/// ⚠️ A metade que importa é que a peça boa **nasce na mesma**: uma recusa que abortasse o lote
/// faria um contorno mal desenhado apagar o trabalho dos outros.
#[test]
fn a_refused_shape_is_named_and_the_good_one_still_lands() {
    let _ = ph2d_panel_model3d::drain_intents();
    let mut scene = VecScene::default();
    let good = scene.push_path(square(1.0));
    // Um «contorno» de dois pontos: fechado, e sem área nenhuma para ser perfil.
    let sliver = VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        closed: true,
        ..VecPath::default()
    };
    let bad = scene.push_path(sliver);

    let msg = crate::field3d_profile::from_selection(&scene, &[good, bad], ProfileShape::Extrude);
    assert!(
        msg.starts_with("Extruded the shape") && msg.contains("skipped"),
        "a boa nasce e a recusada é nomeada: {msg}"
    );

    let mut sim = SimWorld::new();
    crate::field3d_scene::sync_scene_and_birth(&mut sim, Some(&seed()), &[], 0.0, &scene);
    let e = the_profile_node(&mut sim);
    let world = sim.world_mut();
    assert_eq!(
        world.entity(e).get::<FieldProfileSource>().map(|s| s.path),
        Some(good),
        "a peça que nasceu é a do contorno válido"
    );
}

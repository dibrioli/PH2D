//! Os gates do AUTO LAYOUT vivo.
//!
//! O motor (`ph2d-vec-layout`) já prova o que o flexbox produz — em coordenadas de CSS. O que só
//! se pode afirmar aqui é a COSTURA: que a moldura mede-se por si, que a ordem da hierarquia é a
//! ordem do fluxo, que o eixo é trocado UMA vez (o documento é Y-up), que o `grow` chega ao
//! desenho, que o passe **não escreve `Transform`** — e que o mundo de quem nunca pediu layout
//! fica **byte-intocado**.

use super::*;
use ph2d_ecs::{LayoutDir, Transform, VecLayout, VecLayoutItem};
use ph2d_vec_scene::rectangle;

/// Uma moldura de 100×40 na origem com `n` filhos de 10×10 em fila, ainda por colocar (todos
/// empilhados no canto inferior-esquerdo da moldura). Devolve `(sim, scene, map, moldura, filhos)`.
fn frame_with_children(n: usize) -> (SimWorld, VecScene, VecEntityMap, Entity, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let frame_id = scene.push_path(rectangle([0.0, 0.0], [100.0, 40.0]));
    let kids: Vec<VecPathId> = (0..n)
        .map(|_| scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0])))
        .collect();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let frame = Entity::from_bits(map[&frame_id]);
    for k in &kids {
        let kid = Entity::from_bits(map[k]);
        sim.world_mut()
            .entity_mut(kid)
            .insert(ph2d_ecs::ChildOf(frame));
    }
    (sim, scene, map, frame, kids)
}

fn arm(sim: &mut SimWorld, frame: Entity, l: VecLayout) {
    sim.world_mut().entity_mut(frame).insert(l);
}

fn run(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    live: &mut LiveGeometry,
) -> LayoutLive {
    let mut ll = LayoutLive::default();
    ll.recook(
        scene,
        sim,
        map,
        &VecXforms::default(),
        live,
        crate::vec_bindings::TokenCtx::factory(),
    );
    ll
}

/// A caixa de mundo que um caminho DESENHA depois do passe.
fn drawn(live: &LiveGeometry, scene: &VecScene, id: VecPathId) -> ([f64; 2], [f64; 2]) {
    let items = world_of(scene, &VecXforms::default(), live, id);
    bbox_of(&items).expect("o caminho desenha alguma coisa")
}

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.01
}

/// **Três filhos em fila pousam na aritmética exacta — e o eixo foi trocado UMA vez.**
///
/// ⚠️ A segunda metade é a que um gate em coordenadas de CSS não pode ver: o motor mede `y` para
/// baixo a partir do topo da moldura, o documento mede para cima. O topo da moldura é `y = 40`,
/// então o primeiro filho tem de ter o topo em **40** — não em 0, que é onde ele estaria se a
/// troca de eixo tivesse sido esquecida (e a fila sairia por baixo da moldura, fora dela).
#[test]
fn a_row_places_the_children_at_the_arithmetic_positions_in_a_y_up_world() {
    let (mut sim, scene, map, frame, kids) = frame_with_children(3);
    arm(
        &mut sim,
        frame,
        VecLayout {
            dir: LayoutDir::Row,
            gap: [4.0, 4.0],
            ..VecLayout::default()
        },
    );
    let mut live = LiveGeometry::new();
    let ll = run(&sim, &scene, &map, &mut live);
    assert_eq!(ll.placed(), 3, "os tres filhos foram colocados");

    for (i, k) in kids.iter().enumerate() {
        let (lo, hi) = drawn(&live, &scene, *k);
        let want_x = i as f64 * 14.0; // 10 de largura + 4 de vao
        assert!(approx(lo[0], want_x), "filho {i}: x={} != {want_x}", lo[0]);
        assert!(
            approx(hi[1], 40.0),
            "filho {i}: o topo tem de encostar no TOPO da moldura (40), e esta' em {}",
            hi[1]
        );
    }
}

/// **A COLUNA empilha para BAIXO a partir do topo** — o outro lado da mesma troca de eixo.
#[test]
fn a_column_stacks_downwards_from_the_top() {
    let (mut sim, scene, map, frame, kids) = frame_with_children(2);
    arm(
        &mut sim,
        frame,
        VecLayout {
            dir: LayoutDir::Column,
            gap: [2.0, 2.0],
            ..VecLayout::default()
        },
    );
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);
    let first = drawn(&live, &scene, kids[0]);
    let second = drawn(&live, &scene, kids[1]);
    assert!(approx(first.1[1], 40.0), "o 1o encosta no topo");
    assert!(
        approx(second.1[1], 40.0 - 12.0),
        "o 2o desce 10 (altura) + 2 (vao): {}",
        second.1[1]
    );
}

/// **`grow` chega ao DESENHO** — o filho que cresce ocupa a sobra, e o seguinte é empurrado.
///
/// É o caso da barra de ferramentas: um espaçador que empurra o resto para o fim.
#[test]
fn grow_reaches_the_drawing_and_pushes_the_rest() {
    let (mut sim, scene, map, frame, kids) = frame_with_children(3);
    arm(&mut sim, frame, VecLayout::default());
    // O do meio é o espaçador.
    let spacer = Entity::from_bits(map[&kids[1]]);
    sim.world_mut().entity_mut(spacer).insert(VecLayoutItem {
        grow: 1.0,
        ..VecLayoutItem::default()
    });
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);

    let mid = drawn(&live, &scene, kids[1]);
    assert!(
        approx(mid.1[0] - mid.0[0], 10.0 + 70.0),
        "o espacador leva a sobra inteira (100 - 30): {}",
        mid.1[0] - mid.0[0]
    );
    let last = drawn(&live, &scene, kids[2]);
    assert!(
        approx(last.1[0], 100.0),
        "o ultimo encosta no fim da moldura: {}",
        last.1[0]
    );
}

/// **O passe NÃO escreve `Transform`** — a lei do ADR-0153, e o preço de a quebrar é o undo deste
/// editor a encher de passos espúrios (ele regista por DIFF do mundo ECS).
///
/// ⚠️ **Hoje o TIPO já recusa**: `recook` recebe `&SimWorld`, então a mutação que escreveria a pose
/// não compila — este gate não a "matou", e dizer o contrário seria vender uma prova que não houve.
/// Ele é o guarda do dia em que a assinatura mudar, e ela **vai querer mudar**: o gesto de
/// reordenar (a metade seguinte da wave) escreve no mundo, e é exactamente aí que alguém passa um
/// `&mut` por este passe e a lei cai em silêncio.
///
/// O oráculo é a pose de TODOS os filhos, e não só a da moldura: um passe que escrevesse a de um
/// deles passaria num gate que só olhasse para a raiz.
#[test]
fn the_pass_never_writes_the_authored_transform() {
    let (mut sim, scene, map, frame, kids) = frame_with_children(3);
    arm(
        &mut sim,
        frame,
        VecLayout {
            gap: [7.0, 7.0],
            ..VecLayout::default()
        },
    );
    let before: Vec<Transform> = kids
        .iter()
        .map(|k| {
            sim.world()
                .get::<Transform>(Entity::from_bits(map[k]))
                .copied()
                .unwrap_or(Transform::IDENTITY)
        })
        .collect();
    let mut live = LiveGeometry::new();
    let ll = run(&sim, &scene, &map, &mut live);
    assert!(ll.placed() > 0, "a fixture tem de conter o fenomeno");
    let after: Vec<Transform> = kids
        .iter()
        .map(|k| {
            sim.world()
                .get::<Transform>(Entity::from_bits(map[k]))
                .copied()
                .unwrap_or(Transform::IDENTITY)
        })
        .collect();
    assert_eq!(
        before, after,
        "o layout escreveu a pose AUTORADA — cada redimensionamento viraria um passo de undo"
    );
}

/// **Sem o componente o mapa fica VAZIO** — é o que torna todo documento anterior a esta feature
/// byte-idêntico.
#[test]
fn a_frame_without_the_component_never_enters_the_live_map() {
    let (sim, scene, map, _frame, _kids) = frame_with_children(3);
    let mut live = LiveGeometry::new();
    let ll = run(&sim, &scene, &map, &mut live);
    assert_eq!(ll.placed(), 0);
    assert!(
        live.is_empty(),
        "o mapa nasceu com {} entrada(s)",
        live.len()
    );
}

/// **Uma moldura ANINHADA é colocada pela de fora, e coloca os próprios filhos** — e ela não é
/// processada duas vezes.
///
/// ⚠️ A dupla passagem é o modo de falha caro: a segunda leria a caixa que a primeira acabou de
/// mover e empilharia o deslocamento, e a sub-árvore fugiria da moldura a cada frame.
#[test]
fn a_nested_frame_is_placed_by_the_outer_one_and_lays_its_own_children() {
    let (mut sim, mut scene, mut map, frame, _kids) = frame_with_children(1);
    // Uma moldura interna com dois filhos, pendurada na de fora.
    let inner_id = scene.push_path(rectangle([0.0, 0.0], [40.0, 20.0]));
    let g1 = scene.push_path(rectangle([0.0, 0.0], [5.0, 5.0]));
    let g2 = scene.push_path(rectangle([0.0, 0.0], [5.0, 5.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let inner = Entity::from_bits(map[&inner_id]);
    sim.world_mut()
        .entity_mut(inner)
        .insert(ph2d_ecs::ChildOf(frame));
    for g in [g1, g2] {
        let e = Entity::from_bits(map[&g]);
        sim.world_mut()
            .entity_mut(e)
            .insert(ph2d_ecs::ChildOf(inner));
    }
    arm(&mut sim, frame, VecLayout::default());
    arm(&mut sim, inner, VecLayout::default());

    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);

    // A interna foi colocada pela de fora: ela começa onde o primeiro filho acaba (x = 10).
    let inner_box = drawn(&live, &scene, inner_id);
    assert!(approx(inner_box.0[0], 10.0), "a interna: {inner_box:?}");
    // E os netos foram colocados DENTRO dela, em fila, a partir do canto dela.
    let a = drawn(&live, &scene, g1);
    let b = drawn(&live, &scene, g2);
    assert!(approx(a.0[0], 10.0), "o 1o neto: {a:?}");
    assert!(approx(b.0[0], 15.0), "o 2o neto = 10 + 5: {b:?}");
}

/// **O layout coloca o que os outros produtores DESENHARAM, não a fonte crua.**
///
/// ⚠️ É o gate que justifica este módulo TRANSFORMAR o mapa em vez de o estender. A fixture põe no
/// mapa uma geometria que a fonte não tem (um filho três vezes maior): se o passe lesse a fonte,
/// mediria 10 e a fila sairia com os vãos errados.
#[test]
fn the_layout_places_what_the_other_producers_drew() {
    let (mut sim, scene, map, frame, kids) = frame_with_children(2);
    arm(&mut sim, frame, VecLayout::default());
    let mut live = LiveGeometry::new();
    // "Já derivado": o primeiro filho desenha 30 de largura, não 10.
    live.insert(kids[0], vec![rectangle([0.0, 0.0], [30.0, 10.0])]);
    run(&sim, &scene, &map, &mut live);
    let second = drawn(&live, &scene, kids[1]);
    assert!(
        approx(second.0[0], 30.0),
        "o 2o tem de comecar onde o 1o DESENHADO acaba (30), e comeca em {}",
        second.0[0]
    );
}

/// **O VÃO pode seguir um TOKEN, e o passe honra o comprimento resolvido** (W4c.4).
///
/// ⚠️ É o gate da COSTURA, e ele existe porque nenhum dos outros a cobria: `bound_gap` pode estar
/// perfeito e o `frame_style` continuar a ler o literal — a mutação que troca o vão resolvido pelo
/// autorado passava por toda a suíte até este gate nascer.
///
/// ⚠️ O oráculo é a POSIÇÃO do segundo filho, e não o número que a régua devolve: o número seria o
/// espelho do `token_world`, e o que se quer saber é se a moldura de facto espaça por ele.
#[test]
fn a_gap_token_spaces_the_flow_and_the_literal_is_ignored_while_it_is_bound() {
    let tok = crate::vec_bindings::TokenCtx::factory();
    let expected = crate::vec_bindings::token_world("spacing.4xl", tok).expect("o token existe");
    assert!(
        (expected - 3.0).abs() > 0.5,
        "premissa: o token tem de valer um numero DIFERENTE do literal da fixture, senao o gate \
         nao distingue as duas fontes (ele vale {expected})"
    );

    let (mut sim, scene, map, frame, kids) = frame_with_children(2);
    arm(
        &mut sim,
        frame,
        VecLayout {
            dir: LayoutDir::Row,
            gap: [3.0, 0.0],
            ..VecLayout::default()
        },
    );

    // CONTROLE: sem binding, o vão é o literal — o 2º filho começa em 10 + 3.
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);
    let plain = drawn(&live, &scene, kids[1]);
    assert!(
        approx(plain.0[0], 13.0),
        "o literal tem de valer sem binding: {plain:?}"
    );

    // Com o token preso, o passe espaça pelo comprimento RESOLVIDO.
    let mut b = ph2d_ecs::VecBindings::default();
    b.set(ph2d_ecs::BoundProp::LayoutGapMain, "spacing.4xl");
    sim.world_mut().entity_mut(frame).insert(b);
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);
    let bound = drawn(&live, &scene, kids[1]);
    assert!(
        approx(bound.0[0], 10.0 + expected),
        "o 2o filho tinha de comecar em 10 + {expected} (o token), e comeca em {}",
        bound.0[0]
    );
}

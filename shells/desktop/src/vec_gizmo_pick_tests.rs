//! Os gates do que o PONTEIRO acha — irmão do [`super`], que é o produto.
//!
//! ⚠️ Eles mudaram-se do `vec_gizmo_view_tests` quando o hit-test saiu para módulo próprio: um
//! gate que julga o pick tem de viver ao lado dele, senão o próximo corte deixa-o a testar o
//! vizinho.

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use ph2d_vec_scene::line;

/// REGRESSÃO (Enio 2026-07-09: "line e arc não podem ser transformadas com o
/// gizmo"). Uma forma ABERTA não tem interior — sem raio de traço ela nunca é
/// pega, e o gizmo de Select nunca a agarra. Com raio, o clique no traço pega.
#[test]
fn an_open_line_is_picked_by_stroke_proximity_not_interior() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(line([0.0, 0.0], [10.0, 0.0]));
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    let vs = VecViewState::default();
    // Um clique 0.4 ACIMA da linha (fora do traço): sem raio não pega — uma linha
    // aberta não tem interior.
    assert_eq!(
        pick_at_world(
            &sim,
            &scene,
            &Default::default(),
            &vs,
            &map,
            [5.0, 0.4],
            0.0
        ),
        None
    );
    // Com raio 1.0 (> 0.4): pega pela proximidade do traço.
    assert_eq!(
        pick_at_world(
            &sim,
            &scene,
            &Default::default(),
            &vs,
            &map,
            [5.0, 0.4],
            1.0
        ),
        Some(e.to_bits())
    );
    // Longe do traço, mesmo com raio: não pega.
    assert_eq!(
        pick_at_world(
            &sim,
            &scene,
            &Default::default(),
            &vs,
            &map,
            [5.0, 5.0],
            1.0
        ),
        None
    );
}

/// O picking respeita o `Transform`: o interior está onde a forma é DESENHADA,
/// não onde ela é guardada.
#[test]
fn picking_finds_the_shape_where_the_transform_puts_it() {
    let (mut sim, scene, map, e) = scene_with_square();
    let vs = VecViewState::default();
    assert!(
        pick_at_world(
            &sim,
            &scene,
            &Default::default(),
            &vs,
            &map,
            [0.0, 0.0],
            0.0
        )
        .is_some()
    );

    sim.world_mut().entity_mut(e).insert(Transform {
        translation: Vec2::new(50.0, 0.0),
        ..Transform::IDENTITY
    });
    assert_eq!(
        pick_at_world(
            &sim,
            &scene,
            &Default::default(),
            &vs,
            &map,
            [0.0, 0.0],
            0.0
        ),
        None,
        "a origem ficou vazia"
    );
    assert_eq!(
        pick_at_world(
            &sim,
            &scene,
            &Default::default(),
            &vs,
            &map,
            [50.0, 0.0],
            0.0
        ),
        Some(e.to_bits()),
        "a forma está onde o transform a pôs"
    );
}

/// Travada ou escondida não é selecionável no canvas — como um sprite.
#[test]
fn a_hidden_or_locked_shape_is_not_pickable() {
    let (sim, scene, map, _) = scene_with_square();
    let id = scene.paths()[0].id;
    let hidden = VecViewState {
        hidden: vec![id],
        ..Default::default()
    };
    assert_eq!(
        pick_at_world(
            &sim,
            &scene,
            &Default::default(),
            &hidden,
            &map,
            [0.0, 0.0],
            0.0
        ),
        None
    );
    let locked = VecViewState {
        locked: vec![id],
        ..Default::default()
    };
    assert_eq!(
        pick_at_world(
            &sim,
            &scene,
            &Default::default(),
            &locked,
            &map,
            [0.0, 0.0],
            0.0
        ),
        None
    );
}

/// O marquee pega a forma pela bbox de MUNDO.
#[test]
fn the_marquee_selects_a_translated_shape_by_its_world_bbox() {
    let (mut sim, scene, map, e) = scene_with_square();
    let vs = VecViewState::default();
    sim.world_mut().entity_mut(e).insert(Transform {
        translation: Vec2::new(20.0, 20.0),
        ..Transform::IDENTITY
    });
    assert!(
        pick_in_world_rect(
            &sim,
            &scene,
            &Default::default(),
            &vs,
            &map,
            [-5.0, -5.0],
            [5.0, 5.0]
        )
        .is_empty()
    );
    assert_eq!(
        pick_in_world_rect(
            &sim,
            &scene,
            &Default::default(),
            &vs,
            &map,
            [15.0, 15.0],
            [25.0, 25.0]
        ),
        vec![e.to_bits()]
    );
}

// ---------------------------------------------------------------------------
// A lei que este módulo DECLARA — e o mapa de que ela depende.
// ---------------------------------------------------------------------------

/// **A metade ESPELHADA é clicável porque é DESENHADA.**
///
/// O doc do [`super`] afirma que a pergunta *"o que está desenhado aqui?"* é feita ao **MESMO
/// mapa** que o `ph2d_vec_render::dispatch` consome. ⚠️ Até 2026-08-10 a fiação contradizia-o: o
/// `dispatch` recebe a FUSÃO de nove produtores e os seis sítios de pick da `input_dispatch`
/// passavam **só o `offset_live`** — medido nesta fixture, **3 de 3** pontos da metade espelhada
/// estavam na tela com o clique a atravessar.
///
/// A fixture é a mais barata que contém o fenômeno: um meio-perfil com **simetria armada**, para o
/// qual a metade derivada existe e a autorada não a cobre.
///
/// ⚠️ **A metade negativa é o CONTROLE, não um pino de regressão:** sem ela o gate ficaria verde
/// no dia em que a geometria da FONTE passasse a cobrir os pontos amostrados, e deixaria de falar
/// sobre a derivada. Ela afirma *a fixture contém o fenômeno*; a positiva afirma *o pick o vê*.
///
/// ⚠️ E ele é **CEGO à fiação** por construção — entrega o mapa à mão. Quem prova que o produto
/// entrega o mapa certo é o arch-gate `the_pick_reads_the_map_that_was_drawn`.
#[test]
fn the_mirrored_half_is_pickable_because_it_is_drawn() {
    use ph2d_vec_scene::VecVertex;
    use ph2d_vec_scene::symmetry::SymmetrySpec;

    // Meio-perfil ABERTO com as pontas no eixo `x = 0` — a forma para a qual a simetria existe.
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(ph2d_vec_scene::VecPath {
        verts: [[0.0, -1.0], [0.8, -0.3], [0.5, 0.4], [0.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: false,
        ..ph2d_vec_scene::VecPath::default()
    });
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    crate::symmetry_live::arm(&mut sim, &map, &[id], Some(SymmetrySpec::default()));
    let xf = crate::vec_transform::build(&sim, &map);
    let mut sym = crate::symmetry_live::SymmetryLive::default();
    sym.recook(&scene, &sim, &map, &xf, true);

    // O mapa que o RENDERER recebe (a fusão) contra o mapa VAZIO — que é o que sobra quando
    // nenhum produtor vivo entra, e é onde o pick lê a geometria AUTORADA.
    let drawn = sym.live().clone();
    let source_only: LiveGeometry = LiveGeometry::new();
    let vs = VecViewState::default();
    for p in [[-0.8, -0.3], [-0.5, 0.4], [-0.4, 0.0]] {
        assert!(
            pick_at_world(&sim, &scene, &source_only, &vs, &map, p, 0.05).is_none(),
            "CONTROLE: {p:?} nao devia ser coberto pela metade AUTORADA — a fixture deixou de \
             conter o fenomeno, e a metade positiva passaria a nao dizer nada sobre a derivada"
        );
        assert!(
            pick_at_world(&sim, &scene, &drawn, &vs, &map, p, 0.05).is_some(),
            "{p:?} esta' DESENHADO e o clique atravessa — o pick nao esta' a ler o mapa que o \
             `dispatch` recebeu"
        );
    }
}

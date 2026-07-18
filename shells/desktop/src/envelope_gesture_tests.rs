//! Os gates do [`crate::envelope_gesture`] — o gesto de arrastar os cantos da gaiola (ADR-0129), no
//! lado do HOST, com o alvo sendo o **container** do envelope (Fatia 3) sob uma POSE não-identidade.
//!
//! A geometria pura (que canto, e até onde) já está gateada na crate `ph2d-vec-envelope`
//! (`nearest_corner`/`move_corner_convex`). Aqui prova-se o **fio** — e que ele atravessa a POSE do
//! CONTAINER: os cantos vivem em LOCAL no componente, mas o hit-test/desenho acontecem em MUNDO
//! (cantos × pose do container) e o drag desce o cursor à local pela pose inversa. O fixture dá ao
//! container uma translação de `[100, 50]` DE PROPÓSITO: com pose identidade, local == mundo e um
//! `container_world_xform` esquecido passaria despercebido.
//! [[feedback_derived_coordinate_seed_must_match_sample]]

use super::*;
use ph2d_ecs::Transform;
use ph2d_vec_scene::{ShapeKind, VecPath, VecScene, cook};

use crate::vec_entities::VecEntityMap;

/// Uma gaiola retangular de cantos LOCAIS CONHECIDOS `[BL, BR, TR, TL]`.
fn rect_corners() -> [[f64; 2]; 4] {
    [[0.0, 0.0], [10.0, 0.0], [10.0, 6.0], [0.0, 6.0]]
}

/// A pose do fixture (translação), no CONTAINER. NÃO-identidade, para exercitar a conversão
/// local↔mundo.
const POSE: [f64; 2] = [100.0, 50.0];

/// Os cantos LOCAIS levados ao MUNDO pela pose do container (só translação).
fn world_corners() -> [[f64; 2]; 4] {
    std::array::from_fn(|i| [rect_corners()[i][0] + POSE[0], rect_corners()[i][1] + POSE[1]])
}

fn ellipse() -> VecPath {
    cook(ShapeKind::Ellipse, [3.0, 1.0], [7.0, 5.0], &[])
}

/// px→mundo do fixture: raio de captura = `ENVELOPE_HANDLE_R_PX (6) × 0.1 = 0.6` unidades de mundo.
/// Pequeno o bastante para o centro da gaiola não pegar canto nenhum.
const PX_TO_WORLD: f64 = 0.1;

/// Sobrescreve os cantos do container.
fn set_corners(sim: &mut SimWorld, bits: u64, corners: [[f64; 2]; 4]) {
    sim.world_mut()
        .get_mut::<VecEnvelope>(Entity::from_bits(bits))
        .expect("VecEnvelope")
        .corners = corners;
}

/// Uma cena com UM envelope (container de cantos LOCAIS conhecidos) e uma POSE de `[100, 50]` no
/// CONTAINER. Devolve `(sim, scene, map, container_bits)`.
fn scene() -> (SimWorld, VecScene, VecEntityMap, u64) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(ellipse());
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let container = crate::envelope_live::create(&mut sim, &mut scene, &map, &[id]).expect("create");
    set_corners(&mut sim, container, rect_corners());
    // A POSE que o gizmo do Select moveria — aqui posta à mão no CONTAINER para o hit-test/desenho
    // terem de atravessá-la (com identidade, local == mundo e o `container_world_xform` poderia sumir
    // sem falha).
    sim.world_mut()
        .get_mut::<Transform>(Entity::from_bits(container))
        .expect("Transform")
        .translation = ph2d_core::Vec2::new(POSE[0] as f32, POSE[1] as f32);
    (sim, scene, map, container)
}

/// Adiciona uma forma COMUM (sem `VecEnvelope`) e devolve os bits da ENTIDADE dela.
fn add_plain(sim: &mut SimWorld, scene: &mut VecScene, map: &mut VecEntityMap) -> u64 {
    let id = scene.push_path(ellipse());
    crate::vec_entities::sync(sim, scene, map);
    map[&id]
}

/// **O press arma quando um canto (em MUNDO) está sob o cursor** — e no canto CERTO. O TR local
/// `[10,6]` está no mundo em `[110,56]` pela pose do container; um cursor logo fora dele pega o
/// índice 2.
#[test]
fn press_arms_on_the_world_corner_under_the_cursor() {
    let (sim, _scene, _map, container) = scene();
    let mut drag = None;
    let tr = world_corners()[2];
    assert!(press(
        &sim,
        Some(container),
        [tr[0] + 0.3, tr[1] + 0.2],
        PX_TO_WORLD,
        &mut drag
    ));
    assert_eq!(drag, Some((container, 2)), "devia armar o TR (via a pose do container)");
    // E o cursor no lugar LOCAL do canto (ignorando a pose) NÃO pega: prova que a pose foi aplicada.
    let mut miss = None;
    assert!(!press(&sim, Some(container), rect_corners()[2], PX_TO_WORLD, &mut miss));
    assert_eq!(miss, None, "sem aplicar a pose, o hit-test pegaria no lugar errado");
}

/// **O raio é uma cerca:** um cursor no meio da gaiola (mundo) não arma nada.
#[test]
fn press_misses_when_far_from_every_corner() {
    let (sim, _scene, _map, container) = scene();
    let mut drag = None;
    assert!(!press(
        &sim,
        Some(container),
        [5.0 + POSE[0], 3.0 + POSE[1]],
        PX_TO_WORLD,
        &mut drag
    ));
    assert_eq!(drag, None, "o centro nao esta perto de canto nenhum");
}

/// **Uma entidade que NÃO é envelope não tem gaiola** — o press a ignora. É o que deixa o pen
/// (seleção/âncora) seguir dono do clique.
#[test]
fn press_ignores_a_non_envelope_entity() {
    let (mut sim, mut scene, mut map, _container) = scene();
    let plain = add_plain(&mut sim, &mut scene, &mut map);
    let mut drag = None;
    assert!(!press(&sim, Some(plain), [7.0, 5.0], PX_TO_WORLD, &mut drag));
    assert_eq!(drag, None);
}

/// **Sem seleção, nada arma.**
#[test]
fn press_with_no_selection_arms_nothing() {
    let (sim, _scene, _map, _container) = scene();
    let mut drag = None;
    assert!(!press(&sim, None, world_corners()[2], PX_TO_WORLD, &mut drag));
    assert_eq!(drag, None);
}

/// **O drag escreve o canto LOCAL no componente** num movimento convexo — o cursor em MUNDO desce à
/// local pela pose inversa do container, e é o `VecEnvelope` (que o undo/save capturam) que muda.
#[test]
fn drag_writes_the_local_corner_from_a_world_cursor() {
    let (mut sim, _scene, _map, container) = scene();
    // Cursor no MUNDO em `[108, 57]` → local `[8, 7]` (convexo).
    assert!(drag(&mut sim, Some((container, 2)), [8.0 + POSE[0], 7.0 + POSE[1]]));
    let corners = corners_of(&sim, container).expect("envelope");
    assert_eq!(
        corners[2],
        [8.0, 7.0],
        "o TR não desceu para local (a pose inversa do container não foi aplicada)"
    );
}

/// **O guard de convexidade no caminho do HOST:** puxar um canto para o interior (reflexo) NÃO muda a
/// gaiola — o canto para na fronteira (§5, o horizonte fica fora). O gesto ainda CONSOME o Move
/// (devolve `true`), só não escreve.
#[test]
fn drag_refuses_a_non_convex_move_and_freezes_the_corner() {
    let (mut sim, _scene, _map, container) = scene();
    let before = corners_of(&sim, container).expect("envelope");
    // Cursor MUNDO `[103, 52]` → local `[3, 2]` (perto do centro) → quad reflexo.
    assert!(
        drag(&mut sim, Some((container, 2)), [3.0 + POSE[0], 2.0 + POSE[1]]),
        "o gesto consome o Move mesmo recusando o movimento"
    );
    let after = corners_of(&sim, container).expect("envelope");
    assert_eq!(
        before, after,
        "um movimento nao-convexo mudou a gaiola — o horizonte entraria nela"
    );
}

/// **O `view` desenha a gaiola em MUNDO (cantos LOCAIS × pose do container) e marca SÓ o canto sob
/// arrasto DESTE container.** Um arrasto vivo noutro container não acende bolinha aqui; sem arrasto,
/// nenhuma.
#[test]
fn view_draws_world_corners_and_marks_only_this_containers_dragged_corner() {
    let (sim, _scene, _map, container) = scene();

    let cage = view(&sim, Some(container), Some((container, 2))).expect("cage");
    assert_eq!(cage.corners, world_corners(), "os cantos saíram em MUNDO (local × pose)");
    assert_eq!(cage.dragging, Some(2), "o TR deste container esta sob arrasto");

    assert_eq!(
        view(&sim, Some(container), Some((container + 1, 0)))
            .expect("cage")
            .dragging,
        None,
        "um arrasto noutro container nao marca esta gaiola"
    );
    assert_eq!(
        view(&sim, Some(container), None).expect("cage").dragging,
        None,
        "sem arrasto, nenhum canto e' cheio"
    );
}

/// **Não há gaiola para desenhar sobre uma entidade comum nem sobre "nada".**
#[test]
fn view_is_none_without_an_envelope() {
    let (mut sim, mut scene, mut map, _container) = scene();
    let plain = add_plain(&mut sim, &mut scene, &mut map);
    assert!(view(&sim, Some(plain), None).is_none());
    assert!(view(&sim, None, None).is_none());
}

//! Os gates do [`crate::envelope_gesture`] — o gesto de arrastar os cantos da gaiola (ADR-0129,
//! Fatia 1), no lado do HOST.
//!
//! A geometria pura (que canto, e até onde) já está gateada na crate `ph2d-vec-envelope`
//! (`nearest_corner`/`move_corner_convex`). Aqui prova-se o **fio**: o press arma pelo componente
//! certo, o drag escreve `corners` no `VecEnvelope`, a convexidade é honrada no caminho do host, e
//! o `view` marca o canto sob arrasto — nunca o de outra forma.

use super::*;
use ph2d_vec_scene::{ShapeKind, VecPath, VecScene, cook};

use crate::vec_entities::VecEntityMap;

/// Uma gaiola retangular de cantos CONHECIDOS `[BL, BR, TR, TL]` — para posicionar o cursor com
/// precisão. Independe da fonte: o gesto só lê/escreve `corners`.
fn rect_corners() -> [[f64; 2]; 4] {
    [[0.0, 0.0], [10.0, 0.0], [10.0, 6.0], [0.0, 6.0]]
}

fn ellipse() -> VecPath {
    cook(ShapeKind::Ellipse, [3.0, 1.0], [7.0, 5.0], &[])
}

/// px→mundo do fixture: raio de captura = `ENVELOPE_HANDLE_R_PX (6) × 0.1 = 0.6` unidades de mundo.
/// Pequeno o bastante para o centro da gaiola não pegar canto nenhum.
const PX_TO_WORLD: f64 = 0.1;

/// Uma cena com UM envelope de cantos retangulares conhecidos. Devolve `(sim, scene, map, id)`.
fn scene() -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(ellipse());
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let xf = crate::vec_transform::build(&sim, &map);
    let (eid, mut env) = crate::envelope_live::create(&scene, &xf, id).expect("create");
    env.corners = rect_corners();
    assert!(crate::envelope_live::attach(&mut sim, &map, eid, &env));
    (sim, scene, map, id)
}

/// Adiciona uma forma COMUM (sem `VecEnvelope`) e devolve o id dela.
fn add_plain(sim: &mut SimWorld, scene: &mut VecScene, map: &mut VecEntityMap) -> VecPathId {
    let id = scene.push_path(ellipse());
    crate::vec_entities::sync(sim, scene, map);
    id
}

/// **O press arma quando um canto está sob o cursor** — e no canto CERTO. Cursor logo fora do TR
/// (índice 2) pega o TR.
#[test]
fn press_arms_on_the_corner_under_the_cursor() {
    let (sim, _scene, map, id) = scene();
    let mut drag = None;
    assert!(press(
        &sim,
        &map,
        Some(id),
        [10.3, 6.2],
        PX_TO_WORLD,
        &mut drag
    ));
    assert_eq!(drag, Some((id, 2)), "devia armar o TR");
}

/// **O raio é uma cerca:** um cursor no meio da gaiola (longe de todo canto) não arma nada. Sem
/// isto o press pegaria o canto "mais próximo" de qualquer clique — e o `nearest_corner` sem o
/// limiar passaria por aqui verde.
#[test]
fn press_misses_when_far_from_every_corner() {
    let (sim, _scene, map, id) = scene();
    let mut drag = None;
    assert!(!press(
        &sim,
        &map,
        Some(id),
        [5.0, 3.0],
        PX_TO_WORLD,
        &mut drag
    ));
    assert_eq!(drag, None, "o centro nao esta perto de canto nenhum");
}

/// **Uma forma que NÃO é envelope não tem gaiola** — o press a ignora, mesmo com o cursor num
/// "canto" do bbox dela. É o que deixa o pen (seleção/âncora) seguir dono do clique.
#[test]
fn press_ignores_a_non_envelope_shape() {
    let (mut sim, mut scene, mut map, _id) = scene();
    let plain = add_plain(&mut sim, &mut scene, &mut map);
    let mut drag = None;
    assert!(!press(
        &sim,
        &map,
        Some(plain),
        [7.0, 5.0],
        PX_TO_WORLD,
        &mut drag
    ));
    assert_eq!(drag, None);
}

/// **Sem seleção, nada arma.**
#[test]
fn press_with_no_selection_arms_nothing() {
    let (sim, _scene, map, _id) = scene();
    let mut drag = None;
    assert!(!press(
        &sim,
        &map,
        None,
        [10.3, 6.2],
        PX_TO_WORLD,
        &mut drag
    ));
    assert_eq!(drag, None);
}

/// **O drag escreve o canto no componente** num movimento convexo — o TR vai para onde o cursor
/// pediu, e é o `VecEnvelope` (que o undo/save capturam) que muda.
#[test]
fn drag_writes_the_corner_on_a_convex_move() {
    let (mut sim, _scene, map, id) = scene();
    assert!(drag(&mut sim, &map, Some((id, 2)), [8.0, 7.0]));
    let corners = corners_of(&sim, &map, id).expect("envelope");
    assert_eq!(corners[2], [8.0, 7.0], "o TR nao seguiu o cursor");
}

/// **O guard de convexidade no caminho do HOST:** puxar um canto para o interior (reflexo) NÃO muda
/// a gaiola — o canto para na fronteira (§5, o horizonte fica fora). O gesto ainda CONSOME o Move
/// (devolve `true`), só não escreve. Sem o guard, a gaiola viraria côncava e a homografia poria a
/// linha de fuga dentro dela.
#[test]
fn drag_refuses_a_non_convex_move_and_freezes_the_corner() {
    let (mut sim, _scene, map, id) = scene();
    let before = corners_of(&sim, &map, id).expect("envelope");
    // TR puxado para perto do centro → quad reflexo (não-convexo).
    assert!(
        drag(&mut sim, &map, Some((id, 2)), [3.0, 2.0]),
        "o gesto consome o Move mesmo recusando o movimento"
    );
    let after = corners_of(&sim, &map, id).expect("envelope");
    assert_eq!(
        before, after,
        "um movimento nao-convexo mudou a gaiola — o horizonte entraria nela"
    );
}

/// **O `view` desenha a gaiola da forma selecionada e marca SÓ o canto sob arrasto DESTA forma.**
/// Um arrasto vivo noutra forma não acende bolinha aqui; sem arrasto, nenhuma.
#[test]
fn view_marks_only_this_shapes_dragged_corner() {
    let (mut sim, mut scene, mut map, id) = scene();
    let other = add_plain(&mut sim, &mut scene, &mut map);

    let cage = view(&sim, &map, Some(id), Some((id, 2))).expect("cage");
    assert_eq!(
        cage.corners,
        rect_corners(),
        "os cantos vieram do componente"
    );
    assert_eq!(cage.dragging, Some(2), "o TR desta forma esta sob arrasto");

    assert_eq!(
        view(&sim, &map, Some(id), Some((other, 0)))
            .expect("cage")
            .dragging,
        None,
        "um arrasto noutra forma nao marca esta gaiola"
    );
    assert_eq!(
        view(&sim, &map, Some(id), None).expect("cage").dragging,
        None,
        "sem arrasto, nenhum canto e' cheio"
    );
}

/// **Não há gaiola para desenhar sobre uma forma comum nem sobre "nada".**
#[test]
fn view_is_none_without_an_envelope() {
    let (mut sim, mut scene, mut map, _id) = scene();
    let plain = add_plain(&mut sim, &mut scene, &mut map);
    assert!(view(&sim, &map, Some(plain), None).is_none());
    assert!(view(&sim, &map, None, None).is_none());
}

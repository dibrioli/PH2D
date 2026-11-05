//! O ramo da alça em `despacho_clique_gizmo.rs`, pela porta da shell: a caixa de `gizmo_anchor_half`
//! sobre um objecto real, `pivo_da_alca` e a conta do arrasto. `ramo_gizmo_alca` pede `App` + janela.

use super::despacho_clique_gizmo::pivo_da_alca;
use super::despacho_vetor_alinhar_e_forma::gizmo_anchor_half;
use ph2d_editor_core::{GizmoCamera, GizmoDragKind, GizmoDragState, TransformSnapshot as Pose};
use ph2d_editor_core::{GizmoModifiers, GizmoSnap, GizmoTarget, compose_snapshot};

const CAMERA: GizmoCamera = {
    // A cena deste teste não tem banda: a janela dela é a própria fixtura.
    let janela = [800.0, 600.0];
    GizmoCamera {
        center: [0.0, 0.0],
        height_world: 10.0,
        window_w: janela[0],
        window_h: janela[1],
    }
};
const ESPELHO: Pose = Pose {
    translation: [0.0, 0.0],
    rotation: 0.0,
    scale: [-1.0, 1.0],
};

/// Agarra a borda da direita em `mao` e anda até `ate`, e devolve as bordas de mundo em X.
fn arrasta(meio: ([f32; 2], [f32; 2]), pai: Pose, mao: [f32; 2], ate: [f32; 2]) -> [f32; 3] {
    let kind = GizmoDragKind::ScaleEdge { axis: 0, sign: 1.0 };
    let ecra = |w: [f32; 2]| (400.0 + w[0] * 60.0, 300.0 - w[1] * 60.0);
    let mut d = GizmoDragState {
        kind,
        entity_bits: 1,
        start_screen: ecra(mao),
        cursor_screen: ecra(mao),
        start_transform: ESPELHO,
        pivot_world: pivo_da_alca(kind, meio, pai, ESPELHO, mao),
        start_cursor_world: mao,
        sprite_half_intrinsic: meio.1,
        anchor_is_center: false,
        target: GizmoTarget::PrimaryIndividual,
        parent_world: pai,
        turns: 0,
    };
    d.advance_cursor(ecra(ate), &CAMERA);
    let (modos, snap) = (GizmoModifiers::default(), GizmoSnap::default());
    let t = ph2d_editor_core::compute_gizmo_transform(&d, &CAMERA, modos, snap, None);
    let m = compose_snapshot(pai, t);
    let h = (m.scale[0] * meio.1[0]).abs();
    [m.translation[0] - h, m.translation[0] + h, t.scale[0]]
}

fn perto(a: [f32; 3], b: [f32; 3]) {
    assert!(
        a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-2),
        "{a:?} × {b:?}"
    );
}

/// Forma vetorial espelhada, agarrada à direita: a esquerda fica, a direita segue a mão, o sinal
/// é o de partida. *Mutação: `anchor_pivot_world(.., false)` no lugar de `grip_pivot_world`.*
#[test]
fn the_handle_of_a_mirrored_vector_shape_keeps_the_far_border() {
    let (mut sim, mut scene) = (
        ph2d_ecs::SimWorld::default(),
        ph2d_vec_scene::VecScene::new(),
    );
    let mut map = ph2d_vec_entities::entities::VecEntityMap::new();
    let id = scene.push_path(ph2d_vec_scene::rectangle([-2.0, -1.0], [2.0, 1.0]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let e = ph2d_ecs::Entity::from_bits(map[&id]);
    let meio = gizmo_anchor_half(&sim, &scene, &ph2d_flip::FlipDoc::default(), e);
    assert_eq!(meio.1, [2.0, 1.0], "a caixa da forma");
    let raiz = Pose::IDENTITY;
    perto(
        arrasta(meio, raiz, [2.0, 0.0], [3.0, 0.0]),
        [-2.0, 3.0, -1.25],
    );
}

/// Sprite espelhado, filho de um pai deslocado e a meia escala (no mundo: bordas 0 e 2).
/// *Mutação: `pivo_da_alca` sem `compose_snapshot` ⇒ a borda oposta anda.*
#[test]
fn the_handle_of_a_mirrored_sprite_under_a_parent_keeps_the_far_border() {
    let (mut sim, scene) = (
        ph2d_ecs::SimWorld::default(),
        ph2d_vec_scene::VecScene::new(),
    );
    let sprite = ph2d_render::Sprite::atlas(0, [4.0, 2.0], [1.0; 4]);
    let e = sim.world_mut().spawn(sprite).id();
    let meio = gizmo_anchor_half(&sim, &scene, &ph2d_flip::FlipDoc::default(), e);
    assert_eq!(meio.1, [2.0, 1.0], "a caixa do sprite");
    let pai = Pose {
        translation: [1.0, 0.0],
        rotation: 0.0,
        scale: [0.5, 0.5],
    };
    perto(arrasta(meio, pai, [2.0, 0.0], [3.0, 0.0]), [0.0, 3.0, -1.5]);
}

use super::*;
use ph2d_ecs::{PresentWorld, Transform, Visibility};
use ph2d_vec_scene::rectangle;

/// Um grupo com duas formas (a 2.ª escondida) e uma forma solta; as transformações identidade.
fn cena() -> (SimWorld, VecScene, [Entity; 4], [u64; 3]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [2.0, 1.0]));
    let b = scene.push_path(rectangle([10.0, 10.0], [12.0, 11.0]));
    let c = scene.push_path(rectangle([-5.0, -5.0], [-4.0, -4.0]));
    let w = sim.world_mut();
    let grupo = w.spawn(Transform::IDENTITY).id();
    let ea = w
        .spawn((Transform::IDENTITY, VecPathRef(a), ChildOf(grupo)))
        .id();
    let eb = w
        .spawn((
            Transform::IDENTITY,
            VecPathRef(b),
            ChildOf(grupo),
            Visibility::hidden(),
        ))
        .id();
    let ec = w.spawn((Transform::IDENTITY, VecPathRef(c))).id();
    (sim, scene, [grupo, ea, eb, ec], [a, b, c])
}

fn igual(b: Option<WorldBbox>, min: [f32; 2], max: [f32; 2]) {
    let b = b.expect("sem caixa");
    assert!(
        (b.min[0] - min[0]).abs() < 1e-5
            && (b.min[1] - min[1]).abs() < 1e-5
            && (b.max[0] - max[0]).abs() < 1e-5
            && (b.max[1] - max[1]).abs() < 1e-5,
        "{b:?} ≠ {min:?}..{max:?}"
    );
}

/// ⭐ **Um grupo selecionado encaixa os filhos DESENHADOS** — o escondido fica de fora; uma forma é
/// a curva dela; um objecto sem nada desenhado não tem caixa.
#[test]
fn a_selected_group_frames_its_drawn_children_only() {
    let (mut sim, scene, [grupo, ea, _, ec], _) = cena();
    let mut present = PresentWorld::new();
    let xf = VecXforms::new();
    let mut nada = |_| None;
    igual(
        object_box(&sim, present.world_mut(), &scene, &xf, grupo, &mut nada),
        [0.0, 0.0],
        [2.0, 1.0],
    );
    igual(
        object_box(&sim, present.world_mut(), &scene, &xf, ec, &mut nada),
        [-5.0, -5.0],
        [-4.0, -4.0],
    );
    // O que a shell souber de outra família entra pela porta `other`.
    let mut flip = |e: Entity| {
        (e == ea).then_some(WorldBbox {
            min: [0.0, 0.0],
            max: [3.0, 3.0],
        })
    };
    igual(
        object_box(&sim, present.world_mut(), &scene, &xf, grupo, &mut flip),
        [0.0, 0.0],
        [3.0, 3.0],
    );
    let vazio = sim.world_mut().spawn(Transform::IDENTITY).id();
    assert!(object_box(&sim, present.world_mut(), &scene, &xf, vazio, &mut nada).is_none());
}

/// ⭐ **«Tudo» é o que a ORDEM do quadro desenha** — a forma que não entrou nela (a escondida)
/// fica de fora; as caixas das outras famílias somam; sem nada, `None` (a cena vazia).
#[test]
fn everything_is_what_the_frame_order_draws_plus_the_other_families() {
    let (_, scene, _, [a, _, c]) = cena();
    let mut present = PresentWorld::new();
    let xf = VecXforms::new();
    igual(
        scene_box(present.world_mut(), &scene, &xf, [a, c], []),
        [-5.0, -5.0],
        [2.0, 1.0],
    );
    let longe = WorldBbox {
        min: [30.0, 30.0],
        max: [31.0, 31.0],
    };
    igual(
        scene_box(present.world_mut(), &scene, &xf, [a], [longe]),
        [0.0, 0.0],
        [31.0, 31.0],
    );
    assert!(scene_box(present.world_mut(), &scene, &xf, [], []).is_none());
}

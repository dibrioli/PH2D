//! O círculo vivo que o solver de CAD moveu e re-escalou volta a ter a receita e o `Transform` certos — sem mexer no
//! que se vê, e sem re-escrever nada no quadro seguinte (cada escrita seria um passo de undo).

use super::*;
use ph2d_ecs::{MAX_SHAPE_VALUES, Name, Transform, VecPathRef};
use ph2d_vec_scene::{VecPath, ellipse};

fn mundo(scene: &VecScene, sim: &SimWorld, e: Entity, id: VecPathId) -> Vec<[f64; 2]> {
    let xf = crate::transform::xform_of_transform(crate::transform::world_transform(sim, e));
    scene
        .path(id)
        .unwrap()
        .verts
        .iter()
        .map(|v| xf.apply(v.anchor))
        .collect()
}

fn circulo_vivo() -> (SimWorld, VecScene, VecEntityMap, VecPathId, Entity) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(ellipse([0.0, 0.0], 5.0, 5.0));
    let e = sim
        .world_mut()
        .spawn((
            Transform {
                translation: ph2d_core::Vec2::new(30.0, 40.0),
                ..Transform::IDENTITY
            },
            Name::new("C"),
            VecPathRef(id),
            VecShape::Param {
                kind: ShapeKind::Ellipse as u16,
                w: 10.0,
                h: 10.0,
                values: [0.0; MAX_SHAPE_VALUES],
            },
        ))
        .id();
    map.insert(id, e.to_bits());
    (sim, scene, map, id, e)
}

#[test]
fn a_live_circle_the_solver_moved_gets_its_recipe_and_transform_back() {
    let (mut sim, mut scene, map, id, e) = circulo_vivo();
    // O que o solver escreve: o círculo noutro centro e noutro raio, no espaço LOCAL.
    let ids: Vec<_> = scene.path(id).unwrap().verts.iter().map(|v| v.id).collect();
    let mut movido: VecPath = ellipse([2.0, -1.0], 7.0, 7.0);
    for (v, i) in movido.verts.iter_mut().zip(&ids) {
        v.id = *i;
    }
    scene.path_mut(id).unwrap().verts = movido.verts;
    let visto = mundo(&scene, &sim, e, id);

    assenta_circulos_vivos(&mut sim, &mut scene, &map, &[id]);
    assert_eq!(mundo(&scene, &sim, e, id), visto, "em gesto: nada muda");

    assenta_circulos_vivos(&mut sim, &mut scene, &map, &[]);
    let Some(VecShape::Param { w, h, .. }) = sim.world().get::<VecShape>(e).cloned() else {
        panic!("a receita continua");
    };
    assert!(
        (w - 14.0).abs() < 1e-9 && (h - 14.0).abs() < 1e-9,
        "{w} × {h}"
    );
    let t = sim.world().get::<Transform>(e).unwrap().translation;
    assert!(
        (t.x - 32.0).abs() < 1e-4 && (t.y - 39.0).abs() < 1e-4,
        "{t:?}"
    );
    let (c, _) = circulo_de(scene.path(id).unwrap(), Xform::IDENTITY).unwrap();
    assert!(c[0].hypot(c[1]) < 1e-4, "centrado no local 0: {c:?}");
    for (a, b) in mundo(&scene, &sim, e, id).iter().zip(&visto) {
        assert!(
            (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4,
            "a forma não se move"
        );
    }

    let fixo = (scene.clone(), sim.world().get::<Transform>(e).copied());
    assenta_circulos_vivos(&mut sim, &mut scene, &map, &[]);
    assert!(
        (scene.clone(), sim.world().get::<Transform>(e).copied()) == fixo,
        "ponto fixo: o quadro seguinte não escreve"
    );
}

/// ⛔ CONTROLO: o círculo que a receita cozinhou é ponto fixo desde o início.
#[test]
fn a_live_circle_as_cooked_is_left_alone() {
    let (mut sim, mut scene, map, id, e) = circulo_vivo();
    let antes = (scene.clone(), sim.world().get::<Transform>(e).copied());
    assenta_circulos_vivos(&mut sim, &mut scene, &map, &[]);
    assert!((scene.clone(), sim.world().get::<Transform>(e).copied()) == antes);
    assert!(scene.path(id).is_some());
}

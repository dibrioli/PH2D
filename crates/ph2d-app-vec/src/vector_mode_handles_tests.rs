//! A cena 15 faz o que diz (o canto, o vale, o fim da pizza) — com as formas nascidas pelo MESMO
//! commit da ferramenta e tornadas vivas pela MESMA porta da shell.

use super::tests::cena;
use crate::shape_params::panel_shape_target;
use crate::vector_mode_handles::{FORMAS, step};
use ph2d_vec_entities::entities::sync;

#[test]
fn the_scene_pulls_a_corner_a_valley_and_the_end_of_a_pie() {
    let mut c = cena();
    for (kind, at, half) in FORMAS {
        crate::vector_mode::draw_shape(&mut c.vec, &mut c.scene, kind, at, half);
        sync(&mut c.sim, &mut c.scene, &mut c.vec.entities);
        crate::shape_live::make_committed_shape_live(
            &mut c.sim,
            &mut c.scene,
            &c.vec.entities,
            &mut c.vec.shape,
            false,
        );
    }
    let ids: Vec<_> = c.vec.entities.keys().copied().collect();
    let values = |c: &super::tests::Cena, k: usize| {
        panel_shape_target(&c.sim, &c.vec.entities, &[ids[k]])
            .expect("forma viva")
            .3
    };
    let before: Vec<_> = (0..3).map(|k| values(&c, k)).collect();
    c.vec.draw_config.mode = ph2d_tool_vector::DrawMode::Pen;
    assert_eq!(
        step(74, &mut c.vec, &mut c.scene, &mut c.sim, &mut c.hero),
        None,
        "sem o modo Forma na mão, espera"
    );
    c.vec.draw_config.mode = ph2d_tool_vector::DrawMode::Shape;
    for s in 74..=76 {
        assert!(
            step(s, &mut c.vec, &mut c.scene, &mut c.sim, &mut c.hero).is_some(),
            "o estágio {s} não agarrou a alça"
        );
    }
    let after: Vec<_> = (0..3).map(|k| values(&c, k)).collect();
    assert!(
        (after[0][0] - (before[0][0] + 0.45)).abs() < 1e-6,
        "o canto: {:?}",
        after[0]
    );
    assert!((after[1][1] - 0.65).abs() < 1e-6, "o vale: {:?}", after[1]);
    assert!(
        // A pose é `f32` (o `Transform`): o ângulo chega a ~2e-6 graus.
        (after[2][0] - 300.0).abs() < 1e-4,
        "o fim da pizza: {:?}",
        after[2]
    );
    assert!(
        c.vec.shape_handles.grab.is_none(),
        "o arrasto ficou pendurado"
    );
}

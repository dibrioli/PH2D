//! A cena 16 faz o que diz (a ponta arredonda, o vale torce, a aresta enverga, e o *Corner* do
//! cartão é clicado) — com a estrela nascida pelo MESMO commit da ferramenta e tornada viva pela
//! MESMA porta da shell.

use super::tests::cena;
use crate::shape_params::panel_shape_target;
use crate::vector_mode_card::{ESTRELA, step};
use ph2d_vec_entities::entities::sync;
use ph2d_vec_scene::ShapeKind;

#[test]
fn the_scene_rounds_the_tips_and_valleys_and_steps_to_seven_points() {
    let mut c = cena();
    crate::vector_mode::draw_shape(
        &mut c.vec,
        &mut c.scene,
        ShapeKind::Star,
        ESTRELA.0,
        ESTRELA.1,
    );
    sync(&mut c.sim, &mut c.scene, &mut c.vec.entities);
    crate::shape_live::make_committed_shape_live(
        &mut c.sim,
        &mut c.scene,
        &c.vec.entities,
        &mut c.vec.shape,
        false,
    );
    let id = *c.vec.entities.keys().next().expect("a estrela");
    let values = |c: &super::tests::Cena| {
        panel_shape_target(&c.sim, &c.vec.entities, &[id])
            .expect("forma viva")
            .3
    };
    c.vec.draw_config.mode = ph2d_tool_vector::DrawMode::Pen;
    assert_eq!(
        step(82, &mut c.vec, &mut c.scene, &mut c.sim, &mut c.hero),
        None,
        "sem o modo Forma na mão, espera"
    );
    c.vec.draw_config.mode = ph2d_tool_vector::DrawMode::Shape;
    for s in 82..=84 {
        assert!(
            step(s, &mut c.vec, &mut c.scene, &mut c.sim, &mut c.hero).is_some(),
            "o estágio {s} não agarrou a alça"
        );
    }
    let v = values(&c);
    assert!(v[2] > 0.05, "as pontas arredondaram: {v:?}");
    assert!(v[3] > 0.02, "os vales arredondaram: {v:?}");
    assert_eq!(v[0], 7.0, "o ⇕ subiu dois degraus: {v:?}");
    assert_eq!(v[7], 0.0, "nada torceu: {v:?}");
    assert!(
        (v[1] - ShapeKind::Star.defaults()[1]).abs() < 1e-12,
        "o raio interior ficou: {v:?}"
    );
    assert!(
        c.vec.shape_handles.grab.is_none(),
        "o arrasto ficou pendurado"
    );
}

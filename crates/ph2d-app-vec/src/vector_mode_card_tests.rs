//! A cena 16 faz o que diz (a ponta arredonda, o vale torce, a aresta enverga, e o *Corner* do
//! cartão é clicado) — com a estrela nascida pelo MESMO commit da ferramenta e tornada viva pela
//! MESMA porta da shell.

use super::tests::cena;
use crate::shape_params::panel_shape_target;
use crate::vector_mode_card::{ESTRELA, step};
use ph2d_vec_entities::entities::sync;
use ph2d_vec_scene::ShapeKind;

#[test]
fn the_scene_rounds_the_tips_twists_the_valley_bends_the_edges_and_clicks_the_card() {
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
    assert!(v[2] > 0.05, "a ponta arredondou: {v:?}");
    assert!((v[7] - 12.0).abs() < 0.5, "o vale torceu 12°: {v:?}");
    assert!(v[5] > 0.05, "a aresta envergou para fora: {v:?}");
    assert!(
        (v[1] - ShapeKind::Star.defaults()[1]).abs() < 0.02,
        "o raio do vale ficou: {v:?}"
    );
    assert_eq!(
        step(85, &mut c.vec, &mut c.scene, &mut c.sim, &mut c.hero),
        Some(86)
    );
    assert_eq!(
        step(86, &mut c.vec, &mut c.scene, &mut c.sim, &mut c.hero),
        Some(9)
    );
    assert!(
        c.vec.shape_handles.grab.is_none(),
        "o arrasto ficou pendurado"
    );
}

//! A cena 21 faz o que diz: o raio de cada uma das 36 formas sobe pelo MESMO press e arrasto da
//! shell, com as formas nascidas pelo commit da ferramenta e tornadas vivas pela porta da shell.

use super::tests::cena;
use crate::shape_params::panel_shape_target;
use crate::vector_mode_quinas::{FIRST, KINDS, LAST, MEIA, centre, round_handle, step};
use ph2d_vec_entities::entities::sync;

#[test]
fn the_scene_rounds_the_corners_of_every_shape() {
    let mut c = cena();
    for (i, kind) in KINDS.iter().enumerate() {
        crate::vector_mode::draw_shape(&mut c.vec, &mut c.scene, *kind, centre(i), MEIA);
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
    assert_eq!(ids.len(), KINDS.len());
    c.vec.draw_config.mode = ph2d_tool_vector::DrawMode::Shape;
    let pulls = FIRST + KINDS.len() as u8 + 1;
    for s in pulls..LAST {
        assert_eq!(
            step(s, &mut c.vec, &mut c.scene, &mut c.sim, &mut c.hero),
            Some(s + 1),
            "o estágio {s} não agarrou a alça de {:?}",
            KINDS[usize::from(s - pulls)]
        );
        assert!(
            c.vec.shape_handles.grab.is_none(),
            "o estágio {s} ficou pendurado"
        );
    }
    for (i, kind) in KINDS.iter().enumerate() {
        let f = kind.corner_round_field().expect("a forma tem o campo");
        assert!(round_handle(*kind).is_some(), "{kind:?}: a alça");
        let v = panel_shape_target(&c.sim, &c.vec.entities, &[ids[i]])
            .expect("forma viva")
            .3;
        assert!(v[f] > 0.05, "{kind:?}: o raio {}", v[f]);
    }
    assert_eq!(
        step(LAST, &mut c.vec, &mut c.scene, &mut c.sim, &mut c.hero),
        Some(9)
    );
}

//! A cena 19 faz o que diz (a aresta enverga, a estrela sobe a 6, a pizza abre o furo e arredonda as
//! quinas, a ponta da espiral segue a mão, um ⇕ fica a meio do arrasto com a etiqueta) — com as formas
//! nascidas pelo MESMO commit da ferramenta e tornadas vivas pela MESMA porta da shell.

use super::tests::cena;
use crate::shape_handles_live::{Gate, readout, view};
use crate::shape_params::panel_shape_target;
use crate::vector_mode_setas::{FORMAS, MEIA, alca, step, viva};
use ph2d_tool_vector::shape_handles::{Law, handles_of};
use ph2d_vec_entities::entities::sync;

#[test]
fn the_scene_moves_one_handle_of_each_shape_and_holds_an_arrow_mid_drag() {
    let mut c = cena();
    for (kind, at) in FORMAS {
        crate::vector_mode::draw_shape(&mut c.vec, &mut c.scene, kind, at, MEIA);
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
    c.vec.draw_config.mode = ph2d_tool_vector::DrawMode::Shape;
    // O índice da alça é o do ponto desenhado: nos valores de fábrica, toda alça tem ponto.
    for (k, (kind, _)) in FORMAS.iter().enumerate() {
        c.vec.pen.select_many(&[ids[k]]);
        let v = view(
            &c.sim,
            &c.vec.entities,
            &[ids[k]],
            Gate::of(&c.vec),
            0.01,
            None,
            None,
        );
        assert_eq!(
            v.map(|v| v.points.len()),
            Some(handles_of(*kind).len()),
            "{kind:?}"
        );
    }
    let before: Vec<_> = (0..4).map(|k| values(&c, k)).collect();
    for s in 115..=119 {
        assert!(
            step(s, &mut c.vec, &mut c.scene, &mut c.sim, &mut c.hero).is_some(),
            "o estágio {s} não agarrou a alça"
        );
        assert!(
            c.vec.shape_handles.grab.is_none(),
            "o estágio {s} ficou pendurado"
        );
    }
    let after: Vec<_> = (0..4).map(|k| values(&c, k)).collect();
    assert!(
        after[0][3].abs() > 1e-3,
        "a aresta do polígono: {:?}",
        after[0]
    );
    assert_eq!(after[1][0], before[1][0] + 1.0, "a estrela: {:?}", after[1]);
    assert!(after[2][2] > 0.1, "o furo da pizza: {:?}", after[2]);
    assert!(after[2][3] > 0.01, "as quinas da pizza: {:?}", after[2]);
    // ¼ de volta da mão = ¼ de volta da ponta — e a ponta DESENHADA está no ângulo da mão (às 9h).
    assert!(
        (after[3][0] - (before[3][0] + 0.25)).abs() < 1e-6,
        "{:?}",
        after[3]
    );
    let tip = alca(FORMAS[3].0, |h| h.law == Law::Winding).unwrap();
    let p = view(
        &c.sim,
        &c.vec.entities,
        &[ids[3]],
        Gate::of(&c.vec),
        0.01,
        None,
        None,
    )
    .unwrap()
    .points[tip];
    let c3 = FORMAS[3].1;
    let ang = (p[1] - c3[1]).atan2(p[0] - c3[0]).to_degrees();
    assert!((ang.abs() - 180.0).abs() < 1.0, "a ponta às {ang}°");
    // O ⇕ do Expansion fica agarrado, a meio do arrasto, e a etiqueta diz o número dele.
    assert!(step(120, &mut c.vec, &mut c.scene, &mut c.sim, &mut c.hero).is_some());
    let g = c.vec.shape_handles.grab.expect("o ⇕ fica agarrado");
    assert_eq!(Some(g.handle()), viva(FORMAS[3].0));
    assert!(
        values(&c, 3)[1] > 0.1,
        "o Expansion subiu: {:?}",
        values(&c, 3)
    );
    let label = readout(&c.vec, &c.sim, 0.01, Default::default(), None);
    assert!(
        label.as_deref().is_some_and(|s| s.starts_with("Expansion")),
        "{label:?}"
    );
}

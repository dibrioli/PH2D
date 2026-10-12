//! A cena 20 faz o que diz: armado na paleta, um clique na estrela selecciona-a, desarma e traz as
//! alças dela sem nós; armado de novo, um clique DENTRO da elipse passa a selecção — com as formas
//! nascidas pelo MESMO commit da ferramenta e o clique pelo MESMO Up da shell.

use super::tests::cena;
use crate::shape_handles_live::{Gate, hides_the_nodes, view};
use crate::vector_mode_seleciona::{FORMAS, MEIA, step};
use ph2d_tool_vector::shape_handles::handles_of;
use ph2d_vec_entities::entities::sync;

#[test]
fn the_scene_selects_with_the_shape_tool_and_brings_the_handles_back() {
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
    // As posições dos objectos, como a shell as dá à caneta a cada quadro (o pick lê-as).
    let xf = ph2d_vec_entities::transform::build(&c.sim, &c.vec.entities);
    c.vec.pen.set_xforms(xf);
    let ids: Vec<_> = c.vec.entities.keys().copied().collect();
    c.vec.draw_config.mode = ph2d_tool_vector::DrawMode::Shape;
    c.vec.pen.select_many(&[ids[1]]);
    let paths = c.scene.paths().len();
    for (k, s) in [(0, 134), (1, 136)] {
        // O latch da paleta (o quadro dos campos acende-o no quadro do clique no catálogo).
        c.vec.shape_armed = true;
        let alcas = |c: &super::tests::Cena| {
            let sel = c.vec.pen.selected_paths();
            view(&c.sim, &c.vec.entities, sel, Gate::of(&c.vec), 0.01, None, None)
        };
        assert!(alcas(&c).is_none(), "armado, a forma {k} já tinha alças");
        assert!(step(s, &mut c.vec, &mut c.scene, &mut c.hero).is_some());
        assert_eq!(c.vec.pen.selected(), Some(ids[k]), "o clique {s}");
        assert!(!c.vec.shape_armed, "o clique {s} não desarmou");
        // Nem toda alça tem ponto em todo valor (a elipse de fábrica mostra 3 das 4): 1..=todas.
        let n = alcas(&c).map_or(0, |v| v.points.len());
        assert!(
            (1..=handles_of(FORMAS[k].0).len()).contains(&n),
            "o clique {s} não trouxe as alças da forma {k}: {n}"
        );
        assert!(hides_the_nodes(&c.vec, &c.sim), "o clique {s}: nós à vista");
        if k == 0 {
            assert_eq!(step(135, &mut c.vec, &mut c.scene, &mut c.hero), Some(136));
        }
    }
    assert_eq!(c.scene.paths().len(), paths, "um clique deixou uma forma");
}

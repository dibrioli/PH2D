//! A sonda que MEDE a estrutura que a eliminação encontra, e os gates dela.

use super::{holonomy, weld};

/// ⭐⭐⭐ **A SONDA DA SOLDADURA** — quantas variáveis a eliminação apaga, e o que
/// sobra por eliminar.
///
/// ```text
/// cargo test -p ph2d-gridmap --release -- --ignored the_weld_measures --nocapture
/// ```
///
/// ⛔ **O que ela mede é o que a lei NÃO alcança:** as ligações que fecham ciclo. Uma
/// soldadura que só contasse as que eliminou diria sempre `100 %`.
#[test]
#[ignore = "sonda -- a estrutura da soldadura"]
fn the_weld_measures_what_elimination_cannot_reach() {
    for (name, mut mesh) in [
        ("esfera 24x36", ph2d_mesh::shapes::uv_sphere(24, 36, 1.0)),
        ("esfera fina 96x144", ph2d_mesh::shapes::uv_sphere(96, 144, 1.0)),
        ("toro 64x32", ph2d_mesh::shapes::torus(64, 32, 1.0, 0.35)),
    ] {
        let (cut, combed, h, singular) = crate::round::tests::chain(&mut mesh);
        let (w, mut r) = weld(&cut, &combed);
        let (map, rr) = crate::round::round_to_integers(
            &mesh,
            &cut,
            &combed,
            h,
            crate::round::RoundOptions::default(),
            &singular,
        );
        holonomy(&w, &map, &mut r);
        eprintln!(
            "{name}: {} copias -> {} classes ({} ligacoes: {} ELIMINADAS, {} fecham ciclo \
             = {} que rodam + {} planas) | holonomia p50 {:.4} max {:.4} | singulares {} \
             | costura {:.4} -> {:.4}",
            r.copies,
            r.classes,
            r.links,
            r.eliminated,
            r.closures,
            r.turning,
            r.flat,
            r.holonomy_p50,
            r.holonomy_max,
            singular.len(),
            rr.seam_before.1,
            rr.seam_after.1,
        );
    }
}

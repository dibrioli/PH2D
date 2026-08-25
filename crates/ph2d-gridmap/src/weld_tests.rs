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
        (
            "esfera fina 96x144",
            ph2d_mesh::shapes::uv_sphere(96, 144, 1.0),
        ),
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

/// Diagnóstico: de onde vêm as ligações que a contagem não fecha.
#[test]
#[ignore = "sonda -- a contagem das ligações"]
fn where_does_the_link_count_disagree() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(24, 36, 1.0);
    let (cut, combed, _h, _) = crate::round::tests::chain(&mut mesh);
    let (w, r) = weld(&cut, &combed);
    eprintln!(
        "ligacoes {} | eliminadas {} | fechos {} | soma {}",
        r.links,
        r.eliminated,
        r.closures,
        r.eliminated + r.closures
    );
    let mut selfloop = 0usize;
    let mut dup: std::collections::BTreeMap<(u32, u32), usize> = std::collections::BTreeMap::new();
    for c in w.closures() {
        if c.copies[0] == c.copies[1] {
            selfloop += 1;
        }
        *dup.entry((c.copies[0].min(c.copies[1]), c.copies[0].max(c.copies[1])))
            .or_default() += 1;
    }
    let repeated: usize = dup.values().filter(|&&n| n > 1).map(|n| n - 1).sum();
    eprintln!("fechos com a cópia repetida (auto-laço): {selfloop} | pares repetidos: {repeated}");
    // as ligações cruas: pares casados por costura com salto
    let mut raw = 0usize;
    let mut pairs: std::collections::BTreeMap<(u32, u32), usize> =
        std::collections::BTreeMap::new();
    for (s, seam) in cut.seams.iter().enumerate() {
        if combed.jump.get(s).copied().flatten().is_none() {
            continue;
        }
        for (la, lb) in seam.side[0].local.iter().zip(&seam.side[1].local) {
            if let (Some(la), Some(lb)) = (la, lb) {
                raw += 1;
                let (Some(ca), Some(cb)) = (
                    w.copy_index(seam.side[0].patch as usize, *la as usize),
                    w.copy_index(seam.side[1].patch as usize, *lb as usize),
                ) else {
                    continue;
                };
                *pairs.entry((ca.min(cb), ca.max(cb))).or_default() += 1;
            }
        }
    }
    let dupe_raw: usize = pairs.values().filter(|&&n| n > 1).map(|n| n - 1).sum();
    let loops = pairs.keys().filter(|(a, b)| a == b).count();
    eprintln!(
        "pares crus {raw} | pares distintos {} | repetidos {dupe_raw} | auto-laços {loops}",
        pairs.len()
    );
}

//! Os gates do sistema soldado, e a sonda que justifica as constantes dele.

use super::solve_welded;
use crate::solve::{ROUNDS as PENALISED_ROUNDS, SEAM_WEIGHT, solve_with};
use crate::weld::{seam_residual, weld};

/// ⭐⭐⭐ **A SONDA QUE COMPARA AS DUAS ESPÉCIES DE SISTEMA** (`CLAUDE.md` §0.0).
///
/// ```text
/// cargo test -p ph2d-gridmap --release -- --ignored the_welded_system --nocapture
/// ```
///
/// ⛔ **O que ela mede não é «a costura fechou»** — isso é trivialmente verdade quando a
/// variável que a mediria deixou de existir. Ela mede as DUAS colunas ao mesmo tempo: o
/// resíduo **e** o ângulo. *A tabela do [`SEAM_WEIGHT`] mostra que o penalizado não
/// consegue as duas; a pergunta desta sonda é se o soldado consegue.*
#[test]
#[ignore = "sonda -- soldado contra penalizado"]
fn the_welded_system_beats_the_penalised_one_on_both_columns() {
    use crate::weld_solve::{WeldOptions, solve_welded_with};
    for (name, mut mesh) in [
        ("esfera 24x36", ph2d_mesh::shapes::uv_sphere(24, 36, 1.0)),
        ("esfera fina 96x144", ph2d_mesh::shapes::uv_sphere(96, 144, 1.0)),
        ("toro 64x32", ph2d_mesh::shapes::torus(64, 32, 1.0, 0.35)),
    ] {
        let (cut, combed, h, _) = crate::round::tests::chain(&mut mesh);
        let (_, pen) = solve_with(&mesh, &cut, &combed, h, SEAM_WEIGHT, PENALISED_ROUNDS);
        eprintln!(
            "{name}\n  PENALIZADO (w={SEAM_WEIGHT}): angulo p50 {:.2}° | escala {:.3} \
             | costura p50 {:.4} max {:.4}",
            pen.angle_p50, pen.scale_p50, pen.seam_p50, pen.seam_max
        );
        for (label, opts) in [
            ("so as copias      ", WeldOptions { settle_flat: false, singular: false, gauge: false }),
            ("+ singulares      ", WeldOptions { settle_flat: false, singular: true, gauge: false }),
            ("+ CALIBRE         ", WeldOptions { settle_flat: false, singular: false, gauge: true }),
            ("+ CALIBRE+singular", WeldOptions { settle_flat: false, singular: true, gauge: true }),
        ] {
            let (map, r) = solve_welded_with(&mesh, &cut, &combed, h, 8_000, opts);
            let (w, _) = weld(&cut, &combed);
            let sr = seam_residual(&w, &map);
            eprintln!(
                "  SOLDADO {label}: angulo p50 {:>6.2}° | escala {:.3} | ⭐eliminadas max {:.2e} \
                 | ⛔rodam max {:>7.3} | ⛔planos max {:>7.3} | passo {:.2e}",
                r.solve.angle_p50, r.solve.scale_p50, sr.max, sr.turning_max, sr.flat_max, r.last_move
            );
        }
    }
}

/// Sonda de diagnóstico: onde é que o mapa deixa de ser um número.
#[test]
#[ignore = "sonda -- caca ao NaN"]
fn where_does_the_welded_map_stop_being_a_number() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(24, 36, 1.0);
    let (cut, combed, h, _) = crate::round::tests::chain(&mut mesh);
    let (w, wr) = weld(&cut, &combed);
    let mut solve_rep = crate::solve::SolveReport::default();
    let a = crate::solve::assemble(&mesh, &cut, &combed, h, &mut solve_rep);
    let r = crate::weld_solve::WeldRelaxer::new(&a, &w, &cut, &combed);
    let mut map = crate::solve::GridMap {
        uv: cut.origin.iter().map(|o| vec![[0.0f32; 2]; o.len()]).collect(),
        shift: vec![[0.0; 2]; cut.seams.len()],
    };
    eprintln!("fechos: {} ({} rodam) | derivadas {} | orfaos {}", wr.closures, wr.turning, wr.derived, wr.orphans);
    let bad = |m: &crate::solve::GridMap| -> (usize, usize) {
        (
            m.uv.iter().flatten().filter(|z| !z[0].is_finite() || !z[1].is_finite()).count(),
            m.shift.iter().filter(|t| !t[0].is_finite() || !t[1].is_finite()).count(),
        )
    };
    for (label, do_settle, do_sing) in [
        ("so classes+costuras", false, false),
        ("+ assentamento dos planos", true, false),
        ("+ passo singular", false, true),
        ("os dois", true, true),
    ] {
    eprintln!(" == {label} ==");
    for z in map.uv.iter_mut().flatten() { *z = [0.0, 0.0]; }
    for t in &mut map.shift { *t = [0.0, 0.0]; }
    for round in 0..12 {
        // uma varredura decomposta, para saber QUAL passo estraga
        let mut worst = 0.0f32;
        for c in 0..w.classes() {
            if !do_sing && w.singular_class(c).is_some() {
                continue;
            }
            worst = worst.max(r.relax_class(&mut map, c));
            let (u, s) = bad(&map);
            if u + s > 0 {
                eprintln!("  ronda {round}: NaN depois da CLASSE {c} (singular: {}) -> uv {u} shift {s}", w.singular_class(c).is_some());
                return;
            }
        }
        for s in 0..cut.seams.len() {
            r.relax_shift(&mut map, s);
            let (u, sh) = bad(&map);
            if u + sh > 0 {
                eprintln!("  ronda {round}: NaN depois da COSTURA {s} -> uv {u} shift {sh}");
                return;
            }
        }
        if do_settle {
            w.settle(&mut map, crate::weld_solve::SETTLE_PASSES);
        }
        let (u, sh) = bad(&map);
        if u + sh > 0 {
            eprintln!("  ronda {round}: NaN depois do SETTLE -> uv {u} shift {sh}");
            return;
        }
        let mx = map.uv.iter().flatten().fold(0.0f32, |m, z| m.max(z[0].abs()).max(z[1].abs()));
        let mt = map.shift.iter().fold(0.0f32, |m, t| m.max(t[0].abs()).max(t[1].abs()));
        if round % 4 == 3 || round < 2 {
            eprintln!("  ronda {round}: passo {worst:.3e} | |uv|max {mx:.3e} | |t|max {mt:.3e}");
        }
    }
    }
}

/// ⭐⭐⭐ **A TABELA DE DERIVADAS TEM DE PREVER O QUE ACONTECE** — o controlo directo.
///
/// ⚠️ *Um passo de Gauss–Seidel construído sobre uma derivada errada não dá erro
/// nenhum: ele diverge devagar e parece um solver mal condicionado.* Aqui a derivada é
/// **medida**: mexe-se `t` e confere-se onde a cópia foi parar.
#[test]
fn the_crossings_predict_how_a_translation_moves_a_copy() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(24, 36, 1.0);
    let (cut, combed, _h, _) = crate::round::tests::chain(&mut mesh);
    let (w, _) = weld(&cut, &combed);
    let mut map = crate::solve::GridMap {
        uv: cut.origin.iter().map(|o| vec![[0.0f32; 2]; o.len()]).collect(),
        shift: vec![[0.0; 2]; cut.seams.len()],
    };
    // Um mapa qualquer, mas determinista: as raízes recebem um valor e o resto deriva.
    for c in 0..w.classes() {
        #[allow(clippy::cast_precision_loss)]
        let t = c as f32;
        w.set(&mut map, c, [0.5 * t.sin(), 0.25 * t.cos()]);
    }
    let mut checked = 0usize;
    for s in 0..cut.seams.len() {
        if w.crossings(s).is_empty() {
            continue;
        }
        let before: Vec<[f32; 2]> = w
            .crossings(s)
            .iter()
            .map(|&(c, _)| {
                let (p, l) = w.where_is_pub(c);
                map.uv[p as usize][l as usize]
            })
            .collect();
        let eps = [0.125f32, -0.375];
        map.shift[s][0] += eps[0];
        map.shift[s][1] += eps[1];
        for &c in w.shift_classes_pub(s) {
            w.derive(&mut map, c as usize);
        }
        for (i, &(c, m)) in w.crossings(s).iter().enumerate() {
            let (p, l) = w.where_is_pub(c);
            let now = map.uv[p as usize][l as usize];
            let want = crate::solve::turn2(eps, m);
            let got = [now[0] - before[i][0], now[1] - before[i][1]];
            assert!(
                (got[0] - want[0]).abs() < 1.0e-4 && (got[1] - want[1]).abs() < 1.0e-4,
                "costura {s}, cópia {c}: a travessia diz R^{m}·ε = {want:?}, o mapa moveu {got:?}"
            );
            checked += 1;
        }
        map.shift[s][0] -= eps[0];
        map.shift[s][1] -= eps[1];
        for &c in w.shift_classes_pub(s) {
            w.derive(&mut map, c as usize);
        }
    }
    assert!(checked > 100, "a fixtura tem de conter o fenómeno: {checked} travessias");
}

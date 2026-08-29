//! Os gates do sistema soldado, e as sondas que justificam as constantes dele.

use crate::solve::{ROUNDS as PENALISED_ROUNDS, SEAM_WEIGHT, solve_with};
use crate::weld::{seam_residual, weld};
use crate::weld_solve_driver::solve_welded;

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
        uv: cut
            .origin
            .iter()
            .map(|o| vec![[0.0f32; 2]; o.len()])
            .collect(),
        shift: vec![[0.0; 2]; cut.seams.len()],
    };
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
    assert!(
        checked > 100,
        "a fixtura tem de conter o fenómeno: {checked} travessias"
    );
}

/// ⭐⭐⭐ **GATE nº1 DA ESPEC — o resíduo de uma ligação ELIMINADA é o chão da
/// representação, e não uma folga.**
///
/// A igualdade `z_b = R^k·z_a + t` é **exacta em ℝ** depois da eliminação: a variável
/// que a mediria deixou de existir. O que a régua lê é a diferença entre associar a
/// mesma soma de duas maneiras em `f32` — ⛔ um *erro de avaliação*, não uma tolerância.
///
/// ⚠️ **A barra é DERIVADA:** `|z|·ε` com `ε = 2⁻²³`. Nas peças, `|z| ≲ 64` células ⇒
/// `≲ 7,6e-6`. *Escrever aqui o `3,5e-15` dos mapas de referência seria copiar um número
/// de `f64` para um mapa de `f32`* — a espec nomeia esta emenda, e ela é do Enio.
#[test]
fn an_eliminated_seam_link_is_closed_to_the_floor_of_f32() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(24, 36, 1.0);
    let (cut, combed, h, _) = crate::round::tests::chain(&mut mesh);
    let (map, _) = solve_welded(&mesh, &cut, &combed, crate::solve::Step::uniform(h), 2_000);
    let (w, _) = weld(&cut, &combed);
    let sr = seam_residual(&w, &map);
    let biggest = map
        .uv
        .iter()
        .flatten()
        .fold(0.0f32, |m, z| m.max(z[0].abs()).max(z[1].abs()));
    let bar = 8.0 * biggest * f32::EPSILON;
    assert!(
        sr.links > 400,
        "a fixtura tem de conter o fenómeno: {} ligações eliminadas",
        sr.links
    );
    assert!(
        sr.p50 == 0.0,
        "a mediana das eliminadas tem de ser ZERO exacto, deu {:.3e}",
        sr.p50
    );
    assert!(
        sr.max <= bar,
        "o pior resíduo de uma ligação ELIMINADA foi {:.3e}, e o chão de `f32` para \
         |z| = {biggest:.1} é {bar:.3e}",
        sr.max
    );
}

/// ⭐⭐⭐ **A SONDA QUE COMPARA AS DUAS ESPÉCIES DE SISTEMA** (`CLAUDE.md` §0.0).
///
/// ```text
/// cargo test -p ph2d-gridmap --release -- --ignored the_welded_system --nocapture
/// ```
///
/// ⛔ **O que ela mede não é «a costura fechou»** — isso é trivialmente verdade quando a
/// variável que a mediria deixou de existir. Ela mede as DUAS colunas ao mesmo tempo: o
/// resíduo **e** o ângulo. *A tabela do [`SEAM_WEIGHT`] mostra que o penalizado não
/// consegue as duas.*
#[test]
#[ignore = "sonda -- soldado contra penalizado"]
fn the_welded_system_beats_the_penalised_one_on_both_columns() {
    for (name, mut mesh) in [
        ("esfera 24x36", ph2d_mesh::shapes::uv_sphere(24, 36, 1.0)),
        (
            "esfera fina 96x144",
            ph2d_mesh::shapes::uv_sphere(96, 144, 1.0),
        ),
        ("toro 64x32", ph2d_mesh::shapes::torus(64, 32, 1.0, 0.35)),
    ] {
        let (cut, combed, h, _) = crate::round::tests::chain(&mut mesh);
        let t = std::time::Instant::now();
        let (_, pen) = solve_with(&mesh, &cut, &combed, crate::solve::Step::uniform(h), SEAM_WEIGHT, PENALISED_ROUNDS);
        eprintln!(
            "{name}\n  PENALIZADO (w={SEAM_WEIGHT}, {:.1}s): angulo p50 {:.2}° | escala {:.3} \
             | costura p50 {:.4} max {:.4}",
            t.elapsed().as_secs_f64(),
            pen.angle_p50,
            pen.scale_p50,
            pen.seam_p50,
            pen.seam_max
        );
        for rounds in [500usize, 2_000, 8_000] {
            let t = std::time::Instant::now();
            let (map, r) = solve_welded(&mesh, &cut, &combed, crate::solve::Step::uniform(h), rounds);
            let (w, _) = weld(&cut, &combed);
            let sr = seam_residual(&w, &map);
            eprintln!(
                "  SOLDADO ({rounds:>5} rondas, {:>5.1}s): angulo p50 {:>6.2}° | escala {:.3} \
                 | ⭐eliminadas ({}) p50 {:.1e} max {:.2e} | fechos: rodam {:.3} planos {:.3} \
                 | passo {:.2e}",
                t.elapsed().as_secs_f64(),
                r.solve.angle_p50,
                r.solve.scale_p50,
                sr.links,
                sr.p50,
                sr.max,
                sr.turning_max,
                sr.flat_max,
                r.last_move
            );
            eprintln!(
                "        sistema: {} classes | {} ligacoes eliminadas | {} equacoes de fecho, \
                 {} eliminaram, ⛔{} orfas | ciclo {} | pior |det| {:.1} | {} livres",
                r.weld.classes,
                r.weld.eliminated,
                r.flat.equations,
                r.flat.resolved,
                r.flat.orphans,
                r.flat.cyclic,
                r.flat.worst_det,
                r.flat.equations + r.weld.eliminated - r.flat.resolved - r.weld.eliminated,
            );
        }
    }
}

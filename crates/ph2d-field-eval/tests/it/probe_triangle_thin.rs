//! Sonda (`#[ignore]`): o triângulo FINO fura — é o `max` ou é o FILETE?
use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
use ph2d_field_eval::Field;

fn pior(p: Primitive, e: f64) -> f64 {
    let doc =
        FieldDoc::new(vec![ph2d_field_eval::leaf(p, Xform::IDENTITY)], NodeId(0)).expect("peça");
    let f = Field::new(&doc);
    let n = 40;
    let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / n as f64;
    let mut m = 0.0_f64;
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let g = f.gradient_norm(at(i), at(j), at(k), 1.0e-4);
                if g.is_finite() {
                    m = m.max(g);
                }
            }
        }
    }
    m * f64::from(ph2d_field_eval::safe_march_step(&doc))
}

#[test]
#[ignore = "sonda: o triângulo fino"]
fn probe_triangle_thin() {
    let tri = |ax: f32, round: f32| Primitive::Triangle {
        a: [ax, -0.20],
        b: [-0.10, 0.36],
        c: [0.32, -0.08],
        half_height: 0.13,
        round,
        chamfer: 0.0,
    };
    println!("\n── o `ax` a afastar-se (passo × ‖∇f‖; barra 1,02) ──");
    println!(
        "  {:>8} {:>12} {:>12}   menor ângulo",
        "ax", "filete 0", "filete 0,02"
    );
    for ax in [-0.34_f32, -0.6, -1.0, 0.34, 0.68, 1.36] {
        // o menor ângulo do triângulo
        let pts = [[ax, -0.20_f32], [-0.10, 0.36], [0.32, -0.08]];
        let ang = |i: usize| {
            let (p, q, r) = (pts[i], pts[(i + 1) % 3], pts[(i + 2) % 3]);
            let (u, v) = ([q[0] - p[0], q[1] - p[1]], [r[0] - p[0], r[1] - p[1]]);
            let d = (u[0] * v[0] + u[1] * v[1]) / ((u[0].hypot(u[1])) * (v[0].hypot(v[1])));
            f64::from(d.clamp(-1.0, 1.0)).acos().to_degrees()
        };
        let menor = ang(0).min(ang(1)).min(ang(2));
        let limite = ph2d_field::round_limit(&tri(ax, 0.0)).unwrap_or(0.0);
        let r = 0.02_f32.min(limite * 0.9);
        println!(
            "  {ax:>8.2} {:>12.4} {:>12.4}   {menor:>6.1}°  (filete {r:.4}, tecto {limite:.4})",
            pior(tri(ax, 0.0), 1.6),
            pior(tri(ax, r), 1.6)
        );
    }
}

/// A régua HONESTA: a constante de Lipschitz **pela definição** — `|f(p) − f(q)| ≤ L·|p − q|` —, que
/// é imune ao artefacto da diferença central sobre um vinco.
#[test]
#[ignore = "sonda: o campo é mesmo 1-Lipschitz?"]
fn probe_triangle_lipschitz() {
    let tri = |ax: f32, round: f32| Primitive::Triangle {
        a: [ax, -0.20],
        b: [-0.10, 0.36],
        c: [0.32, -0.08],
        half_height: 0.13,
        round,
        chamfer: 0.0,
    };
    println!("\n── L pela DEFINIÇÃO contra o que a diferença central lê ──");
    println!(
        "  {:>8} {:>14} {:>16}",
        "ax", "L (definição)", "diferença central"
    );
    for ax in [-0.34_f32, -1.0, 0.34, 0.68, 1.36] {
        let doc = FieldDoc::new(
            vec![ph2d_field_eval::leaf(tri(ax, 0.0), Xform::IDENTITY)],
            NodeId(0),
        )
        .expect("peça");
        let f = Field::new(&doc);
        let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
        // Pares aleatórios-determinísticos na caixa.
        let mut sem = 12_345_u64;
        let mut rnd = || {
            sem = sem.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            ((sem >> 33) as f64 / f64::from(u32::MAX)) * 3.2 - 1.6
        };
        let mut pior = 0.0_f64;
        for _ in 0..40_000 {
            let p = [rnd(), rnd(), rnd()];
            let q = [rnd(), rnd(), rnd()];
            let d = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
            if d < 1.0e-6 {
                continue;
            }
            let l = (f.at(p[0], p[1], p[2]) - f.at(q[0], q[1], q[2])).abs() / d;
            pior = pior.max(l);
        }
        let mut central = 0.0_f64;
        let n = 40;
        let at = |t: usize| -1.6 + 3.2 * (t as f64 + 0.5) / n as f64;
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    let g = f.gradient_norm(at(i), at(j), at(k), 1.0e-4);
                    if g.is_finite() {
                        central = central.max(g);
                    }
                }
            }
        }
        println!(
            "  {ax:>8.2} {:>14.4} {:>16.4}",
            pior * passo,
            central * passo
        );
    }
    println!("\n  ⚠️ se a coluna da DEFINIÇÃO ficar em 1,00 e a outra não, o defeito é da RÉGUA.");
}

/// Onde o campo do triângulo parte, contra o MENOR ÂNGULO — a régua é a definição de Lipschitz.
///
/// ⚠️ **Mede o OPERADOR e não o documento**: a cerca da lasca recusa exactamente as peças que esta
/// sonda precisa de ver, e *uma régua que não alcança o regime que se quer medir não mede nada*.
#[test]
#[ignore = "sonda: L contra o menor ângulo"]
fn probe_triangle_by_angle() {
    // O passo da marcha com os dois recuos a zero é o `SAFE_STEP` da casa.
    const PASSO: f64 = std::f64::consts::FRAC_1_SQRT_2;
    let l_de = |t: &fidget::context::Tree| {
        let f = Field::from_tree(t);
        let mut sem = 987_654_321_u64;
        let mut rnd = || {
            sem = sem.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            ((sem >> 33) as f64 / f64::from(u32::MAX)) * 3.0 - 1.5
        };
        let mut pior = 0.0_f64;
        for _ in 0..60_000 {
            let p = [rnd(), rnd(), rnd()];
            let q = [rnd(), rnd(), rnd()];
            let d = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
            if d < 1.0e-6 {
                continue;
            }
            pior = pior.max((f.at(p[0], p[1], p[2]) - f.at(q[0], q[1], q[2])).abs() / d);
        }
        pior * PASSO
    };
    println!("\n── L contra o MENOR ÂNGULO (barra 1,02) ──");
    println!(
        "  {:>8} {:>10} {:>10} {:>10} {:>10}",
        "ângulo", "filete 0", "inraio", "f=0,9·inr", "c=0,9·inr"
    );
    for graus in [90.0_f64, 60.0, 40.0, 25.0, 15.0, 10.0, 6.0, 3.0] {
        let t = (graus.to_radians() * 0.5).tan();
        let (ax, ay) = (0.35 * t, 0.35);
        let (a, b, c) = ([-ax, -ay], [ax, -ay], [0.0, ay]);
        let inr = ph2d_field_eval::ops_triangle::inradius(a, b, c).min(0.13);
        let tri =
            |r: f64, ch: f64| ph2d_field_eval::ops_triangle::sd_triangle(a, b, c, 0.13, r, ch);
        println!(
            "  {graus:>7.0}° {:>10.4} {inr:>10.4} {:>10.4} {:>10.4}",
            l_de(&tri(0.0, 0.0)),
            l_de(&tri(inr * 0.9, 0.0)),
            l_de(&tri(0.0, inr * 0.9))
        );
    }
}

/// O caso EXACTO que o censo acusa — medido pelas duas réguas.
#[test]
#[ignore = "sonda: o caso do censo"]
fn probe_triangle_census_case() {
    // O representante do censo com `ay` arrastado para `0,800`.
    for (nome, a, round) in [
        ("nascimento", [-0.34_f32, -0.20_f32], 0.02_f32),
        ("ay = 0,800", [-0.34, 0.80], 0.02),
        ("ay = 0,800, filete 0", [-0.34, 0.80], 0.0),
    ] {
        let p = Primitive::Triangle {
            a,
            b: [-0.10, 0.36],
            c: [0.32, -0.08],
            half_height: 0.13,
            round,
            chamfer: 0.0,
        };
        let mut q = p;
        ph2d_field::clamp_dims(&mut q);
        let doc = FieldDoc::new(
            vec![ph2d_field_eval::leaf(q.clone(), Xform::IDENTITY)],
            NodeId(0),
        );
        let Ok(doc) = doc else {
            println!("  {nome}: o DOCUMENTO recusa");
            continue;
        };
        let f = Field::new(&doc);
        let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
        let mut sem = 424_242_u64;
        let mut rnd = || {
            sem = sem.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            ((sem >> 33) as f64 / f64::from(u32::MAX)) * 3.0 - 1.5
        };
        let mut def = 0.0_f64;
        let (mut px, mut py) = ([0.0; 3], [0.0; 3]);
        for _ in 0..80_000 {
            let u = [rnd(), rnd(), rnd()];
            let v = [rnd(), rnd(), rnd()];
            let d = ((u[0] - v[0]).powi(2) + (u[1] - v[1]).powi(2) + (u[2] - v[2]).powi(2)).sqrt();
            if d < 1.0e-6 {
                continue;
            }
            let l = (f.at(u[0], u[1], u[2]) - f.at(v[0], v[1], v[2])).abs() / d;
            if l > def {
                def = l;
                px = u;
                py = v;
            }
        }
        let mut central = 0.0_f64;
        let n = 40;
        let at = |t: usize| -1.5 + 3.0 * (t as f64 + 0.5) / n as f64;
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    let g = f.gradient_norm(at(i), at(j), at(k), 1.0e-4);
                    if g.is_finite() {
                        central = central.max(g);
                    }
                }
            }
        }
        let (r2, c2) = match q {
            Primitive::Triangle { round, chamfer, .. } => (round, chamfer),
            _ => (0.0, 0.0),
        };
        println!(
            "  {nome:<22} definição {:.4}  central {:.4}   (passo {passo:.4}, filete depois do clamp {r2:.4}/{c2:.4})",
            def * passo,
            central * passo
        );
        if def * passo > 1.02 {
            println!("      o pior par: {px:?} → {py:?}");
        }
    }
}

/// Qual cosseno a SEGUNDA junta deve levar? Ela forma DUAS quinas e só leva um número.
#[test]
#[ignore = "sonda: o cosseno da segunda junta"]
fn probe_triangle_second_join_cos() {
    use ph2d_field_eval::ops_triangle::{cos_corner_of, sd_triangle_with};
    const PASSO: f64 = std::f64::consts::FRAC_1_SQRT_2;
    let l = |t: &fidget::context::Tree| {
        let f = Field::from_tree(t);
        let mut sem = 777_u64;
        let mut rnd = || {
            sem = sem.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            ((sem >> 33) as f64 / f64::from(u32::MAX)) * 3.0 - 1.5
        };
        let mut pior = 0.0_f64;
        for _ in 0..60_000 {
            let u = [rnd(), rnd(), rnd()];
            let v = [rnd(), rnd(), rnd()];
            let d = ((u[0] - v[0]).powi(2) + (u[1] - v[1]).powi(2) + (u[2] - v[2]).powi(2)).sqrt();
            if d < 1.0e-6 {
                continue;
            }
            pior = pior.max((f.at(u[0], u[1], u[2]) - f.at(v[0], v[1], v[2])).abs() / d);
        }
        pior * PASSO
    };
    println!("\n── o cosseno da 2.ª junta (barra 1,02; o filete divide o passo, daí o ×0,7071) ──");
    println!(
        "  {:>26} {:>10} {:>10} {:>10} {:>10}",
        "caso", "zero", "menor", "maior", "média"
    );
    for (nome, a, b, c) in [
        (
            "censo: ay = 0,800",
            [-0.34_f64, 0.80_f64],
            [-0.10_f64, 0.36_f64],
            [0.32_f64, -0.08_f64],
        ),
        ("nascimento", [-0.34, -0.20], [-0.10, 0.36], [0.32, -0.08]),
        (
            "isósceles 10°",
            [-0.0306, -0.35],
            [0.0306, -0.35],
            [0.0, 0.35],
        ),
        ("obtuso 150°", [-0.35, 0.0], [0.35, 0.0], [0.0, 0.047]),
    ] {
        let c1 = cos_corner_of(a, c, b);
        let c2 = cos_corner_of(c, b, a);
        let mut linha = format!("  {nome:>26}");
        for cos2 in [0.0, c1.min(c2), c1.max(c2), (c1 + c2) * 0.5] {
            linha.push_str(&format!(
                " {:>10.4}",
                l(&sd_triangle_with(
                    a,
                    b,
                    c,
                    0.13,
                    0.02,
                    0.0,
                    cos_corner_of(b, a, c),
                    cos2
                ))
            ));
        }
        println!("{linha}   (as duas quinas: {c1:+.3} / {c2:+.3})");
    }
}

/// ⭐ **Quanto filete uma quina AGUDA aguenta** — a fronteira que escolhe o `round_limit`.
#[test]
#[ignore = "sonda: o filete contra a quina"]
fn probe_triangle_fillet_vs_corner() {
    use ph2d_field_eval::ops_triangle::{inradius, sd_triangle};
    const PASSO: f64 = std::f64::consts::FRAC_1_SQRT_2;
    let l = |t: &fidget::context::Tree| {
        let f = Field::from_tree(t);
        let mut sem = 31_337_u64;
        let mut rnd = || {
            sem = sem.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            ((sem >> 33) as f64 / f64::from(u32::MAX)) * 3.0 - 1.5
        };
        let mut pior = 0.0_f64;
        for _ in 0..40_000 {
            let u = [rnd(), rnd(), rnd()];
            let v = [rnd(), rnd(), rnd()];
            let d = ((u[0] - v[0]).powi(2) + (u[1] - v[1]).powi(2) + (u[2] - v[2]).powi(2)).sqrt();
            if d < 1.0e-6 {
                continue;
            }
            pior = pior.max((f.at(u[0], u[1], u[2]) - f.at(v[0], v[1], v[2])).abs() / d);
        }
        pior * PASSO
    };
    // Um triângulo com DUAS quinas agudas iguais — o regime que parte.
    println!("\n── L com o filete a uma fracção do INRAIO (barra 1,02) ──");
    println!("  {:>10} {:>8}", "quina", "inraio");
    print!("  {:>10} {:>8}", "", "");
    for f in [0.9_f64, 0.6, 0.4, 0.25, 0.15, 0.08, 0.0] {
        print!(" {:>7.2}", f);
    }
    println!();
    for graus in [60.0_f64, 30.0, 15.0, 10.0, 7.0, 5.0, 3.0] {
        // isósceles com a BASE longa: dois ângulos iguais a `graus` e o ápice obtuso.
        let t = graus.to_radians().tan();
        let (a, b, c) = ([-0.6, 0.0], [0.6, 0.0], [0.0, 0.6 * t]);
        let inr = inradius(a, b, c);
        let mut linha = format!("  {graus:>9.0}° {inr:>8.4}");
        for frac in [0.9_f64, 0.6, 0.4, 0.25, 0.15, 0.08, 0.0] {
            let r = inr * frac;
            linha.push_str(&format!(" {:>7.3}", l(&sd_triangle(a, b, c, 0.13, r, 0.0))));
        }
        println!("{linha}");
    }
}

/// O caso do censo, com o FILETE a varrer e os dois cossenos à vista.
#[test]
#[ignore = "sonda: o filete no caso do censo"]
fn probe_triangle_census_fillet() {
    use ph2d_field_eval::ops_triangle::{cos_corner_of, inradius, sd_triangle};
    const PASSO: f64 = std::f64::consts::FRAC_1_SQRT_2;
    let l = |t: &fidget::context::Tree| {
        let f = Field::from_tree(t);
        let mut sem = 5_150_u64;
        let mut rnd = || {
            sem = sem.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            ((sem >> 33) as f64 / f64::from(u32::MAX)) * 3.0 - 1.5
        };
        let mut pior = 0.0_f64;
        for _ in 0..40_000 {
            let u = [rnd(), rnd(), rnd()];
            let v = [rnd(), rnd(), rnd()];
            let d = ((u[0] - v[0]).powi(2) + (u[1] - v[1]).powi(2) + (u[2] - v[2]).powi(2)).sqrt();
            if d < 1.0e-6 {
                continue;
            }
            pior = pior.max((f.at(u[0], u[1], u[2]) - f.at(v[0], v[1], v[2])).abs() / d);
        }
        pior * PASSO
    };
    let (a, b, c) = (
        [-0.34_f64, 0.80_f64],
        [-0.10_f64, 0.36_f64],
        [0.32_f64, -0.08_f64],
    );
    let inr = inradius(a, b, c);
    println!("\n── o caso do censo: inraio {inr:.4}, meia-altura 0,13 ──");
    println!(
        "  as três quinas: A {:+.4}  B {:+.4}  C {:+.4}",
        cos_corner_of(a, b, c),
        cos_corner_of(b, a, c),
        cos_corner_of(c, a, b)
    );
    println!("  {:>10} {:>10}", "filete", "L");
    for frac in [0.0_f64, 0.02, 0.05, 0.1, 0.2, 0.4, 0.6, 0.9] {
        let r = inr * frac;
        println!(
            "  {:>10.5} {:>10.4}",
            r,
            l(&sd_triangle(a, b, c, 0.13, r, 0.0))
        );
    }
    println!("\n  ⚠️ e com a MEIA-ALTURA a mudar (filete a 0,55 do inraio):");
    for h in [0.13_f64, 0.06, 0.03, 0.02, 0.01] {
        println!(
            "   h = {h:.3}: {:>8.4}",
            l(&sd_triangle(a, b, c, h, inr * 0.55, 0.0))
        );
    }
}

/// ⭐ **Até onde a LASCA aguenta** — a margem da cerca `MIN_TRIANGLE_INRADIUS_OVER_SIDE`.
#[test]
#[ignore = "sonda: a margem da cerca da lasca"]
fn probe_triangle_sliver_margin() {
    use ph2d_field_eval::ops_triangle::{inradius, sd_triangle};
    const PASSO: f64 = std::f64::consts::FRAC_1_SQRT_2;
    let l = |t: &fidget::context::Tree| {
        let f = Field::from_tree(t);
        let mut sem = 20_260_906_u64;
        let mut rnd = || {
            sem = sem.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            ((sem >> 33) as f64 / f64::from(u32::MAX)) * 3.0 - 1.5
        };
        let mut pior = 0.0_f64;
        for _ in 0..40_000 {
            let u = [rnd(), rnd(), rnd()];
            let v = [rnd(), rnd(), rnd()];
            let d = ((u[0] - v[0]).powi(2) + (u[1] - v[1]).powi(2) + (u[2] - v[2]).powi(2)).sqrt();
            if d < 1.0e-6 {
                continue;
            }
            pior = pior.max((f.at(u[0], u[1], u[2]) - f.at(v[0], v[1], v[2])).abs() / d);
        }
        pior * PASSO
    };
    println!("\n── a LASCA: L contra o inraio/maior-lado (barra 1,02) ──");
    println!(
        "  {:>12} {:>10} {:>10} {:>10}",
        "inr/lado", "inraio", "L (filete 0)", "L (filete máx)"
    );
    for altura in [0.30_f64, 0.10, 0.03, 0.01, 0.003, 0.001, 0.0003] {
        let (a, b, c) = ([-0.6, 0.0], [0.6, 0.0], [0.1, altura]);
        let inr = inradius(a, b, c);
        let maior = 1.2_f64
            .max((0.5_f64).hypot(altura))
            .max((0.7_f64).hypot(altura));
        let r = inr.min(0.13) * 0.999;
        println!(
            "  {:>12.5} {inr:>10.5} {:>10.4} {:>10.4}",
            inr / maior,
            l(&sd_triangle(a, b, c, 0.13, 0.0, 0.0)),
            l(&sd_triangle(a, b, c, 0.13, r, 0.0))
        );
    }
}

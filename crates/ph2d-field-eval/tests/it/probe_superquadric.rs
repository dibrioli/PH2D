//! Sonda (`#[ignore]`) da W127: a fórmula fechada do divisor bate a MEDIÇÃO? E onde ficam as cercas?
use ph2d_field_eval::{Field, ops_super};

/// O maior `‖∇f‖` numa casca densa em volta da superfície, mais uma varredura grossa da caixa.
fn medido(half: [f64; 3], n_xy: f64, n_z: f64) -> f64 {
    let t = ops_super::sd_superquadric(half, n_xy, n_z);
    let f = Field::from_tree(&t);
    let e = half[0].max(half[1]).max(half[2]) * 2.5;
    let mut pior = 0.0_f64;
    let mut varre = |n: usize, e: f64, banda: Option<f64>| {
        let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / n as f64;
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    let (x, y, z) = (at(i), at(j), at(k));
                    if banda.is_some_and(|b| f.at(x, y, z).abs() > b) {
                        continue;
                    }
                    let g = f.gradient_norm(x, y, z, 1.0e-5);
                    if g.is_finite() {
                        pior = pior.max(g);
                    }
                }
            }
        }
    };
    varre(28, e, None);
    varre(90, e, Some(0.02));
    pior
}

#[test]
#[ignore]
fn probe_superquadric() {
    println!("── o divisor FECHADO contra a MEDIÇÃO (‖∇f‖ do campo já dividido; alvo ≤ 1,00) ──");
    println!("   (uma leitura acima de 1 significa que a marcha atravessa a superfície)\n");
    let cubica = [0.35, 0.35, 0.35];
    let torta = [0.42, 0.24, 0.30];
    for (nome, h) in [("cúbica", cubica), ("torta", torta)] {
        println!("  ── meia-medida {nome} = {h:?}");
        for n_xy in [1.0_f64, 1.3, 1.7, 2.0, 3.0, 5.0, 8.0, 16.0, 32.0] {
            let mut linha = format!("    n_xy = {n_xy:>5.1}:");
            for n_z in [1.0_f64, 2.0, 4.0, 16.0] {
                linha.push_str(&format!("  n_z {n_z:>4.1} -> {:.4}", medido(h, n_xy, n_z)));
            }
            println!("{linha}");
        }
    }

    println!(
        "\n── ABAIXO da cerca `n = 1`: o gradiente CRU na superfície (sem divisor honesto) ──"
    );
    for n in [1.0_f64, 0.9, 0.8, 0.6, 0.4] {
        // ⚠️ Aqui o divisor é o de `n = 1` de propósito: abaixo de 1 a fórmula não tem limite, e o
        // que se quer ver é **quanto** ela dispara.
        let t = ops_super::sd_superquadric(cubica, n, n);
        let f = Field::from_tree(&t);
        let e = 0.9;
        let n_g = 100;
        let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / n_g as f64;
        let mut pior = 0.0_f64;
        for i in 0..n_g {
            for j in 0..n_g {
                for k in 0..n_g {
                    let (x, y, z) = (at(i), at(j), at(k));
                    if f.at(x, y, z).abs() > 0.02 {
                        continue;
                    }
                    let g = f.gradient_norm(x, y, z, 1.0e-5);
                    if g.is_finite() {
                        pior = pior.max(g);
                    }
                }
            }
        }
        println!("  n = {n:.2}: ‖∇f‖ na pele = {pior:.4}");
    }

    println!("\n── a ESFERA (n = 2) é EXACTA? o campo contra a distância verdadeira ──");
    let t = ops_super::sd_superquadric([0.35; 3], 2.0, 2.0);
    let f = Field::from_tree(&t);
    let mut pior_erro = 0.0_f64;
    for i in 0..40 {
        for j in 0..40 {
            let (x, y) = (
                -0.8 + 1.6 * f64::from(i) / 39.0,
                -0.8 + 1.6 * f64::from(j) / 39.0,
            );
            let z = 0.13;
            let verdade = (x * x + y * y + z * z).sqrt() - 0.35;
            pior_erro = pior_erro.max((f.at(x, y, z) - verdade).abs());
        }
    }
    println!("  pior erro contra ‖p‖ − r: {pior_erro:.3e}");
}

/// A segunda metade: **onde fica o tecto do expoente**, medido por três recursos distintos.
#[test]
#[ignore]
fn probe_superquadric_ceiling() {
    let h = [0.35_f64; 3];

    println!("── (1) a FORMA: quanto ela ainda muda ao subir o expoente ──");
    println!("   desvio da silhueta contra a CAIXA (n = ∞), em fracção da meia-medida");
    let caixa = |x: f64, y: f64, z: f64| x.abs().max(y.abs()).max(z.abs()) - 0.35;
    let mut anterior: Option<f64> = None;
    for n in [2.0_f64, 4.0, 8.0, 12.0, 16.0, 24.0, 32.0, 48.0, 64.0] {
        let f = Field::from_tree(&ops_super::sd_superquadric(h, n, n));
        // Ao longo da diagonal de cada face e do canto: onde a diferença é maior.
        let mut pior = 0.0_f64;
        let g = 60;
        for i in 0..g {
            for j in 0..g {
                let (a, b) = (
                    -0.5 + 1.0 * f64::from(i) / f64::from(g - 1),
                    -0.5 + 1.0 * f64::from(j) / f64::from(g - 1),
                );
                // procura o raio da superfície na direcção (a, b, 0.35) normalizada
                let d = (a * a + b * b + 0.35 * 0.35).sqrt();
                let (ux, uy, uz) = (a / d, b / d, 0.35 / d);
                let raio = |campo: &dyn Fn(f64, f64, f64) -> f64| {
                    let (mut lo, mut hi) = (0.0_f64, 1.5_f64);
                    for _ in 0..60 {
                        let m = 0.5 * (lo + hi);
                        if campo(ux * m, uy * m, uz * m) < 0.0 {
                            lo = m;
                        } else {
                            hi = m;
                        }
                    }
                    0.5 * (lo + hi)
                };
                let r1 = raio(&|x, y, z| f.at(x, y, z));
                let r2 = raio(&caixa);
                pior = pior.max((r1 - r2).abs() / 0.35);
            }
        }
        let delta = anterior.map_or(f64::NAN, |a: f64| (a - pior).abs());
        println!(
            "  n = {n:>5.1}: desvio da caixa {:>7.4}   (ganho sobre o n anterior {delta:.4})",
            pior
        );
        anterior = Some(pior);
    }

    println!("\n── (2) o RELÓGIO: custo de uma varredura densa, contra a esfera ──");
    let base = {
        let f = Field::from_tree(&ops_super::sd_superquadric(h, 2.0, 2.0));
        let t0 = std::time::Instant::now();
        let mut acc = 0.0;
        for i in 0..60 {
            for j in 0..60 {
                for k in 0..60 {
                    let s = |t: usize| -0.7 + 1.4 * f64::from(u32::try_from(t).unwrap()) / 59.0;
                    acc += f.at(s(i), s(j), s(k));
                }
            }
        }
        std::hint::black_box(acc);
        t0.elapsed().as_secs_f64()
    };
    for n in [2.0_f64, 8.0, 32.0, 64.0, 128.0] {
        let f = Field::from_tree(&ops_super::sd_superquadric(h, n, n));
        let t0 = std::time::Instant::now();
        let mut acc = 0.0;
        for i in 0..60 {
            for j in 0..60 {
                for k in 0..60 {
                    let s = |t: usize| -0.7 + 1.4 * f64::from(u32::try_from(t).unwrap()) / 59.0;
                    acc += f.at(s(i), s(j), s(k));
                }
            }
        }
        std::hint::black_box(acc);
        println!(
            "  n = {n:>6.1}: {:.2}× o mesmo campo a n = 2",
            t0.elapsed().as_secs_f64() / base
        );
    }

    println!("\n── (3) a REPRESENTAÇÃO: onde o `f64` deixa de responder ──");
    for n in [32.0_f64, 64.0, 128.0, 256.0, 512.0, 1024.0] {
        let f = Field::from_tree(&ops_super::sd_superquadric(h, n, n));
        // longe da peça, onde `|q|^n` é maior
        let (mut sadios, mut total) = (0, 0);
        for i in 0..25 {
            for j in 0..25 {
                for k in 0..25 {
                    let s = |t: usize| -2.0 + 4.0 * f64::from(u32::try_from(t).unwrap()) / 24.0;
                    let v = f.at(s(i), s(j), s(k));
                    total += 1;
                    if v.is_finite() {
                        sadios += 1;
                    }
                }
            }
        }
        let no_centro = f.at(0.0, 0.0, 0.0);
        let na_pele = f.at(0.35, 0.0, 0.0);
        println!(
            "  n = {n:>6.1}: {sadios}/{total} finitos na caixa ±2  ·  f(centro) = {no_centro:.5}  ·  f(pele) = {na_pele:+.2e}"
        );
    }
}

/// A terceira metade: com o `f64` fora do caminho, **o que sobra a limitar o expoente?** A MARCHA.
#[test]
#[ignore]
fn probe_superquadric_march() {
    let h = [0.35_f64; 3];
    const PASSO: f64 = std::f64::consts::FRAC_1_SQRT_2;
    // Quantos passos a marcha de esferas dá até tocar a peça, vindo de `1,2` na direcção dada.
    let passos = |f: &Field, dir: [f64; 3]| -> usize {
        let n = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        let d = [dir[0] / n, dir[1] / n, dir[2] / n];
        let mut t = 0.0_f64;
        for i in 0..400 {
            let p = [d[0] * (1.2 - t), d[1] * (1.2 - t), d[2] * (1.2 - t)];
            let v = f.at(p[0], p[1], p[2]);
            if v < 1.0e-4 {
                return i;
            }
            t += v * PASSO;
            if t > 1.4 {
                return i;
            }
        }
        400
    };
    println!("── passos da marcha até tocar (400 = desistiu) ──");
    println!("   direcção:        face        aresta       CANTO");
    for n in [
        2.0_f64, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0, 512.0, 1024.0,
    ] {
        let f = Field::from_tree(&ops_super::sd_superquadric(h, n, n));
        println!(
            "  n = {n:>6.1}:   {:>8}    {:>8}    {:>8}",
            passos(&f, [1.0, 0.0, 0.0]),
            passos(&f, [1.0, 1.0, 0.0]),
            passos(&f, [1.0, 1.0, 1.0])
        );
    }
    println!("\n   (referência: a MESMA marcha sobre a caixa exacta da casa)");
    let caixa = ph2d_field_eval::ops_box::sd_box([0.35; 3], 0.0, 0.0);
    let f = Field::from_tree(&caixa);
    println!(
        "  caixa exacta:   {:>8}    {:>8}    {:>8}",
        passos(&f, [1.0, 0.0, 0.0]),
        passos(&f, [1.0, 1.0, 0.0]),
        passos(&f, [1.0, 1.0, 1.0])
    );
}

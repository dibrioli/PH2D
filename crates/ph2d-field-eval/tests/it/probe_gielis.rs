//! Sonda (`#[ignore]`) da W128: o divisor de duas varreduras 1-D segura a família? E quanto custa?
use ph2d_field_eval::ops_gielis::Curve;
use ph2d_field_eval::{Field, ops_gielis};

/// A referência HONESTA: uma varredura densíssima em `θ`/`φ` — a variável do PRODUTO, para o
/// oráculo não partilhar a mudança de variável que está a ser testada.
fn referencia(half: [f64; 3], top: Curve, side: Curve) -> f64 {
    let n = 400_000_usize;
    let mut q = 0.0_f64;
    for i in 0..n {
        let th = -std::f64::consts::PI + 2.0 * std::f64::consts::PI * (i as f64 + 0.5) / n as f64;
        let (r, dr) = ph2d_field_eval::ops_gielis::r_dr_of(top, th);
        let (st, ct) = th.sin_cos();
        let (er, ea) = (1.0 / r, -dr / (r * r));
        let gx = er * ct - ea * st;
        let gz = er * st + ea * ct;
        q = q.max(gx * gx / (half[0] * half[0]) + gz * gz / (half[2] * half[2]));
    }
    let mut k2 = 0.0_f64;
    for i in 0..n {
        let ph = -std::f64::consts::FRAC_PI_2 + std::f64::consts::PI * (i as f64 + 0.5) / n as f64;
        let (r, dr) = ph2d_field_eval::ops_gielis::r_dr_of(side, ph);
        let (sp, cp) = ph.sin_cos();
        let (er, ea) = (1.0 / r, -dr / (r * r));
        let bs = er * cp - ea * sp;
        let by = er * sp + ea * cp;
        k2 = k2.max(bs * bs * q + by * by / (half[1] * half[1]));
    }
    k2.sqrt()
}

fn cv(m: f64, n1: f64, n2: f64, n3: f64) -> Curve {
    Curve {
        symmetry: m,
        n1,
        n2,
        n3,
    }
}

/// O maior `‖∇f‖` da peça — grossa na caixa e fina junto da pele.
fn medido(half: [f64; 3], top: Curve, side: Curve) -> (f64, f64) {
    let t = ops_gielis::sd_superformula(half, top, side);
    let f = Field::from_tree(&t);
    let e = half[0].max(half[1]).max(half[2]) * 3.0;
    let mut pior = 0.0_f64;
    let mut nao_finitos = 0_u32;
    let mut varre = |n: usize, e: f64, banda: Option<f64>| {
        let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / n as f64;
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    let (x, y, z) = (at(i), at(j), at(k));
                    let v = f.at(x, y, z);
                    if !v.is_finite() {
                        nao_finitos += 1;
                        continue;
                    }
                    if banda.is_some_and(|b| v.abs() > b) {
                        continue;
                    }
                    let g = f.gradient_norm(x, y, z, 1.0e-5);
                    if g.is_finite() {
                        pior = pior.max(g);
                    } else {
                        nao_finitos += 1;
                    }
                }
            }
        }
    };
    varre(30, e, None);
    varre(96, e * 0.85, Some(0.02));
    (pior, f64::from(nao_finitos))
}

/// A fila de formas que a família de facto produz — os nomes são os das publicações.
fn corpus() -> Vec<(&'static str, Curve, Curve)> {
    vec![
        (
            "esfera (m=4, tudo 2)",
            cv(4.0, 2.0, 2.0, 2.0),
            cv(4.0, 2.0, 2.0, 2.0),
        ),
        (
            "estrela do mar 5",
            cv(5.0, 0.6, 1.7, 1.7),
            cv(4.0, 2.0, 2.0, 2.0),
        ),
        ("flor 6", cv(6.0, 1.0, 1.0, 1.0), cv(4.0, 2.0, 2.0, 2.0)),
        ("folha 3", cv(3.0, 4.5, 10.0, 10.0), cv(4.0, 2.0, 2.0, 2.0)),
        ("diamante 4", cv(4.0, 1.0, 1.0, 1.0), cv(4.0, 1.0, 1.0, 1.0)),
        ("gota 1", cv(1.0, 1.0, 1.0, 1.0), cv(4.0, 2.0, 2.0, 2.0)),
        (
            "assimétrica",
            cv(5.0, 2.0, 7.0, 1.5),
            cv(4.0, 2.0, 2.0, 2.0),
        ),
        (
            "perfil de sino",
            cv(4.0, 2.0, 2.0, 2.0),
            cv(2.0, 1.0, 4.0, 1.0),
        ),
        (
            "os DOIS a mexer",
            cv(7.0, 1.2, 3.0, 2.0),
            cv(3.0, 1.5, 2.0, 4.0),
        ),
        (
            "achatada (n1 alto)",
            cv(8.0, 12.0, 6.0, 6.0),
            cv(4.0, 2.0, 2.0, 2.0),
        ),
    ]
}

#[test]
#[ignore]
fn probe_gielis() {
    println!("── o divisor contra a MEDIÇÃO (alvo ≤ 1,00; acima disso a peça rasga) ──\n");
    for (nome, top, side) in corpus() {
        for (etiq, h) in [("cúbica", [0.35_f64; 3]), ("torta", [0.42, 0.24, 0.30])] {
            let k = ops_gielis::bound(h, top, side);
            let (g, nf) = medido(h, top, side);
            println!(
                "  {nome:<22} {etiq:<7} K = {k:>9.3}   ‖∇f‖ medido = {g:.4}{}",
                if nf > 0.0 {
                    format!("   ⛔ {nf} amostras NÃO FINITAS")
                } else {
                    String::new()
                }
            );
        }
    }

    println!("\n── quantas amostras a varredura do divisor precisa? (a barra é o K de 16 384) ──");
    println!(
        "        forma                    32       64      128      256      512     2048    16384"
    );
    for (nome, top, side) in corpus() {
        let h = [0.42_f64, 0.24, 0.30];
        let refer = referencia(h, top, side);
        let mut linha = format!("  {nome:<22}");
        for n in [32_usize, 64, 128, 256, 512, 2048] {
            let k = ops_gielis::bound(h, top, side);
            let _ = n;
            linha.push_str(&format!(" {:>+7.3}%", 100.0 * (k - refer) / refer));
        }
        linha.push_str(&format!("  {refer:>8.4}"));
        println!("{linha}");
    }
    println!("   (um K ABAIXO da referência é o perigoso: o divisor fica curto e a peça rasga)");

    println!("\n── o DÉFICE de `512` sobre TODA a fila de simetrias e expoentes (pior caso) ──");
    let mut pior_defice = 0.0_f64;
    let mut onde = String::new();
    for m in 1..=24 {
        let mut pior_m = 0.0_f64;
        for n1 in [0.2_f64, 0.5, 1.0, 2.0, 6.0, 20.0] {
            for n2 in [1.0_f64, 2.0, 6.0, 20.0] {
                for n3 in [1.0_f64, 3.0, 12.0] {
                    let c = cv(f64::from(m), n1, n2, n3);
                    let h = [0.42_f64, 0.24, 0.30];
                    for (top, side) in [(c, cv(4.0, 2.0, 2.0, 2.0)), (cv(4.0, 2.0, 2.0, 2.0), c)] {
                        let k = ops_gielis::bound(h, top, side);
                        let r = referencia(h, top, side);
                        let d = (r - k) / r;
                        if d > pior_m {
                            pior_m = d;
                        }
                        if d > pior_defice {
                            pior_defice = d;
                            onde = format!("m={m} n1={n1} n2={n2} n3={n3}");
                        }
                    }
                }
            }
        }
        println!("  simetria {m:>3}: pior défice {:+.4}%", 100.0 * pior_m);
    }
    println!("\n  PIOR DE TODOS: {:+.4}%  em {onde}", 100.0 * pior_defice);

    println!("\n── o RELÓGIO: uma varredura densa, contra a esfera da casa ──");
    let base = {
        let f = Field::from_tree(&ph2d_field_eval::ops::sd_sphere(0.35));
        let t0 = std::time::Instant::now();
        let mut acc = 0.0;
        for i in 0..50 {
            for j in 0..50 {
                for k in 0..50 {
                    let s = |t: usize| -0.7 + 1.4 * f64::from(u32::try_from(t).unwrap()) / 49.0;
                    acc += f.at(s(i), s(j), s(k));
                }
            }
        }
        std::hint::black_box(acc);
        t0.elapsed().as_secs_f64()
    };
    let (_, top, side) = corpus()[1];
    let f = Field::from_tree(&ops_gielis::sd_superformula([0.35; 3], top, side));
    let t0 = std::time::Instant::now();
    let mut acc = 0.0;
    for i in 0..50 {
        for j in 0..50 {
            for k in 0..50 {
                let s = |t: usize| -0.7 + 1.4 * f64::from(u32::try_from(t).unwrap()) / 49.0;
                acc += f.at(s(i), s(j), s(k));
            }
        }
    }
    std::hint::black_box(acc);
    println!(
        "  a estrela do mar custa {:.1}× uma esfera",
        t0.elapsed().as_secs_f64() / base
    );

    let t0 = std::time::Instant::now();
    for _ in 0..100 {
        std::hint::black_box(ops_gielis::bound([0.35; 3], top, side));
    }
    println!(
        "  e as DUAS varreduras do divisor: {:.1} µs (correm por quadro)",
        t0.elapsed().as_secs_f64() * 1.0e6 / 100.0
    );

    println!("\n── a origem: `atan2(0,0)` devolve NaN? ──");
    let f = Field::from_tree(&ops_gielis::sd_superformula([0.35; 3], top, side));
    for p in [[0.0, 0.0, 0.0], [0.0, 0.1, 0.0], [1e-9, 0.0, 1e-9]] {
        println!("  f{p:?} = {:.6}", f.at(p[0], p[1], p[2]));
    }
}

/// Diagnóstico do caso que ainda difere da referência.
#[test]
#[ignore]
fn probe_gielis_worst_case() {
    let h = [0.42_f64, 0.24, 0.30];
    let c = cv(23.0, 1.0, 6.0, 1.0);
    let e = cv(4.0, 2.0, 2.0, 2.0);
    for (rot, top, side) in [("c em CIMA", c, e), ("c de LADO", e, c)] {
        let meu = ops_gielis::bound(h, top, side);
        let refe = referencia(h, top, side);
        println!(
            "  {rot}: meu {meu:.5}  referência {refe:.5}   ({:+.3}%)",
            100.0 * (meu - refe) / refe
        );
    }
    // E onde a referência acha o pico do termo do PERFIL?
    let (top, side) = (e, c);
    let n = 400_000_usize;
    let mut q = 0.0_f64;
    for i in 0..n {
        let th = -std::f64::consts::PI + 2.0 * std::f64::consts::PI * (i as f64 + 0.5) / n as f64;
        let (r, dr) = ph2d_field_eval::ops_gielis::r_dr_of(top, th);
        let (st, ct) = th.sin_cos();
        let (er, ea) = (1.0 / r, -dr / (r * r));
        let gx = er * ct - ea * st;
        let gz = er * st + ea * ct;
        q = q.max(gx * gx / (h[0] * h[0]) + gz * gz / (h[2] * h[2]));
    }
    let mut melhor = (0.0_f64, 0.0_f64);
    for i in 0..n {
        let ph = -std::f64::consts::FRAC_PI_2 + std::f64::consts::PI * (i as f64 + 0.5) / n as f64;
        let (r, dr) = ph2d_field_eval::ops_gielis::r_dr_of(side, ph);
        let (sp, cp) = ph.sin_cos();
        let (er, ea) = (1.0 / r, -dr / (r * r));
        let bs = er * cp - ea * sp;
        let by = er * sp + ea * cp;
        let v = bs * bs * q + by * by / (h[1] * h[1]);
        if v > melhor.1 {
            melhor = (ph, v);
        }
    }
    let alpha = side.symmetry * (melhor.0 + std::f64::consts::PI) * 0.25;
    println!(
        "  pico do perfil em φ = {:.6} rad  ⇒  α = {alpha:.6}  (janela varrida: {:?})",
        melhor.0,
        ph2d_field_eval::ops_gielis::alpha_window_of(
            side,
            -std::f64::consts::FRAC_PI_2,
            std::f64::consts::FRAC_PI_2
        )
    );
}

/// As CERCAS: onde cada knob deixa de entregar forma e passa a entregar preço.
#[test]
#[ignore]
fn probe_gielis_fences() {
    const PASSO: f64 = std::f64::consts::FRAC_1_SQRT_2;
    let h = [0.35_f64; 3];
    let esfera = cv(4.0, 2.0, 2.0, 2.0);
    // Passos até tocar, vindo de `1,2` na direcção pedida.
    let passos = |f: &Field, d: [f64; 3]| -> usize {
        let n = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let u = [d[0] / n, d[1] / n, d[2] / n];
        let mut t = 0.0_f64;
        for i in 0..600 {
            let p = [u[0] * (1.2 - t), u[1] * (1.2 - t), u[2] * (1.2 - t)];
            let v = f.at(p[0], p[1], p[2]);
            if !v.is_finite() {
                return 999;
            }
            if v < 1.0e-4 {
                return i;
            }
            t += v * PASSO;
            if t > 1.4 {
                return i;
            }
        }
        600
    };
    let linha = |nome: String, top: Curve, side: Curve| {
        let k = ops_gielis::bound(h, top, side);
        let f = Field::from_tree(&ops_gielis::sd_superformula(h, top, side));
        println!(
            "  {nome:<28} K = {k:>8.2}   passos {:>4} / {:>4} / {:>4}",
            passos(&f, [1.0, 0.0, 0.0]),
            passos(&f, [1.0, 1.0, 0.0]),
            passos(&f, [1.0, 1.0, 1.0])
        );
    };
    println!("── a SIMETRIA (n1 = n2 = n3 = 1, que é o caso mais anguloso) ──");
    for m in [1_u32, 2, 3, 5, 8, 12, 16, 24, 32, 48, 64] {
        linha(format!("m = {m}"), cv(f64::from(m), 1.0, 1.0, 1.0), esfera);
    }
    println!("\n── o `n1` (m = 5, n2 = n3 = 1) — baixo EXAGERA os lobos ──");
    for n1 in [0.05_f64, 0.1, 0.2, 0.3, 0.5, 1.0, 2.0, 8.0, 40.0, 200.0] {
        linha(format!("n1 = {n1}"), cv(5.0, n1, 1.0, 1.0), esfera);
    }
    println!("\n── o `n2`/`n3` (m = 5, n1 = 1) ──");
    for n in [0.6_f64, 1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0] {
        linha(format!("n2 = n3 = {n}"), cv(5.0, 1.0, n, n), esfera);
    }
    println!("\n── e a ESQUINA má: `n1` no piso com o `n2` a subir (m = 5) ──");
    for n in [1.0_f64, 1.5, 2.0, 2.5, 3.0, 4.0] {
        linha(
            format!("n1 = 0,3  n2 = n3 = {n}"),
            cv(5.0, 0.3, n, n),
            esfera,
        );
    }
    println!("\n── e com a SIMETRIA no tecto (m = 16, n1 = 1) ──");
    for n in [1.0_f64, 1.5, 2.0, 3.0, 4.0] {
        linha(
            format!("m = 16  n2 = n3 = {n}"),
            cv(16.0, 1.0, n, n),
            esfera,
        );
    }

    println!("\n── A ESQUINA DA CAIXA: m = 16, n2 = n3 = 4, o `n1` a subir do piso ──");
    for n1 in [0.3_f64, 0.5, 0.75, 1.0, 1.5, 2.0] {
        linha(
            format!("m=16 n2=n3=4  n1 = {n1}"),
            cv(16.0, n1, 4.0, 4.0),
            esfera,
        );
    }
    println!("\n── e o tecto do `n2` com o `n1` a 0,5 e m no tecto ──");
    for n in [1.0_f64, 2.0, 2.5, 3.0, 3.5, 4.0] {
        linha(
            format!("n1=0,5 m=16  n2 = n3 = {n}"),
            cv(16.0, 0.5, n, n),
            esfera,
        );
    }
}

//! Sonda (`#[ignore]`) da W134: o divisor do nó de toro segura a família? quanto custa cada volta?
//! e onde é que a corda funde os fios?
use ph2d_field_eval::{Field, ops_knot};

/// A referência HONESTA: a distância à curva paramétrica por varredura densa em `t` — a variável do
/// PRODUTO não entra aqui, para o oráculo não partilhar a mudança de variável que está a ser testada.
fn dist_a_curva(px: f64, py: f64, pz: f64, r: f64, tb: f64, p: u32, q: u32) -> f64 {
    let n = 20_000_usize;
    let tau = std::f64::consts::TAU;
    let mut melhor = f64::INFINITY;
    for i in 0..n {
        let t = tau * (i as f64) / (n as f64);
        let (sq, cq) = (q as f64 * t).sin_cos();
        let (sp, cp) = (p as f64 * t).sin_cos();
        let rad = tb.mul_add(cq, r);
        let (x, y, z) = (rad * cp, rad * sp, tb * sq);
        let d = ((px - x).powi(2) + (py - y).powi(2) + (pz - z).powi(2)).sqrt();
        melhor = melhor.min(d);
    }
    melhor
}

/// O pior `‖∇f‖` **e onde ele está** — a caixa toda e, à parte, só a banda junto da pele.
///
/// ⚠️ **Os dois números respondem a perguntas diferentes.** A marcha só ATRAVESSA onde `f` mente
/// sobre a distância; um gradiente alto longe da peça, com `f` a subestimar, custa passos e não
/// custa buracos. Separá-los é a única maneira de saber qual dos dois se está a medir.
struct Pico {
    caixa: f64,
    banda: f64,
    onde: [f64; 3],
    rho: f64,
}

fn pior_gradiente(f: &Field, e: f64) -> Pico {
    let mut pico = Pico {
        caixa: 0.0,
        banda: 0.0,
        onde: [0.0; 3],
        rho: 0.0,
    };
    let varre = |n: usize, e: f64, banda: Option<f64>, p: &mut Pico| {
        let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / n as f64;
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    let (x, y, z) = (at(i), at(j), at(k));
                    let v = f.at(x, y, z);
                    if !v.is_finite() || banda.is_some_and(|b| v.abs() > b) {
                        continue;
                    }
                    let g = f.gradient_norm(x, y, z, 1.0e-5);
                    if !g.is_finite() {
                        continue;
                    }
                    if banda.is_some() {
                        p.banda = p.banda.max(g);
                    }
                    if g > p.caixa {
                        p.caixa = g;
                        p.onde = [x, y, z];
                        p.rho = x.hypot(y);
                    }
                }
            }
        }
    };
    varre(30, e, None, &mut pico);
    varre(96, e * 0.95, Some(0.02), &mut pico);
    pico
}

/// Os pares `(p, q)` que a família de facto produz — os nomes são os da literatura.
fn corpus() -> Vec<(&'static str, u32, u32)> {
    vec![
        ("anel (1,1)", 1, 1),
        ("trevo (2,3)", 2, 3),
        ("trevo deitado (3,2)", 3, 2),
        ("cinquefoil (2,5)", 2, 5),
        ("(3,4)", 3, 4),
        ("(5,2)", 5, 2),
        ("(4,3)", 4, 3),
        ("(7,2)", 7, 2),
        ("(2,9)", 2, 9),
        ("(8,3)", 8, 3),
        ("elo duplo (2,2)", 2, 2),
    ]
}

#[test]
#[ignore]
fn probe_knot() {
    let (r, tb) = (0.62_f64, 0.26_f64);
    println!("── o divisor contra a MEDIÇÃO (alvo ≤ 1,00; acima disso a peça rasga) ──");
    println!("R = {r}, tubo = {tb}\n");
    println!("{:<22} {:>5} {:>9} {:>9}", "peça", "cord", "caixa", "banda");
    for (nome, p, q) in corpus() {
        let cord = f64::from(ph2d_field::knot_cord_ceiling(r as f32, tb as f32, p, q)) * 0.6;
        let t = ops_knot::sd_torus_knot(r, tb, cord, p, q);
        let f = Field::from_tree(&t);
        let g = pior_gradiente(&f, (r + tb + cord) * 1.15);
        println!(
            "{nome:<22} {cord:>5.3} {:>9.4} {:>9.4}   pico em ρ={:.3} z={:+.3} (f={:+.4})",
            g.caixa,
            g.banda,
            g.rho,
            g.onde[2],
            f.at(g.onde[0], g.onde[1], g.onde[2])
        );
    }

    println!("\n── o campo é MINORANTE da distância à curva? (pior `f − d`, alvo ≤ 0) ──");
    for (nome, p, q) in corpus() {
        let cord = f64::from(ph2d_field::knot_cord_ceiling(r as f32, tb as f32, p, q)) * 0.6;
        let t = ops_knot::sd_torus_knot(r, tb, cord, p, q);
        let f = Field::from_tree(&t);
        let e = (r + tb + cord) * 1.15;
        let n = 15_usize;
        let at = |i: usize| -e + 2.0 * e * (i as f64 + 0.5) / n as f64;
        let (mut pior, mut soma, mut conta) = (0.0_f64, 0.0_f64, 0_u32);
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    let (x, y, z) = (at(i), at(j), at(k));
                    let v = f.at(x, y, z);
                    if !v.is_finite() || v <= 0.0 {
                        continue;
                    }
                    let d = dist_a_curva(x, y, z, r, tb, p, q) - cord;
                    if d <= 1.0e-6 {
                        continue;
                    }
                    // ⚠️ **A DIFERENÇA, e não o quociente.** Junto da superfície `d → 0` e a razão
                    // explode sobre um campo correcto — ela lia `50,5` no trevo onde a diferença é
                    // `−0,0004`. *Um quociente cujo denominador vai a zero não é uma régua.*
                    pior = pior.max(v - d);
                    soma += v / d.max(0.05);
                    conta += 1;
                }
            }
        }
        println!(
            "{nome:<22} pior f−d {pior:>+9.5}   razão média (d>0,05) {:>7.4}   ({conta} pontos)",
            soma / f64::from(conta.max(1))
        );
    }
}

/// ⚠️ O preço por VOLTA — a árvore cresce com `p`, e é `p` que o tecto tem de nomear.
#[test]
#[ignore]
fn probe_knot_price() {
    let (r, tb) = (0.62_f64, 0.26_f64);
    println!("── o preço de uma volta (nós da árvore e relógio de 100k amostras) ──");
    println!("{:>4} {:>12} {:>12}", "p", "µs/100k", "ns/ponto");
    for p in [1_u32, 2, 3, 4, 6, 8, 12, 16, 24, 32] {
        let cord = f64::from(ph2d_field::knot_cord_ceiling(r as f32, tb as f32, p, 3)) * 0.6;
        let t = ops_knot::sd_torus_knot(r, tb, cord, p, 3);
        let f = Field::from_tree(&t);
        let n = 100_000_usize;
        let inicio = std::time::Instant::now();
        let mut acc = 0.0_f64;
        for i in 0..n {
            let u = (i as f64) / (n as f64);
            acc += f.at(u * 2.0 - 1.0, u * 1.3 - 0.6, u * 0.7 - 0.35);
        }
        let dt = inicio.elapsed();
        println!(
            "{p:>4} {:>12.0} {:>12.1}   (acc {acc:.3})",
            dt.as_secs_f64() * 1.0e6,
            dt.as_secs_f64() * 1.0e9 / n as f64
        );
    }
}

/// ⭐⭐ **A GRELHA `(p, q)` INTEIRA** — onde é que o campo deixa de marchar, e portanto onde ficam os
/// tectos das duas contagens.
///
/// ⚠️ A corda é sempre `60 %` do tecto DELA (que depende de `p` e de `q`), senão a varredura mede
/// peças com folgas diferentes e a coluna lê-se como ruído.
#[test]
#[ignore]
fn probe_knot_grid() {
    let (r, tb) = (0.62_f64, 0.26_f64);
    println!("── ‖∇f‖ na BANDA (|f| ≤ 0,02) por (p, q) — alvo ≤ 1,00 ──");
    print!("{:>4}", "p\\q");
    for q in [1_u32, 2, 3, 4, 6, 8, 12, 16, 24, 32, 40, 48] {
        print!("{q:>7}");
    }
    println!();
    for p in 1..=ph2d_field::MAX_KNOT_WINDS {
        print!("{p:>4}");
        for q in [1_u32, 2, 3, 4, 6, 8, 12, 16, 24, 32, 40, 48] {
            if q > ph2d_field::max_knot_loops(p) {
                print!("{:>7}", "—");
                continue;
            }
            let cord = f64::from(ph2d_field::knot_cord_ceiling(r as f32, tb as f32, p, q)) * 0.6;
            let t = ops_knot::sd_torus_knot(r, tb, cord, p, q);
            let f = Field::from_tree(&t);
            let g = pior_gradiente(&f, (r + tb + cord) * 1.05);
            print!("{:>7.2}", g.banda);
        }
        println!();
    }
}

/// ⭐⭐⭐ **O TECTO DA CORDA CONTRA A CURVA QUE SE MEDE A SI PRÓPRIA.**
///
/// ⚠️ **O oráculo é o ALCANCE (*reach*), e ele tem DUAS metades** — a primeira redacção desta sonda
/// tinha só a primeira, e uma janela vazia devolvia `inf` em metade da grelha:
/// 1. metade da menor distância entre pontos **duplamente críticos** (mínimos locais em `j`, longe
///    da diagonal) — a auto-aproximação;
/// 2. o menor **raio de curvatura** — um tubo mais gordo do que ele cruza-se sozinho, sem vizinho
///    nenhum.
#[test]
#[ignore]
fn probe_knot_cord_ceiling() {
    let (r, tb) = (0.62_f64, 0.26_f64);
    let n = 1_500_usize;
    let ponto = |i: usize, p: u32, q: u32| {
        let t = std::f64::consts::TAU * ((i % n) as f64) / (n as f64);
        let (sq, cq) = (f64::from(q) * t).sin_cos();
        let (sp, cp) = (f64::from(p) * t).sin_cos();
        let rad = tb.mul_add(cq, r);
        [rad * cp, rad * sp, tb * sq]
    };
    let dist = |a: [f64; 3], b: [f64; 3]| {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    };
    println!("── tecto fechado ÷ ALCANCE medido (alvo ≤ 1,00; acima disso o tubo funde-se) ──");
    print!("{:>4}", "p\\q");
    for q in 1..=10 {
        print!("{q:>7}");
    }
    println!();
    for p in 1..=10_u32 {
        print!("{p:>4}");
        for q in 1..=10_u32 {
            // ⚠️ **`gcd(p, q) > 1` percorre a MESMA curva `g` vezes**, e o alcance de uma curva que
            // se repete é ZERO — o oráculo mede a curva reduzida, que é o desenho que ali sai.
            let g = {
                let (mut a, mut b) = (p, q);
                while b != 0 {
                    let t = b;
                    b = a % b;
                    a = t;
                }
                a
            };
            let (p, q) = (p / g, q / g);
            // (1) auto-aproximação por pares duplamente críticos
            let janela = n / 40;
            let mut menor = f64::INFINITY;
            for i in (0..n).step_by(5) {
                let a = ponto(i, p, q);
                for j in (i + janela)..(i + n - janela) {
                    let d = dist(a, ponto(j, p, q));
                    if d < dist(a, ponto(j - 1, p, q)) && d < dist(a, ponto(j + 1, p, q)) {
                        menor = menor.min(d);
                    }
                }
            }
            // (2) raio de curvatura, por diferença central
            let mut raio_min = f64::INFINITY;
            for i in 0..n {
                let (a, b, c) = (ponto(i, p, q), ponto(i + 1, p, q), ponto(i + 2, p, q));
                let (ab, bc, ca) = (dist(a, b), dist(b, c), dist(c, a));
                let s = (ab + bc + ca) * 0.5;
                let area = (s * (s - ab) * (s - bc) * (s - ca)).max(0.0).sqrt();
                if area > 1.0e-18 {
                    raio_min = raio_min.min(ab * bc * ca / (4.0 * area));
                }
            }
            let alcance = (menor * 0.5).min(raio_min);
            let tecto = f64::from(ph2d_field::knot_cord_ceiling(
                r as f32,
                tb as f32,
                p * g,
                q * g,
            ));
            print!("{:>7.2}", tecto / alcance);
        }
        println!();
    }
}

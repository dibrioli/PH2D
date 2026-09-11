//! ⭐⭐⭐ **O `σ_max` DO PRODUTO DAS MATRIZES contra o PRODUTO dos `σ_max`** — o spike que decide a
//! obra do bound da composição (`docs/3DModeling/09_o_bound_da_composicao.md`).
//!
//! # A pergunta, e por que ela não se responde com o campo
//!
//! O divisor cobra `σ(J_taper) · σ(J_twist) · σ(J_bend)`. A verdade é
//! `σ_max(J_taper · J_twist · J_bend)`, e a desigualdade só é igualdade se as três direcções de
//! esticadela coincidirem. ⚠️ **O `‖∇f‖` medido não responde**: ele usa UM `inner` (uma caixa), logo
//! é um **minorante** do que qualquer `inner` poderia produzir. Aqui mede-se a matriz.
//!
//! # ⛔ A cerca desta sonda: as fórmulas têm de ser A ÁRVORE
//!
//! As três leis são reescritas aqui como funções de `f64` para se poderem diferenciar. Uma cópia
//! que divirja da árvore mediria outro programa — por isso a primeira metade compara
//! `campo_a_mao(p)` contra o [`Field::at`] do produto em centenas de pontos. *Uma sonda que reescreve
//! a lei tem de provar que reescreveu a mesma.*

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, Xform};
use ph2d_field_eval::{Field, bounds, bounds_clip, hybrid::Registry};

const HALF: [f32; 3] = [0.35, 0.35, 0.30];
const BEND_TURNS: f32 = 0.12;
const TWIST_TURNS: f32 = 0.35;
const SLOPE: f64 = 0.6;
const BANDA: (f64, f64, f64) = (-2.0, 2.0, 0.1);

fn mods() -> Vec<Unary> {
    use ph2d_field::mods::{BEND_AXIS, TAPER_AXIS, TWIST_AXIS};
    vec![
        Unary::Bend {
            turns: BEND_TURNS,
            lower: BANDA.0 as f32,
            upper: BANDA.1 as f32,
            falloff: BANDA.2 as f32,
            axis: BEND_AXIS,
        },
        Unary::Twist {
            turns: TWIST_TURNS,
            lower: BANDA.0 as f32,
            upper: BANDA.1 as f32,
            falloff: BANDA.2 as f32,
            axis: TWIST_AXIS,
        },
        Unary::Taper {
            slope: SLOPE as f32,
            axis: TAPER_AXIS,
        },
    ]
}

fn doc() -> FieldDoc {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: HALF,
            round: 0.0,
            chamfer: 0.0,
        }),
    );
    n.mods = mods();
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

/// Os números que as três leis usam, tirados das MESMAS portas que a árvore usa.
struct Leis {
    k_bend: f64,
    reach: f64,
    k_twist: f64,
    piso: f64,
    divisor: f64,
}

fn leis(doc: &FieldDoc, reg: &Registry) -> Leis {
    let ms = mods();
    let local = bounds::local_balls(doc, reg)[0].expect("bola local");
    // ⚠️ A MESMA travessia da `stack::stacked`: a dobra tira a curvatura da bola de ANTES dela, e o
    // alcance da bola de DEPOIS.
    let k_bend = ph2d_field_eval::bend_curvature(BEND_TURNS, local);
    let depois = bounds::step_mod(local, ms[0]);
    Leis {
        k_bend,
        reach: ph2d_field_eval::bend_reach(depois),
        k_twist: f64::from(TWIST_TURNS) * std::f64::consts::TAU,
        // ⚠️ O piso da inclinação sai da bola **LOCAL**, e não da corrente.
        piso: ph2d_field_eval::taper_floor(SLOPE, local),
        divisor: f64::from(ph2d_field_eval::field_shrink(doc, reg)),
    }
}

fn soft_clamp(z: f64, lo: f64, hi: f64, w: f64) -> f64 {
    let meia = (hi - lo).abs() * 0.5;
    let w = w.min(meia);
    if w <= 0.0 || !w.is_finite() {
        return z.max(lo).min(hi);
    }
    let suave = |a: f64, b: f64, cima: bool| {
        let d = (a - b).abs();
        let h = (w - d).max(0.0) * (1.0 / w);
        let corda = h * h * (w * 0.25);
        if cima {
            a.max(b) + corda
        } else {
            a.min(b) - corda
        }
    };
    suave(suave(z, lo, true), hi, false)
}

/// O factor de secção da inclinação — o mesmo número que a árvore multiplica no fim.
fn k_taper(y: f64, l: &Leis) -> f64 {
    SLOPE.mul_add(y, 1.0).max(l.piso)
}

fn phi_taper(p: [f64; 3], l: &Leis) -> [f64; 3] {
    let k = k_taper(p[1], l);
    [p[0] / k, p[1], p[2] / k]
}

fn phi_twist(q: [f64; 3], l: &Leis) -> [f64; 3] {
    let banda = soft_clamp(q[2], BANDA.0, BANDA.1, BANDA.2);
    let (s, c) = (banda * -l.k_twist).sin_cos();
    [q[0] * c - q[1] * s, q[0] * s + q[1] * c, q[2]]
}

fn phi_bend(r: [f64; 3], l: &Leis) -> [f64; 3] {
    let s = if l.k_bend < 0.0 { -1.0 } else { 1.0 };
    let rho = (1.0 / l.k_bend).abs();
    let piso = (rho - l.reach.abs()).max(rho * 0.1);
    let a = (rho - r[0] * s).max(piso);
    let b = r[2];
    let rr = a.hypot(b);
    let theta = b.atan2(a);
    let theta_c = soft_clamp(theta, BANDA.0 / rho, BANDA.1 / rho, BANDA.2 / rho);
    let d = theta - theta_c;
    [
        (rho - rr * d.cos()) * s,
        r[1],
        theta_c.mul_add(rho, rr * d.sin()),
    ]
}

fn phi(p: [f64; 3], l: &Leis) -> [f64; 3] {
    phi_bend(phi_twist(phi_taper(p, l), l), l)
}

fn box_sdf(p: [f64; 3]) -> f64 {
    let q = [
        p[0].abs() - f64::from(HALF[0]),
        p[1].abs() - f64::from(HALF[1]),
        p[2].abs() - f64::from(HALF[2]),
    ];
    let fora = [q[0].max(0.0), q[1].max(0.0), q[2].max(0.0)];
    fora[0].hypot(fora[1]).hypot(fora[2]) + q[0].max(q[1]).max(q[2]).min(0.0)
}

fn campo_a_mao(p: [f64; 3], l: &Leis) -> f64 {
    box_sdf(phi(p, l)) * k_taper(p[1], l) / l.divisor
}

/// ⭐⭐⭐ **A METADE QUE TORNA A SONDA ACREDITÁVEL**: as fórmulas à mão SÃO a árvore.
#[test]
fn the_hand_written_maps_reproduce_the_tree() {
    let (d, reg) = (doc(), Registry::default());
    let l = leis(&d, &reg);
    let f = Field::new(&d);
    let bola = bounds::bounding_ball(&d, &reg).expect("bordo");
    let (lo, hi) = bounds_clip::march_clip(bola);
    let n = 12;
    let mut pior = 0.0f64;
    for i in 0..=n {
        for j in 0..=n {
            for k in 0..=n {
                let em = |t: i32, e: usize| {
                    f64::from(lo[e]) + f64::from(t) / f64::from(n) * f64::from(hi[e] - lo[e])
                };
                let p = [em(i, 0), em(j, 1), em(k, 2)];
                let a = campo_a_mao(p, &l);
                let b = f.at(p[0], p[1], p[2]);
                if a.is_finite() && b.is_finite() {
                    pior = pior.max((a - b).abs());
                }
            }
        }
    }
    assert!(
        pior < 2.0e-5,
        "as fórmulas desta sonda divergem da árvore em {pior:.3e} — ela estaria a medir outro \
         programa, e toda conclusão sobre a composição seria sobre ele"
    );
}

/// O maior valor singular de uma `3×3`, por iteração de potência sobre `MᵀM`.
fn sigma_max(m: [[f64; 3]; 3]) -> f64 {
    let mut v = [0.577_350_269, 0.577_350_269, 0.577_350_269];
    let mut s = 0.0f64;
    for _ in 0..60 {
        // `w = M v`, depois `u = Mᵀ w`
        let w = [
            m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
            m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
            m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
        ];
        let u = [
            m[0][0] * w[0] + m[1][0] * w[1] + m[2][0] * w[2],
            m[0][1] * w[0] + m[1][1] * w[1] + m[2][1] * w[2],
            m[0][2] * w[0] + m[1][2] * w[1] + m[2][2] * w[2],
        ];
        let n = u[0].hypot(u[1]).hypot(u[2]);
        if n <= 0.0 {
            return 0.0;
        }
        v = [u[0] / n, u[1] / n, u[2] / n];
        s = n.sqrt();
    }
    s
}

fn jacobiano(p: [f64; 3], l: &Leis, mapa: impl Fn([f64; 3], &Leis) -> [f64; 3]) -> [[f64; 3]; 3] {
    let e = 1.0e-6;
    let mut j = [[0.0f64; 3]; 3];
    for c in 0..3 {
        let (mut a, mut b) = (p, p);
        a[c] += e;
        b[c] -= e;
        let (fa, fb) = (mapa(a, l), mapa(b, l));
        for r in 0..3 {
            j[r][c] = (fa[r] - fb[r]) / (2.0 * e);
        }
    }
    j
}

/// ⭐⭐⭐ **O SPIKE**: `σ_max` do PRODUTO contra o produto dos `σ_max`, sobre o recorte inteiro.
///
/// ```text
/// cargo test -p ph2d-field-eval --release --test the_composed_jacobian_is_not_the_product \
///     -- --ignored --nocapture
/// ```
#[test]
#[ignore = "spike: escolhe a forma do bound"]
fn measure_the_composed_jacobian() {
    let (d, reg) = (doc(), Registry::default());
    let l = leis(&d, &reg);
    let bola = bounds::bounding_ball(&d, &reg).expect("bordo");
    let (lo, hi) = bounds_clip::march_clip(bola);
    let n = 48;
    let (mut comp, mut prod, mut aditivo, mut total) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let (mut onde, mut k_no_pior) = ([0.0f64; 3], 0.0f64);
    // ⭐ Os dois majorantes CANDIDATOS para a versão de intervalos — o que decide o desenho é qual
    // deles fica perto do `σ_max` verdadeiro na ESTRUTURA destas matrizes.
    let (mut frob, mut norma_1inf) = (0.0f64, 0.0f64);
    for i in 0..=n {
        for jj in 0..=n {
            for kk in 0..=n {
                let em = |t: i32, e: usize| {
                    f64::from(lo[e]) + f64::from(t) / f64::from(n) * f64::from(hi[e] - lo[e])
                };
                let p = [em(i, 0), em(jj, 1), em(kk, 2)];
                // ⭐ O jacobiano da COMPOSIÇÃO, de uma vez — é ele que a desigualdade sobrestima.
                let jc = jacobiano(p, &l, phi);
                let sc = sigma_max(jc);
                // E o produto dos três, cada um no seu ponto da cadeia.
                let q = phi_taper(p, &l);
                let r = phi_twist(q, &l);
                let sp = sigma_max(jacobiano(p, &l, phi_taper))
                    * sigma_max(jacobiano(q, &l, phi_twist))
                    * sigma_max(jacobiano(r, &l, phi_bend));
                let k = k_taper(p[1], &l);
                // `∇F = k·Jᵀ∇inner + inner(φ(p))·∇k` — a parte multiplicativa e a ADITIVA.
                let add = box_sdf(phi(p, &l)).abs() * SLOPE;
                let t = k.mul_add(sc, add);
                let f_norm = (0..3)
                    .flat_map(|r| (0..3).map(move |c| (r, c)))
                    .map(|(r, c)| jc[r][c] * jc[r][c])
                    .sum::<f64>()
                    .sqrt();
                let n1 = (0..3)
                    .map(|c| (0..3).map(|r| jc[r][c].abs()).sum::<f64>())
                    .fold(0.0f64, f64::max);
                let ninf = (0..3)
                    .map(|r| (0..3).map(|c| jc[r][c].abs()).sum::<f64>())
                    .fold(0.0f64, f64::max);
                if f_norm.is_finite() {
                    frob = frob.max(f_norm);
                }
                if (n1 * ninf).is_finite() {
                    norma_1inf = norma_1inf.max((n1 * ninf).sqrt());
                }
                if sc.is_finite() && sc > comp {
                    comp = sc;
                    onde = p;
                    k_no_pior = k;
                }
                if sp.is_finite() {
                    prod = prod.max(sp);
                }
                if add.is_finite() {
                    aditivo = aditivo.max(add);
                }
                if t.is_finite() {
                    total = total.max(t);
                }
            }
        }
    }
    println!("| grandeza | valor |");
    println!("|---|---:|");
    println!("| divisor COBRADO hoje | {:.2} |", l.divisor);
    println!("| `max σ(J_taper)·σ(J_twist)·σ(J_bend)` (o produto, ponto a ponto) | {prod:.2} |");
    println!("| **`max σ_max(J_taper·J_twist·J_bend)`** (a COMPOSIÇÃO) | **{comp:.2}** |");
    println!(
        "| majorante de Frobenius `‖M‖_F` | {frob:.2} (`{:.1} %` acima) |",
        (frob / comp - 1.0) * 100.0
    );
    println!(
        "| majorante `√(‖M‖₁·‖M‖∞)` | {norma_1inf:.2} (`{:.1} %` acima) |",
        (norma_1inf / comp - 1.0) * 100.0
    );
    println!("| `max k` no pior ponto | {k_no_pior:.3} |");
    println!("| termo ADITIVO `max |inner|·|slope|` | {aditivo:.2} |");
    println!("| **bound NOVO = `max(k·σ_comp + aditivo)`** | **{total:.2}** |");
    println!(
        "| ⇒ ganho contra o cobrado | **{:.2}×** |",
        l.divisor / total.max(1e-9)
    );
    println!(
        "\npior composição em ({:.3}, {:.3}, {:.3})",
        onde[0], onde[1], onde[2]
    );
}

/// ⭐ **O que a lei nova de facto devolve** — a sonda de diagnóstico do `bounds_lip`.
#[test]
#[ignore = "sonda"]
fn what_the_new_bound_returns() {
    let (d, reg) = (doc(), Registry::default());
    let l = leis(&d, &reg);
    println!("divisor cobrado (produto) = {:.3}", l.divisor);
    let local = bounds::local_balls(&d, &reg)[0].unwrap();
    let t = std::time::Instant::now();
    let v = ph2d_field_eval::stack_lipschitz_probe(&mods(), local);
    println!(
        "bound novo = {:?} em {:.3} ms",
        v.map(|x| format!("{x:.3}")),
        t.elapsed().as_secs_f64() * 1000.0
    );
}

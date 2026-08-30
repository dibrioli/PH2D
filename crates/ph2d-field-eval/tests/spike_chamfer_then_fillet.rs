//! **SONDA** — o chanfro de uma primitiva, e o filete POR CIMA dele.
//!
//! > **Pedido do Enio, 2026-08-30:** *«em todas as peças temos fillet para as bordas arredondadas
//! > mas não temos um slider para chamfer. Poderíamos ter os 2, com chamfer antes de fillet para a
//! > possibilidade de arredondar as bordas geradas por chamfer»*
//!
//! ⚠️ **Ela mede ANTES de o produto existir**, que é a lei desta casa: a derivação à mão é uma
//! hipótese, e três coisas têm de ser medidas antes de um slider aparecer no painel.
//!
//! | pergunta | por que ela decide o desenho |
//! |---|---|
//! | o recuo entregue é o **número pedido**? | os caracteres desta casa medem todos a mesma coisa ([`ph2d_field::Blend`]); um chanfro que entregasse `0,71×` mentiria uma fracção fixa, sempre |
//! | `‖∇f‖` fica **≤ 1**? | acima de `1` o campo **superestima** e a marcha salta por cima da superfície — é o defeito que fura a peça |
//! | o filete por cima **morde** as arestas que o chanfro criou? | é literalmente o que foi pedido |
//!
//! ⛔ **A conta genérica NÃO existe a partir do campo sozinho, e isso está medido aqui:** o filete
//! é a dilatação pela bola de `L²` (`f − r`), e o chanfro é a dilatação pelo octaedro de `L¹` —
//! que **não** é recuperável de uma distância euclidiana. Por isso a junta tem de entrar onde as
//! PEÇAS da primitiva ainda existem, e é exactamente aí que o
//! [`ph2d_field_eval::ops::slab_and_walls`] já põe o `round`.
//!
//! Corre com `cargo test -p ph2d-field-eval --test spike_chamfer_then_fillet -- --nocapture`.

use fidget::context::Tree;
use ph2d_field_eval::{Field, ops};

/// `1/√2` — o factor do plano a 45°, o mesmo do [`ph2d_field_eval::ops::union_chamfer`].
const INV_SQRT2: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// ⭐⭐⭐ **A LEI CANDIDATA** — a junta de duas superfícies com **chanfro** e depois **filete**.
///
/// A intersecção de `a` com `b` chanfrada é `max(max(a,b), (a+b+c)·√½)` — o dual de De Morgan do
/// [`ph2d_field_eval::ops::union_chamfer`], logo o `c` **é** o recuo ao longo de cada face, que é a
/// régua que os quatro caracteres desta casa partilham.
///
/// ⭐ O plano do chanfro é uma **terceira superfície**, e as duas arestas novas que ele cria são
/// `a ∩ plano` e `b ∩ plano`. Arredondá-las é o filete de sempre, aplicado a essas duas juntas — e
/// a aresta velha `a ∩ b` já não está na fronteira (o plano cortou-a fora), logo o filete não lhe
/// toca.
fn chamfer_then_fillet(a: &Tree, b: &Tree, c: f64, r: f64) -> Tree {
    if c <= 0.0 {
        // ⭐ **O caminho de sempre, ao bit** — é o que o `slab_and_walls` faz hoje.
        return ops::intersection(a, b, ops::Blended::Exact(r));
    }
    let plane = (a.clone() + b.clone() + Tree::constant(c)) * Tree::constant(INV_SQRT2);
    if r <= 0.0 {
        return a.max(b.clone()).max(plane);
    }
    ops::intersection(
        &ops::intersection(a, &plane, ops::Blended::Exact(r)),
        b,
        ops::Blended::Exact(r),
    )
}

/// O aro de um cilindro: a parede radial e a laje axial, as **duas peças** que o
/// [`ph2d_field_eval::ops::slab_and_walls`] junta.
fn rim(radius: f64, half_height: f64) -> (Tree, Tree) {
    // ⚠️ O piso da raiz é o da casa (`ops::safe_sqrt`), reescrito aqui porque ele é `pub(crate)` —
    // e uma raiz sem piso devolve `NaN` no gradiente, que é precisamente o que esta sonda mede.
    let radial = (Tree::x().square() + Tree::y().square())
        .max(1.0e-30)
        .sqrt()
        - Tree::constant(radius);
    let axial = Tree::z().abs() - Tree::constant(half_height);
    (radial, axial)
}

/// O `z` da superfície acima do raio `rho`, por bissecção. `None` se não houver peça ali.
fn surface_z(f: &Field, rho: f64, hi: f64) -> Option<f64> {
    if f.at(rho, 0.0, 0.0) > 0.0 {
        return None;
    }
    let (mut lo, mut hi) = (0.0f64, hi);
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if f.at(rho, 0.0, mid) <= 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

/// O `rho` da superfície à altura `z`, por bissecção.
fn surface_rho(f: &Field, z: f64, hi: f64) -> Option<f64> {
    if f.at(0.0, 0.0, z) > 0.0 {
        return None;
    }
    let (mut lo, mut hi) = (0.0f64, hi);
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if f.at(mid, 0.0, z) <= 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

/// O maior `t` em que `probe(t)` ainda está a `tol` do valor de referência — o **fim da face**.
fn last_flat(tol: f64, reference: f64, mut probe: impl FnMut(f64) -> Option<f64>) -> f64 {
    let mut best = 0.0f64;
    for i in 0..=4000 {
        let t = f64::from(i) / 2000.0;
        match probe(t) {
            Some(v) if (v - reference).abs() <= tol => best = t,
            _ => {}
        }
    }
    best
}

/// O pior `‖∇f‖` numa grelha à volta do aro — a pergunta que decide se a peça fura.
fn worst_gradient(f: &Field, eps: f64) -> f64 {
    let mut hi = 0.0f64;
    for i in 0..49 {
        for j in 0..49 {
            for k in 0..49 {
                let p = |n: i32| f64::from(n) / 24.0 * 1.6 - 0.8;
                let g = f.gradient_norm(p(i) + 0.4, p(j), p(k), eps);
                if g.is_finite() && g > 1e-6 {
                    hi = hi.max(g);
                }
            }
        }
    }
    hi
}

/// ⭐ **O RECUO ENTREGUE** — o chanfro tem de começar a `c` da quina em CADA uma das duas faces.
#[test]
fn measure_the_chamfer_setback() {
    const R: f64 = 0.5;
    const H: f64 = 0.4;
    let (radial, axial) = rim(R, H);
    println!("\n  chanfro | recuo na TAMPA | recuo na PAREDE | pior |grad|");
    println!("  --------|----------------|-----------------|------------");
    for c in [0.0f64, 0.05, 0.10, 0.15, 0.20] {
        let tree = chamfer_then_fillet(&radial, &axial, c, 0.0);
        let f = Field::from_tree(&tree);
        // A tampa é o plano `z = H`; ela acaba onde o chanfro começa.
        let fim_da_tampa = last_flat(1e-6, H, |rho| surface_z(&f, rho, H * 4.0));
        // A parede é o cilindro `rho = R`; ela acaba na mesma medida, na outra direcção.
        let fim_da_parede = last_flat(1e-6, R, |z| surface_rho(&f, z, R * 4.0));
        println!(
            "  {:>7.3} | {:>14.5} | {:>15.5} | {:>10.4}",
            c,
            R - fim_da_tampa,
            H - fim_da_parede,
            worst_gradient(&f, 1e-3)
        );
    }
}

/// ⭐⭐ **O FILETE POR CIMA** — ele tem de morder as arestas que o chanfro criou, e só essas.
#[test]
fn measure_the_fillet_on_top_of_the_chamfer() {
    const R: f64 = 0.5;
    const H: f64 = 0.4;
    const C: f64 = 0.15;
    let (radial, axial) = rim(R, H);
    println!("\n  filete | recuo na TAMPA | recuo na PAREDE | pior |grad|");
    println!("  -------|----------------|-----------------|------------");
    for r in [0.0f64, 0.02, 0.04, 0.06] {
        let tree = chamfer_then_fillet(&radial, &axial, C, r);
        let f = Field::from_tree(&tree);
        let fim_da_tampa = last_flat(1e-6, H, |rho| surface_z(&f, rho, H * 4.0));
        let fim_da_parede = last_flat(1e-6, R, |z| surface_rho(&f, z, R * 4.0));
        println!(
            "  {:>6.3} | {:>14.5} | {:>15.5} | {:>10.4}",
            r,
            R - fim_da_tampa,
            H - fim_da_parede,
            worst_gradient(&f, 1e-3)
        );
    }
}

/// ⭐ **A GRELHA não é o que se está a medir** — o mesmo número com `ε` cem vezes menor.
///
/// ⚠️ Ela existe porque esta linha já leu um `‖∇f‖` alto que era artefacto de amostragem, e a única
/// forma de os separar é mudar o `ε` e ver se o número anda.
#[test]
fn measure_the_gradient_is_not_a_sampling_artefact() {
    const R: f64 = 0.5;
    const H: f64 = 0.4;
    let (radial, axial) = rim(R, H);
    println!("\n  chanfro | filete | grad(eps=1e-3) | grad(eps=1e-5)");
    for (c, r) in [(0.15f64, 0.0f64), (0.15, 0.04), (0.0, 0.04)] {
        let tree = chamfer_then_fillet(&radial, &axial, c, r);
        let f = Field::from_tree(&tree);
        println!(
            "  {:>7.3} | {:>6.3} | {:>14.4} | {:>14.4}",
            c,
            r,
            worst_gradient(&f, 1e-3),
            worst_gradient(&f, 1e-5)
        );
    }
}

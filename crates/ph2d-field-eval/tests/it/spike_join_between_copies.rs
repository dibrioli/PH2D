//! **SONDA** — a junta entre as CÓPIAS que um modificador de repetição gera.
//!
//! > **Pedido do Enio, 2026-08-30:** *«em radial e outros modificadores que geram cópias da mesma
//! > peça não temos nem filet nem chamfer para a união entre as peças»*
//!
//! ⚠️ **Três perguntas, e a terceira é a que fura a peça se ninguém a fizer:**
//!
//! | pergunta | por que decide |
//! |---|---|
//! | a junta entre duas cópias **morde** o vinco? | é o pedido |
//! | **duas** células chegam com a junta ligada, ou é preciso a terceira? | o `min` de um subconjunto **superestima**, e superestimar é o erro que salta por cima da superfície |
//! | quanto vale `‖∇f‖`? | ⛔ o [`ph2d_field_eval::gradient_bound`] **declara** hoje que `Array`/`Radial`/`Mirror` lêem `1,000` e por isso não entram na conta. Uma junta filetada torna essa frase **falsa** |
//!
//! Corre com `cargo test -p ph2d-field-eval --test spike_join_between_copies -- --nocapture`.

use fidget::context::Tree;
use ph2d_field_eval::{Field, ops};

/// A esfera de raio `R`, a fixtura em que um vinco côncavo se mede sem ambiguidade.
fn esfera(r: f64) -> Tree {
    (Tree::x().square() + Tree::y().square() + Tree::z().square())
        .max(1.0e-30)
        .sqrt()
        - Tree::constant(r)
}

/// ⭐⭐⭐ **A UNIÃO COM O RAIO PORTADO** — a junta com o número multiplicado por um `gate` de `{0,1}`.
///
/// ⛔⛔ **Ela existe porque `blend(a, a) ≠ a`.** As leis de repetição desta casa **prendem** o índice
/// da cópia vizinha (`clamp` nas pontas da matriz, e `compare` que devolve `0` no centro exacto de
/// uma célula), e nesses pontos a "vizinha" **é a própria cópia**. Com `min` isso é inofensivo — o
/// `min` é idempotente. Com uma mistura não é: `union_round(a, a, r)` vale `r − √2·r = −0,414·r`
/// sobre um ponto da superfície, e a superfície **move-se**.
///
/// ⭐ O `gate` é `|vizinha − própria|`, que para índices inteiros já é exactamente `0` ou `1`.
///
/// ⛔⛔ **ESCALAR O RAIO PELO PORTÃO NÃO CHEGA, e a sonda mediu-o:** com `r = 0` o
/// [`ph2d_field_eval::ops::union_round`] **não** é o `min` — por FORA é, mas por DENTRO
/// `min(a,b).max(0) − ‖a⁻, b⁻‖` afasta-se dele, e o `‖∇f‖` lê `1,4142` num documento em que ninguém
/// pediu junta nenhuma. ⇒ o portão **escolhe entre as duas leis** em vez de encolher o número de uma:
/// `min + gate·(mistura − min)` degenera no `min` **exactamente**, dentro e fora.
fn union_gated(a: &Tree, b: &Tree, blend: ops::Blended, gate: &Tree) -> Tree {
    let duro = a.min(b.clone());
    match blend {
        ops::Blended::Sharp => duro,
        _ => {
            let misturado = ops::union(a, b, blend);
            duro.clone() + gate.clone() * (misturado - duro)
        }
    }
}

/// O sinal de `t` **sem o zero** — `+1` quando `t ≥ 0`, `−1` quando `t < 0`.
///
/// ⚠️ O `compare` da árvore devolve `{−1, 0, +1}`, e o `0` cai exactamente no centro de uma célula,
/// que é onde a superfície da cópia passa. Ver [`union_gated`].
fn sign_no_zero(t: &Tree) -> Tree {
    let step = (t.compare(Tree::constant(0.0)) + Tree::constant(1.0))
        .max(0.0)
        .min(1.0);
    step * Tree::constant(2.0) - Tree::constant(1.0)
}

/// A matriz linear **com junta**, na forma exacta da [`ph2d_field_eval`] — `celulas` diz quantas
/// vizinhas entram, e `gated` liga a lei do [`union_gated`].
fn array_com_junta(
    inner: &Tree,
    count: u32,
    spacing: f64,
    blend: ops::Blended,
    celulas: usize,
    gated: bool,
) -> Tree {
    let s = Tree::constant(spacing);
    let last = f64::from(count - 1);
    let raw = (Tree::x() / s.clone()).round();
    let k = raw.max(0.0).min(last);
    let toward = Tree::x() / s.clone() - k.clone();
    let passo = if gated {
        sign_no_zero(&toward)
    } else {
        toward.compare(Tree::constant(0.0))
    };
    let neighbour = (k.clone() + passo).max(0.0).min(last);
    let gate = if gated {
        (neighbour.clone() - k.clone()).abs()
    } else {
        Tree::constant(1.0)
    };
    let cell = |idx: Tree| inner.remap_xyz(Tree::x() - s.clone() * idx, Tree::y(), Tree::z());
    let mut acc = union_gated(&cell(k.clone()), &cell(neighbour.clone()), blend, &gate);
    if celulas >= 3 {
        // ⭐ As células de SEGURANÇA entram por `min`, nunca pela junta — ver [`union_gated`].
        let extra = |idx: Tree| cell(idx.max(0.0).min(last));
        acc = acc
            .min(extra(k - Tree::constant(1.0)))
            .min(extra(neighbour + Tree::constant(1.0)));
    }
    acc
}

/// A união **exacta de todas as cópias**, escrita cópia a cópia — o oráculo das leis acima.
fn array_oraculo(inner: &Tree, count: u32, spacing: f64, blend: ops::Blended) -> Tree {
    let cell = |i: u32| {
        inner.remap_xyz(
            Tree::x() - Tree::constant(spacing * f64::from(i)),
            Tree::y(),
            Tree::z(),
        )
    };
    let mut acc = cell(0);
    for i in 1..count {
        acc = ops::union(&acc, &cell(i), blend);
    }
    acc
}

/// O `y` da superfície acima de `(x, 0)`, por bissecção a partir de dentro.
fn surface_y(f: &Field, x: f64, hi: f64) -> Option<f64> {
    if f.at(x, 0.0, 0.0) > 0.0 {
        return None;
    }
    let (mut lo, mut hi) = (0.0f64, hi);
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if f.at(x, mid, 0.0) <= 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

/// O pior `‖∇f‖` na caixa que contém as duas cópias.
fn worst_gradient(f: &Field, spacing: f64, eps: f64) -> f64 {
    let mut hi = 0.0f64;
    for i in 0..49 {
        for j in 0..49 {
            for k in 0..49 {
                let u = |n: i32| f64::from(n) / 48.0;
                let x = u(i).mul_add(spacing + 1.2, -0.6);
                let y = u(j).mul_add(1.2, -0.6);
                let z = u(k).mul_add(1.2, -0.6);
                let g = f.gradient_norm(x, y, z, eps);
                if g.is_finite() && g > 1e-6 {
                    hi = hi.max(g);
                }
            }
        }
    }
    hi
}

const R: f64 = 0.35;
/// As duas esferas **cruzam-se**: é preciso haver vinco côncavo para haver o que filetar.
const S: f64 = 0.55;

/// ⭐ **A junta MORDE o vinco** — e o número entregue por cada carácter.
#[test]
fn measure_the_joint_between_two_copies() {
    // Onde o vinco está, sem junta: a circunferência de intersecção das duas esferas.
    let vinco = (R * R - S * S * 0.25).sqrt();
    println!("\n  vinco sem junta: y = {vinco:.5}  (x = {:.3})", S * 0.5);
    println!("\n  carácter | raio | y no vinco | mordida | pior |grad|");
    println!("  ---------|------|------------|---------|------------");
    for (nome, mk) in [
        (
            "Sharp   ",
            &(|_r: f64| ops::Blended::Sharp) as &dyn Fn(f64) -> ops::Blended,
        ),
        ("Exact   ", &(|r: f64| ops::Blended::Exact(r))),
        ("Chamfer ", &(|r: f64| ops::Blended::Chamfer(r))),
        ("Organic ", &(|r: f64| ops::Blended::Organic(r))),
    ] {
        for raio in [0.0f64, 0.04, 0.08, 0.12] {
            let tree = array_com_junta(&esfera(R), 2, S, mk(raio), 2, true);
            let f = Field::from_tree(&tree);
            let y = surface_y(&f, S * 0.5, 1.0).unwrap_or(f64::NAN);
            println!(
                "  {nome} | {raio:>4.2} | {y:>10.5} | {:>7.5} | {:>10.4}",
                y - vinco,
                worst_gradient(&f, S, 1e-3)
            );
        }
    }
}

/// ⭐⭐ **DUAS células chegam?** — a pergunta que o `min` de um subconjunto obriga a fazer.
///
/// ⚠️ Um `min` sobre menos cópias **superestima**, e superestimar salta por cima da superfície. Com
/// a junta ligada a pergunta volta a abrir-se, porque a mistura alcança mais longe do que o `min`.
#[test]
fn measure_two_cells_against_the_oracle() {
    println!(
        "\n  lei              | raio | pior SOBRE-estimativa | pior |Δ| | pior |grad|\n  \
         (a sobre-estimativa é a perigosa: é ela que salta por cima da superfície)"
    );
    for (nome, celulas, gated) in [
        ("2 celulas, cru  ", 2usize, false),
        ("2 celulas, gated", 2, true),
        ("4 celulas, gated", 3, true),
    ] {
        for raio in [0.0f64, 0.04, 0.08, 0.16] {
            let b = ops::Blended::Exact(raio);
            let cand = Field::from_tree(&array_com_junta(&esfera(R), 4, S, b, celulas, gated));
            let orac = Field::from_tree(&array_oraculo(&esfera(R), 4, S, b));
            let (mut acima, mut pior) = (0.0f64, 0.0f64);
            for i in 0..61 {
                for j in 0..61 {
                    for k in 0..61 {
                        let u = |n: i32| f64::from(n) / 60.0;
                        let x = u(i).mul_add(3.0f64.mul_add(S, 1.4), -0.7);
                        let y = u(j).mul_add(1.4, -0.7);
                        let z = u(k).mul_add(1.4, -0.7);
                        let d = cand.at(x, y, z) - orac.at(x, y, z);
                        if d.is_finite() {
                            acima = acima.max(d);
                            pior = pior.max(d.abs());
                        }
                    }
                }
            }
            println!(
                "  {nome} | {raio:>4.2} | {acima:>21.6} | {pior:>8.6} | {:>10.4}",
                worst_gradient(&cand, S, 1e-3)
            );
        }
    }
}

/// ⭐ **A grelha não é o que se mede** — o mesmo `‖∇f‖` com `ε` cem vezes menor.
#[test]
fn measure_the_joint_gradient_is_not_a_sampling_artefact() {
    println!("\n  carácter | raio | grad(1e-3) | grad(1e-5)");
    for (nome, b) in [
        ("Exact  ", ops::Blended::Exact(0.08)),
        ("Chamfer", ops::Blended::Chamfer(0.08)),
        ("Organic", ops::Blended::Organic(0.08)),
        ("Sharp  ", ops::Blended::Sharp),
    ] {
        let f = Field::from_tree(&array_com_junta(&esfera(R), 2, S, b, 2, true));
        println!(
            "  {nome} | 0.08 | {:>10.4} | {:>10.4}",
            worst_gradient(&f, S, 1e-3),
            worst_gradient(&f, S, 1e-5)
        );
    }
}

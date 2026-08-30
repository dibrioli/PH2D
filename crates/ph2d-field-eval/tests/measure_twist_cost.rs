//! **SONDA** — a torção nunca superestima a distância, e quanto o divisor precisa de valer.
//!
//! O irmão do `measure_taper_cost`, e pela mesma razão: a derivação à mão é uma hipótese.

use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
use ph2d_field_eval::{Field, leaf};

/// A fixtura: uma caixa larga o suficiente para haver `r` grande onde a torção morde.
fn caixa() -> FieldDoc {
    FieldDoc::new(
        vec![leaf(
            Primitive::Box {
                half: [0.6, 0.6, 0.35],
                round: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("peça")
}

#[test]
fn measure_twist_cost() {
    // ⚠️ A sonda mede o campo TORCIDO construído à mão, porque a ligação ao `Unary` ainda não
    // existe: o que se está a escolher aqui é a constante que ela vai usar.
    let doc = caixa();
    let cru = ph2d_field_eval::compile(&doc);
    println!(
        "\n{:>10} | {:>9} | {}",
        "voltas/un", "k rad/un", "max |grad| por divisor"
    );
    print!("{:>10} | {:>9} |", "", "");
    for safety in SAFETIES {
        print!(" {safety:>8.2}");
    }
    println!();
    for turns in [0.0f64, 0.05, 0.1, 0.2, 0.3, 0.5, 0.75, 1.0, 1.5] {
        let k = turns * std::f64::consts::TAU;
        print!("{turns:>10.2} | {k:>9.4} |");
        for safety in SAFETIES {
            let f = Field::from_tree(&ph2d_field_eval::probe_twist(&cru, k, safety));
            let mut hi = 0.0f64;
            for i in 0..29 {
                for j in 0..29 {
                    for m in 0..29 {
                        let p = |n: i32| f64::from(n) / 14.0 - 1.0;
                        let g = f.gradient_norm(p(i), p(j), p(m), 1e-3);
                        if g.is_finite() && g > 1e-6 {
                            hi = hi.max(g);
                        }
                    }
                }
            }
            print!(" {hi:>8.4}");
        }
        println!();
    }
}

/// Os divisores varridos — `1,0` é a derivação à mão (o tecto espectral cru).
const SAFETIES: [f64; 5] = [1.0, 1.5, 2.0, 3.0, 4.0];

/// ⭐ **O divisor CONSTANTE** — `σ_max(k·R)` com `R` o raio da peça em torno do eixo.
///
/// A caixa é `half = [0.6, 0.6, 0.35]` ⇒ o canto está a `R = √(0.6² + 0.6²) = 0,8485` do eixo Z.
#[test]
fn measure_twist_with_a_constant_divisor() {
    const R: f64 = 0.848_528_137;
    let doc = caixa();
    let cru = ph2d_field_eval::compile(&doc);
    println!(
        "\n{:>10} | {:>9} | {:>9} | {:>11}",
        "voltas/un", "k", "sigma(kR)", "max |grad|"
    );
    for turns in [0.0f64, 0.05, 0.1, 0.2, 0.3, 0.5, 0.75, 1.0, 1.5, 2.0] {
        let k = turns * std::f64::consts::TAU;
        let sigma = ph2d_field_eval::probe_twist_sigma(k * R);
        let f = Field::from_tree(&ph2d_field_eval::probe_twist_const(&cru, k, sigma));
        let mut hi = 0.0f64;
        for i in 0..29 {
            for j in 0..29 {
                for m in 0..29 {
                    let p = |n: i32| f64::from(n) / 14.0 - 1.0;
                    let g = f.gradient_norm(p(i), p(j), p(m), 1e-3);
                    if g.is_finite() && g > 1e-6 {
                        hi = hi.max(g);
                    }
                }
            }
        }
        println!("{turns:>10.2} | {k:>9.4} | {sigma:>9.4} | {hi:>11.4}");
    }
}

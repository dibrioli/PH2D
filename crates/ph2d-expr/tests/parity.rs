//! CPU↔WGSL parity + lowering coverage (audit C4; ADR-0033 §3 "golden parity
//! per op"). True bit-level GPU parity needs a device (lands with the shader
//! evaluator, Track B); here we (a) gate that every `Func` lowers — the
//! exhaustive match breaks compilation if a variant is added without a parity
//! entry — and (b) pin the semantics that previously diverged (fract, mix,
//! non-finite consts, identifier sanitization, the noise helper).

use ph2d_expr::{BinOp, Bindings, Expr, Func, eval, to_wgsl, wgsl_prelude};

struct Zero;
impl Bindings for Zero {
    fn attr(&self, _: &str) -> f32 {
        0.0
    }
    fn param(&self, _: &str) -> f32 {
        0.0
    }
}

/// Exhaustive over `Func` — adding a variant without handling it here fails to
/// compile, forcing a parity entry (the coverage gate).
fn func_arity(f: Func) -> usize {
    match f {
        Func::Sin
        | Func::Cos
        | Func::Abs
        | Func::Sqrt
        | Func::Floor
        | Func::Fract
        | Func::Noise => 1,
        Func::Min | Func::Max => 2,
        Func::Mix => 3,
    }
}

#[test]
fn every_func_lowers_and_evals() {
    let all = [
        Func::Sin,
        Func::Cos,
        Func::Abs,
        Func::Sqrt,
        Func::Floor,
        Func::Fract,
        Func::Min,
        Func::Max,
        Func::Mix,
        Func::Noise,
    ];
    for f in all {
        let args: Vec<Expr> = (0..func_arity(f)).map(|_| Expr::cnst(0.5)).collect();
        let e = Expr::call(f, args);
        assert!(!to_wgsl(&e).is_empty(), "{f:?} produced empty WGSL");
        let v = eval(&e, &Zero);
        assert!(v.is_finite() || f == Func::Sqrt, "{f:?} eval = {v}");
    }
}

#[test]
fn fract_matches_wgsl_semantics() {
    // WGSL fract(x) = x - floor(x), always >= 0 (audit C1).
    let e = Expr::call(Func::Fract, vec![Expr::cnst(-0.25)]);
    assert_eq!(eval(&e, &Zero), 0.75);
    assert_eq!(to_wgsl(&e), "fract(-0.25)");
}

#[test]
fn mix_is_algebraic_lerp() {
    let e = Expr::call(
        Func::Mix,
        vec![Expr::cnst(2.0), Expr::cnst(10.0), Expr::cnst(0.25)],
    );
    assert_eq!(eval(&e, &Zero), 2.0 * (1.0 - 0.25) + 10.0 * 0.25); // 4.0
    assert_eq!(to_wgsl(&e), "mix(2.0, 10.0, 0.25)");
}

#[test]
fn nonfinite_const_emits_valid_wgsl_bitcast() {
    // audit C2: NaN/inf have no WGSL literal.
    assert_eq!(to_wgsl(&Expr::cnst(f32::NAN)), "bitcast<f32>(0x7fc00000u)");
    assert_eq!(
        to_wgsl(&Expr::cnst(f32::INFINITY)),
        "bitcast<f32>(0x7f800000u)"
    );
    assert_eq!(
        to_wgsl(&Expr::cnst(f32::NEG_INFINITY)),
        "bitcast<f32>(0xff800000u)"
    );
}

#[test]
fn identifiers_are_sanitized() {
    // audit A1: a dotted/hyphenated name must still parse as a WGSL identifier.
    assert_eq!(to_wgsl(&Expr::attr("P.x")), "attr_P_x");
    assert_eq!(to_wgsl(&Expr::param("a-b")), "param_a_b");
}

#[test]
fn comparisons_lower_and_eval_consistently() {
    // exhaustive over the comparison/logic BinOps that have non-arithmetic WGSL.
    let cases = [
        (BinOp::Lt, 3.0, 5.0, 1.0),
        (BinOp::Gt, 3.0, 5.0, 0.0),
        (BinOp::Eq, 5.0, 5.0, 1.0),
        (BinOp::And, 1.0, 0.0, 0.0),
        (BinOp::Or, 1.0, 0.0, 1.0),
    ];
    for (op, a, b, expected) in cases {
        let e = Expr::bin(op, Expr::cnst(a), Expr::cnst(b));
        assert_eq!(eval(&e, &Zero), expected, "{op:?} eval");
        assert!(
            to_wgsl(&e).starts_with("f32("),
            "{op:?} should cast to f32: {}",
            to_wgsl(&e)
        );
    }
}

#[test]
fn noise_helper_is_in_prelude() {
    // audit C3: the GPU noise body must be emitted, bit-matching the CPU hash.
    let prelude = wgsl_prelude();
    assert!(prelude.contains("fn ph2d_noise1(x: f32) -> f32"));
    assert!(prelude.contains("0x9e3779b9u")); // same seed as eval::noise1
    assert_eq!(
        to_wgsl(&Expr::call(Func::Noise, vec![Expr::attr("x")])),
        "ph2d_noise1(attr_x)"
    );
}

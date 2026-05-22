//! Native CPU evaluator — the reference semantics every other lowering must
//! match. A consuming node implements [`Bindings`] to feed the current
//! element's attribute values and the node's parameters.

use crate::expr::{BinOp, Expr, Func, UnaryOp};

/// Supplies the values an expression reads. `attr` is per-element (the current
/// stream element); `param` is constant across the stream. Unknown names
/// return `0.0` (a missing input is zero, not a panic).
pub trait Bindings {
    fn attr(&self, name: &str) -> f32;
    fn param(&self, name: &str) -> f32;
}

/// Evaluate `expr` against `bindings`. Total: malformed arity reads missing
/// args as `0.0` rather than panicking, so a node bug degrades to a value, not
/// a crash. Pure given the same bindings.
pub fn eval(expr: &Expr, bindings: &dyn Bindings) -> f32 {
    match expr {
        Expr::Const(v) => *v,
        Expr::Attr(name) => bindings.attr(name),
        Expr::Param(name) => bindings.param(name),
        Expr::Unary(UnaryOp::Neg, e) => -eval(e, bindings),
        Expr::Binary(op, l, r) => {
            let a = eval(l, bindings);
            let b = eval(r, bindings);
            match op {
                BinOp::Add => a + b,
                BinOp::Sub => a - b,
                BinOp::Mul => a * b,
                BinOp::Div => a / b,
                BinOp::Lt => bool_f32(a < b),
                BinOp::Gt => bool_f32(a > b),
                BinOp::Eq => bool_f32(a == b),
                BinOp::And => bool_f32(truthy(a) && truthy(b)),
                BinOp::Or => bool_f32(truthy(a) || truthy(b)),
            }
        }
        Expr::Call(f, args) => {
            let arg = |i: usize| args.get(i).map(|e| eval(e, bindings)).unwrap_or(0.0);
            match f {
                Func::Sin => arg(0).sin(),
                Func::Cos => arg(0).cos(),
                Func::Abs => arg(0).abs(),
                Func::Sqrt => arg(0).sqrt(),
                Func::Floor => arg(0).floor(),
                Func::Fract => arg(0).fract(),
                Func::Min => arg(0).min(arg(1)),
                Func::Max => arg(0).max(arg(1)),
                Func::Mix => {
                    let (a, b, t) = (arg(0), arg(1), arg(2));
                    a + (b - a) * t
                }
                Func::Noise => noise1(arg(0)),
            }
        }
        Expr::Select { cond, a, b } => {
            if truthy(eval(cond, bindings)) {
                eval(a, bindings)
            } else {
                eval(b, bindings)
            }
        }
    }
}

fn truthy(x: f32) -> bool {
    x != 0.0
}

fn bool_f32(b: bool) -> f32 {
    if b { 1.0 } else { 0.0 }
}

/// Deterministic value noise in `[0, 1)` from the bits of `x` — RNG-free and
/// reproducible (no `thread_rng`), and bit-identical across platforms (integer
/// hashing only). The WGSL lowering emits a matching `ph2d_noise1` helper.
pub fn noise1(x: f32) -> f32 {
    let mut h = x.to_bits();
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
    // top 24 bits → [0,1)
    (h >> 8) as f32 / (1u32 << 24) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::Expr;
    use std::collections::BTreeMap;

    struct Map {
        attrs: BTreeMap<String, f32>,
        params: BTreeMap<String, f32>,
    }
    impl Bindings for Map {
        fn attr(&self, name: &str) -> f32 {
            self.attrs.get(name).copied().unwrap_or(0.0)
        }
        fn param(&self, name: &str) -> f32 {
            self.params.get(name).copied().unwrap_or(0.0)
        }
    }
    fn binds(attrs: &[(&str, f32)], params: &[(&str, f32)]) -> Map {
        Map {
            attrs: attrs.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            params: params.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        }
    }

    #[test]
    fn arithmetic_and_attrs() {
        // (x + 2) * scale
        let e = Expr::bin(
            BinOp::Mul,
            Expr::bin(BinOp::Add, Expr::attr("x"), Expr::cnst(2.0)),
            Expr::param("scale"),
        );
        let b = binds(&[("x", 3.0)], &[("scale", 10.0)]);
        assert_eq!(eval(&e, &b), 50.0);
    }

    #[test]
    fn missing_names_are_zero() {
        let b = binds(&[], &[]);
        assert_eq!(eval(&Expr::attr("nope"), &b), 0.0);
        assert_eq!(eval(&Expr::param("nope"), &b), 0.0);
    }

    #[test]
    fn comparisons_and_select() {
        // select(x < 5, 100, 200)
        let e = Expr::select(
            Expr::bin(BinOp::Lt, Expr::attr("x"), Expr::cnst(5.0)),
            Expr::cnst(100.0),
            Expr::cnst(200.0),
        );
        assert_eq!(eval(&e, &binds(&[("x", 3.0)], &[])), 100.0);
        assert_eq!(eval(&e, &binds(&[("x", 9.0)], &[])), 200.0);
    }

    #[test]
    fn mix_and_funcs() {
        // mix(0, 10, 0.25) = 2.5
        let e = Expr::call(
            Func::Mix,
            vec![Expr::cnst(0.0), Expr::cnst(10.0), Expr::cnst(0.25)],
        );
        assert_eq!(eval(&e, &binds(&[], &[])), 2.5);
        assert_eq!(
            eval(
                &Expr::call(Func::Abs, vec![Expr::cnst(-4.0)]),
                &binds(&[], &[])
            ),
            4.0
        );
    }

    #[test]
    fn noise_is_deterministic_and_in_range() {
        for x in [0.0_f32, 1.0, -3.5, 1234.5] {
            let n = noise1(x);
            assert!((0.0..1.0).contains(&n), "noise out of range: {n}");
            assert_eq!(noise1(x), n, "noise must be pure");
        }
        assert_ne!(noise1(1.0), noise1(2.0));
    }
}

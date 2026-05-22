//! WGSL lowering: emit an expression as a WGSL **expression string** (the GPU
//! target). The shader pass that hosts it (Track B / W2) provides the bindings
//! the identifiers resolve to — `attr_<name>` (per-fragment input), `param_<name>`
//! (uniform) — and a `fn ph2d_noise1(x: f32) -> f32` helper matching
//! [`crate::eval::noise1`]. Everything stays `f32`-typed; comparisons/booleans
//! are cast to `f32` so the result composes like any other scalar.
//!
//! Subexpressions are fully parenthesized — correctness over terseness, no
//! reliance on WGSL operator precedence.

use crate::expr::{BinOp, Expr, Func, UnaryOp};

/// Emit `expr` as a WGSL expression string.
pub fn to_wgsl(expr: &Expr) -> String {
    match expr {
        Expr::Const(v) => format!("{v:?}"), // Debug gives a decimal: 2.0, 0.25, -3.5
        Expr::Attr(name) => format!("attr_{name}"),
        Expr::Param(name) => format!("param_{name}"),
        Expr::Unary(UnaryOp::Neg, e) => format!("(-{})", to_wgsl(e)),
        Expr::Binary(op, l, r) => {
            let (l, r) = (to_wgsl(l), to_wgsl(r));
            match op {
                BinOp::Add => format!("({l} + {r})"),
                BinOp::Sub => format!("({l} - {r})"),
                BinOp::Mul => format!("({l} * {r})"),
                BinOp::Div => format!("({l} / {r})"),
                BinOp::Lt => format!("f32({l} < {r})"),
                BinOp::Gt => format!("f32({l} > {r})"),
                BinOp::Eq => format!("f32({l} == {r})"),
                BinOp::And => format!("f32(({l} != 0.0) && ({r} != 0.0))"),
                BinOp::Or => format!("f32(({l} != 0.0) || ({r} != 0.0))"),
            }
        }
        Expr::Call(f, args) => {
            let rendered: Vec<String> = args.iter().map(to_wgsl).collect();
            format!("{}({})", wgsl_fn(*f), rendered.join(", "))
        }
        Expr::Select { cond, a, b } => {
            // WGSL: select(false_value, true_value, condition).
            format!(
                "select({}, {}, ({}) != 0.0)",
                to_wgsl(b),
                to_wgsl(a),
                to_wgsl(cond)
            )
        }
    }
}

fn wgsl_fn(f: Func) -> &'static str {
    match f {
        Func::Sin => "sin",
        Func::Cos => "cos",
        Func::Abs => "abs",
        Func::Sqrt => "sqrt",
        Func::Floor => "floor",
        Func::Fract => "fract",
        Func::Min => "min",
        Func::Max => "max",
        Func::Mix => "mix",
        Func::Noise => "ph2d_noise1",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::Expr;

    #[test]
    fn arithmetic_with_attr_and_param() {
        // (x + 2) * scale
        let e = Expr::bin(
            BinOp::Mul,
            Expr::bin(BinOp::Add, Expr::attr("x"), Expr::cnst(2.0)),
            Expr::param("scale"),
        );
        assert_eq!(to_wgsl(&e), "((attr_x + 2.0) * param_scale)");
    }

    #[test]
    fn select_and_comparison() {
        let e = Expr::select(
            Expr::bin(BinOp::Lt, Expr::attr("x"), Expr::cnst(5.0)),
            Expr::cnst(100.0),
            Expr::cnst(200.0),
        );
        assert_eq!(
            to_wgsl(&e),
            "select(200.0, 100.0, (f32(attr_x < 5.0)) != 0.0)"
        );
    }

    #[test]
    fn functions() {
        let e = Expr::call(
            Func::Mix,
            vec![Expr::cnst(0.0), Expr::cnst(10.0), Expr::attr("t")],
        );
        assert_eq!(to_wgsl(&e), "mix(0.0, 10.0, attr_t)");
        let n = Expr::call(Func::Noise, vec![Expr::attr("x")]);
        assert_eq!(to_wgsl(&n), "ph2d_noise1(attr_x)");
    }

    #[test]
    fn neg() {
        assert_eq!(
            to_wgsl(&Expr::Unary(UnaryOp::Neg, Box::new(Expr::attr("x")))),
            "(-attr_x)"
        );
    }
}

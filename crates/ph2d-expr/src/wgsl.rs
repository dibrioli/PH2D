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

/// The WGSL helper definitions that emitted expressions may reference. Today:
/// `ph2d_noise1`, **bit-identical** to [`crate::eval::noise1`] (same integer
/// hash) so CPU and GPU noise agree. A shader pass hosting a lowered expression
/// must include this prelude (audit C3).
pub fn wgsl_prelude() -> &'static str {
    "fn ph2d_noise1(x: f32) -> f32 {\n    \
     let xc = select(x, 0.0, x == 0.0);\n    \
     var h: u32 = bitcast<u32>(xc) ^ 0x9e3779b9u;\n    \
     h = h ^ (h >> 16u);\n    \
     h = h * 0x7feb352du;\n    \
     h = h ^ (h >> 15u);\n    \
     h = h * 0x846ca68bu;\n    \
     h = h ^ (h >> 16u);\n    \
     return f32(h >> 8u) / 16777216.0;\n}\n"
}

/// Emit a WGSL `f32` literal. Non-finite values have no valid WGSL literal, so
/// emit a `bitcast` (audit C2).
fn wgsl_float(v: f32) -> String {
    if v.is_finite() {
        format!("{v:?}") // finite f32 Debug is always a valid WGSL float (decimal or exponent)
    } else if v.is_nan() {
        "bitcast<f32>(0x7fc00000u)".to_string()
    } else if v > 0.0 {
        "bitcast<f32>(0x7f800000u)".to_string()
    } else {
        "bitcast<f32>(0xff800000u)".to_string()
    }
}

/// Build a WGSL identifier from a prefix + an attribute/param name, mapping any
/// non-identifier character to `_` so the output always parses (audit A1).
/// Names that differ only in non-identifier chars can collide — the naming
/// convention keeps attribute/param names to plain identifiers.
fn wgsl_ident(prefix: &str, name: &str) -> String {
    let mut s = String::from(prefix);
    for c in name.chars() {
        s.push(if c.is_ascii_alphanumeric() || c == '_' {
            c
        } else {
            '_'
        });
    }
    s
}

/// Emit `expr` as a WGSL expression string.
pub fn to_wgsl(expr: &Expr) -> String {
    match expr {
        Expr::Const(v) => wgsl_float(*v),
        Expr::Attr(name) => wgsl_ident("attr_", name),
        Expr::Param(name) => wgsl_ident("param_", name),
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

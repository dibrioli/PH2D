//! Field → column: evaluate an expression over every element of an attribute
//! [`Stream`], the "VOP → cook" step (ADR-0033). This is the glue a node uses
//! so the fan-out does not reinvent the binding (audit A3).

use crate::eval::{Bindings, eval};
use crate::expr::Expr;
use ph2d_nodegraph::attr::{Column, Stream};

/// [`Bindings`] backed by one element (`row`) of a stream plus a parameter
/// lookup. `attr` reads the row's value from a scalar column (non-scalar or
/// missing → `0.0`; vector attributes are a later extension).
pub struct StreamBindings<'a> {
    pub stream: &'a Stream,
    pub row: usize,
    pub params: &'a dyn Fn(&str) -> f32,
}

impl Bindings for StreamBindings<'_> {
    fn attr(&self, name: &str) -> f32 {
        match self.stream.get(name) {
            Some(Column::Scalar(v)) => v.get(self.row).copied().unwrap_or(0.0),
            _ => 0.0,
        }
    }
    fn param(&self, name: &str) -> f32 {
        (self.params)(name)
    }
}

/// Evaluate `expr` once per element of `stream`, producing a scalar column of
/// length `stream.count()`. `params` resolves the expression's `Param` reads.
pub fn eval_column(expr: &Expr, stream: &Stream, params: &dyn Fn(&str) -> f32) -> Vec<f32> {
    (0..stream.count())
        .map(|row| {
            let b = StreamBindings {
                stream,
                row,
                params,
            };
            eval(expr, &b)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{BinOp, Expr};
    use ph2d_nodegraph::attr::{Column, Stream};

    #[test]
    fn evaluates_a_field_over_a_stream() {
        // out = x * 2 + scale, over x = [1,2,3], scale = 10  ->  [12,14,16]
        let stream = Stream::new(3).with("x", Column::Scalar(vec![1.0, 2.0, 3.0]));
        let expr = Expr::bin(
            BinOp::Add,
            Expr::bin(BinOp::Mul, Expr::attr("x"), Expr::cnst(2.0)),
            Expr::param("scale"),
        );
        let params = |name: &str| if name == "scale" { 10.0 } else { 0.0 };
        assert_eq!(eval_column(&expr, &stream, &params), vec![12.0, 14.0, 16.0]);
    }

    #[test]
    fn missing_attr_is_zero() {
        let stream = Stream::new(2).with("x", Column::Scalar(vec![1.0, 2.0]));
        let expr = Expr::attr("nonexistent");
        let params = |_: &str| 0.0;
        assert_eq!(eval_column(&expr, &stream, &params), vec![0.0, 0.0]);
    }
}

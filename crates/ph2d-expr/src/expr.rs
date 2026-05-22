//! The expression IR. Scalar-`f32`-valued (the common case for per-element
//! math); vector/matrix expressions are a later extension. Comparisons and
//! booleans are encoded as `0.0`/`1.0` so the whole IR stays single-typed.

/// A per-element scalar expression.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// A literal constant.
    Const(f32),
    /// Read a named attribute of the current stream element (Blender-style
    /// field input).
    Attr(String),
    /// Read a static node parameter (constant across the stream).
    Param(String),
    /// Unary operation.
    Unary(UnaryOp, Box<Expr>),
    /// Binary operation.
    Binary(BinOp, Box<Expr>, Box<Expr>),
    /// Built-in function call.
    Call(Func, Vec<Expr>),
    /// `if cond != 0 then a else b` (branch-free select).
    Select {
        cond: Box<Expr>,
        a: Box<Expr>,
        b: Box<Expr>,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    /// `<` → `1.0`/`0.0`.
    Lt,
    /// `>` → `1.0`/`0.0`.
    Gt,
    /// `==` → `1.0`/`0.0`.
    Eq,
    /// logical AND of truthiness → `1.0`/`0.0`.
    And,
    /// logical OR of truthiness → `1.0`/`0.0`.
    Or,
}

/// Built-in functions. Each documents its arity; the evaluator and the WGSL
/// emitter agree on it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Func {
    Sin,   // (x)
    Cos,   // (x)
    Abs,   // (x)
    Sqrt,  // (x)
    Floor, // (x)
    Fract, // (x)
    Min,   // (a, b)
    Max,   // (a, b)
    Mix,   // (a, b, t) → a + (b-a)*t
    Noise, // (x) → deterministic value noise in [0,1)
}

impl Expr {
    // Small ergonomic constructors so node code reads cleanly.
    pub fn cnst(v: f32) -> Self {
        Expr::Const(v)
    }
    pub fn attr(name: &str) -> Self {
        Expr::Attr(name.to_string())
    }
    pub fn param(name: &str) -> Self {
        Expr::Param(name.to_string())
    }
    pub fn call(f: Func, args: Vec<Expr>) -> Self {
        Expr::Call(f, args)
    }
    pub fn bin(op: BinOp, a: Expr, b: Expr) -> Self {
        Expr::Binary(op, Box::new(a), Box::new(b))
    }
    pub fn select(cond: Expr, a: Expr, b: Expr) -> Self {
        Expr::Select {
            cond: Box::new(cond),
            a: Box::new(a),
            b: Box::new(b),
        }
    }
}

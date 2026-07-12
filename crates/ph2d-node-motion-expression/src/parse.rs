//! A tiny **VEX-lite parser**: text → the frozen `ph2d_expr::Expr` IR (ADR-0033). A
//! recursive-descent grammar over arithmetic, comparisons, the ten built-in functions,
//! and a `select(cond, a, b)` ternary. Identifiers become `Expr::Attr` (the node's
//! `Bindings` resolves `i`/`n`/`t`/columns/params); numbers become `Expr::Const`.
//! Dependency-free, deterministic — no parser generator, just a hand-written pratt-ish
//! descent so the error path is a plain `Result` the node turns into a fallback.

use ph2d_expr::{BinOp, Expr, Func, UnaryOp};

/// Tokens of the little language.
#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f32),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Lt,
    Gt,
    EqEq,
    AndAnd,
    OrOr,
    LParen,
    RParen,
    Comma,
}

/// Split `src` into tokens (whitespace-skipping). Returns an error string on an
/// unexpected character or a malformed number.
fn lex(src: &str) -> Result<Vec<Tok>, String> {
    let b = src.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < b.len() {
        let c = b[i] as char;
        match c {
            c if c.is_whitespace() => i += 1,
            '+' => {
                out.push(Tok::Plus);
                i += 1;
            }
            '-' => {
                out.push(Tok::Minus);
                i += 1;
            }
            '*' => {
                out.push(Tok::Star);
                i += 1;
            }
            '/' => {
                out.push(Tok::Slash);
                i += 1;
            }
            '<' => {
                out.push(Tok::Lt);
                i += 1;
            }
            '>' => {
                out.push(Tok::Gt);
                i += 1;
            }
            '(' => {
                out.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                out.push(Tok::RParen);
                i += 1;
            }
            ',' => {
                out.push(Tok::Comma);
                i += 1;
            }
            '=' if b.get(i + 1) == Some(&b'=') => {
                out.push(Tok::EqEq);
                i += 2;
            }
            '&' if b.get(i + 1) == Some(&b'&') => {
                out.push(Tok::AndAnd);
                i += 2;
            }
            '|' if b.get(i + 1) == Some(&b'|') => {
                out.push(Tok::OrOr);
                i += 2;
            }
            c if c.is_ascii_digit() || c == '.' => {
                let start = i;
                while i < b.len() && ((b[i] as char).is_ascii_digit() || b[i] == b'.') {
                    i += 1;
                }
                let num: f32 = src[start..i]
                    .parse()
                    .map_err(|_| format!("bad number `{}`", &src[start..i]))?;
                out.push(Tok::Num(num));
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < b.len() && ((b[i] as char).is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                out.push(Tok::Ident(src[start..i].to_string()));
            }
            other => return Err(format!("unexpected char `{other}`")),
        }
    }
    Ok(out)
}

/// A recursive-descent parser over the token stream.
struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn eat(&mut self, t: &Tok) -> Result<(), String> {
        if self.peek() == Some(t) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("expected {t:?}, found {:?}", self.peek()))
        }
    }

    // expr := or
    fn expr(&mut self) -> Result<Expr, String> {
        self.or()
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut lhs = self.and()?;
        while self.peek() == Some(&Tok::OrOr) {
            self.pos += 1;
            let rhs = self.and()?;
            lhs = Expr::Binary(BinOp::Or, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut lhs = self.cmp()?;
        while self.peek() == Some(&Tok::AndAnd) {
            self.pos += 1;
            let rhs = self.cmp()?;
            lhs = Expr::Binary(BinOp::And, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn cmp(&mut self) -> Result<Expr, String> {
        let lhs = self.add()?;
        let op = match self.peek() {
            Some(Tok::Lt) => BinOp::Lt,
            Some(Tok::Gt) => BinOp::Gt,
            Some(Tok::EqEq) => BinOp::Eq,
            _ => return Ok(lhs),
        };
        self.pos += 1;
        let rhs = self.add()?;
        Ok(Expr::Binary(op, Box::new(lhs), Box::new(rhs)))
    }

    fn add(&mut self) -> Result<Expr, String> {
        let mut lhs = self.mul()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Plus) => BinOp::Add,
                Some(Tok::Minus) => BinOp::Sub,
                _ => break,
            };
            self.pos += 1;
            let rhs = self.mul()?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn mul(&mut self) -> Result<Expr, String> {
        let mut lhs = self.unary()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Star) => BinOp::Mul,
                Some(Tok::Slash) => BinOp::Div,
                _ => break,
            };
            self.pos += 1;
            let rhs = self.unary()?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.peek() == Some(&Tok::Minus) {
            self.pos += 1;
            return Ok(Expr::Unary(UnaryOp::Neg, Box::new(self.unary()?)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Tok::Num(n)) => Ok(Expr::Const(n)),
            Some(Tok::LParen) => {
                let e = self.expr()?;
                self.eat(&Tok::RParen)?;
                Ok(e)
            }
            Some(Tok::Ident(name)) => {
                if self.peek() == Some(&Tok::LParen) {
                    self.pos += 1;
                    let args = self.args()?;
                    self.eat(&Tok::RParen)?;
                    make_call(&name, args)
                } else {
                    Ok(Expr::Attr(name))
                }
            }
            other => Err(format!("unexpected token {other:?}")),
        }
    }

    fn args(&mut self) -> Result<Vec<Expr>, String> {
        let mut out = Vec::new();
        if self.peek() == Some(&Tok::RParen) {
            return Ok(out);
        }
        loop {
            out.push(self.expr()?);
            if self.peek() == Some(&Tok::Comma) {
                self.pos += 1;
            } else {
                break;
            }
        }
        Ok(out)
    }
}

/// Build a function call node from a name + args (or the `select` ternary).
fn make_call(name: &str, args: Vec<Expr>) -> Result<Expr, String> {
    let want = |n: usize| -> Result<(), String> {
        if args.len() == n {
            Ok(())
        } else {
            Err(format!("`{name}` wants {n} args, got {}", args.len()))
        }
    };
    let f = match name {
        "sin" => Func::Sin,
        "cos" => Func::Cos,
        "abs" => Func::Abs,
        "sqrt" => Func::Sqrt,
        "floor" => Func::Floor,
        "fract" => Func::Fract,
        "min" => Func::Min,
        "max" => Func::Max,
        "mix" => Func::Mix,
        "noise" => Func::Noise,
        "select" => {
            want(3)?;
            let mut it = args.into_iter();
            return Ok(Expr::Select {
                cond: Box::new(it.next().unwrap()),
                a: Box::new(it.next().unwrap()),
                b: Box::new(it.next().unwrap()),
            });
        }
        other => return Err(format!("unknown function `{other}`")),
    };
    let arity = match f {
        Func::Min | Func::Max => 2,
        Func::Mix => 3,
        _ => 1,
    };
    want(arity)?;
    Ok(Expr::Call(f, args))
}

/// Parse `src` into an `Expr`, or an error message.
pub(crate) fn parse(src: &str) -> Result<Expr, String> {
    let toks = lex(src)?;
    let mut p = Parser { toks, pos: 0 };
    let e = p.expr()?;
    if p.pos != p.toks.len() {
        return Err(format!("trailing tokens from {:?}", p.peek()));
    }
    Ok(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Precedence: `1 + 2 * 3` parses as `1 + (2*3)`, not `(1+2)*3`.
    #[test]
    fn multiplication_binds_tighter_than_addition() {
        let e = parse("1 + 2 * 3").unwrap();
        match e {
            Expr::Binary(BinOp::Add, a, b) => {
                assert!(matches!(*a, Expr::Const(x) if x == 1.0));
                assert!(matches!(*b, Expr::Binary(BinOp::Mul, _, _)), "rhs is 2*3");
            }
            _ => panic!("expected Add at the top: {e:?}"),
        }
    }

    /// Functions, identifiers and select parse; unknowns error.
    #[test]
    fn functions_identifiers_and_select() {
        assert!(matches!(parse("sin(i)").unwrap(), Expr::Call(Func::Sin, _)));
        assert!(matches!(parse("t").unwrap(), Expr::Attr(_)));
        assert!(matches!(
            parse("select(i, 1, 2)").unwrap(),
            Expr::Select { .. }
        ));
        assert!(parse("bogus(x)").is_err(), "unknown function errors");
        assert!(parse("min(1)").is_err(), "wrong arity errors");
        assert!(parse("1 +").is_err(), "incomplete errors");
        assert!(parse("1 2").is_err(), "trailing tokens error");
    }

    /// Unary minus and parentheses.
    #[test]
    fn unary_and_parens() {
        assert!(matches!(parse("-i").unwrap(), Expr::Unary(UnaryOp::Neg, _)));
        // `-(1+2)` negates the group.
        assert!(matches!(
            parse("-(1 + 2)").unwrap(),
            Expr::Unary(UnaryOp::Neg, _)
        ));
    }
}

#![forbid(unsafe_code)]
//! A tiny **VEX-lite parser**: text -> the frozen `ph2d_expr::Expr` IR (ADR-0033/0039).
//!
//! A hand-written recursive-descent grammar over arithmetic, comparisons, the ten
//! built-in functions, a `select(cond, a, b)` ternary, and the `wiggle(freq, amp)`
//! sugar. Identifiers become `Expr::Attr` — the CONSUMER's [`ph2d_expr::Bindings`]
//! resolves them (`i`/`n`/`t` for a Motion node; `time`/`value`/`Name.prop` for a
//! timeline expression). Numbers become `Expr::Const`. Dependency-free and
//! deterministic; the error path is a plain `Result` the caller turns into a fallback.
//!
//! ⚠️ **This is the ONE parser** (ADR-0144). It was extracted from
//! `ph2d-node-motion-expression` so the timeline and the Motion node share it — two
//! parsers for one IR would drift in silence. It lives in its own leaf crate rather
//! than inside the FROZEN `ph2d-expr` (ADR-0039): the IR shape is a contract, but
//! *how text becomes that IR* is free to grow.

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
                // A `.` joins the identifier ONLY when an alphabetic/underscore
                // follows it (ADR-0144 prop-links: `Sprite.x`). So `2.5`/`.5`
                // still lex as numbers, and `x.5` stops the ident at `x`.
                while i < b.len() {
                    let ch = b[i] as char;
                    let dot_join = ch == '.'
                        && b.get(i + 1)
                            .is_some_and(|n| (*n as char).is_ascii_alphabetic() || *n == b'_');
                    if ch.is_ascii_alphanumeric() || ch == '_' || dot_join {
                        i += 1;
                    } else {
                        break;
                    }
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

/// The most octaves `wiggle` unrolls — each is a `Noise` node in the IR, so an
/// unbounded literal would blow up the tree. AE's practical range is well under this.
const WIGGLE_MAX_OCTAVES: u32 = 8; // LITERAL-PX-OK: wiggle octave cap (tree size)

/// `wiggle(freq, amp [, octaves [, amp_mult]])` — smooth pseudo-random motion, the
/// AE staple. NOT an IR `Func` (that would change the frozen `ph2d-expr` contract):
/// it LOWERS at parse time to a SUM of `Noise` octaves over `time`, offset by a
/// per-consumer `__seed` the `Bindings` supplies. One octave is
///
/// `amp * 2 * (noise((time + __seed) * freq) - 0.5)`  in `[-amp, amp]`,
///
/// and octave `o` (0-based) adds the same at frequency `freq·2^o`, amplitude
/// `amp·amp_mult^o`, phase-shifted by `o` so the octaves decorrelate. Total is
/// bounded by `amp·(1 + amp_mult + … )`. `octaves`/`amp_mult` must be numeric
/// literals (they size the unrolled tree); defaults are `1` and `0.5` (AE's), and
/// with `octaves = 1` this is byte-identical to the single-octave lowering.
///
/// `noise` is bit-deterministic and seeded, so a wiggle is reproducible for a given
/// seed and two wiggles with different seeds diverge (ADR-0144).
fn wiggle(freq: Expr, amp: Expr, octaves: u32, amp_mult: f32) -> Expr {
    let base = Expr::bin(BinOp::Add, Expr::attr("time"), Expr::attr("__seed"));
    let mut sum: Option<Expr> = None;
    let (mut freq_scale, mut amp_scale) = (1.0f32, 1.0f32);
    for o in 0..octaves {
        // phase = (time + seed) * (freq · 2^o), shifted by `o` for o > 0 so the
        // octaves do not sample the same noise (o == 0 stays byte-identical).
        let f = Expr::bin(BinOp::Mul, freq.clone(), Expr::cnst(freq_scale));
        let mut phase = Expr::bin(BinOp::Mul, base.clone(), f);
        if o > 0 {
            phase = Expr::bin(BinOp::Add, phase, Expr::cnst(o as f32));
        }
        let centred = Expr::bin(
            BinOp::Sub,
            Expr::call(Func::Noise, vec![phase]),
            Expr::cnst(0.5),
        );
        let term = Expr::bin(
            BinOp::Mul,
            Expr::bin(BinOp::Mul, amp.clone(), Expr::cnst(amp_scale * 2.0)),
            centred,
        );
        sum = Some(match sum {
            Some(s) => Expr::bin(BinOp::Add, s, term),
            None => term,
        });
        freq_scale *= 2.0;
        amp_scale *= amp_mult;
    }
    sum.unwrap_or_else(|| Expr::cnst(0.0))
}

/// Build a function call node from a name + args (or `select`/`wiggle` sugar).
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
        "wiggle" => {
            if args.len() < 2 || args.len() > 4 {
                return Err(format!("`wiggle` wants 2..4 args, got {}", args.len()));
            }
            let mut it = args.into_iter();
            let freq = it.next().unwrap();
            let amp = it.next().unwrap();
            // octaves/amp_mult size the unrolled tree, so they must be literals.
            let octaves = match it.next() {
                None => 1,
                Some(Expr::Const(n)) if n >= 1.0 => (n as u32).min(WIGGLE_MAX_OCTAVES),
                Some(_) => return Err("`wiggle` octaves must be a number >= 1".into()),
            };
            let amp_mult = match it.next() {
                None => 0.5,
                Some(Expr::Const(m)) => m,
                Some(_) => return Err("`wiggle` amp_mult must be a number".into()),
            };
            return Ok(wiggle(freq, amp, octaves, amp_mult));
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

/// Parse `src` into an [`Expr`], or an error message.
pub fn parse(src: &str) -> Result<Expr, String> {
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
        assert!(matches!(
            parse("-(1 + 2)").unwrap(),
            Expr::Unary(UnaryOp::Neg, _)
        ));
    }

    /// A dotted identifier (`Sprite.x`) is ONE `Attr` — the prop-link syntax
    /// (ADR-0144). But `2.5` and `.5` stay numbers, and `x.5` stops at `x`.
    #[test]
    fn dotted_identifier_is_one_attr_but_numbers_survive() {
        assert!(matches!(parse("Sprite.x").unwrap(), Expr::Attr(n) if n == "Sprite.x"));
        assert!(matches!(parse("2.5").unwrap(), Expr::Const(v) if v == 2.5));
        assert!(matches!(parse(".5").unwrap(), Expr::Const(v) if v == 0.5));
        // `x.5`: ident `x`, then number `.5` -> two adjacent atoms -> trailing.
        assert!(parse("x.5").is_err(), "a digit does not join an identifier");
    }

    /// `wiggle(f, a)` lowers to `Noise` over `time`+`__seed` (no new IR Func),
    /// and wants exactly two args.
    #[test]
    fn wiggle_lowers_to_noise_over_time_and_seed() {
        let e = parse("wiggle(3, 20)").unwrap();
        // The tree must CONTAIN a Noise call and reference both `time` and `__seed`.
        let s = format!("{e:?}");
        assert!(s.contains("Noise"), "wiggle uses Noise: {s}");
        assert!(s.contains("\"time\""), "wiggle reads time: {s}");
        assert!(s.contains("\"__seed\""), "wiggle reads the seed: {s}");
        assert!(parse("wiggle(3)").is_err(), "wiggle wants at least 2 args");
    }

    /// `wiggle(f, a, octaves [, amp_mult])` unrolls to ONE `Noise` per octave
    /// (AE parity). `octaves`/`amp_mult` must be numeric literals; the octave count
    /// is capped so the tree cannot explode.
    #[test]
    fn wiggle_octaves_unroll_to_one_noise_each() {
        let count_noise = |src: &str| {
            format!("{:?}", parse(src).unwrap())
                .matches("Noise")
                .count()
        };
        assert_eq!(count_noise("wiggle(2, 20)"), 1, "default is one octave");
        assert_eq!(
            count_noise("wiggle(2, 20, 3)"),
            3,
            "three octaves -> three Noise"
        );
        assert_eq!(
            count_noise("wiggle(2, 20, 4, 0.6)"),
            4,
            "amp_mult is the 4th arg"
        );
        // Capped, not unbounded.
        assert_eq!(
            count_noise("wiggle(2, 20, 99)"),
            WIGGLE_MAX_OCTAVES as usize,
            "octaves are capped at WIGGLE_MAX_OCTAVES"
        );
        // Non-literal octaves / too many args error.
        assert!(
            parse("wiggle(2, 20, t)").is_err(),
            "octaves must be a literal"
        );
        assert!(parse("wiggle(2, 20, 2, 0.5, 1)").is_err(), "at most 4 args");
    }
}

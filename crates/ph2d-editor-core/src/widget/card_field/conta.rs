//! As CONTAS do campo de cartão (`eval_number`) — saíram de `card_field.rs` pelo tecto de 500 dos widgets.

/// ⭐ **O texto do campo aceita CONTAS** (`0.35*2`, `120+16`, `(1+2)/3`) e tolera um sufixo de
/// unidade (`12px`, `45deg`, `50%`). `None` = não é um número.
///
/// ⚠️ Um parser aritmético de 40 linhas, e não o `ph2d-expr-parse` (VEX-lite): aquele arrasta o
/// `ph2d-nodegraph` para dentro do editor-core, a mesma aresta que a `ph2d-tokens` recusou
/// (`ph2d-tokens/src/num_expr.rs`). Aqui só há números, `+ - * /` e parênteses.
#[must_use]
pub fn eval_number(text: &str) -> Option<f64> {
    let t = text.trim().replace(',', ".");
    let t = t.trim_end_matches(|c: char| c.is_alphabetic() || c == '°' || c == '%' || c == ' ');
    let mut p = Expr {
        s: t.as_bytes(),
        i: 0,
    };
    let v = p.sum()?;
    p.ws();
    (p.i == p.s.len() && v.is_finite()).then_some(v)
}

struct Expr<'a> {
    s: &'a [u8],
    i: usize,
}

impl Expr<'_> {
    fn ws(&mut self) {
        while self.s.get(self.i).is_some_and(u8::is_ascii_whitespace) {
            self.i += 1;
        }
    }
    fn eat(&mut self, c: u8) -> bool {
        self.ws();
        let ok = self.s.get(self.i) == Some(&c);
        if ok {
            self.i += 1;
        }
        ok
    }
    fn sum(&mut self) -> Option<f64> {
        let mut v = self.product()?;
        loop {
            if self.eat(b'+') {
                v += self.product()?;
            } else if self.eat(b'-') {
                v -= self.product()?;
            } else {
                return Some(v);
            }
        }
    }
    fn product(&mut self) -> Option<f64> {
        let mut v = self.unary()?;
        loop {
            if self.eat(b'*') {
                v *= self.unary()?;
            } else if self.eat(b'/') {
                v /= self.unary()?;
            } else {
                return Some(v);
            }
        }
    }
    fn unary(&mut self) -> Option<f64> {
        if self.eat(b'-') {
            return self.unary().map(|v| -v);
        }
        if self.eat(b'+') {
            return self.unary();
        }
        if self.eat(b'(') {
            let v = self.sum()?;
            return self.eat(b')').then_some(v);
        }
        self.ws();
        let start = self.i;
        while self
            .s
            .get(self.i)
            .is_some_and(|c| c.is_ascii_digit() || *c == b'.')
        {
            self.i += 1;
        }
        std::str::from_utf8(&self.s[start..self.i])
            .ok()?
            .parse()
            .ok()
    }
}

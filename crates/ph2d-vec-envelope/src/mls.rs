//! **Pinos** (ADR-0129 §4, Fatia E): *Moving Least Squares* rígido (Schaefer, Sederberg & Warren,
//! SIGGRAPH 2006) — o *puppet warp*, como um mapa [`Warp`].
//!
//! # O puppet NÃO precisa de malha, e é aqui que a suposição comum quebra
//!
//! *Puppet = ARAP = triangulação* é falso: o MLS-rigid é `R2→R2` puro, e na forma complexa cabe num
//! multiply-accumulate — sem solver, sem fatoração, sem re-malhar a cada edição. Para cada ponto `v`:
//!
//! ```text
//! wᵢ = 1/|pᵢ − v|²      p⋆ = Σwᵢpᵢ/Σwᵢ      q⋆ = Σwᵢqᵢ/Σwᵢ
//! S  = Σ wᵢ·q̂ᵢ·conj(p̂ᵢ)                      f(v) = (S/|S|)·(v − p⋆) + q⋆
//! ```
//!
//! `S/|S|` é um complexo unitário: **rotação pura**, que é o que "rígido" quer dizer — a similaridade
//! ajustada com a escala isotrópica eliminada.
//!
//! # ⚠️ O NOME MENTE: "rigid MLS" não é localmente rígido
//!
//! A restrição rígida vale para a transformação **ajustada** `l_v(x)`, **não** para a diferencial do
//! mapa `f(v) = l_v(v)` — porque `p⋆` e `q⋆` são **funções de `v`**. Medido no ADR: a 45° de torção
//! de pino os valores singulares vão a `[0,38 · 1,44]`, e a 90° `det J` **muda de sinal**. Quem ler
//! "rigid" e esperar isometria local vai se enganar; o que É exato é a reprodução de um movimento
//! rígido GLOBAL.
//!
//! # ⚠️ Armadilha de dia-um: com 2 pinos não se dobra nada
//!
//! Qualquer isometria de um PAR de pinos devolve movimento rígido do plano inteiro (`det J = 1`
//! exato) — segue da precisão rígida mais o facto de duas correspondências isométricas determinarem
//! uma rigidez única. **Deformar exige o 3º pino**, e ele não pode ser colinear com os outros dois.
//! Não é bug: é o que o método é. Há gate.
//!
//! # A jacobiana é FECHADA, e o que a torna tratável é um cancelamento
//!
//! O [`Warp`] exige a derivada real (uma diferença finita faz o fitter **não convergir**), e derivar
//! isto à mão parece proibitivo — até se notar que **`Σwᵢp̂ᵢ = 0` e `Σwᵢq̂ᵢ = 0` por definição do
//! centróide ponderado**. Os dois termos de correção de `∂S` (os que vêm de `p⋆` e `q⋆` dependerem de
//! `v`) são exatamente essas somas, e **desaparecem**:
//!
//! ```text
//! ∂S/∂v = Σ (∂wᵢ/∂v)·q̂ᵢ·conj(p̂ᵢ)          com  ∂wᵢ/∂v = wᵢ/(pᵢ − v)
//! ```
//!
//! O resto é regra do quociente em [cálculo de Wirtinger] — `∂/∂v` e `∂/∂v̄` por estágio, e a
//! jacobiana real sai de `A = ∂f/∂v` e `B = ∂f/∂v̄` por [`real_jacobian`].
//!
//! [cálculo de Wirtinger]: https://en.wikipedia.org/wiki/Wirtinger_derivatives

use crate::Warp;

/// Distância abaixo da qual `v` **é** o pino: o peso seria infinito, e o mapa ali é o pino movido.
///
/// É o guard `if (v == p[i]) return q[i]` que o ADR-0129 mandou copiar do Krita. Sem ele, pôr um
/// pino em cima de uma âncora produz `NaN` no primeiro frame.
const PIN_EPS: f64 = 1e-9;

/// `|S|` abaixo disto: a rotação é indeterminada, e o mapa é **translação pura**.
///
/// **Este guard É o caso de 1 pino**, e não um caso à parte: com um pino só, `p̂ = q̂ = 0` ⇒ `S = 0`,
/// e o limite correto é transladar (não há informação de rotação nenhuma). Cai daqui o critério de
/// aceitação #5 do ADR — *1 pino = translação pura, não NaN*.
const S_EPS: f64 = 1e-12;

/// Resolução do amostrador de dobra, por eixo — irmão do `FOLD_GRID` do Coons, e com a mesma
/// ressalva: é **amostragem**, não teorema.
const FOLD_GRID: usize = 16;

/// Um complexo. Existe para a derivação caber na página: em coordenadas reais, `S = Σwᵢq̂ᵢconj(p̂ᵢ)`
/// vira quatro somas cruzadas e a regra do quociente vira ilegível.
#[derive(Clone, Copy, Debug, PartialEq)]
struct C {
    re: f64,
    im: f64,
}

impl C {
    const ZERO: Self = Self { re: 0.0, im: 0.0 };

    const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    const fn from_pt(p: [f64; 2]) -> Self {
        Self { re: p[0], im: p[1] }
    }

    const fn to_pt(self) -> [f64; 2] {
        [self.re, self.im]
    }

    const fn add(self, o: Self) -> Self {
        Self::new(self.re + o.re, self.im + o.im)
    }

    const fn sub(self, o: Self) -> Self {
        Self::new(self.re - o.re, self.im - o.im)
    }

    const fn mul(self, o: Self) -> Self {
        Self::new(
            self.re * o.re - self.im * o.im,
            self.re * o.im + self.im * o.re,
        )
    }

    const fn scale(self, k: f64) -> Self {
        Self::new(self.re * k, self.im * k)
    }

    const fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }

    fn norm2(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    fn abs(self) -> f64 {
        self.norm2().sqrt()
    }
}

/// A jacobiana REAL a partir das duas derivadas de Wirtinger de `f`.
///
/// Com `df = A·dv + B·dv̄` e `v = x + iy`, sai `∂f/∂x = A + B` e `∂f/∂y = i(A − B)`; separando parte
/// real e imaginária dá a matriz abaixo, na ordem de linha que o [`Warp`] pede.
const fn real_jacobian(a: C, b: C) -> [[f64; 2]; 2] {
    let s = a.add(b);
    let d = a.sub(b);
    [[s.re, -d.im], [s.im, d.re]]
}

/// Um pino: onde ele **estava** (no espaço-fonte) e para onde o artista o **levou**.
pub type Pin = [[f64; 2]; 2];

/// O gesto **Pinos** como [`Warp`]: MLS-rigid sobre uma lista de correspondências.
///
/// ⚠️ **O suporte é GLOBAL** — o deslocamento cresce linearmente com a distância, e nenhum parâmetro
/// conserta isso (o α do paper não tem efeito no campo distante; ver a recusa do "Flexibility" do
/// Krita no ADR-0129 §6). A mitigação é estrutural: **o container é o escopo**, então os pinos
/// deformam os filhos do envelope e mais nada.
#[derive(Clone, Debug)]
pub struct MlsWarp {
    p: Vec<C>,
    q: Vec<C>,
}

impl MlsWarp {
    /// `None` sem pino nenhum (não há mapa a definir; o chamador mantém a arte como está).
    #[must_use]
    pub fn new(pins: &[Pin]) -> Option<Self> {
        if pins.is_empty() {
            return None;
        }
        Some(Self {
            p: pins.iter().map(|pin| C::from_pt(pin[0])).collect(),
            q: pins.iter().map(|pin| C::from_pt(pin[1])).collect(),
        })
    }

    /// Os estágios comuns a `map` e `jacobian`, calculados uma vez.
    ///
    /// `None` quando `v` **é** um pino: ali o mapa é o pino movido, por definição, e os pesos
    /// estourariam. Devolve o índice para o chamador responder o que couber.
    fn solve(&self, v: C) -> Result<Solved, usize> {
        let n = self.p.len();
        let (mut w_sum, mut pw, mut qw) = (0.0_f64, C::ZERO, C::ZERO);
        let mut w = Vec::with_capacity(n);
        for i in 0..n {
            let d = self.p[i].sub(v);
            if d.abs() < PIN_EPS {
                return Err(i);
            }
            let wi = 1.0 / d.norm2();
            w.push(wi);
            w_sum += wi;
            pw = pw.add(self.p[i].scale(wi));
            qw = qw.add(self.q[i].scale(wi));
        }
        let p_star = pw.scale(1.0 / w_sum);
        let q_star = qw.scale(1.0 / w_sum);
        let mut s = C::ZERO;
        for ((p, q), &wi) in self.p.iter().zip(&self.q).zip(&w) {
            s = s.add(q.sub(q_star).mul(p.sub(p_star).conj()).scale(wi));
        }
        Ok(Solved {
            w,
            w_sum,
            pw,
            qw,
            p_star,
            q_star,
            s,
        })
    }
}

/// Os estágios do MLS num ponto — partilhados entre o valor e a derivada, para as duas não poderem
/// discordar sobre os pesos.
struct Solved {
    w: Vec<f64>,
    w_sum: f64,
    pw: C,
    qw: C,
    p_star: C,
    q_star: C,
    s: C,
}

impl Warp for MlsWarp {
    fn map(&self, v: [f64; 2]) -> [f64; 2] {
        let v = C::from_pt(v);
        let sol = match self.solve(v) {
            Ok(s) => s,
            // `v` é um pino: o mapa ali é o destino dele. Exato, não aproximado.
            Err(i) => return self.q[i].to_pt(),
        };
        let u = v.sub(sol.p_star);
        if sol.s.abs() < S_EPS {
            // Sem informação de rotação (1 pino, ou pinos degenerados): translação pura.
            return u.add(sol.q_star).to_pt();
        }
        sol.s
            .scale(1.0 / sol.s.abs())
            .mul(u)
            .add(sol.q_star)
            .to_pt()
    }

    fn jacobian(&self, v: [f64; 2]) -> [[f64; 2]; 2] {
        let v = C::from_pt(v);
        let Ok(sol) = self.solve(v) else {
            // Em cima de um pino o mapa é constante-por-definição; a derivada ali não é usada pelo
            // fitter (o ponto é isolado) e a identidade é a resposta que não desestabiliza nada.
            return [[1.0, 0.0], [0.0, 1.0]];
        };
        let n = self.p.len();

        // ── ∂w/∂v = w/(pᵢ − v); como wᵢ é REAL, ∂wᵢ/∂v̄ = conj(∂wᵢ/∂v).
        let mut dw = Vec::with_capacity(n);
        let (mut dw_sum, mut dpw, mut dqw) = (C::ZERO, C::ZERO, C::ZERO);
        for ((p, q), &wi) in self.p.iter().zip(&self.q).zip(&sol.w) {
            let d = p.sub(v);
            let g = d.conj().scale(wi / d.norm2()); // wᵢ/dᵢ
            dw.push(g);
            dw_sum = dw_sum.add(g);
            dpw = dpw.add(p.mul(g));
            dqw = dqw.add(q.mul(g));
        }
        let (dw_sum_b, dpw_b, dqw_b) = {
            let mut a = C::ZERO;
            let (mut b, mut c) = (C::ZERO, C::ZERO);
            for ((p, q), g0) in self.p.iter().zip(&self.q).zip(&dw) {
                let g = g0.conj();
                a = a.add(g);
                b = b.add(p.mul(g));
                c = c.add(q.mul(g));
            }
            (a, b, c)
        };

        // ── Regra do quociente para p⋆ = pw/W e q⋆ = qw/W.
        let inv_w2 = 1.0 / (sol.w_sum * sol.w_sum);
        let quot =
            |num_d: C, num: C, den_d: C| num_d.scale(sol.w_sum).sub(num.mul(den_d)).scale(inv_w2);
        let dp_star = quot(dpw, sol.pw, dw_sum);
        let dp_star_b = quot(dpw_b, sol.pw, dw_sum_b);
        let dq_star = quot(dqw, sol.qw, dw_sum);
        let dq_star_b = quot(dqw_b, sol.qw, dw_sum_b);

        // ── ∂S: os termos de correção somam Σwᵢp̂ᵢ e Σwᵢq̂ᵢ, que são ZERO por definição do
        //    centróide ponderado. É esse cancelamento que torna a derivada escrevível à mão.
        let (mut ds, mut ds_b) = (C::ZERO, C::ZERO);
        for ((p, q), g) in self.p.iter().zip(&self.q).zip(&dw) {
            let term = q.sub(sol.q_star).mul(p.sub(sol.p_star).conj());
            ds = ds.add(term.mul(*g));
            ds_b = ds_b.add(term.mul(g.conj()));
        }

        let u = v.sub(sol.p_star);
        let m = sol.s.abs();
        if m < S_EPS {
            // Translação pura: f = (v − p⋆) + q⋆.
            let a = C::new(1.0, 0.0).sub(dp_star).add(dq_star);
            let b = C::ZERO.sub(dp_star_b).add(dq_star_b);
            return real_jacobian(a, b);
        }

        // ── R = S/|S|. Com m² = S·conj(S): 2m·∂m = ∂S·conj(S) + S·conj(∂S/∂v̄).
        let dm = ds
            .mul(sol.s.conj())
            .add(sol.s.mul(ds_b.conj()))
            .scale(1.0 / (2.0 * m));
        let dm_b = dm.conj(); // m é real
        let inv_m2 = 1.0 / (m * m);
        let dr = ds.scale(m).sub(sol.s.mul(dm)).scale(inv_m2);
        let dr_b = ds_b.scale(m).sub(sol.s.mul(dm_b)).scale(inv_m2);
        let r = sol.s.scale(1.0 / m);

        // ── f = R·(v − p⋆) + q⋆.
        let a = dr
            .mul(u)
            .add(r.mul(C::new(1.0, 0.0).sub(dp_star)))
            .add(dq_star);
        let b = dr_b
            .mul(u)
            .add(r.mul(C::ZERO.sub(dp_star_b)))
            .add(dq_star_b);
        real_jacobian(a, b)
    }
}

/// **Esta configuração de pinos dobra a arte dentro de `domain`?**
///
/// `domain` é `(origem, tamanho)` — o retângulo-fonte do envelope, que é exatamente o pedaço de
/// plano onde há arte. Amostra o sinal de `det J` numa grade; `true` se ele zera ou vira.
///
/// É o guard do gesto, e o irmão do `cage_folds`. **Ele restitui a premissa do `break_cusp`:** o
/// `fit.rs` avisava que a Fatia E quebraria o *"nenhum mapa em escopo dobra"*, e a resposta desta
/// linha à degenerescência é sempre a mesma — **torná-la inalcançável pela mão**, em vez de
/// aproximá-la bem. Dobra em vetor não é um bico a fitar: é um contorno auto-interseccionado, que é
/// a saga da lasca da booleana de novo.
///
/// ⚠️ **O preço está registado:** o artista não consegue torcer um pino além de ~90°. É o limite do
/// método (medido no ADR), não do guard — e se o smoke mostrar que o gesto quer posar personagem, a
/// decisão de usar MLS é **reaberta**, não calibrada (ADR-0129 §4, o contra-sinal do ARAP).
#[must_use]
pub fn pins_fold(pins: &[Pin], origin: [f64; 2], size: [f64; 2]) -> bool {
    let Some(w) = MlsWarp::new(pins) else {
        return false;
    };
    for i in 0..=FOLD_GRID {
        for j in 0..=FOLD_GRID {
            let p = [
                origin[0] + size[0] * (i as f64 / FOLD_GRID as f64),
                origin[1] + size[1] * (j as f64 / FOLD_GRID as f64),
            ];
            let j2 = w.jacobian(p);
            if j2[0][0] * j2[1][1] - j2[0][1] * j2[1][0] <= 0.0 {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
#[path = "mls_tests.rs"]
mod tests;

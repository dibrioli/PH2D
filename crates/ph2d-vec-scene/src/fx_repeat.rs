//! **Repeater** — o terceiro efeito da pilha (ADR-0132), e o primeiro que MULTIPLICA contornos.
//!
//! N cópias da forma, cada uma com a transformação aplicada mais uma vez. É o *Repeater* do
//! After Effects e o *Transform* do Illustrator, e é o staple de motion graphics: fileiras,
//! catavento, escadas, espirais.
//!
//! # O que o distingue dos dois primeiros efeitos
//!
//! O Trim e o Zig Zag são **por-contorno** — cada contorno entra, um contorno sai, e o
//! `apply_per_contour` trata os buracos de um compound independentemente. O Repeater não cabe
//! nessa porta: ele toma a forma INTEIRA (contorno primário **mais** os buracos) e emite `n`
//! cópias dela. Um buraco copiado sozinho deixaria de ser buraco.
//!
//! Então este é o primeiro efeito a construir a saída diretamente, e é por isso que
//! `PathEffect::apply` ramifica por variante em vez de encaminhar tudo para uma função só.
//!
//! # A transformação é CUMULATIVA, e é isso que dá a espiral
//!
//! A cópia `k` leva `M^k`, não `k·M`: a matriz é composta consigo mesma. Com só translação as
//! duas coincidem (uma fileira); com rotação **e** translação a composição espirala, enquanto o
//! múltiplo desenharia um arco de raio fixo. A espiral é o comportamento do AE, e é o que o
//! artista espera quando mexe nos dois botões ao mesmo tempo.
//!
//! # As distâncias são PERCENTAGEM da forma
//!
//! `Move X = 100` desloca cada cópia por uma largura-média da forma. É a mesma lei do `Size` do
//! Zig Zag, e pela mesma razão: as formas da cena têm poucas unidades, então um slider em
//! unidades de mundo é inútil (Enio, 2026-07-18).

use crate::effect::FxCtx;
use crate::{Contour, VecPath, VecVertex};

/// Abaixo disto uma distância é zero.
const EPS: f64 = 1e-12;

/// Menos de duas cópias não é repetição — é a própria forma.
const MIN_COPIES: f64 = 1.0;

/// Meia-volta em graus, para converter sem constante mágica.
const HALF_TURN_DEG: f64 = 180.0;

/// **Os parâmetros do Repeater.** Neutro em `copies <= 1`.
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RepeatSpec {
    /// Quantas cópias ao todo, contando o original. `1` = neutro.
    pub copies: f64,
    /// Deslocamento por cópia no eixo x, em **percentagem da forma** ([`FxCtx::ref_size`]).
    pub move_x: f64,
    /// Idem em y.
    pub move_y: f64,
    /// Rotação por cópia, em **graus**, em torno do centro da forma. Cumulativa.
    pub rotate: f64,
}

impl Default for RepeatSpec {
    fn default() -> Self {
        Self {
            copies: 1.0,
            move_x: 0.0,
            move_y: 0.0,
            rotate: 0.0,
        }
    }
}

impl RepeatSpec {
    /// Uma cópia é a forma. O neutro tem de ser no-op byte-idêntico, senão a pilha não pode
    /// saltá-lo e o `Cow::Borrowed` morre (ADR-0132).
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.copies < MIN_COPIES + 1.0
    }
}

/// Um afim 2D em linha-maior: `[a, b, tx; c, d, ty]`.
#[derive(Copy, Clone, Debug)]
struct Affine {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    tx: f64,
    ty: f64,
}

impl Affine {
    const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    /// `self ∘ rhs` — aplica `rhs` primeiro.
    fn then(self, rhs: Self) -> Self {
        Self {
            a: rhs.a.mul_add(self.a, rhs.c * self.b),
            b: rhs.b.mul_add(self.a, rhs.d * self.b),
            c: rhs.a.mul_add(self.c, rhs.c * self.d),
            d: rhs.b.mul_add(self.c, rhs.d * self.d),
            tx: rhs.tx.mul_add(self.a, rhs.ty.mul_add(self.b, self.tx)),
            ty: rhs.tx.mul_add(self.c, rhs.ty.mul_add(self.d, self.ty)),
        }
    }

    fn apply(self, p: [f64; 2]) -> [f64; 2] {
        [
            p[0].mul_add(self.a, p[1].mul_add(self.b, self.tx)),
            p[0].mul_add(self.c, p[1].mul_add(self.d, self.ty)),
        ]
    }
}

/// O passo de uma cópia: rodar em torno de `center`, depois transladar.
fn step_of(spec: &RepeatSpec, ctx: &FxCtx) -> Affine {
    // `sin`/`cos` UMA vez por efeito, não por ponto — o custo do Repeater é o número de pontos
    // vezes o número de cópias, e não há razão para pagar transcendental em cada um.
    let rad = spec.rotate / HALF_TURN_DEG * core::f64::consts::PI;
    let (s, c) = rad.sin_cos();
    let k = ctx.ref_size / 100.0;
    let (dx, dy) = (spec.move_x * k, spec.move_y * k);
    let (ox, oy) = (ctx.center[0], ctx.center[1]);
    // T(centro) ∘ R(θ) ∘ T(−centro), com a translação somada no fim.
    Affine {
        a: c,
        b: -s,
        c: s,
        d: c,
        tx: dx + ox - (c * ox - s * oy),
        ty: dy + oy - s.mul_add(ox, c * oy),
    }
}

fn map_vert(v: &VecVertex, m: Affine) -> VecVertex {
    VecVertex {
        anchor: m.apply(v.anchor),
        in_handle: m.apply(v.in_handle),
        out_handle: m.apply(v.out_handle),
        kind: v.kind,
        // O afim é rotação + translação (sem escala), então um comprimento LOCAL sobrevive
        // intacto. Um parâmetro de escala teria de multiplicá-lo aqui — a mesma conversão que
        // o raio do gradiente radial já faz.
        corner_radius: v.corner_radius,
    }
}

/// **Aplica o Repeater à forma inteira.** O original fica onde está; as cópias entram como
/// contornos adicionais.
///
/// A `fill_rule` do caminho é preservada — duas cópias sobrepostas de uma forma com buraco
/// continuam a vazar pela regra que o artista escolheu.
#[must_use]
pub fn repeat_path(path: &VecPath, spec: &RepeatSpec, ctx: &FxCtx) -> VecPath {
    let mut out = path.clone();
    if spec.is_neutral() {
        return out;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let copies = spec.copies.floor().max(MIN_COPIES) as usize;
    let step = step_of(spec, ctx);
    // Um passo que não move NADA (sem distância e sem ângulo) empilharia `n` cópias exatamente
    // em cima da forma: invisível, e caro. É o neutro pela outra porta.
    if step.tx.abs() <= EPS
        && step.ty.abs() <= EPS
        && (step.a - 1.0).abs() <= EPS
        && step.b.abs() <= EPS
    {
        return out;
    }

    // Os contornos ORIGINAIS, capturados antes de a saída crescer — copiar de uma lista que se
    // está a estender daria cópias das cópias.
    let source: Vec<(Vec<VecVertex>, bool)> = (0..path.contour_count())
        .filter_map(|k| path.contour(k).map(|(v, cl)| (v.to_vec(), cl)))
        .collect();

    let mut m = Affine::IDENTITY;
    for _ in 1..copies {
        m = step.then(m);
        for (verts, closed) in &source {
            out.subpaths.push(Contour {
                verts: verts.iter().map(|v| map_vert(v, m)).collect(),
                closed: *closed,
            });
        }
    }
    out
}

#[cfg(test)]
#[path = "fx_repeat_tests.rs"]
mod tests;

//! **A DOBRA** — quanto o pano resiste a MUDAR a curvatura que ele já tem.
//!
//! # ⛔⛔ Por que NÃO é o modelo quadrático de dobra, que a pesquisa tinha indicado
//!
//! O [`01_pesquisa`](../../../docs/3D/cloth/01_pesquisa_o_estado_da_arte.md) §5
//! prescrevia o **modelo quadrático** (Bergou et al. 2006) com o argumento — real
//! — de que a Hessiana dele é **constante e semi-definida positiva**, logo
//! pré-computável uma vez por traço. A implementação refutou a premissa: *aquele
//! modelo assume o repouso PLANO* (é a condição de isometria que o torna válido),
//! e o repouso de um pincel de escultura é a **superfície esculpida**, que é curva
//! em todo lugar interessante.
//!
//! Usá-lo aqui daria força no repouso — a peça se mexeria sozinha ao encostar o
//! pincel, que é exatamente o que o gate *«o repouso é ponto fixo»* existe para
//! proibir.
//!
//! ⇒ **ângulo diedro com ângulo de REPOUSO** (Grinspun, Hirani, Desbrun &
//! Schröder — *Discrete Shells*, SCA 2003):
//!
//! ```text
//! E = k · (3‖ē‖² / (A₀+A₁)) · (θ − θ̄)²
//! ```
//!
//! Ele é invariante a rotação, é zero no repouso **por construção**, e vale numa
//! superfície de qualquer curvatura.
//!
//! # ⚠️ A Hessiana é de GAUSS-NEWTON, e a escolha é deliberada
//!
//! `∂²E/∂xᵢ² ≈ 2k·w·(∂θ/∂xᵢ)(∂θ/∂xᵢ)ᵀ` — o termo com `∂²θ` é descartado.
//!
//! ⭐ **O produto externo é semi-definido positivo por construção**, então o passo
//! local é garantidamente de descida. O VBD tolera Hessiana indefinida (e o paper
//! argumenta por que), mas **o gradiente tem de ser exato** — e é: só a métrica
//! que orienta o passo é aproximada. *Trocar a exatidão do passo pela garantia de
//! descida é o negócio certo num pincel, onde quem trunca as iterações é o relógio
//! do quadro.*

use crate::{V3, add, cross, dot, norm, scale, sub};

/// **UMA DOBRADIÇA** — uma aresta interior e os dois ápices que a ladeiam.
///
/// A ordem é a lei: `edge[0] → edge[1]` é o sentido em que a face da FRENTE
/// percorre a aresta, e é o que dá **sinal** ao ângulo. Sem sinal, dobrar para
/// um lado e para o outro leriam igual, e o pano não teria como voltar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hinge {
    /// A aresta partilhada, no sentido da face da frente.
    pub edge: [u32; 2],
    /// Os ápices: `[frente, verso]`.
    pub apex: [u32; 2],
}

impl Hinge {
    /// Os quatro vértices, na ordem dos *slots*: `[e₀, e₁, ápice frente, ápice verso]`.
    #[must_use]
    pub fn verts(&self) -> [u32; 4] {
        [self.edge[0], self.edge[1], self.apex[0], self.apex[1]]
    }
}

/// As duas normais (não normalizadas) e a aresta.
fn frame(x: &[V3], h: Hinge) -> (V3, V3, V3) {
    let v = h.verts();
    let (e0, e1) = (x[v[0] as usize], x[v[1] as usize]);
    let (a0, a1) = (x[v[2] as usize], x[v[3] as usize]);
    let e = sub(e1, e0);
    (cross(e, sub(a0, e0)), cross(scale(e, -1.0), sub(a1, e1)), e)
}

/// **O ÂNGULO DIEDRO, com sinal** — `atan2` sobre a aresta, e não `acos`.
///
/// ⚠️ O `acos` do produto das normais dá `|θ|` e perde o lado; e perto de `0` ele
/// perde precisão exatamente onde o pano passa a maior parte do tempo.
#[must_use]
pub(crate) fn dihedral(x: &[V3], h: Hinge) -> f64 {
    let (n0, n1, e) = frame(x, h);
    let (l0, l1, le) = (norm(n0), norm(n1), norm(e));
    if l0 < 1e-18 || l1 < 1e-18 || le < 1e-18 {
        return 0.0;
    }
    let (u0, u1) = (scale(n0, 1.0 / l0), scale(n1, 1.0 / l1));
    let s = dot(cross(u0, u1), scale(e, 1.0 / le));
    s.atan2(dot(u0, u1))
}

/// O peso do *Discrete Shells*: `‖ē‖/h̄ = 3‖ē‖²/(A₀+A₁)`.
///
/// ⚠️ **Ele é medido no REPOUSO e nunca mais** — é o que faz uma dobradiça longa
/// e magra pesar diferente de uma curta e gorda sem que a razão mude enquanto o
/// artista deforma.
#[must_use]
pub(crate) fn weight_of(x: &[V3], h: Hinge) -> f64 {
    let (n0, n1, e) = frame(x, h);
    let a = 0.5 * (norm(n0) + norm(n1));
    if a < 1e-18 {
        return 0.0;
    }
    3.0 * dot(e, e) / a
}

/// **AS QUATRO DERIVADAS do ângulo**, uma por *slot*.
///
/// Os dois ápices saem da forma fechada `∂θ/∂a = −‖e‖·n/‖n‖²`; os dois vértices
/// da aresta saem daí por **duas invariâncias**, e não por uma segunda derivação:
///
/// - transladar os quatro não muda `θ` ⇒ a soma das quatro é zero;
/// - **deslizar um vértice ao longo da aresta** não muda `θ` ⇒ o peso de cada
///   ápice sobre cada ponta é a projeção `t` dele na aresta.
///
/// ⭐ É por isso que a soma dar zero é um **gate**, e não um comentário: ela é a
/// própria construção, então um sinal trocado a quebra.
pub(crate) fn grads(x: &[V3], h: Hinge) -> [V3; 4] {
    let v = h.verts();
    let e0 = x[v[0] as usize];
    let (a0, a1) = (x[v[2] as usize], x[v[3] as usize]);
    let (n0, n1, e) = frame(x, h);
    let (q0, q1, le2) = (dot(n0, n0), dot(n1, n1), dot(e, e));
    if q0 < 1e-24 || q1 < 1e-24 || le2 < 1e-24 {
        return [[0.0; 3]; 4];
    }
    let le = le2.sqrt();
    let ga0 = scale(n0, -le / q0);
    let ga1 = scale(n1, -le / q1);
    let t0 = dot(sub(a0, e0), e) / le2;
    let t1 = dot(sub(a1, e0), e) / le2;
    let ge0 = add(scale(ga0, t0 - 1.0), scale(ga1, t1 - 1.0));
    let ge1 = add(scale(ga0, -t0), scale(ga1, -t1));
    [ge0, ge1, ga0, ga1]
}

/// **QUANTO ESTA DOBRADIÇA SAIU DO REPOUSO** — a porta ÚNICA, e ela existe por um
/// defeito que a implementação quase deixou passar.
///
/// ⚠️⚠️ **A diferença é DOBRADA para `(−π, π]`.** Sem isso, uma dobradiça que
/// cruza `±π` (o pano a fechar sobre si mesmo) lê um salto de `2π` e a força
/// explode — o modo de falha é uma agulha na malha.
///
/// ⛔ **E a 1.ª versão dobrava só no GRADIENTE**, deixando a energia a medir o
/// salto cru: as duas leis discordariam exatamente onde o pano dobra ao máximo, e
/// o gate de diferenças finitas — que compara as duas — só o veria numa fixtura
/// que cruzasse `π`. *Uma lei escrita em dois sítios ainda não é uma lei.*
pub(crate) fn delta(x: &[V3], h: Hinge, r: &HingeRest) -> f64 {
    let mut d = dihedral(x, h) - r.angle;
    while d > core::f64::consts::PI {
        d -= 2.0 * core::f64::consts::PI;
    }
    while d < -core::f64::consts::PI {
        d += 2.0 * core::f64::consts::PI;
    }
    d
}

/// O gradiente e a Hessiana de Gauss-Newton num *slot* da dobradiça.
#[derive(Clone, Debug)]
pub(crate) struct HingeRest {
    /// O ângulo diedro de repouso, em radianos.
    pub(crate) angle: f64,
    /// `3‖ē‖²/(A₀+A₁)`, o peso do *Discrete Shells*.
    pub(crate) weight: f64,
}

pub(crate) fn accumulate(
    x: &[V3],
    h: Hinge,
    r: &HingeRest,
    k: f64,
    slot: usize,
) -> (V3, [[f64; 3]; 3]) {
    if r.weight <= 0.0 || k <= 0.0 {
        return ([0.0; 3], [[0.0; 3]; 3]);
    }
    let g = grads(x, h)[slot];
    let kw = k * r.weight;
    let grad = scale(g, 2.0 * kw * delta(x, h, r));
    let mut hess = [[0.0f64; 3]; 3];
    for r in 0..3 {
        for c in 0..3 {
            hess[r][c] = 2.0 * kw * g[r] * g[c];
        }
    }
    (grad, hess)
}

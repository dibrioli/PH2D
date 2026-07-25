//! **Twist** — o remoinho: cada ponto gira em torno do CENTRO por um ângulo que cresce com a
//! distância. O centro fica parado, a borda gira o ângulo inteiro, e uma ponta que ultrapassa o
//! raio de referência gira MAIS — o *pinwheel*. É o *Distort & Transform > Twist* do Illustrator /
//! o *Twist* do After Effects.
//!
//! # ⚠️ Este efeito foi CORTADO uma vez (2026-07-18), e a razão importa
//!
//! A 1ª versão mapeava só os **pontos de controle** (âncora + 2 alças) — um campo não-afim
//! amostrado grosso, o *"como se torcesse um lowpoly"* que o Enio viu. Depois vieram quatro
//! tentativas com uma subdivisão adaptativa (força ∝ raio, força ∝ 1−raio, raio de referência pela
//! média, pelo máximo, subdivisão seis vezes mais fina) e **todas rasgavam** — a cerca ficou no
//! cabeçalho do [`crate::fx_warp`], com o veredito de que era um defeito do MODELO sem referência
//! verificável.
//!
//! **O que mudou:** aquela subdivisão era ANTERIOR ao esqueleto maduro que os presets de Warp
//! trouxeram ([`crate::fx_warp_presets::resample_displace`] — reamostragem densa por ARCO + união
//! com as âncoras + Catmull-Rom). Os warps paramétricos são campos não-afins tão fortes quanto um
//! remoinho e **não rasgam** sobre esse esqueleto; e a sonda de olhar (`tests/fx_look.rs`) passou a
//! preencher em **nonzero-winding**, então uma silhueta que cruza a si mesma (o que um twist forte
//! faz de propósito) aparece TINTA CHEIA, não rasgada — o artefato de even-odd que confundia o
//! diagnóstico morreu. A cerca pedia *render-and-look sobre uma forma com quinas, não um palpite*;
//! é o que a folha de contacto entrega, e é o que autoriza esta volta.

use crate::VecVertex;
use crate::effect::FxCtx;
use crate::fx_falloff::Falloff;
use crate::fx_warp_presets::resample_displace;

/// Abaixo deste ângulo (radianos) o efeito é o ponto neutro.
const EPS: f64 = 1e-12;

/// Meia-volta em graus — a conversão graus→radianos do ângulo autorado.
const HALF_TURN_DEG: f64 = 180.0;

/// **Os parâmetros de um Twist.** Neutro em `angle == 0`.
#[derive(Copy, Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TwistSpec {
    /// O ângulo, em **graus**, entregue na BORDA da forma (o raio de referência). O centro não
    /// gira; a borda gira isto; uma ponta que passa do raio gira mais. Positivo = anti-horário.
    pub angle: f64,
}

impl TwistSpec {
    /// Um Twist novo, no ponto NEUTRO.
    #[must_use]
    pub fn new() -> Self {
        Self { angle: 0.0 }
    }

    /// Sem ângulo não há giro — e o neutro tem de ser no-op byte-idêntico (ADR-0132), o que
    /// mantém o `Cow::Borrowed` do `cooked()` vivo.
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.angle.abs() <= EPS
    }
}

/// **Aplica o Twist a UM contorno.** Devolve `(verts, closed)` — girar não abre nem fecha.
///
/// O raio de referência é meia [`FxCtx::ref_size`] — a mesma medição do caminho AUTORADO que dá
/// sentido a todo parâmetro de distância da pilha, então o mesmo ângulo desenha o mesmo remoinho
/// numa forma pequena e numa grande, e o significado não depende da ORDEM na pilha.
///
/// `falloff` (opcional) escala a força por-amostra na posição ORIGINAL: `w = 0` deixa a amostra na
/// curva de entrada, `w = 1` é o giro cheio. `None` é byte-idêntico ao Twist sem campo.
///
/// A deformação corre pelo esqueleto [`resample_displace`] — reamostragem densa por arco —, e é ISSO
/// que o distingue da versão cortada, que mapeava só os pontos de controle (ver o cabeçalho).
#[must_use]
pub fn twist_contour(
    verts: &[VecVertex],
    closed: bool,
    spec: &TwistSpec,
    ctx: &FxCtx,
    falloff: Option<&Falloff>,
) -> (Vec<VecVertex>, bool) {
    let r = ctx.ref_size * 0.5;
    if spec.is_neutral() || verts.len() < 2 || r <= EPS {
        return (verts.to_vec(), closed);
    }
    let full = spec.angle / HALF_TURN_DEG * core::f64::consts::PI;
    let twist = |p: [f64; 2]| -> [f64; 2] {
        let (dx, dy) = (p[0] - ctx.center[0], p[1] - ctx.center[1]);
        // A força cresce com a distância: no centro é zero, na borda (r) é o ângulo inteiro, e uma
        // ponta que passa do raio enrola mais que o corpo — o pinwheel.
        let t = dx.hypot(dy) / r;
        let (s, c) = (full * t).sin_cos();
        [
            dx.mul_add(c, -(dy * s)) + ctx.center[0],
            dx.mul_add(s, dy * c) + ctx.center[1],
        ]
    };
    match resample_displace(verts, closed, falloff, twist) {
        Some(v) => (v, closed),
        None => (verts.to_vec(), closed),
    }
}

#[cfg(test)]
#[path = "fx_twist_tests.rs"]
mod tests;

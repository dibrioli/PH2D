//! **Twist** e **Pucker & Bloat** — os dois deformadores radiais do Illustrator, num motor só.
//!
//! Ambos movem cada ponto em função da posição dele **relativa ao centro da forma**, e diferem
//! só na direção: o Twist gira (tangencial, mais forte longe do centro) e o Pucker/Bloat puxa ou
//! empurra (radial). Um parâmetro cada, e o mesmo laço.
//!
//! # Porque estes dois e não um pacote maior
//!
//! Eles existem para **medir** a promessa do ADR-0132 — *"o próximo efeito custa zero painel"* —
//! e não para encher o menu. Se um efeito de duzentas linhas entra sem uma linha de painel, a
//! promessa é verdadeira e fica medida; se não entra, quero saber onde o desenho vaza enquanto é
//! barato descobri-lo.
//!
//! # Alças acompanham a âncora, e isso é uma escolha
//!
//! A deformação exata de uma cúbica sob um campo não-afim não é uma cúbica. Aqui cada ponto de
//! controle é mapeado pelo MESMO campo — é o que o Illustrator faz, é estável, e mantém a
//! continuidade nas junções (dois vértices que partilham uma alça mapeiam-na para o mesmo sítio,
//! porque o campo é função só da posição).

use crate::VecVertex;
use crate::effect::FxCtx;

/// Abaixo disto o efeito é o ponto neutro.
const EPS: f64 = 1e-12;

/// Meia-volta em graus.
const HALF_TURN_DEG: f64 = 180.0;

/// **Twist** — gira em torno do centro, com força proporcional à distância.
#[derive(Copy, Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TwistSpec {
    /// Ângulo em **graus** na borda da forma. O centro não roda; a borda roda isto.
    pub angle: f64,
}

impl TwistSpec {
    /// Sem ângulo não há giro.
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.angle.abs() <= EPS
    }
}

/// **Pucker & Bloat** — puxa os pontos para o centro (negativo) ou empurra para fora (positivo).
#[derive(Copy, Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BloatSpec {
    /// Quanto, em **percentagem** da distância ao centro. `100` duplica o raio de cada ponto;
    /// `-100` colapsa a forma no centro.
    pub amount: f64,
}

impl BloatSpec {
    /// Sem quantidade não há deformação.
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.amount.abs() <= EPS
    }
}

/// O raio da forma — metade da referência, que é a média das dimensões da caixa. É a distância
/// em que o Twist entrega o ângulo inteiro.
fn radius_of(ctx: &FxCtx) -> f64 {
    ctx.ref_size * 0.5
}

/// **Aplica o Twist a um contorno.** Devolve `(verts, closed)` — girar não abre nem fecha.
#[must_use]
pub fn twist_contour(
    verts: &[VecVertex],
    closed: bool,
    spec: &TwistSpec,
    ctx: &FxCtx,
) -> (Vec<VecVertex>, bool) {
    let r = radius_of(ctx);
    if spec.is_neutral() || r <= EPS {
        return (verts.to_vec(), closed);
    }
    let full = spec.angle / HALF_TURN_DEG * core::f64::consts::PI;
    let map = |p: [f64; 2]| -> [f64; 2] {
        let (dx, dy) = (p[0] - ctx.center[0], p[1] - ctx.center[1]);
        // A força cresce com a distância: no centro é zero, na borda é o ângulo inteiro. Fora da
        // borda continua a crescer — é o que faz uma ponta de estrela enrolar mais que o corpo.
        let t = dx.hypot(dy) / r;
        let (s, c) = (full * t).sin_cos();
        [
            dx.mul_add(c, -(dy * s)) + ctx.center[0],
            dx.mul_add(s, dy * c) + ctx.center[1],
        ]
    };
    (
        verts
            .iter()
            .map(|v| VecVertex {
                anchor: map(v.anchor),
                in_handle: map(v.in_handle),
                out_handle: map(v.out_handle),
                kind: v.kind,
                // O campo não é afim, então um comprimento local deixa de ter significado
                // exato. Zerar seria perder o raio autorado; mantê-lo é o erro menor, e o
                // estágio da quina já correu ANTES desta pilha (o `cooked()` cozinha na ordem).
                corner_radius: v.corner_radius,
            })
            .collect(),
        closed,
    )
}

/// **Aplica o Pucker & Bloat a um contorno.**
#[must_use]
pub fn bloat_contour(
    verts: &[VecVertex],
    closed: bool,
    spec: &BloatSpec,
    ctx: &FxCtx,
) -> (Vec<VecVertex>, bool) {
    if spec.is_neutral() {
        return (verts.to_vec(), closed);
    }
    let k = 1.0 + spec.amount / 100.0;
    let map = |p: [f64; 2]| -> [f64; 2] {
        [
            (p[0] - ctx.center[0]).mul_add(k, ctx.center[0]),
            (p[1] - ctx.center[1]).mul_add(k, ctx.center[1]),
        ]
    };
    (
        verts
            .iter()
            .map(|v| VecVertex {
                anchor: map(v.anchor),
                in_handle: map(v.in_handle),
                out_handle: map(v.out_handle),
                kind: v.kind,
                // Aqui o campo É uma escala uniforme, então um comprimento local escala com ela
                // — a mesma conversão que o raio do gradiente radial faz.
                corner_radius: v.corner_radius * k.abs(),
            })
            .collect(),
        closed,
    )
}

#[cfg(test)]
#[path = "fx_warp_tests.rs"]
mod tests;

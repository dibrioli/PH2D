//! **COMO uma forma é pintada** — cor, gradiente e o `Paint` que os une.
//!
//! Split de `lib.rs` pelo teto de LOC (700). O corte é por ASSUNTO, e a linha é a mesma que o
//! resto da crate já usa: `lib.rs` fica com *o que uma forma É* (`VecVertex`, `VecPath`,
//! `VecScene` e a árvore) e este módulo com *com que tinta ela aparece*.
//!
//! ⚠️ Os tipos são **re-exportados na raiz** — `ph2d_vec_scene::Paint` continua a resolver, e
//! nenhum dos consumidores do workspace muda de caminho.

use serde::{Deserialize, Serialize};

/// Cor de estilo (sRGB 8-bit). Fase 0: representação mínima; a cor canônica
/// OKLCH (via `ph2d-color`) é refinamento de Fase 1.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8 {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// Um stop de gradiente linear/radial: cor numa posição `offset ∈ [0,1]` ao longo
/// da rampa. Ordenados por `offset` crescente pelo editor (o render assume isso).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GradientStop {
    pub offset: f64,
    pub color: Rgba8,
}

impl GradientStop {
    #[must_use]
    pub fn new(offset: f64, color: Rgba8) -> Self {
        Self { offset, color }
    }
}

/// Um ponto de um gradiente **multi-ponto** (freeform, estilo Cavalry / Illustrator
/// Freeform Gradient): uma cor posicionada em `pos` no espaço NORMALIZADO da bbox
/// do path (`pos` em WORLD-space, mesmo espaço das âncoras, então transforma junto
/// com a shape) + uma `influence` (força/peso IDW) + `jitter` (0..1: ruído
/// determinístico por-texel na contribuição do ponto — grão estilo Cavalry). O
/// render mistura por inverse-distance weighting: `c(p) = Σ wᵢcᵢ / Σ wᵢ`,
/// `wᵢ = influenceᵢ / (dist² + ε)`, com `wᵢ` perturbado por `jitter`.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GradientPoint {
    pub pos: [f64; 2],
    pub color: Rgba8,
    pub influence: f64,
    /// Ruído per-texel na contribuição do ponto, `0..1` (0 = blend liso, default).
    pub jitter: f64,
}

impl GradientPoint {
    /// Ponto liso (`jitter = 0`): o caminho comum. Use [`Self::with_jitter`] para o grão.
    #[must_use]
    pub fn new(pos: [f64; 2], color: Rgba8, influence: f64) -> Self {
        Self {
            pos,
            color,
            influence,
            jitter: 0.0,
        }
    }

    /// Igual a [`Self::new`] mas com `jitter` explícito (0..1).
    #[must_use]
    pub fn with_jitter(pos: [f64; 2], color: Rgba8, influence: f64, jitter: f64) -> Self {
        Self {
            pos,
            color,
            influence,
            jitter,
        }
    }
}

/// Preenchimento de um path: cor sólida ou um dos três gradientes. A geometria do
/// gradiente é armazenada em **WORLD-space** (mesmo espaço das âncoras) e
/// **transforma junto com o path** (translate/scale/rotate/flip movem os pontos do
/// gradiente igual às âncoras) — então rotacionar a shape roda o gradiente
/// rigidamente, sem "respirar" (o bug do gradiente bbox-relativo). Linear/Radial
/// rasterizam nativo no Vello; MultiPoint (freeform) por IDW num image-brush.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Paint {
    /// Cor chapada.
    Solid(Rgba8),
    /// Rampa linear do ponto `start` ao `end` (world-space).
    Linear {
        stops: Vec<GradientStop>,
        start: [f64; 2],
        end: [f64; 2],
    },
    /// Rampa radial do `center` (world-space) até `radius` (world-units).
    Radial {
        stops: Vec<GradientStop>,
        center: [f64; 2],
        radius: f64,
    },
    /// Multi-ponto freeform (Cavalry): blend IDW de pontos em world-space.
    MultiPoint { points: Vec<GradientPoint> },
}

impl Paint {
    /// Cor sólida (o caminho comum; `Rgba8` também converte via [`From`]).
    #[must_use]
    pub fn solid(color: Rgba8) -> Self {
        Paint::Solid(color)
    }

    /// Cor representativa (sólida / 1º stop / 1º ponto) — pra swatch de UI e para
    /// caminhos legados que esperam uma cor única. Preto opaco se um gradiente
    /// estiver (invalidamente) vazio.
    #[must_use]
    pub fn primary_color(&self) -> Rgba8 {
        match self {
            Paint::Solid(c) => *c,
            Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => {
                stops.first().map_or(Rgba8::new(0, 0, 0, 255), |s| s.color)
            }
            Paint::MultiPoint { points } => {
                points.first().map_or(Rgba8::new(0, 0, 0, 255), |p| p.color)
            }
        }
    }
}

impl From<Rgba8> for Paint {
    fn from(c: Rgba8) -> Self {
        Paint::Solid(c)
    }
}

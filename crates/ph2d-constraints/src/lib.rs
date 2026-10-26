#![forbid(unsafe_code)]
//! **A lei das constraints 2D** — o que cada tipo faz a uma pose, sem saber o que é uma entidade.
//!
//! Plano e oráculo: [`docs/BlenderConstraints/01_plano.md`](../../../docs/BlenderConstraints/01_plano.md).
//! Uma constraint lê a pose MUNDIAL do dono (já com as constraints de cima aplicadas) e devolve a
//! pose mundial que ela pede; [`blend`] mistura as duas pela força. A ordem da pilha é a lei
//! (oráculo `ordem_da_pilha`): quem chama aplica os itens de cima para baixo, sem excepções.
//!
//! O ângulo é do `ph2d-skeleton` ([`blend_angle`], [`clamp_to_limit`]): uma lei, um dono.

mod caminho;
mod kind;
mod ossos;

pub use caminho::{FollowPath, PathBy};
pub use kind::{
    Canais, ChildOf, CopyTransform, Distance, DistanceMode, Faixa, Kind, Limit, LookAt, Mix, Space,
};
pub use ossos::{Ik, StretchTo, Volume, bend_in_world, solve_chain, stretch_factor};
use ph2d_affine::Xform;
pub use ph2d_skeleton::BendSide;
pub use ph2d_skeleton::{blend_angle, clamp_to_limit, wrap_pi};
use serde::{Deserialize, Serialize};

/// A pose 2D: translação, rotação (radianos) e escala por eixo. O cisalhamento não entra — a
/// composição de uma escala não uniforme rodada perde-o, como no `Transform` sem skew.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pose2 {
    pub x: f64,
    pub y: f64,
    pub rot: f64,
    pub sx: f64,
    pub sy: f64,
}

impl Default for Pose2 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Pose2 {
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        rot: 0.0,
        sx: 1.0,
        sy: 1.0,
    };

    #[must_use]
    pub fn to_xform(self) -> Xform {
        let (s, c) = self.rot.sin_cos();
        Xform([
            c * self.sx,
            s * self.sx,
            -s * self.sy,
            c * self.sy,
            self.x,
            self.y,
        ])
    }

    /// A rotação é a do eixo +X; o sinal do determinante vai para `sy` (espelho).
    #[must_use]
    pub fn from_xform(m: Xform) -> Self {
        let [a, b, _, _, e, f] = m.0;
        let sx = a.hypot(b);
        let (rot, sy) = if sx > 0.0 {
            (b.atan2(a), m.det() / sx)
        } else {
            (0.0, 0.0)
        };
        Self {
            x: e,
            y: f,
            rot,
            sx,
            sy,
        }
    }

    /// `parent ∘ self` — a pose mundial de um filho cuja pose local é `self`.
    #[must_use]
    pub fn under(self, parent: Self) -> Self {
        Self::from_xform(self.to_xform().then(&parent.to_xform()))
    }

    /// A pose local que, sob `parent`, dá `self`. `None` quando `parent` colapsa (escala 0).
    #[must_use]
    pub fn relative_to(self, parent: Self) -> Option<Self> {
        let inv = parent.to_xform().inverse()?;
        Some(Self::from_xform(self.to_xform().then(&inv)))
    }

    #[must_use]
    pub fn is_finite(&self) -> bool {
        [self.x, self.y, self.rot, self.sx, self.sy]
            .iter()
            .all(|v| v.is_finite())
    }
}

/// ⭐ **A mistura pela força `t`** — a porta única. Translação em linha recta, rotação pelo arco curto
/// (oráculo `mistura_rotacao_copy_rotation`: 170° → −170° a ½ dá 180°), escala em proporção
/// (log-lerp) quando os sinais concordam. `t ≤ 0` devolve `a` e `t ≥ 1` devolve `b` ao bit.
#[must_use]
pub fn blend(a: Pose2, b: Pose2, t: f64) -> Pose2 {
    if t.is_nan() || t <= 0.0 {
        return a;
    }
    if t >= 1.0 {
        return b;
    }
    let lerp = |p: f64, q: f64| p + (q - p) * t;
    Pose2 {
        x: lerp(a.x, b.x),
        y: lerp(a.y, b.y),
        rot: blend_angle(a.rot, b.rot, t),
        sx: blend_scale(a.sx, b.sx, t),
        sy: blend_scale(a.sy, b.sy, t),
    }
}

fn blend_scale(p: f64, q: f64, t: f64) -> f64 {
    if p * q > 0.0 {
        p.signum() * (p.abs().ln() + (q.abs().ln() - p.abs().ln()) * t).exp()
    } else {
        p + (q - p) * t
    }
}

/// O que uma constraint vê: a pose mundial do dono (com as de cima já aplicadas), a do pai dele, e a
/// do alvo e do pai do alvo (identidade sem pai).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ctx {
    pub owner: Pose2,
    pub parent: Pose2,
    pub target: Option<Pose2>,
    pub target_parent: Pose2,
}

/// Porque uma constraint não muda nada — o cartão e o gizmo leem isto, nunca recalculam.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inerte {
    SemAlvo,
    /// O pai (ou o pai do alvo) colapsou numa escala 0: não há espaço local.
    EspacoColapsado,
    /// O tipo escreve uma corrente de ossos, não só o dono: avalia-o a fase.
    Corrente,
}

/// Aplica UMA constraint com força `influence`: `Ok(pose)` ou `Err` com o motivo de não fazer nada.
///
/// # Errors
/// [`Inerte`] quando falta o alvo de um tipo que o pede, ou o espaço local não existe.
pub fn apply(kind: &Kind, ctx: &Ctx, influence: f64) -> Result<Pose2, Inerte> {
    apply_with(kind, ctx, None, influence)
}

/// [`apply`] com o caminho do Follow Path.
///
/// # Errors
/// Os de [`Kind::resolve_with`].
pub fn apply_with(
    kind: &Kind,
    ctx: &Ctx,
    path: Option<&[[f64; 2]]>,
    influence: f64,
) -> Result<Pose2, Inerte> {
    let alvo = kind.resolve_with(ctx, path)?;
    Ok(blend(ctx.owner, alvo, influence))
}

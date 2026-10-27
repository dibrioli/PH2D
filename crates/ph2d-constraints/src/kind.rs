//! Os tipos da onda 1 e a lei de cada um. Cada `resolve` devolve a pose MUNDIAL que o tipo pede.

use serde::{Deserialize, Serialize};

use crate::{
    Action, Ctx, Floor, FollowPath, Ik, Inerte, Pivot, PontoDeForma, Pose2, SplineIk, Spring,
    StretchTo, Transformation, clamp_to_limit, wrap_pi,
};

/// Em que referencial se lêem e escrevem os canais.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Space {
    #[default]
    World,
    /// Relativo ao pai — do dono, e do alvo para o que se copia dele.
    Local,
}

/// Que canais o tipo toca; os desligados ficam como estavam.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Canais {
    pub x: bool,
    pub y: bool,
    pub rot: bool,
    pub sx: bool,
    pub sy: bool,
}

impl Default for Canais {
    fn default() -> Self {
        Self::ALL
    }
}

impl Canais {
    pub const ALL: Self = Self {
        x: true,
        y: true,
        rot: true,
        sx: true,
        sy: true,
    };
    pub const LOCATION: Self = Self {
        x: true,
        y: true,
        rot: false,
        sx: false,
        sy: false,
    };
    pub const ROTATION: Self = Self {
        x: false,
        y: false,
        rot: true,
        sx: false,
        sy: false,
    };
    pub const SCALE: Self = Self {
        x: false,
        y: false,
        rot: false,
        sx: true,
        sy: true,
    };

    fn pick(self, on: Pose2, off: Pose2) -> Pose2 {
        Pose2 {
            x: if self.x { on.x } else { off.x },
            y: if self.y { on.y } else { off.y },
            rot: if self.rot { on.rot } else { off.rot },
            sx: if self.sx { on.sx } else { off.sx },
            sy: if self.sy { on.sy } else { off.sy },
        }
    }
}

/// Como o valor copiado entra: substitui o do dono, ou soma-se a ele (a escala multiplica).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mix {
    #[default]
    Replace,
    Add,
}

/// O alvo passa a ser pai: `mundo = offset ∘ alvo`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChildOf {
    pub canais: Canais,
    pub offset: Pose2,
}

/// Copia canais do alvo. `offset` soma-se depois (x, y, rot) ou multiplica (sx, sy).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CopyTransform {
    pub canais: Canais,
    pub mix: Mix,
    pub space: Space,
    pub offset: Pose2,
}

/// Aponta o eixo +X DESENHADO ao alvo — os três «track» do Blender são este no plano (oráculo
/// `apontar_3_tipos`). A fase avalia-o pela mira da casa (`ph2d-constraints-ecs`, `olha`), que é esta
/// lei mais o limite de junta e a mistura no local.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LookAt {
    pub offset: f64,
}

/// Faixa de um canal; um lado `None` não apara.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Faixa {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl Faixa {
    fn clamp(self, v: f64) -> f64 {
        let v = self.min.map_or(v, |m| v.max(m));
        self.max.map_or(v, |m| v.min(m))
    }
}

/// Apara canais. A rotação é uma faixa de ARCO: fora dela vai à borda mais próxima no círculo
/// (oráculo `limit_rotation_z_-45_45`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Limit {
    pub space: Space,
    pub x: Faixa,
    pub y: Faixa,
    pub rot: Option<[f64; 2]>,
    pub sx: Faixa,
    pub sy: Faixa,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMode {
    /// Nunca mais longe que `distance`.
    #[default]
    Inside,
    /// Nunca mais perto.
    Outside,
    /// Sempre exactamente a `distance`.
    Surface,
}

/// Mantém a distância ao alvo, projectando pelo raio (oráculo `limit_distance_inside_2`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Distance {
    pub distance: f64,
    pub mode: DistanceMode,
}

impl Default for Distance {
    fn default() -> Self {
        Self {
            distance: 1.0,
            mode: DistanceMode::Inside,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Kind {
    ChildOf(ChildOf),
    CopyTransform(CopyTransform),
    LookAt(LookAt),
    Limit(Limit),
    Distance(Distance),
    Ik(Ik),
    StretchTo(StretchTo),
    FollowPath(FollowPath),
    Pivot(Pivot),
    Floor(Floor),
    Transformation(Transformation),
    SplineIk(SplineIk),
    PontoDeForma(PontoDeForma),
    Action(Action),
    Spring(Spring),
    Valor(crate::Valor),
    Comportamento(crate::Comportamento),
}

impl Kind {
    #[must_use]
    pub fn needs_target(&self) -> bool {
        // O Pivot mostra o alvo mas não o exige: sem ele, o pivô é o dono + o desvio.
        match self {
            Self::Comportamento(c) => c.perfil.mostra_alvo(),
            _ => !matches!(self, Self::Limit(_) | Self::Spring(_)),
        }
    }

    /// Ao NASCER (e ao trocar de alvo), o tipo guarda a pose actual em vez de saltar? O Child Of
    /// sim (oráculo `child_of.*.ao_ligar` = pose de antes); o Distance guarda a distância actual.
    /// Copy e Look At apontam logo ao alvo, como no Blender — o «manter desvio» é um gesto.
    #[must_use]
    pub fn keeps_current_on_create(&self) -> bool {
        matches!(
            self,
            Self::ChildOf(_) | Self::Distance(_) | Self::StretchTo(_)
        )
    }

    /// ⭐ **A porta única de «o offset»:** acerta o desvio para que `resolve(ctx) == ctx.owner`.
    ///
    /// # Errors
    /// [`Inerte`] se falta o alvo ou o espaço colapsou — nada é mudado.
    pub fn keep_current(&mut self, ctx: &Ctx) -> Result<(), Inerte> {
        match self {
            Self::ChildOf(c) => {
                let t = ctx.target.ok_or(Inerte::SemAlvo)?;
                c.offset = ctx.owner.relative_to(t).ok_or(Inerte::EspacoColapsado)?;
            }
            Self::CopyTransform(c) => {
                let (base, src) = copy_frames(c.space, ctx)?;
                let sem = copy_channels(c.mix, base, src, Pose2::IDENTITY);
                c.offset = Pose2 {
                    x: base.x - sem.x,
                    y: base.y - sem.y,
                    rot: wrap_pi(base.rot - sem.rot),
                    sx: ratio(base.sx, sem.sx),
                    sy: ratio(base.sy, sem.sy),
                };
            }
            Self::LookAt(l) => {
                let t = ctx.target.ok_or(Inerte::SemAlvo)?;
                if let Some(aim) = aim(ctx.owner, t) {
                    l.offset = wrap_pi(ctx.owner.rot + eixo(ctx.owner) - aim);
                }
            }
            Self::Distance(d) => {
                let t = ctx.target.ok_or(Inerte::SemAlvo)?;
                d.distance = (ctx.owner.x - t.x).hypot(ctx.owner.y - t.y);
            }
            Self::StretchTo(st) => {
                let t = ctx.target.ok_or(Inerte::SemAlvo)?;
                st.rest = (ctx.owner.x - t.x).hypot(ctx.owner.y - t.y);
            }
            Self::Limit(_)
            | Self::Ik(_)
            | Self::FollowPath(_)
            | Self::Pivot(_)
            | Self::Floor(_)
            | Self::Transformation(_)
            | Self::SplineIk(_)
            | Self::PontoDeForma(_)
            | Self::Action(_)
            | Self::Spring(_)
            | Self::Valor(_)
            | Self::Comportamento(_) => {}
        }
        Ok(())
    }

    /// A pose mundial que o tipo pede, a força 1.
    ///
    /// # Errors
    /// [`Inerte`] se falta o alvo ou o espaço colapsou.
    pub fn resolve(&self, ctx: &Ctx) -> Result<Pose2, Inerte> {
        self.resolve_with(ctx, None)
    }

    /// [`Self::resolve`] com o caminho do Follow Path (poligonal em mundo).
    ///
    /// # Errors
    /// [`Inerte`] se falta o alvo ou o espaço colapsou; [`Inerte::Corrente`] para a IK, que escreve a
    /// corrente inteira e é avaliada pela fase (`ph2d-constraints-ecs`).
    pub fn resolve_with(&self, ctx: &Ctx, path: Option<&[[f64; 2]]>) -> Result<Pose2, Inerte> {
        match self {
            Self::Ik(_) | Self::SplineIk(_) => Err(Inerte::Corrente),
            Self::Action(_) | Self::Spring(_) | Self::Valor(_) | Self::Comportamento(_) => {
                Err(Inerte::Relogio)
            }
            Self::Pivot(p) => Ok(p.resolve(ctx)),
            Self::Floor(f) => f.resolve(ctx),
            Self::Transformation(t) => t.resolve(ctx),
            Self::PontoDeForma(_) => {
                let t = ctx.target.ok_or(Inerte::SemAlvo)?;
                Ok(Pose2 {
                    x: t.x,
                    y: t.y,
                    ..ctx.owner
                })
            }
            Self::StretchTo(st) => st.resolve(ctx),
            Self::FollowPath(f) => f.resolve(ctx.owner, path),
            Self::ChildOf(c) => {
                let t = ctx.target.ok_or(Inerte::SemAlvo)?;
                Ok(c.canais.pick(c.offset.under(t), ctx.owner))
            }
            Self::CopyTransform(c) => {
                let (base, src) = copy_frames(c.space, ctx)?;
                let local = c
                    .canais
                    .pick(copy_channels(c.mix, base, src, c.offset), base);
                Ok(back_to_world(c.space, local, ctx.parent))
            }
            Self::LookAt(l) => {
                let t = ctx.target.ok_or(Inerte::SemAlvo)?;
                Ok(aim(ctx.owner, t).map_or(ctx.owner, |a| Pose2 {
                    rot: a - eixo(ctx.owner) + l.offset,
                    ..ctx.owner
                }))
            }
            Self::Limit(l) => {
                let base = in_space(l.space, ctx.owner, ctx.parent)?;
                let rot = l
                    .rot
                    .map_or(base.rot, |[lo, hi]| clamp_to_limit(base.rot, lo, hi));
                let local = Pose2 {
                    x: l.x.clamp(base.x),
                    y: l.y.clamp(base.y),
                    rot,
                    sx: l.sx.clamp(base.sx),
                    sy: l.sy.clamp(base.sy),
                };
                Ok(back_to_world(l.space, local, ctx.parent))
            }
            Self::Distance(d) => {
                let t = ctx.target.ok_or(Inerte::SemAlvo)?;
                let (dx, dy) = (ctx.owner.x - t.x, ctx.owner.y - t.y);
                let r = dx.hypot(dy);
                let fora = match d.mode {
                    DistanceMode::Inside => r > d.distance,
                    DistanceMode::Outside => r < d.distance,
                    DistanceMode::Surface => r != d.distance,
                };
                if !fora || r == 0.0 {
                    return Ok(ctx.owner);
                }
                let k = d.distance / r;
                Ok(Pose2 {
                    x: t.x + dx * k,
                    y: t.y + dy * k,
                    ..ctx.owner
                })
            }
        }
    }
}

fn aim(owner: Pose2, target: Pose2) -> Option<f64> {
    let (dx, dy) = (target.x - owner.x, target.y - owner.y);
    (dx != 0.0 || dy != 0.0).then(|| dy.atan2(dx))
}

/// O ângulo que a escala dá ao eixo +X desenhado: π sob um espelho em X (pose de herança TRS, como a
/// do `Transform` da casa; uma pose vinda de um afim leva o espelho em `sy` e dá 0).
fn eixo(p: Pose2) -> f64 {
    0.0_f64.atan2(p.sx)
}

fn ratio(a: f64, b: f64) -> f64 {
    if b == 0.0 { 1.0 } else { a / b }
}

fn in_space(space: Space, world: Pose2, parent: Pose2) -> Result<Pose2, Inerte> {
    match space {
        Space::World => Ok(world),
        Space::Local => world.relative_to(parent).ok_or(Inerte::EspacoColapsado),
    }
}

pub(crate) fn back_to_world(space: Space, p: Pose2, parent: Pose2) -> Pose2 {
    match space {
        Space::World => p,
        Space::Local => p.under(parent),
    }
}

/// O dono e o alvo no espaço da cópia.
pub(crate) fn copy_frames(space: Space, ctx: &Ctx) -> Result<(Pose2, Pose2), Inerte> {
    let t = ctx.target.ok_or(Inerte::SemAlvo)?;
    Ok((
        in_space(space, ctx.owner, ctx.parent)?,
        in_space(space, t, ctx.target_parent)?,
    ))
}

fn copy_channels(mix: Mix, base: Pose2, src: Pose2, off: Pose2) -> Pose2 {
    let v = match mix {
        Mix::Replace => src,
        Mix::Add => Pose2 {
            x: base.x + src.x,
            y: base.y + src.y,
            rot: base.rot + src.rot,
            sx: base.sx * src.sx,
            sy: base.sy * src.sy,
        },
    };
    Pose2 {
        x: v.x + off.x,
        y: v.y + off.y,
        rot: v.rot + off.rot,
        sx: v.sx * off.sx,
        sy: v.sy * off.sy,
    }
}

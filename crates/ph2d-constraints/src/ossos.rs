//! Os tipos da onda 2 — a IK de uma corrente e o Stretch To. Desenho e oráculo:
//! [`docs/BlenderConstraints/02_W2_ossos.md`](../../../docs/BlenderConstraints/02_W2_ossos.md).

use ph2d_skeleton::{BendSide, Reach, reach};
use serde::{Deserialize, Serialize};

use crate::{Ctx, Inerte, Pose2};

/// A IK de uma corrente de ossos, da ponta (o dono) para cima. O alvo e a força (o *Mix*) são os do item.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ik {
    /// Quantos ossos, contados da ponta. `0` = até à raiz do esqueleto.
    pub chain: u32,
    /// Fracção do alcance da corrente onde ela começa a abrandar (`ph2d_skeleton::softened_distance`).
    pub softness: f64,
    /// De que lado o joelho dobra, no referencial da corrente (troca com o espelho).
    pub bend: BendSide,
    /// Desvio do apontar (radianos), só lido quando a corrente resolvida tem UM osso.
    pub offset: f64,
    /// Além do alcance, a corrente estica-se para a ponta chegar ao alvo.
    pub stretch: bool,
}

impl Default for Ik {
    fn default() -> Self {
        Self {
            chain: 2,
            softness: 0.0,
            bend: BendSide::Keep,
            offset: 0.0,
            stretch: false,
        }
    }
}

/// O factor de esticar (`≥ 1`): além do joelho da suavidade (`alcance − suavidade`) a corrente estica
/// `d / joelho`, e é contínua no joelho. A recusa medida do Blender (encolher ao alcance) está no doc.
#[must_use]
pub fn stretch_factor(d: f64, alcance: f64, suavidade: f64) -> f64 {
    let joelho = alcance - suavidade.clamp(0.0, alcance);
    if joelho > f64::EPSILON && d > joelho {
        d / joelho
    } else {
        1.0
    }
}

/// O lado da dobra no MUNDO: sob um referencial espelhado (determinante negativo) o lado autorado troca.
#[must_use]
pub fn bend_in_world(bend: BendSide, espelhado: bool) -> BendSide {
    if espelhado { bend.flipped() } else { bend }
}

/// ⭐ **A lei da IK de uma corrente**: as juntas (`n + 1`, em mundo, raiz primeiro) vão para o alvo;
/// devolve o factor de esticar a aplicar ao osso raiz. A lei dos 2 ossos e a cadeia são do
/// [`ph2d_skeleton::reach`] — uma lei, um dono.
pub fn solve_chain(
    juntas: &mut [[f64; 2]],
    comps: &[f64],
    alvo: [f64; 2],
    ik: &Ik,
    espelhado: bool,
) -> f64 {
    let alcance: f64 = comps.iter().sum();
    let Some(&raiz) = juntas.first() else {
        return 1.0;
    };
    if alcance <= f64::EPSILON || juntas.len() != comps.len() + 1 {
        return 1.0;
    }
    let suave = ik.softness.max(0.0) * alcance;
    let (dx, dy) = (alvo[0] - raiz[0], alvo[1] - raiz[1]);
    let k = if ik.stretch {
        stretch_factor(dx.hypot(dy), alcance, suave)
    } else {
        1.0
    };
    let pedido = [raiz[0] + dx / k, raiz[1] + dy / k];
    reach(
        juntas,
        comps,
        pedido,
        Reach {
            softness: suave,
            bend: bend_in_world(ik.bend, espelhado),
            ..Reach::default()
        },
    );
    if k > 1.0 {
        for j in juntas.iter_mut().skip(1) {
            *j = [
                raiz[0] + (j[0] - raiz[0]) * k,
                raiz[1] + (j[1] - raiz[1]) * k,
            ];
        }
    }
    k
}

/// Como a largura responde a esticar `s` (oráculo `stretch_to.*`; o `VOLUME_Z` do Blender é fora do plano).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Volume {
    /// `1/√s` — o `VOLUME_XZX` (o default do Blender).
    #[default]
    Xz,
    /// `1/s` — a área do plano mantém-se (`VOLUME_X`).
    Area,
    /// A largura não muda (`NO_VOLUME`).
    Nenhum,
}

/// Aponta o +X ao alvo e estica até ele; `rest` é o comprimento a `s = 1`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StretchTo {
    pub rest: f64,
    pub volume: Volume,
}

impl Default for StretchTo {
    fn default() -> Self {
        Self {
            rest: 1.0,
            volume: Volume::Xz,
        }
    }
}

impl StretchTo {
    pub(crate) fn resolve(&self, ctx: &Ctx) -> Result<Pose2, Inerte> {
        let t = ctx.target.ok_or(Inerte::SemAlvo)?;
        let (dx, dy) = (t.x - ctx.owner.x, t.y - ctx.owner.y);
        let d = dx.hypot(dy);
        if d == 0.0 || self.rest <= 0.0 {
            return Ok(ctx.owner);
        }
        let s = d / self.rest;
        let largura = match self.volume {
            Volume::Xz => 1.0 / s.sqrt(),
            Volume::Area => 1.0 / s,
            Volume::Nenhum => 1.0,
        };
        Ok(Pose2 {
            rot: dy.atan2(dx),
            sx: ctx.owner.sx * s,
            sy: ctx.owner.sy * largura,
            ..ctx.owner
        })
    }
}

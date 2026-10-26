//! Os tipos da onda 3. Desenho e oráculo:
//! [`docs/BlenderConstraints/03_W3_poderosas.md`](../../../docs/BlenderConstraints/03_W3_poderosas.md).

use serde::{Deserialize, Serialize};

use crate::kind::{back_to_world, copy_frames};
use crate::{Ctx, Inerte, Mix, Pose2, Space, wrap_pi};

/// A rotação autorada do dono acontece à volta do pivô: `p' = pivô + R(rot)(p − pivô)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Pivot {
    /// Somado ao alvo (mundo); sem alvo, ao próprio dono.
    pub offset: [f64; 2],
}

/// De que lado do plano o dono fica: `Mais*` acima (do lado da normal), `Menos*` abaixo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EixoDoChao {
    #[default]
    MaisY,
    MenosY,
    MaisX,
    MenosX,
}

/// O dono não atravessa o plano do alvo; o plano passa em `alvo + desvio · normal`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Floor {
    pub eixo: EixoDoChao,
    pub desvio: f64,
    /// A normal roda com o alvo (o chão inclinado); sem isto, é a do mundo.
    pub usa_rotacao: bool,
}

/// Um canal da pose.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Canal {
    #[default]
    X,
    Y,
    Rot,
    Sx,
    Sy,
}

impl Canal {
    #[must_use]
    pub fn of(self, p: Pose2) -> f64 {
        match self {
            Self::X => p.x,
            Self::Y => p.y,
            Self::Rot => p.rot,
            Self::Sx => p.sx,
            Self::Sy => p.sy,
        }
    }

    fn set(self, p: &mut Pose2, v: f64) {
        match self {
            Self::X => p.x = v,
            Self::Y => p.y = v,
            Self::Rot => p.rot = v,
            Self::Sx => p.sx = v,
            Self::Sy => p.sy = v,
        }
    }
}

/// UM canal do alvo, numa faixa, mapeado para UM canal do dono, noutra. A rotação vai em radianos.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transformation {
    pub de: Canal,
    pub de_faixa: [f64; 2],
    pub para: Canal,
    pub para_faixa: [f64; 2],
    /// Fora da faixa de entrada continua a recta; sem isto, apara nas bordas.
    pub extrapola: bool,
    /// `Add` soma ao autorado (a escala multiplica).
    pub mix: Mix,
    pub space: Space,
}

impl Default for Transformation {
    fn default() -> Self {
        Self {
            de: Canal::X,
            de_faixa: [0.0, 1.0],
            para: Canal::Rot,
            para_faixa: [0.0, std::f64::consts::FRAC_PI_2],
            extrapola: false,
            mix: Mix::Replace,
            space: Space::World,
        }
    }
}

/// Como a corrente se deita na curva.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ajuste {
    /// Os ossos mantêm o comprimento; além do fim, a tangente final.
    #[default]
    Nenhum,
    /// A corrente cobre a curva inteira: cada osso estica até à corda.
    Esticar,
}

/// A corrente de ossos (da ponta, o dono, para cima) deita-se na curva do alvo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplineIk {
    /// Quantos ossos, contados da ponta. `0` = até à raiz do esqueleto.
    pub chain: u32,
    pub ajuste: Ajuste,
}

impl Default for SplineIk {
    fn default() -> Self {
        Self {
            chain: 3,
            ajuste: Ajuste::Nenhum,
        }
    }
}

/// Copy Location de um vértice AUTORADO da forma alvo (o índice; o id estável da W4 migra-o). A fase
/// põe no alvo o ponto já no mundo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PontoDeForma {
    pub indice: u32,
}

impl Pivot {
    pub(crate) fn resolve(&self, ctx: &Ctx) -> Pose2 {
        let o = ctx.owner;
        let base = ctx.target.unwrap_or(o);
        let c = [base.x + self.offset[0], base.y + self.offset[1]];
        let (s, co) = o.rot.sin_cos();
        let (dx, dy) = (o.x - c[0], o.y - c[1]);
        Pose2 {
            x: c[0] + co * dx - s * dy,
            y: c[1] + s * dx + co * dy,
            ..o
        }
    }
}

impl Floor {
    pub(crate) fn resolve(&self, ctx: &Ctx) -> Result<Pose2, Inerte> {
        let t = ctx.target.ok_or(Inerte::SemAlvo)?;
        let (n, lado) = match self.eixo {
            EixoDoChao::MaisY => ([0.0, 1.0], 1.0),
            EixoDoChao::MenosY => ([0.0, 1.0], -1.0),
            EixoDoChao::MaisX => ([1.0, 0.0], 1.0),
            EixoDoChao::MenosX => ([1.0, 0.0], -1.0),
        };
        let n = if self.usa_rotacao {
            let (s, c) = t.rot.sin_cos();
            [c * n[0] - s * n[1], s * n[0] + c * n[1]]
        } else {
            n
        };
        let p = [t.x + self.desvio * n[0], t.y + self.desvio * n[1]];
        let o = ctx.owner;
        let d = (o.x - p[0]) * n[0] + (o.y - p[1]) * n[1];
        if d * lado >= 0.0 {
            return Ok(o);
        }
        Ok(Pose2 {
            x: o.x - d * n[0],
            y: o.y - d * n[1],
            ..o
        })
    }
}

impl Transformation {
    pub(crate) fn resolve(&self, ctx: &Ctx) -> Result<Pose2, Inerte> {
        let (base, src) = copy_frames(self.space, ctx)?;
        let v = match self.de {
            Canal::Rot => wrap_pi(src.rot),
            c => c.of(src),
        };
        let [a, b] = self.de_faixa;
        let t = if b == a {
            if v >= a { 1.0 } else { 0.0 }
        } else {
            (v - a) / (b - a)
        };
        let t = if self.extrapola { t } else { t.clamp(0.0, 1.0) };
        let [c, d] = self.para_faixa;
        let out = c + t * (d - c);
        let agora = self.para.of(base);
        let v = match (self.mix, self.para) {
            (Mix::Replace, _) => out,
            (Mix::Add, Canal::Sx | Canal::Sy) => agora * out,
            (Mix::Add, _) => agora + out,
        };
        let mut p = base;
        self.para.set(&mut p, v);
        Ok(back_to_world(self.space, p, ctx.parent))
    }
}

fn comprimentos(curva: &[[f64; 2]]) -> Vec<f64> {
    curva
        .windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .collect()
}

fn no_arco(curva: &[[f64; 2]], segs: &[f64], s: f64) -> [f64; 2] {
    let mut resto = s.max(0.0);
    for (i, &l) in segs.iter().enumerate() {
        if resto <= l && l > 0.0 {
            let (a, b) = (curva[i], curva[i + 1]);
            let u = resto / l;
            return [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u];
        }
        resto -= l;
    }
    curva[curva.len() - 1]
}

/// O menor `u ∈ [desde, 1]` com `|a + u(b − a) − p| = l`.
fn corda_no_segmento(a: [f64; 2], b: [f64; 2], p: [f64; 2], l: f64, desde: f64) -> Option<f64> {
    let d = [b[0] - a[0], b[1] - a[1]];
    let f = [a[0] - p[0], a[1] - p[1]];
    let qa = d[0] * d[0] + d[1] * d[1];
    if qa <= 0.0 {
        return None;
    }
    let qb = 2.0 * (f[0] * d[0] + f[1] * d[1]);
    let qc = f[0] * f[0] + f[1] * f[1] - l * l;
    let disc = qb * qb - 4.0 * qa * qc;
    if disc < 0.0 {
        return None;
    }
    let r = disc.sqrt();
    [(-qb - r) / (2.0 * qa), (-qb + r) / (2.0 * qa)]
        .into_iter()
        .find(|&u| u >= desde && u <= 1.0)
}

/// ⭐ **A lei da Spline IK**: as juntas (`n + 1`, raiz primeiro) da corrente de comprimentos `comps`
/// deitada na poligonal `curva` (mundo). A raiz vai ao início; [`Ajuste::Nenhum`]: cada junta é o
/// primeiro ponto da curva à CORDA = comprimento do osso, e além do fim segue a tangente final;
/// [`Ajuste::Esticar`]: a junta `k` na fracção do arco = comprimento acumulado ÷ total. `None` sem
/// curva ou sem ossos.
#[must_use]
pub fn lay_on_curve(comps: &[f64], curva: &[[f64; 2]], ajuste: Ajuste) -> Option<Vec<[f64; 2]>> {
    let segs = comprimentos(curva);
    let total: f64 = segs.iter().sum();
    let repouso: f64 = comps.iter().sum();
    if comps.is_empty() || segs.is_empty() || !(total > 0.0 && total.is_finite()) {
        return None;
    }
    let mut juntas = vec![curva[0]];
    match ajuste {
        Ajuste::Esticar => {
            if repouso <= 0.0 {
                return None;
            }
            let mut acc = 0.0;
            for &l in comps {
                acc += l;
                juntas.push(no_arco(curva, &segs, acc / repouso * total));
            }
        }
        Ajuste::Nenhum => {
            let fim = curva[curva.len() - 1];
            let i_fim = segs.iter().rposition(|&l| l > 0.0)?;
            let rumo = [
                (curva[i_fim + 1][0] - curva[i_fim][0]) / segs[i_fim],
                (curva[i_fim + 1][1] - curva[i_fim][1]) / segs[i_fim],
            ];
            let (mut seg, mut desde) = (0, 0.0);
            for &l in comps {
                let p = juntas[juntas.len() - 1];
                let mut achou = None;
                while seg < segs.len() {
                    if let Some(u) = corda_no_segmento(curva[seg], curva[seg + 1], p, l, desde) {
                        achou = Some(u);
                        break;
                    }
                    seg += 1;
                    desde = 0.0;
                }
                let q = if let Some(u) = achou {
                    desde = u;
                    let (a, b) = (curva[seg], curva[seg + 1]);
                    [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u]
                } else {
                    // Além do fim: o ponto da recta tangente final à corda `l`.
                    let f = [fim[0] - p[0], fim[1] - p[1]];
                    let qb = 2.0 * (f[0] * rumo[0] + f[1] * rumo[1]);
                    let qc = f[0] * f[0] + f[1] * f[1] - l * l;
                    let s = ((-qb + (qb * qb - 4.0 * qc).max(0.0).sqrt()) / 2.0).max(0.0);
                    [fim[0] + rumo[0] * s, fim[1] + rumo[1] * s]
                };
                juntas.push(q);
            }
        }
    }
    Some(juntas)
}

/// O canal do alvo, numa faixa, escolhe o instante de um clip do DONO, que o dono toca (o Smart
/// Bone generalizado: a lei do tempo é a dele, [`ph2d_skeleton::action_time`]).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub canal: Canal,
    /// A faixa do canal do alvo (a rotação em radianos), aparada nas bordas.
    pub faixa: [f64; 2],
    /// A posição do clip na animação do dono (`TimelineDoc::clips_of`).
    pub clip: u32,
    /// O instante do clip na borda de baixo e na de cima da faixa (segundos).
    pub de_t: f64,
    pub ate_t: f64,
}

impl Default for Action {
    fn default() -> Self {
        Self {
            canal: Canal::Rot,
            faixa: [0.0, std::f64::consts::FRAC_PI_2],
            clip: 0,
            de_t: 0.0,
            ate_t: 1.0,
        }
    }
}

impl Action {
    /// ⭐ O instante do clip que o alvo pede.
    #[must_use]
    pub fn tempo(&self, alvo: Pose2) -> f64 {
        let v = match self.canal {
            Canal::Rot => wrap_pi(alvo.rot),
            c => c.of(alvo),
        };
        self.de_t
            + ph2d_skeleton::action_time(v, self.faixa[0], self.faixa[1], self.ate_t - self.de_t)
    }
}

/// O dono segue a própria pose ANIMADA com atraso de mola (os knobs do nó `motion.spring`, modo
/// Time). O estado é função do quadro — a fase re-simula no scrub (`ph2d-constraints-ecs`, `mola`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spring {
    /// Só `x`, `y` e `rot` têm mola.
    pub canais: crate::Canais,
    /// Segundos de uma oscilação.
    pub duracao: f64,
    /// `0` = sem ressalto; perto de `1` = vibra muito.
    pub ressalto: f64,
}

impl Default for Spring {
    fn default() -> Self {
        Self {
            canais: crate::Canais::LOCATION,
            duracao: 0.5,
            ressalto: 0.5,
        }
    }
}

impl Spring {
    /// A pose com o atraso `atraso = (x, y, rot)` da mola somado nos canais ligados, à força `forca`.
    #[must_use]
    pub fn aplica(&self, owner: Pose2, atraso: [f64; 3], forca: f64) -> Pose2 {
        let k = forca.clamp(0.0, 1.0);
        let c = self.canais;
        Pose2 {
            x: owner.x + if c.x { atraso[0] * k } else { 0.0 },
            y: owner.y + if c.y { atraso[1] * k } else { 0.0 },
            rot: owner.rot + if c.rot { atraso[2] * k } else { 0.0 },
            ..owner
        }
    }
}

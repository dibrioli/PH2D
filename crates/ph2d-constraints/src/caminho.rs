//! Follow Path — o dono vai a um ponto do caminho (pelo comprimento do arco) e, se pedido, vira-se
//! para a tangente. O caminho chega como poligonal em MUNDO (quem a lê da forma é a família).

use serde::{Deserialize, Serialize};

use crate::{Inerte, Pose2};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathBy {
    /// `value` em fracção `0..=1` do comprimento.
    #[default]
    Fraction,
    /// `value` em unidades do mundo, a contar do início.
    Distance,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FollowPath {
    pub by: PathBy,
    pub value: f64,
    /// Rodar para a tangente (o +X para a frente).
    pub follow: bool,
    /// Somado à tangente (radianos).
    pub offset: f64,
}

impl Default for FollowPath {
    fn default() -> Self {
        Self {
            by: PathBy::Fraction,
            value: 0.0,
            follow: true,
            offset: 0.0,
        }
    }
}

impl FollowPath {
    /// Num caminho fechado (primeiro ponto = último) o ponto dá a volta; aberto, apara nas pontas.
    /// A tangente é a do segmento: num canto muda de uma vez (recusa medida do Blender no doc).
    pub(crate) fn resolve(&self, owner: Pose2, path: Option<&[[f64; 2]]>) -> Result<Pose2, Inerte> {
        let pts = path.filter(|p| p.len() >= 2).ok_or(Inerte::SemAlvo)?;
        let segs: Vec<f64> = pts
            .windows(2)
            .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
            .collect();
        let total: f64 = segs.iter().sum();
        if total <= 0.0 || !total.is_finite() {
            return Ok(owner);
        }
        let pedido = match self.by {
            PathBy::Fraction => self.value * total,
            PathBy::Distance => self.value,
        };
        let fechado = pts.first() == pts.last();
        let s = if fechado {
            pedido.rem_euclid(total)
        } else {
            pedido.clamp(0.0, total)
        };
        let mut resto = s;
        let mut i = 0;
        while i + 1 < segs.len() && (resto > segs[i] || segs[i] == 0.0) {
            resto -= segs[i];
            i += 1;
        }
        let (a, b) = (pts[i], pts[i + 1]);
        let t = if segs[i] > 0.0 {
            (resto / segs[i]).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let rot = if self.follow {
            (b[1] - a[1]).atan2(b[0] - a[0]) + self.offset
        } else {
            owner.rot
        };
        Ok(Pose2 {
            x: a[0] + (b[0] - a[0]) * t,
            y: a[1] + (b[1] - a[1]) * t,
            rot,
            ..owner
        })
    }
}

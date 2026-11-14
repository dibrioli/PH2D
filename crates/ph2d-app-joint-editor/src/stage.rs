//! **O desenho do gabarito**: onde cada forma-guia fica no ecrã, que forma está debaixo do cursor, e onde
//! nasce cada junta. Puro (metros → píxeis); a pintura mora em [`crate::paint_stage`].
//!
//! ⚠️ O desenho é o do GABARITO (as formas-guia de `ph2d-joint-templates`), não o da cena: a peça do
//! artista pode ter outro tamanho, e o que a aba ensina é o papel. Só a cota do entre-eixos mede a cena.

use ph2d_editor_core::zones::Rect;
use ph2d_joint_templates::{Anchor, Guide, Role, Side, Spot, Template};

/// Metros → píxeis do palco: `y` cresce para cima no gabarito e para baixo no ecrã.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StageMap {
    /// O píxel do ponto `(0, 0)` do gabarito (o chão, no meio).
    pub origin: (f32, f32),
    /// Píxeis por metro.
    pub scale: f32,
}

/// Quantas formas um papel mostra: as de fábrica, ou mais se o artista já pôs mais.
#[must_use]
pub fn shown_count(role: &Role, placed: usize) -> u16 {
    let factory = role.many.map_or(1, |m| m.count);
    factory.max(u16::try_from(placed).unwrap_or(u16::MAX))
}

/// O centro da `n`-ésima forma de um papel, metros.
#[must_use]
pub fn centre_of(role: &Role, n: u16) -> [f32; 2] {
    let step = role.many.map_or([0.0, 0.0], |m| m.step);
    let k = f32::from(n);
    [role.at[0] + step[0] * k, role.at[1] + step[1] * k]
}

impl StageMap {
    /// Enquadra o gabarito em `r` (com `pad` píxeis de folga), sem deformar e sem passar de `max_scale` px/m.
    #[must_use]
    pub fn fit(t: &Template, counts: &[u16], r: Rect, pad: f32, max_scale: f32) -> Self {
        let (mut lo, mut hi) = ([f32::MAX, 0.0_f32], [f32::MIN, 0.0_f32]);
        for (i, role) in t.roles.iter().enumerate() {
            let n = counts.get(i).copied().unwrap_or(1).max(1);
            let h = role.guide.half();
            for k in 0..n {
                let c = centre_of(role, k);
                lo = [lo[0].min(c[0] - h[0]), lo[1].min(c[1] - h[1])];
                hi = [hi[0].max(c[0] + h[0]), hi[1].max(c[1] + h[1])];
            }
        }
        if lo[0] > hi[0] {
            lo = [-1.0, 0.0];
            hi = [1.0, 1.0];
        }
        let (bw, bh) = (
            (hi[0] - lo[0]).max(f32::EPSILON),
            (hi[1] - lo[1]).max(f32::EPSILON),
        );
        let (aw, ah) = ((r.w - pad * 2.0).max(1.0), (r.h - pad * 2.0).max(1.0));
        let scale = (aw / bw).min(ah / bh).min(max_scale);
        let cx = r.x + r.w * 0.5 - (lo[0] + hi[0]) * 0.5 * scale;
        let cy = r.y + r.h * 0.5 + (lo[1] + hi[1]) * 0.5 * scale;
        Self {
            origin: (cx, cy),
            scale,
        }
    }

    #[must_use]
    pub fn px(&self, m: [f32; 2]) -> (f32, f32) {
        (
            self.origin.0 + m[0] * self.scale,
            self.origin.1 - m[1] * self.scale,
        )
    }

    /// De volta: o ponto do gabarito sob o píxel.
    #[must_use]
    pub fn metres(&self, x: f32, y: f32) -> [f32; 2] {
        [
            (x - self.origin.0) / self.scale,
            (self.origin.1 - y) / self.scale,
        ]
    }

    /// A caixa no ecrã da forma `n` do papel.
    #[must_use]
    pub fn shape_rect(&self, role: &Role, n: u16) -> Rect {
        let c = self.px(centre_of(role, n));
        let h = role.guide.half();
        let (hw, hh) = (h[0] * self.scale, h[1] * self.scale);
        Rect::new(c.0 - hw, c.1 - hh, hw * 2.0, hh * 2.0)
    }
}

/// O ponto `s` na forma-guia de quem ele cita, metros.
fn spot_at(t: &Template, a: u16, b: u16, s: Spot, n: [u16; 2]) -> Option<[f32; 2]> {
    let (role, k) = match s.on {
        Side::A => (t.role(a)?, n[0]),
        Side::B => (t.role(b)?, n[1]),
    };
    let c = centre_of(role, k);
    let h = role.guide.half();
    Some([c[0] + s.u * h[0], c[1] + s.v * h[1]])
}

/// As duas pontas de uma junta da regra `rule` entre as formas `n[0]` (de A) e `n[1]` (de B), metros.
/// Num ponto só (pino, roda, trilho), as duas são iguais.
#[must_use]
pub fn rule_ends(t: &Template, rule: u16, n: [u16; 2]) -> Option<([f32; 2], [f32; 2])> {
    let r = t.rule(rule)?;
    match r.anchor {
        Anchor::At(s) => {
            let p = spot_at(t, r.a, r.b, s, n)?;
            Some((p, p))
        }
        Anchor::Ends(sa, sb) => Some((spot_at(t, r.a, r.b, sa, n)?, spot_at(t, r.a, r.b, sb, n)?)),
        Anchor::Seam => {
            let (ra, rb) = (t.role(r.a)?, t.role(r.b)?);
            let (ca, cb) = (centre_of(ra, n[0]), centre_of(rb, n[1]));
            let p = [(ca[0] + cb[0]) * 0.5, (ca[1] + cb[1]) * 0.5];
            Some((p, p))
        }
    }
}

/// O ponto `(x, y)` está dentro da forma-guia `role`/`n` (o disco é redondo de verdade)?
#[must_use]
pub fn inside(map: &StageMap, role: &Role, n: u16, x: f32, y: f32) -> bool {
    let r = map.shape_rect(role, n);
    match role.guide {
        Guide::Box { .. } => r.contains(x, y),
        Guide::Disc { .. } => {
            let (cx, cy, rad) = (r.x + r.w * 0.5, r.y + r.h * 0.5, r.w * 0.5);
            (x - cx).powi(2) + (y - cy).powi(2) <= rad * rad
        }
    }
}

/// O papel debaixo do cursor. Os discos (rodas) vêm à frente das caixas (o corpo por cima delas).
#[must_use]
pub fn role_at(map: &StageMap, t: &Template, counts: &[u16], x: f32, y: f32) -> Option<u16> {
    let order = t
        .roles
        .iter()
        .enumerate()
        .filter(|(_, r)| matches!(r.guide, Guide::Disc { .. }))
        .chain(
            t.roles
                .iter()
                .enumerate()
                .filter(|(_, r)| matches!(r.guide, Guide::Box { .. })),
        );
    for (i, role) in order {
        let n = counts.get(i).copied().unwrap_or(1).max(1);
        if (0..n).any(|k| inside(map, role, k, x, y)) {
            return u16::try_from(i).ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_joint_templates::{JointKind, by_name};

    fn car() -> &'static Template {
        by_name(JointKind::Wheel, "Car").expect("o carro")
    }

    /// ⭐ O gabarito cabe inteiro no palco e o metro vira píxel nos dois sentidos.
    #[test]
    fn o_carro_cabe_no_palco_e_volta() {
        let r = Rect::new(100.0, 50.0, 600.0, 300.0);
        let m = StageMap::fit(car(), &[1, 1, 1], r, 20.0, f32::MAX);
        for (i, role) in car().roles.iter().enumerate() {
            let s = m.shape_rect(role, 0);
            assert!(
                s.x >= r.x && s.x + s.w <= r.x + r.w + 0.01,
                "papel {i} sai na horizontal"
            );
            assert!(
                s.y >= r.y && s.y + s.h <= r.y + r.h + 0.01,
                "papel {i} sai na vertical"
            );
        }
        let p = m.px([0.7, 0.3]);
        let back = m.metres(p.0, p.1);
        assert!((back[0] - 0.7).abs() < 1e-4 && (back[1] - 0.3).abs() < 1e-4);
    }

    /// ⭐ **A roda debaixo do cursor ganha ao corpo**, e o centro de cada forma é dela.
    ///
    /// Mutação que deve sangrar: pôr as caixas antes dos discos em [`role_at`].
    #[test]
    fn a_roda_ganha_ao_corpo_sob_o_cursor() {
        let r = Rect::new(0.0, 0.0, 800.0, 400.0);
        let m = StageMap::fit(car(), &[1, 1, 1], r, 20.0, f32::MAX);
        for (i, role) in car().roles.iter().enumerate() {
            let c = m.px(centre_of(role, 0));
            assert_eq!(role_at(&m, car(), &[1, 1, 1], c.0, c.1), Some(i as u16));
        }
        // A borda de cima da roda da frente está DENTRO do corpo e da roda: ganha a roda.
        let front = &car().roles[2];
        let c = m.px(centre_of(front, 0));
        let top = (c.0, c.1 - front.guide.half()[1] * m.scale * 0.9);
        assert_eq!(role_at(&m, car(), &[1, 1, 1], top.0, top.1), Some(2));
        assert_eq!(role_at(&m, car(), &[1, 1, 1], r.x + 1.0, r.y + 1.0), None);
    }

    /// ⭐ Um papel em fileira (os elos da corrente) responde em TODAS as cópias, não só na primeira.
    ///
    /// Mutação que deve sangrar: ignorar `counts` em [`role_at`] (tratar todo papel como um só).
    #[test]
    fn a_fileira_responde_em_cada_copia() {
        let chain = by_name(JointKind::Rope, "Chain").expect("a corrente");
        let counts = [1, 6];
        let m = StageMap::fit(
            chain,
            &counts,
            Rect::new(0.0, 0.0, 400.0, 800.0),
            20.0,
            f32::MAX,
        );
        for k in 0..6 {
            let c = m.px(centre_of(&chain.roles[1], k));
            assert_eq!(role_at(&m, chain, &counts, c.0, c.1), Some(1), "elo {k}");
        }
    }

    /// O eixo da roda do carro nasce no centro da roda (`at_b(0, 0)`).
    #[test]
    fn a_junta_da_roda_nasce_no_centro_dela() {
        let (a, b) = rule_ends(car(), 1, [0, 0]).expect("a regra da frente");
        assert_eq!(a, b);
        assert_eq!(a, car().roles[2].at);
    }
}

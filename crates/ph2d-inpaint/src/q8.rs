//! [`Q8`] — one pyramid level as both backends see it: premultiplied RGBA8. The match score and the
//! vote are then integer arithmetic, so the CPU and the GPU give the same byte (B38.3).

use crate::plane::{Plane, clampi};

/// Largest patch radius whose vote fits the `u32` accumulators: `(2r+1)² · 2¹⁶ · 255 < 2³²` ⇒ `r ≤ 7`
/// (the Heal asks 2–6).
pub const MAX_PATCH_RADIUS: u32 = 7;

/// `2¹⁶ · 255²`: the Q16 weight `1/(1 + ssd/255²)` is `PESO_UM / (255² + ssd)`; still `< 2³²`.
pub(crate) const PESO_UM: u32 = 65_536 * 65_025;

/// Premultiplied RGBA8 level image, `px.len() == w * h`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Q8 {
    pub w: usize,
    pub h: usize,
    pub px: Vec<[u8; 4]>,
}

impl Q8 {
    pub fn from_plane(p: &Plane) -> Q8 {
        let px =
            p.px.as_chunks::<4>()
                .0
                .iter()
                .map(|c| c.map(|v| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8))
                .collect();
        Q8 { w: p.w, h: p.h, px }
    }

    pub fn to_plane(&self) -> Plane {
        let mut p = Plane::new(self.w, self.h);
        for (d, s) in p.px.as_chunks_mut::<4>().0.iter_mut().zip(&self.px) {
            *d = s.map(|v| f32::from(v) / 255.0);
        }
        p
    }

    /// Clamp-to-edge read.
    #[inline]
    pub fn get(&self, x: i32, y: i32) -> [u8; 4] {
        self.px[clampi(y, self.h) * self.w + clampi(x, self.w)]
    }
}

/// The Q16 vote weight of a match with score `ssd`.
#[inline]
pub(crate) fn peso(ssd: u32) -> u32 {
    PESO_UM / (65_025 + ssd)
}

/// The rounded weighted mean `(Σ w·c + Σw/2) / Σw` of one channel.
#[inline]
pub(crate) fn media(soma: u32, wsum: u32) -> u8 {
    ((soma + wsum / 2) / wsum) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    /// B38.3 — o voto é a média ponderada ARREDONDADA, ao byte, em todos os canais (o alfa incluído): mata
    /// a truncagem e o recíproco `× (1/Σw)` (a mutação sobrevivente do B38.2 era o alfa por `× iw`).
    #[test]
    fn o_voto_e_a_media_arredondada_em_todos_os_canais() {
        let mut casos = 0u32;
        let mut x = 0x9e37_79b9u32;
        for _ in 0..20_000 {
            let n = 2 + (x % 48) as usize;
            let (mut soma, mut wsum, mut exacto) = (0u32, 0u32, 0f64);
            for _ in 0..n {
                x = crate::hash::hash32(x);
                let w = peso(x % 12_000_000);
                let c = (x >> 8) as u8;
                soma += w * u32::from(c);
                wsum += w;
                exacto += f64::from(w) * f64::from(c);
            }
            let m = media(soma, wsum);
            assert_eq!(
                f64::from(m),
                (exacto / f64::from(wsum) + 0.5).floor(),
                "Σ={soma} Σw={wsum}"
            );
            assert_eq!(media(255 * wsum, wsum), 255, "todo opaco dá 255 exacto");
            casos += 1;
        }
        assert_eq!(casos, 20_000);
        assert_eq!(
            (peso(0), peso(12_744_900)),
            (65_536, 332),
            "os extremos do peso Q16"
        );
    }
}

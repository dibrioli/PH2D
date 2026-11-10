//! **O dono do halo** (B16.2): o texel SEM dono (o halo de um traço, a amostra deslocada que cai nele)
//! resolve-se pelo dono MAIS PRÓXIMO dentro do alcance, não pelo pincel vivo. Com o pincel vivo, trocar
//! o raio entre traços reescrevia o halo dos traços velhos, e o pen-up pelo traço deixava de dar os
//! bytes do pen-up pela união (90 texels no 3.º traço, raio 3→5).
//!
//! O mapa é função só do mapa de donos do canvas e do alcance, nunca da janela: a transformada corre
//! sobre a janela ⊕ alcance, e o caminho chanfrado óptimo de uma semente ao texel fica na caixa dos dois.

use super::super::watercolor_field::Dono;

#[cfg(test)]
thread_local! {
    /// O oráculo da prancha B16.2: o halo pelo pincel VIVO (o caminho antigo).
    pub(super) static HALO_PELO_PINCEL_VIVO: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

/// O dono resolvido de cada texel da janela de leitura `(rx0, ry0, rw, rh)`.
pub(super) struct DonoDoHalo {
    rx0: usize,
    ry0: usize,
    rw: usize,
    rh: usize,
    local: Vec<Dono>,
}

impl DonoDoHalo {
    /// `owner` = o mapa do canvas (`fw × fh`); `alcance` em px (o `pad` da janela). Empate: o mais
    /// próximo pela métrica chanfrada 3-4; na mesma distância, o dono mais RECENTE (o id maior).
    pub(super) fn build(
        owner: &[Dono],
        (fw, fh): (usize, usize),
        (rx0, ry0, rw, rh): (usize, usize, usize, usize),
        alcance: usize,
    ) -> Self {
        let ex0 = rx0.saturating_sub(alcance);
        let ey0 = ry0.saturating_sub(alcance);
        let ex1 = (rx0 + rw + alcance).min(fw);
        let ey1 = (ry0 + rh + alcance).min(fh);
        let (ew, eh) = (ex1 - ex0, ey1 - ey0);
        const INF: u64 = u64::MAX / 4;
        let limite = 3 * alcance as u64;
        // Chave = distância · 2¹⁶ + (MAX − dono): o mínimo é o mais próximo e, no empate, o mais recente.
        let chave = |d: u64, o: Dono| (d << 16) | u64::from(Dono::MAX - o);
        let mut k: Vec<u64> = Vec::with_capacity(ew * eh);
        for y in ey0..ey1 {
            for &o in &owner[y * fw + ex0..y * fw + ex1] {
                k.push(if o != 0 { chave(0, o) } else { INF });
            }
        }
        let relaxa = |k: &mut [u64], i: usize, j: usize, custo: u64| {
            if k[j] < INF {
                let c = k[j] + (custo << 16);
                if c < k[i] {
                    k[i] = c;
                }
            }
        };
        for y in 0..eh {
            for x in 0..ew {
                let i = y * ew + x;
                if x > 0 {
                    relaxa(&mut k, i, i - 1, 3);
                }
                if y > 0 {
                    relaxa(&mut k, i, i - ew, 3);
                    if x > 0 {
                        relaxa(&mut k, i, i - ew - 1, 4);
                    }
                    if x + 1 < ew {
                        relaxa(&mut k, i, i - ew + 1, 4);
                    }
                }
            }
        }
        for y in (0..eh).rev() {
            for x in (0..ew).rev() {
                let i = y * ew + x;
                if x + 1 < ew {
                    relaxa(&mut k, i, i + 1, 3);
                }
                if y + 1 < eh {
                    relaxa(&mut k, i, i + ew, 3);
                    if x + 1 < ew {
                        relaxa(&mut k, i, i + ew + 1, 4);
                    }
                    if x > 0 {
                        relaxa(&mut k, i, i + ew - 1, 4);
                    }
                }
            }
        }
        let mut local = vec![0; rw * rh];
        for wy in 0..rh {
            let ey = ry0 + wy - ey0;
            for wx in 0..rw {
                let c = k[ey * ew + (rx0 + wx - ex0)];
                if c < INF && (c >> 16) <= limite {
                    local[wy * rw + wx] = Dono::MAX - (c & 0xFFFF) as Dono;
                }
            }
        }
        Self {
            rx0,
            ry0,
            rw,
            rh,
            local,
        }
    }

    /// Sem mapa de estilos: todo texel devolve o `cru`.
    pub(super) fn vazio() -> Self {
        Self {
            rx0: 0,
            ry0: 0,
            rw: 0,
            rh: 0,
            local: Vec::new(),
        }
    }

    /// O dono do texel `(gx, gy)` do canvas; fora da janela, o do mapa (`cru`).
    #[inline]
    pub(super) fn em(&self, gx: usize, gy: usize, cru: Dono) -> Dono {
        if cru != 0 {
            return cru;
        }
        #[cfg(test)]
        if HALO_PELO_PINCEL_VIVO.get() {
            return cru;
        }
        let (Some(wx), Some(wy)) = (gx.checked_sub(self.rx0), gy.checked_sub(self.ry0)) else {
            return cru;
        };
        if wx < self.rw && wy < self.rh {
            self.local[wy * self.rw + wx]
        } else {
            cru
        }
    }
}

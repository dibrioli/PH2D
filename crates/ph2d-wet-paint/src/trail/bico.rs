//! **O BICO** — os passos 1–2 da transferência (a auto-limpeza e a recolha da cor do canvas) e o reservatório
//! (filho de [`super`], LOC cap, a partir de `transfer.rs`). Movimento de código puro; o fingerprint da sessão
//! prende a identidade ao byte.

use super::*;

impl Trail {
    /// Steps 1–2 of the transfer — **o BICO**: a auto-limpeza (toda a janela) e a recolha da cor
    /// do canvas (as linhas tocadas), numa passada por linha.
    ///
    /// ⚠️ **Byte-idêntico em qualquer rota, e é estrutural** (as três condições do
    /// [`crate::par`], ADR-0175): cada texel do bico é escrito só na própria linha; o que a recolha
    /// LÊ é o grid (`sett`, `susp` e as duas cores), que este passo não escreve; e não há redução.
    /// A ordem por texel é a de sempre — primeiro limpa, depois recolhe —, porque as duas vivem no
    /// MESMO corpo de linha em vez de dois laços sobre a janela.
    ///
    /// ⛔ **Os passos 3 e 4 ficam em série, e não por gosto:** o 3 é uma SOMA em `f64` (a ordem
    /// muda os bits), e o arrasto do 4 lê `susp[si]`/`sett[si]`/`film[si]` de uma célula que o
    /// mesmo laço pode já ter escrito — é Gauss-Seidel, e trocá-lo por Jacobi mudaria a tinta.
    pub(super) fn tip_rows(&mut self, g: &Grid, p: &Params, mode: Option<crate::par::Rows>) {
        let clean = p.k(Knob::TipClean);
        let pickup = (self.lx1 >= self.lx0).then_some((self.lx0, self.ly0, self.lx1, self.ly1));
        if clean <= 0.0 && pickup.is_none() {
            return;
        }
        let size = self.size as usize;
        let (base_r, base_g, base_b) = (self.base_r, self.base_g, self.base_b);
        let (half, ax, ay) = (self.half, self.anchor_x, self.anchor_y);
        let (s, w, h) = (g.s, g.w as i32, g.h as i32);
        let tip_retain = 1.0 - p.k(Knob::Pickup);
        let reservatorio = p.k(Knob::TipReservoir);
        let dabs = self.dab_count.max(1) as i32;
        let mode =
            mode.unwrap_or_else(|| crate::par::Rows::pick(size, size, crate::par::MIN_CELLS_TIP));
        // `limpo` (com a cor por célula): o peso da cor limpa em cada texel segue as MESMAS duas
        // interpolações — a limpeza devolve-lhe `clean`, a recolha guarda `k` dele.
        let linha = |ly: usize,
                     tr: &mut [f32],
                     tg: &mut [f32],
                     tb: &mut [f32],
                     mut limpo: Option<&mut [f32]>| {
            // 1. Tip self-cleaning: every texel eases back toward the stroke color.
            //
            // ⚠️ **O laço mede a janela VIVA, nunca a do PISO.** Ele ia `0..N` (a
            // área de `TRAIL_SIZE²`), o que era o mesmo número enquanto a janela
            // era fixa e virou um bug mudo no dia em que ela passou a seguir o
            // pincel: os 15129 primeiros índices LINEARES não são uma região, são
            // as ~18 primeiras LINHAS de uma janela de 845 de largura, e o corpo do
            // pincel nunca mais limpava. Num pincel dentro do teto do modelo os
            // dois números coincidem ⇒ o fingerprint segue byte-idêntico POR
            // CONSTRUÇÃO. Gate: `the_tip_cleaning_covers_the_live_window_…`.
            if clean > 0.0 {
                for l in 0..tr.len() {
                    tr[l] = (tr[l] as f64 + (base_r - tr[l] as f64) * clean) as f32;
                    tg[l] = (tg[l] as f64 + (base_g - tg[l] as f64) * clean) as f32;
                    tb[l] = (tb[l] as f64 + (base_b - tb[l] as f64) * clean) as f32;
                }
                if let Some(w) = limpo.as_deref_mut() {
                    for v in w.iter_mut() {
                        *v = (*v as f64 + (1.0 - *v as f64) * clean) as f32;
                    }
                }
            }
            // 2. Tip pickup (the dirty brush) — reads the PRE-deposit canvas.
            let Some((lx0, ly0, lx1, ly1)) = pickup else {
                return;
            };
            let ly = ly as i32;
            if ly < ly0 || ly > ly1 {
                return;
            }
            let cy = ay + (ly - half);
            if cy < 2 || cy > h - 1 {
                return;
            }
            for lx in lx0..=lx1 {
                let cx = ax + (lx - half);
                if cx < 2 || cx > w - 1 {
                    continue;
                }
                let i = cx as usize + cy as usize * s;
                // A tinta líquida re-molha a SECA (o reservatório: as cinco tintas do Rebelle misturam-se 15–38 px
                // antes da faixa seca); os outros pincéis só apanham a molhada (`pickup_assentado` do produto 0).
                let assentado = if reservatorio > 0.0 {
                    1.0
                } else {
                    p.pickup_assentado
                };
                let w_s = alpha_of_mass(g.sett[i] as f64) * assentado;
                let w_f = alpha_of_mass(g.susp[i] as f64);
                let fe = w_s + w_f;
                if fe <= 0.0 {
                    continue; // blank paper keeps the tip color
                }
                let inv = 1.0 / fe;
                let sc = g.sett_rgb[i];
                let uc = g.susp_rgb[i];
                let cr = (sc[0] as f64 * w_s + uc[0] as f64 * w_f) * inv;
                let cg = (sc[1] as f64 * w_s + uc[1] as f64 * w_f) * inv;
                let cb = (sc[2] as f64 * w_s + uc[2] as f64 * w_f) * inv;
                // Com o reservatório, `r` é a fracção da tinta debaixo da ponta que entra nele a cada DAB (a janela
                // junta `dabs` dabs: o passo não muda a tinta; o `Pickup` do painel acaba em 0,2 por janela — fraco
                // demais para o borrão das tintas do Rebelle).
                let k = if reservatorio > 0.0 {
                    (1.0 - reservatorio * fe.min(1.0)).powi(dabs)
                } else {
                    tip_retain + (1.0 - tip_retain) * (1.0 - fe.min(1.0))
                };
                let l = lx as usize;
                // O reservatório de tinta líquida (`Knob::TipReservoir`): o pigmento apanhado de OUTRA tinta ABSORVE
                // na da ponta (o produto, Beer–Lambert) — o azul sobre o vermelho escurece, nunca vira roxo; a mesma
                // tinta ([`crate::colorops::outra_tinta`] 0) não se escurece a si mesma.
                let (cr, cg, cb) = if reservatorio > 0.0 {
                    let ponta = [tr[l] as f64, tg[l] as f64, tb[l] as f64];
                    let d = crate::colorops::outra_tinta(ponta, [cr, cg, cb]);
                    let prod = |t: f64, c: f64| t + (t * c / 255.0 - t) * d;
                    (prod(ponta[0], cr), prod(ponta[1], cg), prod(ponta[2], cb))
                } else {
                    (cr, cg, cb)
                };
                tr[l] = (tr[l] as f64 * k + cr * (1.0 - k)) as f32;
                tg[l] = (tg[l] as f64 * k + cg * (1.0 - k)) as f32;
                tb[l] = (tb[l] as f64 * k + cb * (1.0 - k)) as f32;
                if let Some(w) = limpo.as_deref_mut() {
                    w[l] = (w[l] as f64 * k) as f32;
                }
            }
        };
        if self.limpo.len() == self.tip_r.len() {
            crate::par::walk_rows4(
                mode,
                [
                    &mut self.tip_r,
                    &mut self.tip_g,
                    &mut self.tip_b,
                    &mut self.limpo,
                ],
                size,
                |ly, [tr, tg, tb, w]| linha(ly, tr, tg, tb, Some(w)),
            );
        } else {
            crate::par::walk_rows3(
                mode,
                &mut self.tip_r,
                &mut self.tip_g,
                &mut self.tip_b,
                size,
                |ly, tr, tg, tb| linha(ly, tr, tg, tb, None),
            );
        }
        if reservatorio > 0.0 && pickup.is_some() {
            self.mistura_o_reservatorio();
        }
    }

    /// **O reservatório** ([`Knob::TipReservoir`]): cada texel da pegada (onde esta janela pousa pigmento) passa a
    /// média da pegada — a tinta líquida não guarda o que cada ponto do bico tocou. Em série: a média é uma soma em
    /// `f64` (a ordem fixa os bits).
    fn mistura_o_reservatorio(&mut self) {
        let mut soma = [0.0f64; 4];
        let mut n = 0u32;
        let com_limpo = self.limpo.len() == self.tip_r.len();
        for ly in self.ly0..=self.ly1 {
            for lx in self.lx0..=self.lx1 {
                let l = (lx + ly * self.size) as usize;
                if self.pig[l] > 0.0 {
                    soma[0] += self.tip_r[l] as f64;
                    soma[1] += self.tip_g[l] as f64;
                    soma[2] += self.tip_b[l] as f64;
                    if com_limpo {
                        soma[3] += self.limpo[l] as f64;
                    }
                    n += 1;
                }
            }
        }
        if n == 0 {
            return;
        }
        let m = soma.map(|v| v / n as f64);
        let puxa = |v: &mut f32, alvo: f64| *v = alvo as f32;
        for ly in self.ly0..=self.ly1 {
            for lx in self.lx0..=self.lx1 {
                let l = (lx + ly * self.size) as usize;
                if self.pig[l] > 0.0 {
                    puxa(&mut self.tip_r[l], m[0]);
                    puxa(&mut self.tip_g[l], m[1]);
                    puxa(&mut self.tip_b[l], m[2]);
                    if com_limpo {
                        puxa(&mut self.limpo[l], m[3]);
                    }
                }
            }
        }
    }
}

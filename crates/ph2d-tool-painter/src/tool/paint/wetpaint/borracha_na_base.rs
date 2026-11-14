//! **A Borracha no Wet Paint apaga tudo o que está sob o círculo** (Bug #70, decisão do dono 05/11): além do fluido, a
//! tinta antiga, que mora na [`WetSession::base`] congelada. A cobertura sai da borracha normal, medida num escudo opaco
//! como a do Composite ([`Self::aplica_borracha_do_traco`]); o undo devolve a base pela chave do passo (`undo_vivo`).

use super::borracha_morta::{Caixa, Sulco, caixa_de_escrita, escreve_o_sulco, parede_dos_dabs};
use super::relevo_da_borracha::{alfa_na_caixa, razao_do_alfa};
use super::*;
use crate::tool::paint::region::dabs_bounds;

impl PainterTool {
    /// A cobertura da Borracha normal sobre `r`, num escudo opaco (alfa 255 − cobertura) que o chamador devolve à
    /// `pilha`. Plano de medição, fora do journal do undo; sem sessão os dabs caem na Borracha normal.
    pub(super) fn mede_o_escudo(
        &mut self,
        dabs: &[Dab],
        r: crate::compositor::Region,
    ) -> Arc<Vec<u8>> {
        let (w, h) = self.source_size;
        let stride = w as usize * 4;
        let len = stride * h as usize;
        if self.paint.pilha.escudo.len() != len {
            self.paint.pilha.escudo = Arc::new(vec![255u8; len]);
        }
        {
            let e = Arc::make_mut(&mut self.paint.pilha.escudo);
            for row in 0..r.h as usize {
                let o = (r.y as usize + row) * stride + r.x as usize * 4;
                e[o..o + r.w as usize * 4].fill(255);
            }
        }
        let mut plano = std::mem::take(&mut self.paint.pilha.escudo);
        let rng = self.paint.tex_rng;
        crate::tool::paint::plane_fork::swap_canvas_plane(
            &mut self.canvas_rgba,
            &mut plano,
            &self.undo.write_state,
        );
        self.paint.wetpaint.medindo_o_escudo = true;
        self.stamp_dabs_inner(dabs);
        self.paint.wetpaint.medindo_o_escudo = false;
        crate::tool::paint::plane_fork::swap_canvas_plane(
            &mut self.canvas_rgba,
            &mut plano,
            &self.undo.write_state,
        );
        self.paint.tex_rng = rng;
        plano
    }

    /// A Borracha sobre a `base` segue a lei gradual da tinta morta? As miniaturas e as macros ficam com a de sempre.
    fn base_na_lei_gradual(&self, sess: &WetSession) -> bool {
        let w = &self.paint.wetpaint;
        #[cfg(test)]
        if super::borracha_defeitos_tests::corte_seco::get() {
            return false;
        }
        self.paint.brush.stroke_method.is_incremental()
            && !w.na_miniatura
            && !w.deposit_pass
            && w.fita.is_none()
            && w.ritmo.is_none()
            && sess.fita.is_none()
            && sess.ritmo.is_none()
            && !self.undo.marca.ativa()
    }

    /// Apaga da base o que os `dabs` cobrem, sob a selecção e a protecção. O alfa preso não deixa apagar. A base só se
    /// copia quando há tinta a apagar; cada passo copia-a uma vez, por o undo segurar a de antes.
    ///
    /// Sob tinta da grade a base sai logo: quem aparece é a tinta, que sai aos poucos. Sem ela, a base é o que se vê e
    /// segue a [lei gradual aprovada](super::borracha_morta) (dono 06/11). Entre as duas, o [`PesoDaViva`] de cada píxel.
    pub(super) fn apaga_a_base_sob_os_dabs(&mut self, dabs: &[Dab]) {
        if self.paint.acumulando_no_plano {
            return;
        }
        let (w, h) = self.source_size;
        let stride = w as usize * 4;
        let len = stride * h as usize;
        let Some(r) = dabs_bounds(dabs, w, h, self.paint.brush.tip_reach()) else {
            return;
        };
        let (sel, prot, alock) = self.wet_splat_gates();
        if alock.is_some() || self.canvas_rgba.len() != len {
            return;
        }
        let Some(mut sess) = self.paint.wetpaint.session.take() else {
            return;
        };
        if sess.base.len() != len {
            self.paint.wetpaint.session = SessaoDoWet::aqui(sess);
            return;
        }
        let plano = self.mede_o_escudo(dabs, r);
        let linhas = || {
            (0..r.h as usize).flat_map(move |row| {
                let o = (r.y as usize + row) * stride + r.x as usize * 4;
                (0..r.w as usize).map(move |col| o + col * 4)
            })
        };
        let gate = |p: usize| {
            watercolor_accum::splat_keep(
                sel.as_deref().map(Vec::as_slice),
                prot.as_deref().map(Vec::as_slice),
                None,
                p,
            )
        };
        let keep = |i: usize| 1.0 - (1.0 - f32::from(plano[i + 3]) / 255.0) * gate(i / 4);
        let tam = (w as usize, h as usize);
        let caixa = Caixa {
            x0: r.x as usize,
            y0: r.y as usize,
            x1: (r.x + r.w) as usize,
            y1: (r.y + r.h) as usize,
        };
        let q = usize::from(sess.ratio);
        let (gw, gh) = sess.grid;
        let celula = |p: usize| (p / tam.0 / q).min(gh - 1) * gw + (p % tam.0 / q).min(gw - 1);
        let mut sulco = self.base_na_lei_gradual(&sess).then(|| {
            sess.bring_home();
            self.paint.wetpaint.sulco.take().unwrap_or_else(|| {
                Sulco::novo(
                    Arc::clone(&sess.base),
                    tam.0 * tam.1,
                    celulas_com_tinta(&sess),
                    sess.planos.pigment.clone(),
                )
            })
        });
        if let Some(s) = sulco.as_mut() {
            s.acumula(&plano, dabs, caixa, tam.0);
        }
        let mut relevo = None;
        if linhas().any(|i| sess.base[i + 3] > 0 && keep(i) < 1.0) {
            let forca = self.forca_do_sulco();
            let parede = parede_dos_dabs(dabs);
            let esc = if sulco.is_some() {
                caixa_de_escrita(caixa, parede, tam)
            } else {
                caixa
            };
            let antes = alfa_na_caixa(&sess.base, esc, tam.0);
            let base = Arc::make_mut(&mut sess.base);
            if sulco.is_none() {
                for i in linhas() {
                    let k = keep(i);
                    if k < 1.0 {
                        // Para baixo: com alfa 1 e k > 0,5 o arredondar nunca chegava a 0.
                        base[i + 3] = (f32::from(base[i + 3]) * k).floor() as u8;
                    }
                }
            }
            let (x0, y0, x1, y1) = (esc.x0, esc.y0, esc.x1, esc.y1);
            if let Some(s) = &sulco {
                let viva = PesoDaViva::novo(s, esc, q, sess.grid);
                escreve_o_sulco(s, caixa, tam, parede, forca, base, &|p| {
                    // Sem nada na base sob a tinta, não há o que escrever: o óleo vivo sozinho fica ao bit.
                    if s.com_tinta[celula(p)] && s.antes[p * 4 + 3] == 0 {
                        return (0.0, 0.0);
                    }
                    let (g, v) = (gate(p), viva.em(p, tam.0));
                    (g * (1.0 - v), g * v)
                });
            }
            relevo = razao_do_alfa(&antes, &sess.base, esc, tam.0).map(|r| (esc, r));
            let q = q as i32;
            sess.engine.merge_dirty_otica(
                x0 as i32 / q + 1,
                y0 as i32 / q + 1,
                x1 as i32 / q + 2,
                y1 as i32 / q + 2,
            );
        }
        self.paint.pilha.escudo = plano;
        self.paint.wetpaint.sulco = sulco;
        self.paint.wetpaint.session = SessaoDoWet::aqui(sess);
        if let Some((esc, razao)) = relevo {
            self.relevo_segue_o_alfa(esc, &razao);
        }
    }
}

/// Quanto a tinta viva cobre cada píxel da caixa, sem degrau: o que ela esconde sai de uma vez, o que ela deixa ver segue
/// a lei gradual.
struct PesoDaViva {
    caixa: Caixa,
    plano: Vec<f32>,
}

impl PesoDaViva {
    fn novo(s: &Sulco, caixa: Caixa, q: usize, (gw, gh): (usize, usize)) -> Self {
        let mut plano = Vec::with_capacity((caixa.x1 - caixa.x0) * (caixa.y1 - caixa.y0));
        let amostra = super::grid_map::SampleU::new(q as u8, gw, gh);
        for y in caixa.y0..caixa.y1 {
            let linha = amostra.row(y);
            for x in caixa.x0..caixa.x1 {
                #[cfg(test)]
                if super::borracha_fronteira_tests::com_degrau::get() {
                    let c = (y / q).min(gh - 1) * gw + (x / q).min(gw - 1);
                    plano.push(f32::from(u8::from(s.com_tinta[c])));
                    continue;
                }
                plano.push(if s.pigmento.len() == gw * gh * 4 {
                    linha.at(&s.pigmento, x)[3] as f32 / 255.0
                } else {
                    0.0
                });
            }
        }
        Self { caixa, plano }
    }

    fn em(&self, p: usize, w: usize) -> f32 {
        let (x, y) = (p % w, p / w);
        self.plano[(y - self.caixa.y0) * (self.caixa.x1 - self.caixa.x0) + x - self.caixa.x0]
    }
}

/// As células da grade activa com tinta agora, `gw × gh`.
fn celulas_com_tinta(sess: &WetSession) -> Vec<bool> {
    let g = sess.engine.active_grid();
    let (gw, gh) = sess.grid;
    let mut v = vec![false; gw * gh];
    for cy in 0..gh {
        for cx in 0..gw {
            let i = (cy + 1) * g.s + cx + 1;
            v[cy * gw + cx] = g.sett[i] > 0.0 || g.susp[i] > 0.0;
        }
    }
    v
}

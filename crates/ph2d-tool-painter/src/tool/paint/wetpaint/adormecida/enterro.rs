//! **O enterro** ([`super`]): o que outro meio pintou sobre a tinta viva enterra as células sob ele — a `base` delas
//! passa a ser a tela, a orla que perdeu cobertura fica na `base`, e a tela congela-se como a verdade das células que
//! a óptica não mudar. Partido de `adormecida.rs` por LOC (B52.R9).

use super::*;

#[cfg(test)]
#[path = "enterro_tests.rs"]
mod tests;

/// A falta de cobertura que conta na orla do enterro: menos do que isto é o arredondar da tela.
const ORLA_QUE_DESTAPA: f32 = 2.0 / 255.0;

#[cfg(test)]
thread_local! {
    /// Os píxeis que os laços do enterro visitaram (B02.A3): o gate `o_enterro_visita_so_a_caixa_tocada`.
    pub(super) static VISITAS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[inline]
fn visita(_n: usize) {
    #[cfg(test)]
    VISITAS.with(|v| v.set(v.get() + _n as u64));
}

/// A caixa (0-based, inclusiva) das células marcadas, ou `None`.
fn caixa_das(
    marcadas: impl Iterator<Item = bool>,
    gw: usize,
) -> Option<(usize, usize, usize, usize)> {
    let mut caixa: Option<(usize, usize, usize, usize)> = None;
    for (k, _) in marcadas.enumerate().filter(|(_, m)| *m) {
        let (x, y) = (k % gw, k / gw);
        caixa = Some(match caixa {
            None => (x, y, x, y),
            Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
        });
    }
    caixa
}

/// Os píxeis de uma caixa de células (0-based, inclusiva): a última linha/coluna da grade leva o resto da tela, como o
/// `min(g - 1)` da amostra.
fn pixeis_da_caixa(
    (cx0, cy0, cx1, cy1): (usize, usize, usize, usize),
    q: usize,
    (gw, gh): (usize, usize),
    (w, h): (usize, usize),
) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
    let fim = |c: usize, g: usize, n: usize| if c + 1 >= g { n } else { ((c + 1) * q).min(n) };
    (
        (cx0 * q).min(w)..fim(cx1, gw, w),
        (cy0 * q).min(h)..fim(cy1, gh, h),
    )
}

/// A tela no instante de um enterro, com os planos e a `base` de que ela saiu. ⛔ Medido: recompor uma caixa à volta de
/// células enterradas re-ilumina a orla delas, numa linha que acaba na borda da caixa (1 345 px com Δ ≥ 10); a orla só
/// muda onde a óptica mudou, sob o pincel.
pub(crate) struct Congelada {
    tela: Arc<Vec<u8>>,
    base: Arc<Vec<u8>>,
    pigment: Vec<u8>,
    sombra: Vec<u8>,
    /// As células enterradas: a `base` delas já tem a sombra assada, e uma mudança mínima não a põe outra vez por cima.
    enterradas: Vec<bool>,
    /// As células cuja óptica o enterro mudou: as enterradas e a orla delas. Só estas ficam: uma longe do enterro é a da
    /// tinta viva, ao bit.
    mudou: Vec<bool>,
    gw: usize,
    gh: usize,
    rr: usize,
}

impl Congelada {
    /// O píxel de quando enterrou, se nem a óptica da célula nem a `base` dele mudaram.
    #[inline]
    pub(in crate::tool::paint::wetpaint) fn pixel(
        &self,
        (px, py, o): (usize, usize, usize),
        pigment: &[u8],
        sombra: &[u8],
        base: &[u8],
    ) -> Option<&[u8]> {
        let c = (py / self.rr).min(self.gh - 1) * self.gw + (px / self.rr).min(self.gw - 1);
        (self.mudou[c]
            && pigment[c * 4..c * 4 + 4] == self.pigment[c * 4..c * 4 + 4]
            && (self.enterradas[c] || sombra.get(c) == self.sombra.get(c))
            && base[o..o + 4] == self.base[o..o + 4])
            .then(|| &self.tela[o..o + 4])
    }

    pub(in crate::tool::paint::wetpaint) fn bytes(&self) -> usize {
        self.tela.len() + self.pigment.len() + self.sombra.len()
    }

    /// Os bytes só dela, sem os `Arc` que ela partilha.
    pub(in crate::tool::paint::wetpaint) fn bytes_proprios(&self) -> usize {
        self.pigment.len() + self.sombra.len() + self.enterradas.len() + self.mudou.len()
    }

    pub(in crate::tool::paint::wetpaint) fn tela(&self) -> &Arc<Vec<u8>> {
        &self.tela
    }

    pub(in crate::tool::paint::wetpaint) fn base(&self) -> &Arc<Vec<u8>> {
        &self.base
    }
}

impl PainterTool {
    /// As células sob o que outro meio pintou acabam: a imagem de agora passa a ser a `base` delas. Devolve-as.
    pub(in crate::tool::paint::wetpaint) fn wet_enterra_o_que_mudou(
        &self,
        sess: &mut WetSession,
        velha: &[u8],
    ) -> Vec<bool> {
        self.wet_enterra_no_casco(sess, velha, None)
    }

    /// [`Self::wet_enterra_o_que_mudou`] só nas células do `casco` (1-based, inclusivo); `None` = a grade inteira.
    pub(in crate::tool::paint::wetpaint) fn wet_enterra_no_casco(
        &self,
        sess: &mut WetSession,
        velha: &[u8],
        casco: Option<(usize, usize, usize, usize)>,
    ) -> Vec<bool> {
        let nova = &self.canvas_rgba[..];
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let q = usize::from(sess.ratio);
        let (gw, gh) = sess.grid;
        let mut tocada = vec![false; gw * gh];
        // A caixa (0-based) das células tocadas sai da 1.ª passagem: os laços de píxel seguintes só andam nela (B02.A3).
        let (mut x0, mut y0, mut x1, mut y1) = (gw, gh, 0, 0);
        let (cx0, cy0, cx1, cy1) = casco.unwrap_or((1, 1, gw, gh));
        let (px0, px1) = (((cx0 - 1) * q).min(w), (cx1 * q).min(w));
        for y in ((cy0 - 1) * q).min(h)..(cy1 * q).min(h) {
            let o = y * w * 4;
            if velha[o + px0 * 4..o + px1 * 4] == nova[o + px0 * 4..o + px1 * 4] {
                continue;
            }
            for x in px0..px1 {
                let p = o + x * 4;
                if velha[p..p + 4] != nova[p..p + 4] {
                    let (gx, gy) = (x / q, y / q);
                    tocada[gy * gw + gx] = true;
                    (x0, y0, x1, y1) = (x0.min(gx), y0.min(gy), x1.max(gx), y1.max(gy));
                }
            }
        }
        if x0 <= x1 {
            let base = Arc::make_mut(&mut sess.base);
            let (xs, ys) = pixeis_da_caixa((x0, y0, x1, y1), q, (gw, gh), (w, h));
            for y in ys {
                let o = y * w * 4;
                visita(xs.len());
                for x in xs.clone() {
                    if tocada[(y / q) * gw + x / q] {
                        let p = o + x * 4;
                        base[p..p + 4].copy_from_slice(&nova[p..p + 4]);
                    }
                }
            }
        }
        sess.engine.enterra_celulas(&tocada, gw);
        // A caixa 1-based do resto: `x1 == 0` = nada tocou.
        let (x0, y0, x1, y1) = if x0 <= x1 {
            (x0 + 1, y0 + 1, x1 + 1, y1 + 1)
        } else {
            (gw, gh, 0, 0)
        };
        let (pigmento, sombra) = (sess.planos.pigment.clone(), sess.planos.sombra.clone());
        if x1 > 0 {
            self.wet_planos_em_dia(sess, (x0, y0, x1, y1));
        }
        let p = &sess.planos;
        let mudou: Vec<bool> = (0..gw * gh)
            .map(|c| {
                tocada[c]
                    || pigmento[c * 4..c * 4 + 4] != p.pigment[c * 4..c * 4 + 4]
                    || sombra.get(c).copied().unwrap_or(0) != p.sombra.get(c).copied().unwrap_or(0)
            })
            .collect();
        self.wet_orla_vai_para_a_base(sess, &mudou, &tocada);
        self.wet_congela(sess, Some((&tocada, &mudou)));
        tocada
    }

    /// Na orla do enterro a tinta perdeu cobertura que a tela ainda mostra: fica na `base`, ou a Borracha que refaz estes
    /// píxeis destapa o papel de repente.
    pub(in crate::tool::paint::wetpaint) fn wet_orla_vai_para_a_base(
        &self,
        sess: &mut WetSession,
        mudou: &[bool],
        tocada: &[bool],
    ) {
        #[cfg(test)]
        if super::borracha_fronteira_tests::com_degrau::get() {
            return;
        }
        let nova = &self.canvas_rgba[..];
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let q = usize::from(sess.ratio);
        let (gw, gh) = sess.grid;
        let amostra = super::grid_map::SampleU::new(sess.ratio, gw, gh);
        let (pigment, sombra) = (&sess.planos.pigment, &sess.planos.sombra);
        let base = &mut sess.base;
        // Só a caixa da orla (as células que mudaram sem ser tocadas): fora dela o laço só fazia `continue` (B02.A3).
        let Some(caixa) = caixa_das(mudou.iter().zip(tocada).map(|(m, t)| *m && !*t), gw) else {
            return;
        };
        let (xs, ys) = pixeis_da_caixa(caixa, q, (gw, gh), (w, h));
        for y in ys {
            let linha = amostra.row(y);
            let cy = (y / q).min(gh - 1);
            visita(xs.len());
            for x in xs.clone() {
                let c = cy * gw + (x / q).min(gw - 1);
                if !mudou[c] || tocada[c] {
                    continue;
                }
                let o = (y * w + x) * 4;
                let (pa, s) = (
                    linha.at(pigment, x)[3] as f32 / 255.0,
                    f32::from(sombra.get(c).copied().unwrap_or(0)) / 255.0,
                );
                let ba = f32::from(base[o + 3]) / 255.0;
                let ta = f32::from(nova[o + 3]) / 255.0;
                let deixa = (1.0 - pa) * (1.0 - s);
                if ta > 1.0 - deixa * (1.0 - ba) + ORLA_QUE_DESTAPA {
                    // A `base` que, sob esta tinta, dá a cobertura da tela: sem degrau de píxel a píxel.
                    let b = &mut Arc::make_mut(base)[o..o + 4];
                    b[..3].copy_from_slice(&nova[o..o + 3]);
                    b[3] = ((1.0 - (1.0 - ta) / deixa) * 255.0).round() as u8;
                }
            }
        }
    }

    /// A tela de agora é a verdade das células cuja óptica não mudar.
    pub(in crate::tool::paint::wetpaint) fn wet_congela(
        &self,
        sess: &mut WetSession,
        agora: Option<(&[bool], &[bool])>,
    ) {
        let n = sess.grid.0 * sess.grid.1;
        let (mut enterradas, mut mudou) = match sess.congelada.take() {
            Some(c) => (c.enterradas.clone(), c.mudou.clone()),
            None => (vec![false; n], vec![false; n]),
        };
        if let Some((t, m)) = agora {
            for (e, t) in enterradas.iter_mut().zip(t) {
                *e |= *t;
            }
            for (e, m) in mudou.iter_mut().zip(m) {
                *e |= *m;
            }
        }
        sess.congelada = Some(Arc::new(Congelada {
            mudou,
            enterradas,
            tela: Arc::clone(&self.canvas_rgba),
            base: Arc::clone(&sess.base),
            pigment: sess.planos.pigment.clone(),
            sombra: sess.planos.sombra.clone(),
            gw: sess.grid.0,
            gh: sess.grid.1,
            rr: usize::from(sess.ratio),
        }));
    }

    /// Sem recompor a tela, os planos da óptica têm de seguir a grade: o laço de píxel de uma caixa seguinte lê-os para lá
    /// das células que ela refaz, e um plano velho punha a tinta enterrada de volta.
    pub(in crate::tool::paint::wetpaint) fn wet_planos_em_dia(
        &self,
        sess: &mut WetSession,
        caixa: (usize, usize, usize, usize),
    ) {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if sess.base.len() != w * h * 4 {
            return;
        }
        let factos = self.factos_da_otica();
        sess.bring_home();
        sess.planos.campo_cache = None; // a altura guardada é a de antes do enterro
        let entrada = super::composite_otica::EntradaDaOtica {
            factos,
            marca: fita::MarcaDaAgua::de(&sess.engine),
            trabalho: Some(super::composite_otica::TrabalhoDaOtica {
                caixa,
                params: sess.engine.sim.gather_params(&sess.engine.tuning),
            }),
            ratio: sess.ratio,
            grade: sess.grid,
            tela: (w, h),
            revisao: sess.engine.revisao(),
        };
        let layers: Vec<ph2d_wet_paint::render::RenderLayer<'_>> = sess
            .engine
            .layers
            .iter()
            .map(|l| ph2d_wet_paint::render::RenderLayer {
                grid: &l.grid,
                opacity: l.opacity,
                visible: l.visible,
            })
            .collect();
        super::composite_otica::compoe_a_otica(&entrada, &layers, &mut sess.planos);
    }
}

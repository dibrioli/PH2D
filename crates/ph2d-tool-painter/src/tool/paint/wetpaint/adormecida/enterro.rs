//! **O enterro** ([`super`]): o que outro meio pintou sobre a tinta viva enterra as células sob ele — a `base` delas
//! passa a ser a tela, a orla que perdeu cobertura fica na `base`, e a tela congela-se como a verdade das células que
//! a óptica não mudar. Partido de `adormecida.rs` por LOC (B52.R9). Cada laço anda só a caixa do que mudou (B02.A3).

use super::*;
use rayon::prelude::*;

#[cfg(test)]
#[path = "enterro_tests.rs"]
mod tests;

/// A falta de cobertura que conta na orla do enterro: menos do que isto é o arredondar da tela.
const ORLA_QUE_DESTAPA: f32 = 2.0 / 255.0;

#[cfg(test)]
thread_local! {
    /// Os píxeis e as células que os laços do enterro visitaram (B02.A3): o gate `o_enterro_visita_so_a_caixa_tocada`.
    pub(super) static VISITAS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[inline]
fn visita(_n: usize) {
    #[cfg(test)]
    VISITAS.with(|v| v.set(v.get() + _n as u64));
}

/// Uma caixa de células `(x0, y0, x1, y1)`, inclusiva.
type Caixa = (usize, usize, usize, usize);

/// A caixa `c` com a célula `(x, y)`.
#[inline]
fn junta(c: Option<Caixa>, (x, y): (usize, usize)) -> Option<Caixa> {
    Some(match c {
        None => (x, y, x, y),
        Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
    })
}

fn une(a: Option<Caixa>, b: Option<Caixa>) -> Option<Caixa> {
    match (a, b) {
        (Some(a), Some(b)) => Some((a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))),
        (a, b) => a.or(b),
    }
}

/// Os píxeis de uma caixa de células (0-based, inclusiva): a última linha/coluna da grade leva o resto da tela, como o
/// `min(g - 1)` da amostra.
fn pixeis_da_caixa(
    (cx0, cy0, cx1, cy1): Caixa,
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

/// Os planos da óptica de antes de ela correr no enterro, onde ela os reescreve.
enum Antes {
    /// A óptica não correu.
    Nada,
    /// A caixa (1-based) do sujo crescida pelo alcance do Óleo Real, com o pigmento e a sombra dela linha a linha (a
    /// sombra vazia = toda a zero).
    Caixa {
        caixa: Caixa,
        pigment: Vec<u8>,
        sombra: Vec<u8>,
    },
    /// Sem Óleo Real a sombra sai da grade inteira: os planos todos.
    Tudo { pigment: Vec<u8>, sombra: Vec<u8> },
}

/// Os planos de antes onde a óptica de `e` os reescreve: a caixa de `composite_otica::compoe_a_caixa` (o
/// `debug_assert` de [`PainterTool::wet_compoe_os_planos`] confere-a contra a que ela devolve).
fn antes_da_otica(
    e: &super::composite_otica::EntradaDaOtica,
    layers: &[ph2d_wet_paint::render::RenderLayer<'_>],
    p: &super::composite_otica::PlanosDaOtica,
) -> Antes {
    let Some(t) = &e.trabalho else {
        return Antes::Nada;
    };
    let (gw, gh) = e.grade;
    let oleo = e.factos.oleo_da_tela(layers);
    if oleo.is_none() && !p.sombra.is_empty() {
        visita(gw * gh);
        return Antes::Tudo {
            pigment: p.pigment.clone(),
            sombra: p.sombra.clone(),
        };
    }
    let (cx0, cy0, cx1, cy1) = t.caixa;
    let (cx0, cy0, cx1, cy1) = match &oleo {
        Some(o) => {
            let grades: Vec<_> = layers.iter().map(|l| (l.grid, l.visible)).collect();
            let a =
                ph2d_wet_paint::oleo_real::alcance_na(o, &grades, usize::from(e.ratio), t.caixa);
            (
                cx0.saturating_sub(a).max(1),
                cy0.saturating_sub(a).max(1),
                (cx1 + a).min(gw),
                (cy1 + a).min(gh),
            )
        }
        None => (cx0, cy0, cx1, cy1),
    };
    let (lw, lh) = (cx1 + 1 - cx0, cy1 + 1 - cy0);
    let mut pigment = Vec::with_capacity(lw * lh * 4);
    let mut sombra = Vec::with_capacity(if p.sombra.is_empty() { 0 } else { lw * lh });
    for y in cy0 - 1..cy1 {
        visita(lw);
        let o = y * gw;
        pigment.extend_from_slice(&p.pigment[(o + cx0 - 1) * 4..(o + cx1) * 4]);
        if !p.sombra.is_empty() {
            sombra.extend_from_slice(&p.sombra[o + cx0 - 1..o + cx1]);
        }
    }
    Antes::Caixa {
        caixa: (cx0, cy0, cx1, cy1),
        pigment,
        sombra,
    }
}

/// As células cuja óptica o enterro mudou (as tocadas e as que a óptica reescreveu diferentes), com a caixa (0-based)
/// de todas e a da orla (as que mudaram sem ser tocadas). Só anda as caixas do toque e da óptica.
fn mudou_do_enterro(
    tocada: &[bool],
    toque: Option<Caixa>,
    antes: &Antes,
    p: &super::composite_otica::PlanosDaOtica,
    (gw, gh): (usize, usize),
) -> (Vec<bool>, Option<Caixa>, Option<Caixa>) {
    let mut mudou = vec![false; gw * gh];
    let (mut caixa, mut orla) = (None, None);
    if let Some((x0, y0, x1, y1)) = toque {
        for y in y0..=y1 {
            visita(x1 + 1 - x0);
            for x in x0..=x1 {
                if tocada[y * gw + x] {
                    mudou[y * gw + x] = true;
                    caixa = junta(caixa, (x, y));
                }
            }
        }
    }
    let difere = |pig: &[u8], s: u8, k: usize| {
        pig != &p.pigment[k * 4..k * 4 + 4] || s != p.sombra.get(k).copied().unwrap_or(0)
    };
    match antes {
        Antes::Nada => {}
        Antes::Caixa {
            caixa: (cx0, cy0, cx1, cy1),
            pigment,
            sombra,
        } => {
            let lw = cx1 + 1 - cx0;
            for (j, y) in (cy0 - 1..*cy1).enumerate() {
                visita(lw);
                for (i, x) in (cx0 - 1..*cx1).enumerate() {
                    let (k, l) = (y * gw + x, j * lw + i);
                    if !tocada[k]
                        && difere(
                            &pigment[l * 4..l * 4 + 4],
                            sombra.get(l).copied().unwrap_or(0),
                            k,
                        )
                    {
                        mudou[k] = true;
                        orla = junta(orla, (x, y));
                    }
                }
            }
        }
        Antes::Tudo { pigment, sombra } => {
            visita(gw * gh);
            for k in (0..gw * gh).filter(|&k| !tocada[k]) {
                if difere(
                    &pigment[k * 4..k * 4 + 4],
                    sombra.get(k).copied().unwrap_or(0),
                    k,
                ) {
                    mudou[k] = true;
                    orla = junta(orla, (k % gw, k / gw));
                }
            }
        }
    }
    (mudou, une(caixa, orla), orla)
}

/// A tela no instante de um enterro, com os planos e a `base` de que ela saiu. ⛔ Medido: recompor uma caixa à volta de
/// células enterradas re-ilumina a orla delas, numa linha que acaba na borda da caixa (1 345 px com Δ ≥ 10); a orla só
/// muda onde a óptica mudou, sob o pincel.
#[derive(Clone)]
pub(crate) struct Congelada {
    tela: Arc<Vec<u8>>,
    base: Arc<Vec<u8>>,
    /// O pigmento e a sombra de quando congelou, em dia só na `caixa`: fora dela são inertes (nenhuma célula `mudou`).
    pigment: Vec<u8>,
    sombra: Vec<u8>,
    /// As células enterradas: a `base` delas já tem a sombra assada, e uma mudança mínima não a põe outra vez por cima.
    enterradas: Vec<bool>,
    /// As células cuja óptica o enterro mudou: as enterradas e a orla delas. Só estas ficam: uma longe do enterro é a da
    /// tinta viva, ao bit.
    mudou: Vec<bool>,
    /// A caixa (0-based) das células `mudou` (B02.A3).
    caixa: Option<Caixa>,
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
    #[cfg(test)]
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
        casco: Option<Caixa>,
    ) -> Vec<bool> {
        self.wet_enterra_no_casco_com_toque(sess, velha, casco).0
    }

    /// [`Self::wet_enterra_no_casco`] com a caixa (0-based, inclusiva) das células tocadas.
    pub(in crate::tool::paint::wetpaint) fn wet_enterra_no_casco_com_toque(
        &self,
        sess: &mut WetSession,
        velha: &[u8],
        casco: Option<Caixa>,
    ) -> (Vec<bool>, Option<Caixa>) {
        let nova = &self.canvas_rgba[..];
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let q = usize::from(sess.ratio);
        let (gw, gh) = sess.grid;
        let mut tocada = vec![false; gw * gh];
        // A caixa (0-based) das células tocadas sai da 1.ª passagem: os laços seguintes só andam nela.
        let mut toque = None;
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
                    toque = junta(toque, (gx, gy));
                }
            }
        }
        if let Some(t) = toque {
            // Linhas independentes, cada uma escreve só a sua: o mesmo bit em qualquer nº de fios.
            let (xs, ys) = pixeis_da_caixa(t, q, (gw, gh), (w, h));
            visita(xs.len() * ys.len());
            let (passo, tocada) = (w * 4, &tocada);
            Arc::make_mut(&mut sess.base)[ys.start * passo..ys.end * passo]
                .par_chunks_mut(passo)
                .zip(nova[ys.start * passo..ys.end * passo].par_chunks(passo))
                .enumerate()
                .for_each(|(k, (linha, de))| {
                    let cy = (ys.start + k) / q;
                    for x in xs.clone() {
                        if tocada[cy * gw + x / q] {
                            linha[x * 4..x * 4 + 4].copy_from_slice(&de[x * 4..x * 4 + 4]);
                        }
                    }
                });
        }
        sess.engine.enterra_celulas(&tocada, gw);
        let antes = match toque {
            Some((x0, y0, x1, y1)) => {
                self.wet_compoe_os_planos(sess, (x0 + 1, y0 + 1, x1 + 1, y1 + 1), true)
            }
            None => Antes::Nada,
        };
        let (mudou, caixa, orla) = mudou_do_enterro(&tocada, toque, &antes, &sess.planos, (gw, gh));
        self.wet_orla_vai_para_a_base(sess, &mudou, &tocada, orla);
        self.wet_congela(sess, caixa.map(|c| (&tocada[..], &mudou[..], c)));
        (tocada, toque)
    }

    /// Na orla do enterro a tinta perdeu cobertura que a tela ainda mostra: fica na `base`, ou a Borracha que refaz estes
    /// píxeis destapa o papel de repente. `orla` = a caixa (0-based) das células que mudaram sem ser tocadas.
    pub(in crate::tool::paint::wetpaint) fn wet_orla_vai_para_a_base(
        &self,
        sess: &mut WetSession,
        mudou: &[bool],
        tocada: &[bool],
        orla: Option<Caixa>,
    ) {
        #[cfg(test)]
        if super::borracha_fronteira_tests::com_degrau::get() {
            return;
        }
        let Some(caixa) = orla else {
            return;
        };
        let nova = &self.canvas_rgba[..];
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let q = usize::from(sess.ratio);
        let (gw, gh) = sess.grid;
        let amostra = super::grid_map::SampleU::new(sess.ratio, gw, gh);
        let (pigment, sombra) = (&sess.planos.pigment, &sess.planos.sombra);
        let base = &mut sess.base;
        // Em série: o `make_mut` só copia a `base` partilhada (congelada, passos) quando um píxel a muda.
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

    /// A tela de agora é a verdade das células cuja óptica não mudar. `agora` = as tocadas e as que mudaram neste
    /// enterro, com a caixa (0-based) de todas; os planos guardados põem-se em dia só na caixa de todas as `mudou`.
    pub(in crate::tool::paint::wetpaint) fn wet_congela(
        &self,
        sess: &mut WetSession,
        agora: Option<(&[bool], &[bool], Caixa)>,
    ) {
        let (gw, gh) = sess.grid;
        let (n, rr) = (gw * gh, usize::from(sess.ratio));
        // Sem passo que a partilhe, a congelada de antes é reusada sem cópia.
        let mut c = match sess.congelada.take().map(Arc::unwrap_or_clone) {
            Some(c) if (c.gw, c.gh, c.rr) == (gw, gh, rr) => c,
            _ => Congelada {
                tela: Arc::clone(&self.canvas_rgba),
                base: Arc::clone(&sess.base),
                pigment: Vec::new(),
                sombra: Vec::new(),
                enterradas: vec![false; n],
                mudou: vec![false; n],
                caixa: None,
                gw,
                gh,
                rr,
            },
        };
        if let Some((t, m, (x0, y0, x1, y1))) = agora {
            for y in y0..=y1 {
                visita(x1 + 1 - x0);
                for k in y * gw + x0..=y * gw + x1 {
                    c.enterradas[k] |= t[k];
                    c.mudou[k] |= m[k];
                }
            }
            c.caixa = une(c.caixa, Some((x0, y0, x1, y1)));
        }
        let p = &sess.planos;
        if p.pigment.len() != n * 4 {
            c.pigment = p.pigment.clone();
        } else if c.pigment.len() != n * 4 {
            c.pigment = vec![0; n * 4];
        }
        if c.sombra.len() != p.sombra.len() {
            c.sombra = vec![0; p.sombra.len()];
        }
        if let (Some((x0, y0, x1, y1)), true) = (c.caixa, p.pigment.len() == n * 4) {
            for y in y0..=y1 {
                visita(x1 + 1 - x0);
                let (a, b) = (y * gw + x0, y * gw + x1 + 1);
                c.pigment[a * 4..b * 4].copy_from_slice(&p.pigment[a * 4..b * 4]);
                if p.sombra.len() == n {
                    c.sombra[a..b].copy_from_slice(&p.sombra[a..b]);
                }
            }
        }
        c.tela = Arc::clone(&self.canvas_rgba);
        c.base = Arc::clone(&sess.base);
        sess.congelada = Some(Arc::new(c));
    }

    /// Sem recompor a tela, os planos da óptica têm de seguir a grade: o laço de píxel de uma caixa seguinte lê-os para lá
    /// das células que ela refaz, e um plano velho punha a tinta enterrada de volta.
    pub(in crate::tool::paint::wetpaint) fn wet_planos_em_dia(
        &self,
        sess: &mut WetSession,
        caixa: Caixa,
    ) {
        self.wet_compoe_os_planos(sess, caixa, false);
    }

    /// [`Self::wet_planos_em_dia`]; `guarda` = devolve os planos de antes onde a óptica os reescreve.
    fn wet_compoe_os_planos(&self, sess: &mut WetSession, caixa: Caixa, guarda: bool) -> Antes {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if sess.base.len() != w * h * 4 {
            return Antes::Nada;
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
        let antes = if guarda {
            antes_da_otica(&entrada, &layers, &sess.planos)
        } else {
            Antes::Nada
        };
        let _saida = super::composite_otica::compoe_a_otica(&entrada, &layers, &mut sess.planos);
        #[cfg(debug_assertions)]
        if let (Antes::Caixa { caixa: g, .. }, Some(s)) = (&antes, &_saida.caixas) {
            let c = s.caixa_cel;
            debug_assert!(
                c.0 >= g.0 && c.1 >= g.1 && c.2 <= g.2 && c.3 <= g.3,
                "a óptica reescreveu {c:?} fora da caixa guardada {g:?}"
            );
        }
        antes
    }
}

//! **O tique e o compositor da óptica** (J13b P6, [plano](../../../../../../docs/Painter/Arquitetura/J13b_plano_otica_fora.md)
//! §2 passo 1 e «Validade»; filho de [`super`] por LOC). Com o compositor de pé, o tique PEDE a imagem da água e
//! APLICA a que chega, sem trazer o motor para casa:
//!
//! - um pedido no ar de cada vez; o seguinte sai no tique em que o anterior é respondido;
//! - `uniao_local` é a união das caixas compostas EM CASA desde o último pedido. Uma imagem que a toca pode ser
//!   mais velha do que a tela já mostra ali: descarta-se e a caixa vai para a `divida`, que entra no pedido seguinte;
//! - a pausa da autoria toca a tela inteira: nenhuma imagem se aplica por cima do esboço.

use super::composite_otica::{CaixasDaOtica, SaidaDaOtica};
use super::offthread::otica::pixel::EntradaDoPixel;
use super::offthread::otica::{CaixaCel, FormaDaTela, Imagem, PedidoDeImagem, RespostaDaOtica};
use super::veu::{FilmeEMolha, VeuDe};
use super::*;

/// O estado do tique perante o compositor (vive na [`WetSession`]).
#[derive(Clone, Default)]
pub(super) struct OticaDoTique {
    /// A época do pedido por responder.
    pub(super) no_ar: Option<u64>,
    epoca: u64,
    pub(super) uniao_local: Option<CaixaCel>,
    /// As caixas que a tela deve, em pedaços que não se tocam (B14.1: a última envolvente).
    pub(super) divida: Vec<CaixaCel>,
    /// A água do passo herdada no pen-down (B14.1): volta ao worker como sujo do PASSO, que o espelho reduz ao que
    /// mudou; pela `divida` era a caixa do fluido inteira.
    pub(super) agua: Option<CaixaCel>,
    #[cfg(test)]
    pub(super) aplicadas: u32,
    /// Imagens cuja caixa tocou a `uniao_local`.
    #[cfg(test)]
    pub(super) tocadas: u32,
    /// Faixas cujos RGBA do compositor a tela copiou (B11.3).
    #[cfg(test)]
    pub(super) rgba_usadas: u32,
}

/// Junta `c` aos pedaços: os que ela toca fundem-se com ela (até nenhum tocar), os outros ficam à parte (B14.1).
pub(super) fn junta(v: &mut Vec<CaixaCel>, mut c: CaixaCel) {
    while let Some(k) = v.iter().position(|&p| toca(p, c)) {
        c = une(Some(v.swap_remove(k)), c);
    }
    v.push(c);
}

pub(super) fn une(a: Option<CaixaCel>, b: CaixaCel) -> CaixaCel {
    a.map_or(b, |a| {
        (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))
    })
}

fn toca(a: CaixaCel, b: CaixaCel) -> bool {
    a.0 <= b.2 && b.0 <= a.2 && a.1 <= b.3 && b.1 <= a.3
}

impl OticaDoTique {
    /// Uma composição em casa escreveu esta caixa.
    pub(super) fn compos_local(&mut self, c: CaixaCel) {
        self.uniao_local = Some(une(self.uniao_local, c));
    }

    /// A pausa da autoria: a tela inteira conta como composta em casa.
    pub(super) fn toca_tudo(&mut self, (gw, gh): (usize, usize)) {
        self.compos_local((1, 1, gw.max(1), gh.max(1)));
    }

    /// Há sujo tirado do motor que a tela ainda não mostra (uma imagem no ar ou a dívida).
    pub(super) fn pendente(&self) -> bool {
        self.no_ar.is_some() || !self.divida.is_empty() || self.agua.is_some()
    }

    /// O compositor morreu: o que estava no ar e por servir já voltou ao motor (B11.A1).
    pub(super) fn esquece_o_que_estava_no_ar(&mut self) {
        self.no_ar = None;
        self.uniao_local = None;
        self.divida.clear();
        self.agua = None;
    }

    fn deve(&mut self, c: CaixaCel) {
        junta(&mut self.divida, c);
    }
}

impl WetSession {
    /// J13b P8 ([doc](../../../../../../docs/Painter/Arquitetura/J13b_P8_pen_down.md)): o sujo que o motor traz do
    /// worker é da ÁGUA (a tela de casa não tem traço por compor) e vai pela dívida, para a composição de casa do lote
    /// levar só o que o traço muda. O sujo é uma caixa só: sem isto o traço longe da poça compunha a UNIÃO das duas.
    /// A porta alheia vem em pedaços e fica em pedaços (B14.1).
    pub(super) fn a_agua_herdada_vai_pela_divida(&mut self) {
        let (passo, alheio) = self.engine.take_dirty_partido();
        for d in alheio.em_vec() {
            if let Some(c) = offthread::otica::caixa_da_grade(d, self.grid) {
                self.otica.deve(c);
            }
        }
        if let Some(c) = offthread::otica::caixa_da_grade(passo, self.grid) {
            self.otica.agua = Some(une(self.otica.agua, c));
        }
    }

    /// O sujo que saiu do motor e a tela ainda não mostra (a dívida e a imagem no ar) volta ao motor. Motor em casa: a
    /// composição seguinte leva só o que falta, nunca a tela inteira.
    pub(super) fn o_sujo_por_mostrar_volta_ao_motor(&mut self) {
        let (pedido, agua) = self
            .otica
            .no_ar
            .map_or((Vec::new(), None), |e| self.sujo_do_pedido(e));
        let i = |v: usize| i32::try_from(v).unwrap_or(i32::MAX);
        for &(x0, y0, x1, y1) in pedido.iter().chain(&self.otica.divida) {
            self.engine.merge_dirty_otica(i(x0), i(y0), i(x1), i(y1));
        }
        for (x0, y0, x1, y1) in [agua, self.otica.agua].into_iter().flatten() {
            self.engine
                .devolve_sujo_do_passo(i(x0), i(y0), i(x1), i(y1));
        }
    }
}

impl PainterTool {
    /// O gesto está vivo ou o traço do motor aberto (pinceladas sobrepostas, W16) — o `paint.stroke` sai a meio do lote.
    pub(super) fn wet_pincel_na_tela(&self) -> bool {
        let w = &self.paint.wetpaint;
        w.live_gesture || w.session.as_ref().is_some_and(|s| s.stroke_open)
    }

    /// O tique mostra a água pelo compositor? Ligado e de pé; desligado a meio, acaba o que está no ar.
    pub(super) fn wet_otica_fora(&self) -> bool {
        self.paint.wetpaint.session.as_ref().is_some_and(|s| {
            s.tem_compositor() && (offthread::otica::ligada() || s.otica.pendente())
        })
    }

    /// Recolhe as respostas do compositor, sem esperar, e aplica a que ainda vale.
    pub(super) fn wet_recebe_imagens(&mut self) {
        loop {
            let Some(sess) = self.paint.wetpaint.session.as_mut() else {
                return;
            };
            let Some(r) = sess.try_recv_imagem() else {
                return;
            };
            match r {
                RespostaDaOtica::Nada {
                    epoca,
                    passos,
                    marca,
                } => {
                    if sess.otica.no_ar == Some(epoca) {
                        sess.otica.no_ar = None;
                        sess.seen_steps = passos;
                        self.paint.wetpaint.composta = Some(marca);
                    }
                }
                RespostaDaOtica::Imagem(im) => {
                    if sess.otica.no_ar != Some(im.epoca) {
                        for c in &im.caixas {
                            sess.otica.deve(c.sujo);
                        }
                        continue;
                    }
                    sess.otica.no_ar = None;
                    self.wet_aplica_imagem(*im);
                }
            }
        }
    }

    fn wet_aplica_imagem(&mut self, mut im: Imagem) {
        let show_wet = self.paint.wetpaint.show_wet;
        // B11.3: os RGBA do compositor valem se a entrada do píxel é a mesma de quando se pediu.
        let agora = self.wet_entrada_do_pixel(show_wet);
        let valido = matches!((&im.pixel, &agora), (Some(a), Some(b)) if a.mesma(b));
        drop(agora);
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        // A recusa é POR FAIXA (B14.1): a que a casa tocou vai à dívida, as outras aplicam-se (as faixas não partilham
        // célula nem píxel). A dívida é o sujo da faixa: a composição seguinte cresce-o pelo alcance.
        let (uniao_local, compostas) = (sess.otica.uniao_local, im.caixas.len());
        let (otica, planos) = (&mut sess.otica, &sess.planos);
        im.caixas.retain(|c| {
            // Pela caixa dos degraus, a maior que `Imagem::aplica` cola (`plano_do_suave`: contém a do pigmento).
            debug_assert!(toca(c.caixa_cel, c.caixa_suave) && c.caixa_suave.0 <= c.caixa_cel.0);
            let tocou = uniao_local.is_some_and(|u| toca(u, c.caixa_suave));
            #[cfg(test)]
            {
                otica.tocadas += u32::from(tocou);
            }
            // A casa pintou depois do pedido e tem um plano que a imagem diz vazio (o 1.º óleo, o 1.º Smooth Edges):
            // a imagem é velha, e aplicá-la apagava esse plano fora da caixa dela.
            let velha = uniao_local.is_some() && c.esvazia(planos);
            let recusa = tocou || velha || (show_wet && c.veu.is_none());
            if recusa {
                otica.deve(c.sujo);
            }
            !recusa
        });
        if compostas > 0 && im.caixas.is_empty() {
            return;
        }
        im.aplica(&mut sess.planos, sess.grid);
        // Os RGBA de cada faixa valem se os planos da tela, já com as faixas todas coladas, são os que o laço do
        // compositor leu à volta dela (B11.3): uma faixa recusada ou um dab composto em casa ficam só num dos lados.
        let (planos, grade) = (&sess.planos, sess.grid);
        let validos: Vec<bool> = im
            .caixas
            .iter()
            .map(|c| valido && c.lido.as_ref().is_some_and(|l| l.confere(planos, grade)))
            .collect();
        #[cfg(test)]
        {
            sess.otica.rgba_usadas += validos.iter().filter(|v| **v).count() as u32;
        }
        sess.seen_steps = im.passos;
        #[cfg(test)]
        {
            sess.otica.aplicadas += 1;
        }
        // A imagem não tira o sujo do motor: o pendente do quadro continua de quem o tinha.
        let pendente = self.paint.wetpaint.composite_pendente;
        let antes = self.wetpaint_antes_da_otica();
        self.paint.wetpaint.composite_pendente = pendente;
        let Imagem {
            oleo,
            marca,
            caixas,
            ..
        } = im;
        if caixas.is_empty() {
            let saida = SaidaDaOtica {
                oleo,
                marca,
                caixas: None,
            };
            self.wetpaint_depois_da_otica(
                &antes,
                saida,
                show_wet,
                VeuDe::DaImagem(sem_veu()),
                None,
            );
        }
        // O laço de píxel por faixa (B14.1).
        for (c, valido) in caixas.into_iter().zip(validos) {
            let linhas = match &c.veu {
                Some(v) => FilmeEMolha {
                    film: &v.film,
                    wet: &v.wet,
                    s: v.s,
                    linha0: v.linha0,
                    col0: v.col0,
                },
                None => sem_veu(),
            };
            let pronto = c.rgba.as_deref().filter(|_| valido);
            let saida = SaidaDaOtica {
                oleo,
                marca,
                caixas: Some(CaixasDaOtica {
                    caixa_cel: c.caixa_cel,
                    caixa_px: c.caixa_px,
                    caixa_suave: c.caixa_suave,
                    sem_oleo: c.sem_oleo,
                }),
            };
            self.wetpaint_depois_da_otica(&antes, saida, show_wet, VeuDe::DaImagem(linhas), pronto);
        }
    }

    /// Com água nova a mostrar (ou dívida) e nada no ar, publica o pedido; `done` = os passos que o tique leu.
    pub(super) fn wet_pede_imagem(&mut self, done: u64) {
        let com_veu = self.paint.wetpaint.show_wet;
        let factos = self.factos_da_otica();
        let pixel = self.wet_entrada_do_pixel(com_veu);
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        if sess.otica.no_ar.is_some()
            || sess.base.len() != w * h * 4
            || (done == sess.seen_steps
                && sess.otica.divida.is_empty()
                && sess.otica.agua.is_none())
        {
            return;
        }
        let epoca = sess.otica.epoca + 1;
        let p = PedidoDeImagem {
            factos,
            epoca,
            divida: sess.otica.divida.clone(),
            agua: sess.otica.agua,
            forma: FormaDaTela {
                ratio: sess.ratio,
                grade: sess.grid,
                tela: (w, h),
            },
            com_veu,
            pixel,
        };
        if let Ok(antigo) = sess.pede_imagem(p) {
            let o = &mut sess.otica;
            o.epoca = epoca;
            o.no_ar = Some(epoca);
            o.uniao_local = None;
            o.divida = antigo
                .as_ref()
                .map(|a| a.divida.clone())
                .unwrap_or_default();
            o.agua = antigo.and_then(|a| a.agua);
        }
    }
}

impl PainterTool {
    /// B11.3: o que o laço de píxel lê da ferramenta, para o compositor o correr — `None` (a casa corre-o) com vidro na
    /// camada, com a marca do artista ou com uma porta de teste do Smooth Edges (são do fio da tela).
    pub(super) fn wet_entrada_do_pixel(&self, veil: bool) -> Option<EntradaDoPixel> {
        #[cfg(test)]
        if super::smooth_edges::so_na_cor::get()
            || super::smooth_edges::sem_suave::get()
            || super::smooth_edges::raio_escalar::get().is_some()
        {
            return None;
        }
        let sess = self.paint.wetpaint.session.as_ref()?;
        if self.undo.marca.ativa()
            || sess
                .marca
                .is_some_and(|_| self.undo.marca.molhada_viva(sess.marca))
        {
            return None;
        }
        let camada = self.layers.active().filter(|&id| !self.layers.is_mask(id));
        #[cfg(test)]
        let camada = camada.filter(|_| !self.sem_vidro);
        if camada.is_some_and(|c| self.vidros.contains_key(&c)) {
            return None;
        }
        let wp = &self.paint.wetpaint;
        let linhas = (wp.tilt_on && wp.tilt_ring > 0).then(|| {
            let [gx, gy] = ph2d_wet_paint::sim::tilt_dir_for_spoke(wp.tilt_spoke);
            gy.abs() >= gx.abs()
        });
        Some(EntradaDoPixel {
            base: Arc::clone(&sess.base),
            gates: self.wet_splat_gates(),
            congelada: sess.congelada.clone().filter(|_| !veil),
            modo: sess.blend,
            vazio: sess.vazio,
            linhas,
        })
    }
}

/// O véu de uma imagem sem o filme (sem caixas ou sem `show_wet`).
const fn sem_veu() -> FilmeEMolha<'static> {
    FilmeEMolha {
        film: &[],
        wet: &[],
        s: 0,
        linha0: 0,
        col0: 0,
    }
}

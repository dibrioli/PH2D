//! **O tique e o compositor da óptica** (J13b P6, [plano](../../../../../../docs/Painter/Arquitetura/J13b_plano_otica_fora.md)
//! §2 passo 1 e «Validade»; filho de [`super`] por LOC). Com o compositor de pé, o tique PEDE a imagem da água e
//! APLICA a que chega, sem trazer o motor para casa:
//!
//! - um pedido no ar de cada vez; o seguinte sai no tique em que o anterior é respondido;
//! - `uniao_local` é a união das caixas compostas EM CASA desde o último pedido. Uma imagem que a toca pode ser
//!   mais velha do que a tela já mostra ali: descarta-se e a caixa vai para a `divida`, que entra no pedido seguinte;
//! - a pausa da autoria toca a tela inteira: nenhuma imagem se aplica por cima do esboço.

use super::composite_otica::{CaixasDaOtica, SaidaDaOtica};
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
    pub(super) divida: Option<CaixaCel>,
    #[cfg(test)]
    pub(super) aplicadas: u32,
    /// Imagens cuja caixa tocou a `uniao_local`.
    #[cfg(test)]
    pub(super) tocadas: u32,
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
        self.no_ar.is_some() || self.divida.is_some()
    }

    fn deve(&mut self, c: CaixaCel) {
        self.divida = Some(une(self.divida, c));
    }
}

impl WetSession {
    /// J13b P8 ([doc](../../../../../../docs/Painter/Arquitetura/J13b_P8_pen_down.md)): o sujo que o motor traz do
    /// worker é da ÁGUA (a tela de casa não tem traço por compor) e vai pela dívida, para a composição de casa do lote
    /// levar só o que o traço muda. O sujo é uma caixa só: sem isto o traço longe da poça compunha a UNIÃO das duas.
    pub(super) fn a_agua_herdada_vai_pela_divida(&mut self) {
        if let Some(c) = offthread::otica::caixa_da_grade(self.engine.take_dirty(), self.grid) {
            self.otica.deve(c);
        }
    }

    /// O sujo que saiu do motor e a tela ainda não mostra (a dívida e a imagem no ar) volta ao motor. Motor em casa: a
    /// composição seguinte leva só o que falta, nunca a tela inteira.
    pub(super) fn o_sujo_por_mostrar_volta_ao_motor(&mut self) {
        let no_ar = self
            .otica
            .no_ar
            .map_or([None, None], |e| self.sujo_do_pedido(e));
        for (x0, y0, x1, y1) in no_ar.into_iter().chain([self.otica.divida]).flatten() {
            let i = |v: usize| i32::try_from(v).unwrap_or(i32::MAX);
            self.engine.merge_dirty_otica(i(x0), i(y0), i(x1), i(y1));
        }
    }
}

impl PainterTool {
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
                        if let Some(c) = &im.caixas {
                            sess.otica.deve(c.caixa_cel);
                        }
                        continue;
                    }
                    sess.otica.no_ar = None;
                    self.wet_aplica_imagem(*im);
                }
            }
        }
    }

    fn wet_aplica_imagem(&mut self, im: Imagem) {
        let show_wet = self.paint.wetpaint.show_wet;
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        if let Some(c) = &im.caixas {
            // Pela caixa dos degraus, a maior que `Imagem::aplica` cola (`plano_do_suave`: contém a do pigmento).
            debug_assert!(toca(c.caixa_cel, c.caixa_suave) && c.caixa_suave.0 <= c.caixa_cel.0);
            let tocou = sess
                .otica
                .uniao_local
                .is_some_and(|u| toca(u, c.caixa_suave));
            #[cfg(test)]
            {
                sess.otica.tocadas += u32::from(tocou);
            }
            // A casa pintou depois do pedido e tem um plano que a imagem diz vazio (o 1.º óleo, o 1.º Smooth Edges):
            // a imagem é velha, e aplicá-la apagava esse plano fora da caixa dela.
            let velha = sess.otica.uniao_local.is_some() && c.esvazia(&sess.planos);
            if tocou || velha || (show_wet && c.veu.is_none()) {
                sess.otica.deve(c.caixa_cel);
                return;
            }
        }
        im.aplica(&mut sess.planos, sess.grid);
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
        let (caixas, veu) = match caixas {
            Some(c) => (
                Some(CaixasDaOtica {
                    caixa_cel: c.caixa_cel,
                    caixa_px: c.caixa_px,
                    caixa_suave: c.caixa_suave,
                    sem_oleo: c.sem_oleo,
                }),
                c.veu,
            ),
            None => (None, None),
        };
        let linhas = match &veu {
            Some(v) => FilmeEMolha {
                film: &v.film,
                wet: &v.wet,
                s: v.s,
                linha0: v.linha0,
            },
            None => FilmeEMolha {
                film: &[],
                wet: &[],
                s: 0,
                linha0: 0,
            },
        };
        let saida = SaidaDaOtica {
            oleo,
            marca,
            caixas,
        };
        self.wetpaint_depois_da_otica(&antes, saida, show_wet, VeuDe::DaImagem(linhas));
    }

    /// Com água nova a mostrar (ou dívida) e nada no ar, publica o pedido; `done` = os passos que o tique leu.
    pub(super) fn wet_pede_imagem(&mut self, done: u64) {
        let com_veu = self.paint.wetpaint.show_wet;
        let factos = self.factos_da_otica();
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        if sess.otica.no_ar.is_some()
            || sess.base.len() != w * h * 4
            || (done == sess.seen_steps && sess.otica.divida.is_none())
        {
            return;
        }
        let epoca = sess.otica.epoca + 1;
        let p = PedidoDeImagem {
            factos,
            epoca,
            divida: sess.otica.divida,
            forma: FormaDaTela {
                ratio: sess.ratio,
                grade: sess.grid,
                tela: (w, h),
            },
            com_veu,
        };
        if let Ok(antigo) = sess.pede_imagem(p) {
            let o = &mut sess.otica;
            o.epoca = epoca;
            o.no_ar = Some(epoca);
            o.uniao_local = None;
            o.divida = antigo.and_then(|a| a.divida);
        }
    }
}

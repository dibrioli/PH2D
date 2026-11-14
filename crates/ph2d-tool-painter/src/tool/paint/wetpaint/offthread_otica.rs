//! **O COMPOSITOR DA ÓPTICA fora do fio da tela** (J13b P5, [plano](../../../../../../../docs/Painter/Arquitetura/J13b_plano_otica_fora.md)
//! §2–§3; filho de [`super`] por LOC). A maquinaria; o tique que pede e aplica é [`super::super::otica_no_tique`] (P6):
//!
//! - o fio principal publica um [`PedidoDeImagem`] na CAIXA DE CORREIO do worker ([`DoFioPrincipal::pede`]) e
//!   recolhe as [`RespostaDaOtica`] sem esperar ([`DoFioPrincipal::try_recv`]);
//! - o worker, num passo FECHADO e com folga ([`DoWorker::serve`]), junta a dívida ao sujo, tira-o, copia as
//!   linhas mudadas para o [`Espelho`] e entrega-o ao compositor;
//! - o compositor corre [`compoe_a_otica`] sobre o espelho (planos próprios, sem `CacheCampo`), manda a
//!   [`Imagem`] ao fio principal e devolve o espelho ao worker.
//!
//! Só existe no worker AO RELÓGIO com o interruptor [`ligada`] (`PH2D_WET_OTICA_FORA`); nunca no ritmado, no
//! roteiro nem no `wet_step_sync`. Morre com o worker: o worker sai (o `Drop` do `SimWorker`), o canal de
//! trabalho fecha e o compositor acaba no `recv`.

use super::super::composite_oleo::SemOleo;
use super::super::composite_otica::PlanosDaOtica;
#[cfg(test)]
use super::super::composite_otica::{EntradaDaOtica, TrabalhoDaOtica, compoe_a_otica};
use super::super::fita::MarcaDaAgua;
use super::super::otica_factos::FactosDaOtica;
use super::STEP_S;
use ph2d_wet_paint::oleo_real::OleoReal;
use ph2d_wet_paint::painter::Dirty;
#[cfg(test)]
use ph2d_wet_paint::painter::Engine;
#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use std::time::Instant;

#[path = "offthread_agenda.rs"]
mod agenda; // quando servir: a regra pura (T1-42)
#[path = "otica_compositor.rs"]
mod compositor; // o fio do compositor: compõe sobre o espelho e recorta as caixas (B52.R8)
#[path = "otica_correio.rs"]
mod correio; // a caixa de correio e as duas pontas (B52.R8)
use compositor::cola;
#[path = "otica_pixel.rs"]
pub(in crate::tool::paint::wetpaint) mod pixel; // o laço de píxel no compositor (B11.3)
pub(in crate::tool::paint::wetpaint) use correio::DoFioPrincipal;
pub(super) use correio::{DoWorker, par};

/// Caixa em células, `(x0, y0, x1, y1)` inclusiva, `1..=gw × 1..=gh`.
pub(in crate::tool::paint::wetpaint) type CaixaCel = (usize, usize, usize, usize);

/// A geometria da tela com que se compõe (o que o `composite` lê da sessão e da ferramenta).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::tool::paint::wetpaint) struct FormaDaTela {
    pub(in crate::tool::paint::wetpaint) ratio: u8,
    /// `(gw, gh)`, células.
    pub(in crate::tool::paint::wetpaint) grade: (usize, usize),
    /// `(w, h)`, píxeis.
    pub(in crate::tool::paint::wetpaint) tela: (usize, usize),
}

/// `(dívida por servir ++ faixas servidas, água por servir)` de um pedido (`DoFioPrincipal::sujo_do_pedido`).
pub(in crate::tool::paint::wetpaint) type SujoDoPedido = (Vec<CaixaCel>, Option<CaixaCel>);

/// O pedido de UMA imagem, escrito só pelo fio principal.
#[derive(Clone)]
pub(in crate::tool::paint::wetpaint) struct PedidoDeImagem {
    pub(in crate::tool::paint::wetpaint) factos: FactosDaOtica,
    /// Ecoada na resposta: o fio principal sabe a que pedido ela responde.
    pub(in crate::tool::paint::wetpaint) epoca: u64,
    /// As caixas das imagens descartadas (tocaram a composição local), em pedaços: entram no sujo antes de o tirar.
    pub(in crate::tool::paint::wetpaint) divida: Vec<CaixaCel>,
    /// A água herdada no pen-down: entra como sujo do PASSO (o espelho redu-la ao que mudou).
    pub(in crate::tool::paint::wetpaint) agua: Option<CaixaCel>,
    pub(in crate::tool::paint::wetpaint) forma: FormaDaTela,
    /// `show_wet`: a imagem leva o filme e a molha da caixa para o véu.
    pub(in crate::tool::paint::wetpaint) com_veu: bool,
    /// B11.3: com ela o compositor corre também o laço de píxel e a imagem traz os RGBA da caixa.
    pub(in crate::tool::paint::wetpaint) pixel: Option<pixel::EntradaDoPixel>,
}

/// O filme e a molha da grade ACTIVA na caixa de células que a caixa de píxeis toca, com uma de folga de cada lado
/// (o menisco lê `film[i ± 1]` e `film[i ± s]`): linhas `linha0..`, colunas `col0..col0 + s` (B11.A2).
pub(in crate::tool::paint::wetpaint) struct VeuDaCaixa {
    pub(in crate::tool::paint::wetpaint) linha0: usize,
    pub(in crate::tool::paint::wetpaint) col0: usize,
    pub(in crate::tool::paint::wetpaint) s: usize,
    pub(in crate::tool::paint::wetpaint) film: Vec<f32>,
    pub(in crate::tool::paint::wetpaint) wet: Vec<u8>,
}

/// As caixas de uma imagem: os planos SÓ da `caixa_cel`, linha a linha.
pub(in crate::tool::paint::wetpaint) struct CaixasDaImagem {
    /// A faixa do sujo antes do alcance: a dívida se a imagem não se aplica.
    pub(in crate::tool::paint::wetpaint) sujo: CaixaCel,
    pub(in crate::tool::paint::wetpaint) caixa_cel: CaixaCel,
    pub(in crate::tool::paint::wetpaint) caixa_px: (usize, usize, usize, usize),
    /// `4` bytes por célula.
    pub(in crate::tool::paint::wetpaint) pigmento: Vec<u8>,
    /// `None` = o plano ficou VAZIO (sem campo do óleo), como a óptica o deixa.
    pub(in crate::tool::paint::wetpaint) sombra: Option<Vec<u8>>,
    pub(in crate::tool::paint::wetpaint) grao: Option<Vec<u8>>,
    /// O plano do Smooth Edges, na caixa dele (`None` = vazio, como a óptica o deixa).
    pub(in crate::tool::paint::wetpaint) suave: Option<Vec<u8>>,
    pub(in crate::tool::paint::wetpaint) caixa_suave: CaixaCel,
    pub(in crate::tool::paint::wetpaint) sem_oleo: SemOleo,
    pub(in crate::tool::paint::wetpaint) veu: Option<VeuDaCaixa>,
    /// B11.3: os RGBA da `caixa_px` (`(px1 - px0) * 4` por linha), compostos pelo laço de píxel no compositor.
    pub(in crate::tool::paint::wetpaint) rgba: Option<Vec<u8>>,
    /// Os planos que o laço do compositor leu para os `rgba` (o fio da tela confere-os com os seus).
    pub(in crate::tool::paint::wetpaint) lido: Option<pixel::Lido>,
}

/// Uma imagem composta fora: o que a óptica decidiu e as caixas.
pub(in crate::tool::paint::wetpaint) struct Imagem {
    pub(in crate::tool::paint::wetpaint) oleo: Option<OleoReal>,
    pub(in crate::tool::paint::wetpaint) marca: MarcaDaAgua,
    pub(in crate::tool::paint::wetpaint) epoca: u64,
    /// Os passos do worker no instante da cópia (⇒ `seen_steps`).
    pub(in crate::tool::paint::wetpaint) passos: u64,
    /// Uma por faixa composta (B14.1); vazio = a caixa ficou vazia depois de presa à grade.
    pub(in crate::tool::paint::wetpaint) caixas: Vec<CaixasDaImagem>,
    /// A entrada do píxel com que os `rgba` se compuseram (o fio da tela confere-a antes de os copiar).
    pub(in crate::tool::paint::wetpaint) pixel: Option<pixel::EntradaDoPixel>,
}

/// O que volta ao fio principal por pedido servido.
pub(in crate::tool::paint::wetpaint) enum RespostaDaOtica {
    /// O motor estava limpo (`Dirty::Clean`, sem dívida): nada a compor.
    Nada {
        epoca: u64,
        passos: u64,
        marca: MarcaDaAgua,
    },
    Imagem(Box<Imagem>),
}

impl CaixasDaImagem {
    /// Algum plano que `planos` tem e esta imagem diz vazio?
    pub(in crate::tool::paint::wetpaint) fn esvazia(&self, planos: &PlanosDaOtica) -> bool {
        [
            (&self.sombra, &planos.sombra),
            (&self.grao, &planos.grao),
            (&self.suave, &planos.suave),
        ]
        .iter()
        .any(|(dela, nosso)| dela.is_none() && !nosso.is_empty())
    }
}

impl Imagem {
    /// Escreve a imagem nos planos da sessão: o mesmo que [`compoe_a_otica`] lá teria escrito.
    pub(in crate::tool::paint::wetpaint) fn aplica(
        &self,
        planos: &mut PlanosDaOtica,
        grade: (usize, usize),
    ) {
        let (gw, gh) = grade;
        for c in &self.caixas {
            cola(&mut planos.pigment, 4, gw, c.caixa_cel, &c.pigmento);
            for (plano, caixa, onde) in [
                (&mut planos.sombra, &c.sombra, c.caixa_cel),
                (&mut planos.grao, &c.grao, c.caixa_cel),
                (&mut planos.suave, &c.suave, c.caixa_suave),
            ] {
                match caixa {
                    Some(b) => {
                        if plano.len() != gw * gh {
                            *plano = vec![0u8; gw * gh];
                        }
                        cola(plano, 1, gw, onde, b);
                    }
                    None if !plano.is_empty() => *plano = Vec::new(),
                    None => {}
                }
            }
        }
        if self.caixas.is_empty() {
            return;
        }
        if self.oleo.is_none() {
            planos.campo_cache = None;
        }
    }
}

#[cfg(test)]
thread_local! {
    static FORCA: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// A porta de teste do interruptor: lida por [`ligada`] no fio que sobe o worker.
#[cfg(test)]
pub(in crate::tool::paint::wetpaint) fn forca_no_teste(v: Option<bool>) {
    FORCA.with(|c| c.set(v));
}

/// O compositor existe? (`PH2D_WET_OTICA_FORA`, [`crate::wet_diag::otica_fora`]).
pub(in crate::tool::paint::wetpaint) fn ligada() -> bool {
    #[cfg(test)]
    if let Some(v) = FORCA.with(std::cell::Cell::get) {
        return v;
    }
    crate::wet_diag::otica_fora()
}

/// O sujo do motor preso à grade (o mesmo do `composite`).
pub(in crate::tool::paint::wetpaint) fn caixa_da_grade(
    d: Dirty,
    (gw, gh): (usize, usize),
) -> Option<CaixaCel> {
    match d {
        Dirty::Clean => None,
        Dirty::Full => Some((1, 1, gw, gh)),
        Dirty::Rect { x0, y0, x1, y1 } => Some((
            (x0.max(1) as usize).min(gw),
            (y0.max(1) as usize).min(gh),
            (x1.max(1) as usize).min(gw),
            (y1.max(1) as usize).min(gh),
        )),
    }
}

/// A sessão não tem compositor: o pedido de imagem não foi publicado.
#[derive(Debug)]
pub(in crate::tool::paint::wetpaint) struct SemCompositor;

impl super::super::WetSession {
    /// Publica um pedido de imagem; `Err(SemCompositor)` = não há compositor (interruptor desligado, worker
    /// ritmado ou ainda sem worker) e o pedido cai. `Ok(Some(antigo))` = o pedido ainda por servir que este
    /// substituiu.
    pub(in crate::tool::paint::wetpaint) fn pede_imagem(
        &self,
        p: PedidoDeImagem,
    ) -> Result<Option<PedidoDeImagem>, SemCompositor> {
        let o = self
            .worker
            .as_ref()
            .and_then(|w| w.otica.as_ref())
            .filter(|o| o.vivo())
            .ok_or(SemCompositor)?;
        Ok(o.pede(p))
    }

    /// Esta sessão tem compositor de pé (o worker ao relógio subiu-o e ele não morreu)?
    pub(in crate::tool::paint::wetpaint) fn tem_compositor(&self) -> bool {
        self.worker
            .as_ref()
            .and_then(|w| w.otica.as_ref())
            .is_some_and(DoFioPrincipal::vivo)
    }

    /// O compositor desta sessão morreu e o tique ainda não o largou (B11.A1)?
    pub(in crate::tool::paint::wetpaint) fn compositor_morto(&self) -> bool {
        self.worker
            .as_ref()
            .and_then(|w| w.otica.as_ref())
            .is_some_and(|o| !o.vivo())
    }

    /// Com o motor em casa: o sujo que a tela ainda não mostra volta ao motor, e a sessão larga o compositor morto
    /// — daqui em diante compõe em casa (B11.A1).
    pub(in crate::tool::paint::wetpaint) fn larga_o_compositor_morto(&mut self) {
        self.o_sujo_por_mostrar_volta_ao_motor();
        self.otica.esquece_o_que_estava_no_ar();
        if let Some(w) = self.worker.as_mut() {
            w.otica = None;
        }
    }

    /// [`DoFioPrincipal::sujo_do_pedido`] desta sessão.
    pub(in crate::tool::paint::wetpaint) fn sujo_do_pedido(&self, epoca: u64) -> SujoDoPedido {
        self.worker
            .as_ref()
            .and_then(|w| w.otica.as_ref())
            .map_or((Vec::new(), None), |o| o.sujo_do_pedido(epoca))
    }

    /// A próxima resposta do compositor, sem esperar.
    pub(in crate::tool::paint::wetpaint) fn try_recv_imagem(&self) -> Option<RespostaDaOtica> {
        self.worker.as_ref()?.otica.as_ref()?.try_recv()
    }
}

#[cfg(test)]
#[path = "offthread_otica_tests.rs"]
mod tests;

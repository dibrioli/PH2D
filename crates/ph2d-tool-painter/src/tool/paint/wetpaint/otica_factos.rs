//! **Os factos da óptica** (J13b P3, [plano](../../../../../../../docs/Painter/Arquitetura/J13b_plano_otica_fora.md)
//! §1): tudo o que a metade de cima do composite tira do `PainterTool`, como um VALOR `Copy + Send` — viaja dentro
//! do pedido de imagem para a thread de composição, e cada imagem leva os factos com que foi feita. A parte que lê o
//! MOTOR (há tinta com Real Oil numa camada visível?) é [`FactosDaOtica::oleo_da_tela`], que corre do lado de quem
//! tem as grades.

use ph2d_wet_paint::oleo_real::OleoReal;
use ph2d_wet_paint::render::{PigmentVisual, RenderLayer};

/// Os interruptores de teste que a óptica lê, capturados por quem a pede (um `thread_local!` não se vê noutra
/// thread). Os que a óptica lê no fio da tela ficam ASSADOS no valor: `grao_no_filme::sem_grao` no
/// [`FactosDaOtica::tem_grao`].
#[cfg(test)]
#[derive(Clone, Copy, PartialEq)]
pub(super) struct InterruptoresDaOtica {
    /// [`super::oleo_plano::curas_desligadas`]: `(sombra antiga, campo em série)`.
    pub(super) curas_desligadas: (bool, bool),
    /// [`super::oleo_plano::sem_cache_do_campo`].
    pub(super) sem_cache: bool,
    /// [`super::oleo_plano::desligado`]: o atalho do papel liso na sombra.
    pub(super) papel_liso_desligado: bool,
    /// [`super::oleo_plano::lei_da_caixa`] ligado ⇒ `Some(há cartão do Óleo Real)`, a decisão que substitui a das
    /// grades.
    pub(super) lei_da_caixa: Option<bool>,
    /// [`super::smooth_edges::so_na_cor`]: o campo não vê o Smooth Edges.
    pub(super) so_na_cor: bool,
}

/// O que a óptica sabe da ferramenta — sem empréstimo nenhum.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct FactosDaOtica {
    pub(super) visual: PigmentVisual,
    /// O Óleo Real adiado (`wetpaint.oleo_adiado`): a óptica do óleo não corre.
    pub(super) oleo_adiado: bool,
    /// Os números do Óleo Real da TELA, já construídos (papel linear, `dente_papel`; o `suave_px` é o tecto das grades).
    pub(super) oleo: OleoReal,
    /// Há dente do papel para assar na tinta (o `GraoNoFilme` existe, se houver óleo).
    pub(super) tem_grao: bool,
    #[cfg(test)]
    pub(super) teste: InterruptoresDaOtica,
}

const _: fn() = || {
    fn viaja<T: Copy + Send>() {}
    viaja::<FactosDaOtica>();
};

impl FactosDaOtica {
    /// Os números do Óleo Real deste composite (`None` = sem óleo na tela). O Real Oil é do TRAÇO (dono 28/10): corre
    /// quando ALGUMA camada visível tem tinta com Real Oil (`Grid::oleo_real`; o campo sozinho pode ser só o meio —
    /// J9 §3), com os números da TELA do cartão — nunca pela caixa de agora.
    pub(super) fn oleo_da_tela(&self, layers: &[RenderLayer<'_>]) -> Option<OleoReal> {
        let tem_oleo = !self.oleo_adiado && layers.iter().any(|l| l.visible && l.grid.oleo_real);
        #[cfg(test)]
        let tem_oleo = self.teste.lei_da_caixa.unwrap_or(tem_oleo);
        tem_oleo.then(|| {
            // O alcance e a janela do campo lêem o tecto; cada célula amacia pelo dela.
            let grades: Vec<_> = layers.iter().map(|l| (l.grid, l.visible)).collect();
            let mut o = self.oleo;
            o.suave_px = f64::from(ph2d_wet_paint::grid::aparencia::raio_do_teto(&grades));
            #[cfg(test)]
            if self.teste.so_na_cor {
                o.suave_px = 0.0;
            }
            o
        })
    }
}

impl crate::PainterTool {
    /// Os factos da óptica de agora ([`FactosDaOtica`]).
    pub(super) fn factos_da_otica(&self) -> FactosDaOtica {
        let (w, h) = self.source_size;
        let tem_grao = super::grao_no_filme::GraoNoFilme::do_papel(
            self.papel_na_escala(super::super::substrate_relief::papel_do_documento(
                self.paint.brush.paper,
            )),
            (self.paint.substrate_depth, self.paint.substrate_rough),
            self.paint.paper_image.as_ref().map(|i| i.as_mask()),
            &self.paint.impasto_rig,
            (w as usize, h as usize),
        )
        .is_some();
        // A Paint Texture imprime o PAPEL: sem papel escolhido não há o que imprimir (dono 10/10). É do TRAÇO (dono
        // 28/10): QUEM imprime é a célula (`grid::aparencia::textura`), nunca a caixa de agora.
        let visual = PigmentVisual {
            paper: self.paint.brush.paper.is_active(),
            // O relevo da tinta é da tinta (dono 11/10): o knob Emboss decide, com ou sem papel.
            relevo: true,
        };
        // W3b: o papel do documento sob a tinta (a cor que a Transmission tinge) e o dente dele (o Relevo do papel).
        let mut o = self.paint.wetpaint.oleo_real_valores;
        crate::wet_diag::note_factos(self.wet_pincel_na_tela());
        o.papel = self.papel.map_or([1.0; 3], |p| {
            p.map(|c| ph2d_wet_paint::colorops::transfer::srgb255_to_linear(f64::from(c)))
        });
        // U2 W3b: o Paper Relief morde no estado de fábrica (Relief do papel 0) — a textura do papel na tinta tem o
        // piso de profundidade [`super::grao_no_filme::RELIEF_NA_TINTA_MIN`].
        o.dente_papel = f64::from(
            super::grao_no_filme::relief_na_tinta(self.paint.substrate_depth)
                * super::super::substrate_relief::MAX_TOOTH_PX,
        );
        FactosDaOtica {
            visual,
            oleo_adiado: self.paint.wetpaint.oleo_adiado,
            oleo: o,
            tem_grao,
            #[cfg(test)]
            teste: InterruptoresDaOtica {
                curas_desligadas: super::oleo_plano::curas_desligadas::get(),
                sem_cache: super::oleo_plano::sem_cache_do_campo::get(),
                papel_liso_desligado: super::oleo_plano::desligado::get(),
                lei_da_caixa: super::oleo_plano::lei_da_caixa::get()
                    .then(|| self.paint.wetpaint.oleo_real.is_some()),
                so_na_cor: super::smooth_edges::so_na_cor::get(),
            },
        }
    }
}

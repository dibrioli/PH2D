//! Os PLANOS DO ÓLEO que o composite tira do campo, ainda do lado do motor (filho de [`super`], LOC cap de
//! [`super::composite`]): a sombra no papel, o dente de cada célula e a tinta sem óleo do Smooth Edges.
//!
//! Recebem os CAMPOS da sessão e não a sessão: o campo guardado empresta `campo_cache` até ao fim do composite.

use ph2d_wet_paint::oleo_real::{CampoAltura, OleoReal};
use rayon::prelude::*;

/// O campo do Óleo Real no composite: os valores e a altura (guardada ou feita agora).
pub(super) type Campo<'a> = Option<(OleoReal, &'a CampoAltura)>;

/// A sombra do Óleo Real no PAPEL: a luz que a tinta à volta lhe tira — também por BAIXO da orla fina,
/// que se compõe sobre o papel já sombreado (cortada em «alfa 0» a orla ficava um aro claro entre a
/// tinta e a sombra). Fica num plano à parte — o alfa do pigmento continua a ser só cobertura.
pub(super) fn sombra_do_papel(
    sombra: &mut Vec<u8>,
    campo: &Campo<'_>,
    feita: &[bool],
    (cx0, cy0, cx1, cy1): (usize, usize, usize, usize),
    (gw, gh): (usize, usize),
) {
    if let Some((o, c)) = campo {
        // O papel LISO (campo a zero até ao alcance da óptica) dá a mesma sombra em toda a parte: calcula-se
        // uma vez por linha ([`super::oleo_plano`]) — na água parada a região é a caixa molhada inteira.
        #[cfg(test)]
        let atalho = !super::oleo_plano::desligado::get();
        #[cfg(not(test))]
        let atalho = true;
        let plano =
            atalho.then(|| super::oleo_plano::PapelLiso::de(c, (cx0, cy0, cx1, cy1), (gw, gh)));
        sombra[(cy0 - 1) * gw..cy1 * gw]
            .par_chunks_mut(gw)
            .enumerate()
            .for_each(|(k, row)| {
                let cy = cy0 + k;
                let feita = feita.get(k * gw..(k + 1) * gw);
                let mut liso: Option<u8> = None;
                for cx in cx0..=cx1 {
                    if feita.is_some_and(|f| f[cx - 1]) {
                        continue;
                    }
                    // O papel nu: a sombra atravessa a tinta que a projecta (a Transparency e a Transmission
                    // da célula: o cartão não as tem — W3b ronda 3).
                    let calcula = || {
                        ph2d_wet_paint::oleo_real::sombra_em_u8(
                            ph2d_wet_paint::oleo_real::sombra_transparente(
                                ph2d_wet_paint::oleo_real::sombra_no_papel(o, c, cx, cy),
                                0.0,
                                ph2d_wet_paint::oleo_real::transparencia_de_quem_tapa(o, c, cx, cy),
                            ),
                        )
                    };
                    row[cx - 1] = if plano.as_ref().is_some_and(|p| p.liso(cx, cy)) {
                        *liso.get_or_insert_with(calcula)
                    } else {
                        calcula()
                    };
                }
            });
    } else if !sombra.is_empty() {
        *sombra = Vec::new();
    }
}

/// W3b: o dente que a tinta de cada célula mostra — o píxel lê-o depois da fronteira do motor.
pub(super) fn grao_das_celulas(
    grao_cel: &mut Vec<u8>,
    campo: &Campo<'_>,
    grao: &Option<super::grao_no_filme::GraoNoFilme<'_>>,
    (cx0, cy0, cx1, cy1): (usize, usize, usize, usize),
    (gw, gh): (usize, usize),
) {
    match (campo, grao) {
        (Some((o, c)), Some(_)) => {
            if grao_cel.len() != gw * gh {
                *grao_cel = vec![0u8; gw * gh];
            }
            for cy in cy0..=cy1 {
                for cx in cx0..=cx1 {
                    // A tinta sem óleo não mostra o dente do papel do óleo (a fracção dela).
                    let d =
                        ph2d_wet_paint::oleo_real::grao_do_papel(o, c, cx, cy) * c.oleo_em(cx, cy);
                    grao_cel[(cy - 1) * gw + cx - 1] = (d.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        }
        _ if !grao_cel.is_empty() => *grao_cel = Vec::new(),
        _ => {}
    }
}

/// A tinta sem óleo de cada célula da caixa, `(cx0, cy0, largura, célula sem óleo)`.
pub(super) type SemOleo = Option<(usize, usize, usize, Vec<bool>)>;

/// O Smooth Edges numa tela com óleo (dono 28/10): a silhueta amaciada pelo campo é a da tinta COM óleo; a tinta
/// SEM ele (e o papel à volta só dela) guarda a orla de sempre do Smooth Edges — a tinta sem óleo não muda por
/// haver óleo noutro sítio. `(cx0, cy0, largura, célula sem óleo)`.
pub(super) fn tinta_sem_oleo(
    engine: &ph2d_wet_paint::painter::Engine,
    campo: &Campo<'_>,
    (suave, raio): (bool, f32),
    ratio: u8,
    (cx0, cy0, cx1, cy1): (usize, usize, usize, usize),
    (gw, gh): (usize, usize),
) -> SemOleo {
    match (campo, suave) {
        (Some((_, c)), true) => {
            let tinta = |i: usize| {
                engine
                    .layers
                    .iter()
                    .any(|l| l.visible && l.grid.sett[i] + l.grid.susp[i] > 0.0)
            };
            let s = engine.layers[0].grid.s;
            let r = (raio / f32::from(ratio)).ceil() as usize + 1;
            let cw = cx1 + 1 - cx0;
            let mut v = vec![false; cw * (cy1 + 1 - cy0)];
            for cy in cy0..=cy1 {
                for cx in cx0..=cx1 {
                    v[(cy - cy0) * cw + cx - cx0] = if tinta(cx + cy * s) {
                        c.oleo_em(cx, cy) < 0.5
                    } else {
                        // O papel segue a tinta à volta: só sem óleo se nenhuma tinta com óleo o rodeia.
                        let (mut com, mut sem) = (false, false);
                        for y in cy.saturating_sub(r).max(1)..=(cy + r).min(gh) {
                            for x in cx.saturating_sub(r).max(1)..=(cx + r).min(gw) {
                                if tinta(x + y * s) {
                                    if c.oleo_em(x, y) < 0.5 {
                                        sem = true;
                                    } else {
                                        com = true;
                                    }
                                }
                            }
                        }
                        sem && !com
                    };
                }
            }
            Some((cx0, cy0, cw, v))
        }
        _ => None,
    }
}

impl crate::PainterTool {
    /// Os números do Óleo Real deste composite (`None` = sem óleo na tela); `raio` = o do Smooth Edges, px.
    pub(super) fn oleo_do_composite(&mut self, raio: f32) -> Option<OleoReal> {
        // O Real Oil é do TRAÇO (dono 28/10): a óptica corre quando ALGUMA camada visível tem tinta com óleo (o campo
        // `grid::oleo` existe), com os números da TELA do cartão — nunca pela caixa de agora.
        let tem_oleo = !self.paint.wetpaint.oleo_adiado
            && self.paint.wetpaint.session.as_mut().is_some_and(|s| {
                s.bring_home();
                s.engine
                    .layers
                    .iter()
                    .any(|l| l.visible && !l.grid.oleo.is_empty())
            });
        #[cfg(test)]
        let tem_oleo = if super::oleo_plano::lei_da_caixa::get() {
            self.paint.wetpaint.oleo_real.is_some()
        } else {
            tem_oleo
        };
        self.paint.wetpaint.oleo_na_tela = tem_oleo;
        // W3b: o papel do documento sob a tinta (a cor que a Transmission tinge) e o dente dele (o Relevo do papel).
        tem_oleo
            .then_some(self.paint.wetpaint.oleo_real_valores)
            .map(|mut o| {
                o.papel = self.papel.map_or([1.0; 3], |p| {
                    p.map(|c| ph2d_wet_paint::colorops::transfer::srgb255_to_linear(f64::from(c)))
                });
                // D1 W3b: o Smooth Edges amacia a SILHUETA no campo — o relevo e a cobertura seguem o mesmo contorno.
                o.suave_px = f64::from(raio);
                #[cfg(test)]
                if super::smooth_edges::so_na_cor::get() {
                    o.suave_px = 0.0;
                }
                // U2 W3b: o Paper Relief morde no estado de fábrica (Relief do papel 0) — a textura do papel na tinta tem
                // o piso de profundidade [`super::grao_no_filme::RELIEF_NA_TINTA_MIN`].
                o.dente_papel = f64::from(
                    super::grao_no_filme::relief_na_tinta(self.paint.substrate_depth)
                        * super::super::substrate_relief::MAX_TOOTH_PX,
                );
                o
            })
    }
}

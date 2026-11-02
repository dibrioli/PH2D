//! **A metade de cima do composite** (a óptica, acima da FRONTEIRA DO MOTOR): o pigmento de cada célula da caixa,
//! a sombra no papel, o dente e a tinta sem óleo do Smooth Edges, escritos nos [`PlanosDaOtica`]. Função PURA: lê
//! só a [`EntradaDaOtica`] e as camadas, nunca o `PainterTool` nem a sessão — o passo que a leva a outra thread
//! ([J13b](../../../../../../../docs/Painter/Arquitetura/J13b_plano_otica_fora.md) §1–§2).

#[cfg(test)]
use super::composite_contagem as contagem;
use super::grid_map;
use ph2d_wet_paint::oleo_real::{CacheCampo, OleoReal};
use ph2d_wet_paint::render::{PigmentVisual, RenderLayer, render_pigment_row_visual};
use rayon::prelude::*;

/// Os planos da GRADE que o laço de píxel lê (única verdade dele) e o campo guardado do Óleo Real.
#[derive(Default)]
pub(super) struct PlanosDaOtica {
    /// O pigmento, **`gw * gh * 4`, o tamanho da GRADE e não o do canvas** (o render escreve uma linha de célula
    /// por vez; o composite é quem sobe para o canvas). A caixa pedida é reescrita inteira: bytes velhos fora dela
    /// são inertes.
    pub(super) pigment: Vec<u8>,
    /// **A sombra do Óleo Real no papel** (`gw * gh`, `0..255` de escurecimento) onde a célula não tem
    /// pigmento — VAZIO enquanto o Óleo Real está desligado (quem não o usa não paga).
    pub(super) sombra: Vec<u8>,
    /// W3b: o DENTE do papel que a tinta de cada célula mostra (`gw * gh`, `0..255`,
    /// [`ph2d_wet_paint::oleo_real::grao_do_papel`]) — VAZIO sem papel, sem luz ou sem Óleo Real.
    pub(super) grao: Vec<u8>,
    /// **O campo de altura do Óleo Real guardado por dependência** (W3, `oleo_real::CacheCampo`): a varredura do
    /// cartão (Specular, fenda, material sem Subsurface, Transparency) reusa-o enquanto as grades e a chave ficam.
    pub(super) campo_cache: Option<CacheCampo>,
}

impl PlanosDaOtica {
    /// Os planos novos de uma grade `(gw, gh)`: o pigmento transparente, o resto vazio.
    pub(super) fn da_grade((gw, gh): (usize, usize)) -> Self {
        Self {
            pigment: vec![0u8; gw * gh * 4],
            ..Self::default()
        }
    }

    /// A cópia do instantâneo da sessão: os três planos; o campo guardado NÃO viaja (recalcula-se, é exacto).
    pub(super) fn copia_sem_campo(&self) -> Self {
        Self {
            pigment: self.pigment.clone(),
            sombra: self.sombra.clone(),
            grao: self.grao.clone(),
            campo_cache: None,
        }
    }
}

/// Os interruptores de teste que a óptica lê, capturados por quem a pede (um `thread_local!` não se vê noutra thread).
#[cfg(test)]
#[derive(Clone, Copy)]
pub(super) struct InterruptoresDaOtica {
    /// [`super::oleo_plano::curas_desligadas`]: `(sombra antiga, campo em série)`.
    pub(super) curas_desligadas: (bool, bool),
    /// [`super::oleo_plano::sem_cache_do_campo`].
    pub(super) sem_cache: bool,
}

#[cfg(test)]
impl InterruptoresDaOtica {
    pub(super) fn de_agora() -> Self {
        Self {
            curas_desligadas: super::oleo_plano::curas_desligadas::get(),
            sem_cache: super::oleo_plano::sem_cache_do_campo::get(),
        }
    }
}

/// Tudo o que a óptica lê além das camadas — o que hoje vinha do `self`.
pub(super) struct EntradaDaOtica<'a, 'g> {
    pub(super) visual: PigmentVisual,
    /// O raio do Smooth Edges, px (`0` = desligado).
    pub(super) raio: f32,
    /// Os números do Óleo Real deste composite, já construídos (`None` = sem óleo na tela).
    pub(super) oleo: Option<OleoReal>,
    /// O dente do papel na tinta: a óptica só pergunta se há (`Some`).
    pub(super) grao: &'a Option<super::grao_no_filme::GraoNoFilme<'g>>,
    pub(super) params: &'a ph2d_wet_paint::sim::Params,
    /// ⚠️ Só para [`super::composite_oleo::tinta_sem_oleo`], que ainda lê o motor (sai no P3 do J13b).
    pub(super) motor: &'a ph2d_wet_paint::painter::Engine,
    /// O sujo do motor, em CÉLULAS, já clampado à grade `(x0, y0, x1, y1)`.
    pub(super) caixa: (usize, usize, usize, usize),
    pub(super) ratio: u8,
    /// A grade `(gw, gh)`, células.
    pub(super) grade: (usize, usize),
    /// A tela `(w, h)`, píxeis.
    pub(super) tela: (usize, usize),
    /// A `revisao` do motor (a chave do campo guardado).
    pub(super) revisao: u64,
    #[cfg(test)]
    pub(super) teste: InterruptoresDaOtica,
}

/// O que a óptica escreveu: as caixas e a tinta sem óleo do Smooth Edges.
pub(super) struct SaidaDaOtica {
    /// A caixa composta em células (o sujo crescido pelo alcance do Óleo Real).
    pub(super) caixa_cel: (usize, usize, usize, usize),
    /// A caixa em píxeis que o laço de baixo escreve (crescida pelo alcance do Smooth Edges).
    pub(super) caixa_px: (usize, usize, usize, usize),
    pub(super) sem_oleo: super::composite_oleo::SemOleo,
}

/// A metade de cima do composite. `None` = nada a escrever (a caixa ficou vazia).
pub(super) fn compoe_a_otica(
    e: &EntradaDaOtica<'_, '_>,
    layers: &[RenderLayer<'_>],
    planos: &mut PlanosDaOtica,
) -> Option<SaidaDaOtica> {
    let (w, h) = e.tela;
    let (gw, gh) = e.grade;
    let ratio = e.ratio;
    let (raio, suave) = (e.raio, e.raio > 0.0);
    let (oleo, visual, params) = (e.oleo, e.visual, e.params);
    let (cx0, cy0, cx1, cy1) = e.caixa;
    #[cfg(test)]
    let area_motor = (cx1 + 1).saturating_sub(cx0) * (cy1 + 1).saturating_sub(cy0);
    // Óleo Real: a fenda e a sombra de uma célula leem a tinta à volta — a região cresce pelo alcance.
    let (cx0, cy0, cx1, cy1) = match &oleo {
        Some(o) => {
            let grades: Vec<_> = layers.iter().map(|l| (l.grid, l.visible)).collect();
            let a = ph2d_wet_paint::oleo_real::alcance_na(
                o,
                &grades,
                usize::from(ratio),
                (cx0, cy0, cx1, cy1),
            );
            (
                cx0.saturating_sub(a).max(1),
                cy0.saturating_sub(a).max(1),
                (cx1 + a).min(gw),
                (cy1 + a).min(gh),
            )
        }
        None => (cx0, cy0, cx1, cy1),
    };
    if cx1 < cx0 || cy1 < cy0 {
        return None;
    }
    #[cfg(test)]
    contagem::nota(area_motor, (cx1 + 1 - cx0) * (cy1 + 1 - cy0));
    let (px0, py0, px1, py1) = grid_map::cell_rect_to_px((cx0, cy0, cx1, cy1), ratio, w, h);
    if px1 <= px0 || py1 <= py0 {
        return None;
    }
    // Smooth Edges: um pixel até ao alcance do raio também muda, logo o retângulo cresce por ele.
    let (px0, py0, px1, py1) = if suave {
        let a = super::smooth_edges::alcance(raio);
        (
            px0.saturating_sub(a),
            py0.saturating_sub(a),
            (px1 + a).min(w),
            (py1 + a).min(h),
        )
    } else {
        (px0, py0, px1, py1)
    };
    // The visual terms (doc 22): the paper printed into the pigment and
    // the K–M glaze stacking ride the SAME single render body; both off
    // is the historical pigment render, byte for byte (engine-gated).
    //
    // ROW-PARALLEL ([ADR-0109](../../../../../../docs/architecture/decisions/0109-rayon-exception-watercolor-composite.md),
    // amendment 2): the same shape of work the ADR already sanctioned for
    // the watercolor optical composite — a pure per-pixel map whose rows
    // are independent, in the crate the ADR already opened. The ENGINE
    // stays dependency-free and spawns nothing; it exposes one row
    // (`render_pigment_row_visual`) and the fan-out lives here.
    //
    // Byte-identical by construction: every row computes exactly what the
    // serial loop computed, and rows never read each other's output.
    #[cfg(test)]
    let t_engine = std::time::Instant::now();
    // O plano de pigmento é da GRADE (uma linha por linha de célula), e é
    // por isso que uma razão maior corta o render de pigmento por `ratio²`
    // — é o mesmo trabalho por célula, e há `ratio²` vezes menos delas.
    let g_stride = gw * 4;
    #[cfg(test)]
    let t_campo = std::time::Instant::now();
    // As duas curas de custo (27/10, bit a bit): o campo em faixas pelo rayon e a sombra no papel das células
    // PINTADAS tirada dos termos que a cor delas já calculou ([`super::oleo_plano`]).
    #[cfg(test)]
    let (sombra_antiga, campo_em_serie) = e.teste.curas_desligadas;
    #[cfg(not(test))]
    let (sombra_antiga, campo_em_serie) = (false, false);
    // O campo guardado por DEPENDÊNCIA (`oleo_real::CacheCampo`): a mesma chave e as mesmas grades reusam-no.
    let revisao = e.revisao;
    let chave = oleo.map(|o| {
        let grades: Vec<_> = layers.iter().map(|l| (l.grid, l.visible)).collect();
        ph2d_wet_paint::oleo_real::ChaveCampo::de(&o, &grades, usize::from(ratio))
    });
    #[cfg(test)]
    let sem_cache = e.teste.sem_cache;
    #[cfg(not(test))]
    let sem_cache = false;
    let guardado = match (&oleo, &chave) {
        (Some(o), Some(k)) if !sem_cache => {
            CacheCampo::serve(planos.campo_cache.as_ref(), k, revisao, o, (cy0, cy1))
        }
        _ => None,
    };
    let proprio = oleo.filter(|_| guardado.is_none()).map(|o| {
        let grades: Vec<_> = layers.iter().map(|l| (l.grid, l.visible)).collect();
        let janela = (cx0, cy0, cx1, cy1);
        if campo_em_serie {
            ph2d_wet_paint::oleo_real::CampoAltura::de_com(
                &ph2d_wet_paint::oleo_real::EmSerie,
                &grades,
                &o,
                usize::from(ratio),
                janela,
            )
        } else {
            ph2d_wet_paint::oleo_real::CampoAltura::de_com(
                &super::oleo_plano::EmFaixas,
                &grades,
                &o,
                usize::from(ratio),
                janela,
            )
        }
    });
    #[cfg(test)]
    if proprio.is_some() {
        super::oleo_plano::sem_cache_do_campo::conta();
    }
    let campo = oleo.zip(guardado.or(proprio.as_ref()));
    #[cfg(test)]
    contagem::campo(t_campo.elapsed());
    #[cfg(test)]
    super::fps_j13_tests::campo_bit::gancho(
        campo.as_ref().map(|(o, c)| (o, *c)),
        (cx0, cy0, cx1, cy1),
        (gw, gh),
    );
    if campo.is_some() && planos.sombra.len() != gw * gh {
        planos.sombra = vec![0u8; gw * gh];
    }
    // `feita[k·gw + cx − 1]`: a sombra da célula já veio da linha da tinta.
    let mut feita = if campo.is_some() && !sombra_antiga {
        vec![false; gw * (cy1 + 1 - cy0)]
    } else {
        Vec::new()
    };
    #[cfg(test)]
    let t_pig = std::time::Instant::now();
    let pig_rows = planos.pigment[(cy0 - 1) * g_stride..cy1 * g_stride].par_chunks_mut(g_stride);
    if let (Some((o, c)), false) = (&campo, feita.is_empty()) {
        pig_rows
            .zip(planos.sombra[(cy0 - 1) * gw..cy1 * gw].par_chunks_mut(gw))
            .zip(feita.par_chunks_mut(gw))
            .enumerate()
            .for_each(|(k, ((row, srow), frow))| {
                ph2d_wet_paint::render::render_pigment_row_oleo_com_sombra(
                    Some(params),
                    layers,
                    visual,
                    o,
                    c,
                    cy0 + k,
                    cx0,
                    cx1,
                    row,
                    srow,
                    frow,
                );
            });
    } else {
        pig_rows.enumerate().for_each(|(k, row)| match &campo {
            Some((o, c)) => ph2d_wet_paint::render::render_pigment_row_oleo(
                Some(params),
                layers,
                visual,
                o,
                c,
                cy0 + k,
                cx0,
                cx1,
                row,
            ),
            None => render_pigment_row_visual(Some(params), layers, visual, cy0 + k, cx0, cx1, row),
        });
    }
    #[cfg(test)]
    let t_pig = t_pig.elapsed();
    #[cfg(test)]
    let t_som = std::time::Instant::now();
    let caixa = (cx0, cy0, cx1, cy1);
    super::composite_oleo::sombra_do_papel(&mut planos.sombra, &campo, &feita, caixa, (gw, gh));
    super::composite_oleo::grao_das_celulas(&mut planos.grao, &campo, e.grao, caixa, (gw, gh));
    let sem_oleo = super::composite_oleo::tinta_sem_oleo(
        e.motor,
        &campo,
        (suave, raio),
        ratio,
        caixa,
        (gw, gh),
    );
    if oleo.is_none() {
        // Sem tinta com óleo na tela o campo guardado só ocupava memória (~28 B por célula, 230 MB a 4K).
        planos.campo_cache = None;
    }
    if let (Some(c), Some(k)) = (proprio.as_ref(), chave) {
        CacheCampo::guarda(
            &mut planos.campo_cache,
            k,
            revisao,
            c,
            (gw, gh),
            (cx0, cy0, cx1, cy1),
        );
    }
    #[cfg(test)]
    contagem::tempos(t_pig, t_som.elapsed());
    #[cfg(test)]
    super::composite::split::note_engine(t_engine.elapsed());
    Some(SaidaDaOtica {
        caixa_cel: (cx0, cy0, cx1, cy1),
        caixa_px: (px0, py0, px1, py1),
        sem_oleo,
    })
}

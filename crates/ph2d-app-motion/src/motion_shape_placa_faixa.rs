//! ⭐ GEF 19 E3 — **a placa colada por FAIXA da ordem total** (lei do dono 01/11: a linha de cima
//! da Hierarchy desenha por baixo). Cada faixa `Family::Motion` cola só as cópias dos objectos dela
//! (o dono de cada cópia, [`super::luz::donos`]), redesenhando as corridas de cada segmento cortadas
//! a essas cópias. Uma faixa só com todos os objectos = a colagem de sempre ([`cola_no_mundo`]).
//!
//! ⚠️ A faixa do TOPO leva também o que não tem dono na ordem (cópias sem objecto vivo e as imagens
//! de grupo, que não têm `fonte`), a camada filtrada inteira e a lente — à altura do objecto Motion
//! mais à frente. Com o filtro da camada ligado as outras faixas não colam nada (a camada é uma só).

use std::ops::Range;

use ph2d_gpu::GpuContext;
use ph2d_render::draw_bands::{Band, Family, FrameOrder};

use super::{Corrida, PlacaDeFormas, cola_no_mundo};
use crate::motion_filter::Engrenagem;
use crate::motion_state::MotionState;

/// A ordem do quadro, para colar as faixas Motion dela.
#[derive(Clone, Copy)]
pub struct Colagem<'a> {
    pub gpu: &'a GpuContext,
    pub ordem: &'a FrameOrder,
    pub bands: &'a [Band],
}

/// O que o topo do Motion ainda tem de fazer ([`cola_o_topo`]).
#[derive(Clone, Copy)]
pub enum Topo<'a> {
    /// Nenhum objecto Motion na ordem: a placa inteira, por cima do mundo (o quadro de sempre).
    SemFaixa,
    /// A faixa do topo é esta.
    Faixa(Colagem<'a>, Band),
}

impl Colagem<'_> {
    /// A faixa Motion mais à frente.
    #[must_use]
    pub fn topo(&self) -> Option<Band> {
        self.bands
            .iter()
            .rev()
            .find(|b| b.family == Family::Motion)
            .copied()
    }

    /// Cola no `alvo` as cópias da faixa Motion `band`.
    pub fn cola_a_faixa(
        &self,
        placa: &mut PlacaDeFormas,
        motion: &MotionState,
        band_blit: &ph2d_render::BandBlit,
        alvo: &wgpu::TextureView,
        band: Band,
    ) {
        if motion.filtro.ligada() {
            return;
        }
        let topo = self.topo() == Some(band);
        let ids = self.ordem.motion_ids_in(band);
        let fora: Vec<u64> = self
            .ordem
            .motion_ranks
            .iter()
            .map(|(b, _)| *b)
            .filter(|b| !ids.contains(b))
            .collect();
        if fora.is_empty() {
            if topo {
                cola_no_mundo(placa, self.gpu, band_blit, alvo);
            }
            return;
        }
        if !placa.ativa {
            return;
        }
        let n = placa
            .do_dispositivo
            .unwrap_or_else(|| u32::try_from(placa.copias.len()).unwrap_or(u32::MAX));
        let donos = super::luz::donos(motion, placa);
        let faixa = if topo {
            complemento(&super::luz::pedidas(&donos, &fora, n), n)
        } else {
            super::luz::pedidas(&donos, &ids, n)
        };
        for k in 0..placa.segmentos() {
            let corridas = corta(&placa.corridas[placa.corridas_do_segmento(k)], &faixa, topo);
            if corridas.is_empty() {
                continue;
            }
            let pousa = placa.pousa_do_segmento(k);
            if let Some(camada) = placa.pinta_corridas(self.gpu, &corridas) {
                let formas = ph2d_render::BandSource::Formas;
                band_blit.blit_pousa(self.gpu, alvo, camada, formas, None, pousa);
            }
        }
    }
}

/// ⭐ A colagem de uma faixa Motion (`Some`), na metade de baixo ou na de cima, ou da placa inteira sem
/// objecto Motion na ordem (`None`). A faixa do TOPO leva a camada filtrada e a lente — no sítio DELA,
/// também abaixo da última faixa de sprites (GEF 19, resto da E3). Uma faixa Palco pousa os palcos dela.
pub fn cola(
    c: Colagem<'_>,
    (placa, motion, palcos): (
        &mut PlacaDeFormas,
        &mut MotionState,
        &mut crate::macro_track::palcos::PalcosDasMacros,
    ),
    e: Engrenagem<'_>,
    cena: Option<[f32; 4]>,
    band: Option<Band>,
) {
    if let Some(b) = band.filter(|b| b.family == Family::Palco) {
        let ids = c.ordem.palco_ids_in(b);
        let (gpu, mundo) = (e.gpu, e.mundo);
        palcos.pousa(
            crate::macro_track::palcos::Colagem {
                gpu,
                renderer: e.renderer,
                camera: e.camera,
                window: e.window,
                scene_viewport: e.scene_viewport,
                mundo,
                band_blit: e.band_blit,
            },
            |m| ids.contains(&m),
        );
        return palcos.lentes(gpu, mundo, cena, |m| ids.contains(&m));
    }
    if let Some(b) = band.filter(|b| c.topo() != Some(*b)) {
        return c.cola_a_faixa(placa, motion, e.band_blit, e.mundo.blend_view(), b);
    }
    let quem = band.map_or(Topo::SemFaixa, |b| Topo::Faixa(c, b));
    cola_o_topo(motion, placa, e, cena, quem);
}

/// ⭐ O TOPO do Motion: a camada filtrada (inteira) ou a faixa do topo, e a lente sobre o mundo.
pub fn cola_o_topo(
    motion: &mut MotionState,
    placa: &mut PlacaDeFormas,
    e: Engrenagem<'_>,
    cena: Option<[f32; 4]>,
    topo: Topo<'_>,
) {
    let (gpu, mundo, band_blit) = (e.gpu, e.mundo, e.band_blit);
    let filtrada = motion.filtra_a_camada(Engrenagem {
        formas: Some(&*placa),
        ..e
    });
    if !filtrada {
        let alvo = mundo.blend_view();
        match topo {
            Topo::SemFaixa => cola_no_mundo(placa, gpu, band_blit, alvo),
            Topo::Faixa(c, band) => c.cola_a_faixa(placa, motion, band_blit, alvo, band),
        }
    }
    motion.lente.aplica(gpu, mundo, cena);
}

/// O que sobra de `0..n` fora de `r` (ordenadas e disjuntas, como as [`super::luz::pedidas`]).
fn complemento(r: &[Range<u32>], n: u32) -> Vec<Range<u32>> {
    let mut out = Vec::new();
    let mut em = 0;
    for f in r {
        if f.start > em {
            out.push(em..f.start);
        }
        em = em.max(f.end);
    }
    if em < n {
        out.push(em..n);
    }
    out
}

/// As `corridas` cortadas às cópias de `faixa`; as imagens ficam só com a faixa do topo.
fn corta(corridas: &[Corrida], faixa: &[Range<u32>], imagens: bool) -> Vec<Corrida> {
    let mut out = Vec::new();
    for c in corridas {
        let mut leva_imagens = imagens && !c.imagens.is_empty();
        for r in faixa {
            let (ini, fim) = (c.copias.start.max(r.start), c.copias.end.min(r.end));
            if ini < fim {
                let imagens = if leva_imagens {
                    c.imagens.clone()
                } else {
                    0..0
                };
                leva_imagens = false;
                out.push(Corrida {
                    copias: ini..fim,
                    imagens,
                    ..c.clone()
                });
            }
        }
        if leva_imagens {
            out.push(Corrida {
                copias: c.copias.start..c.copias.start,
                ..c.clone()
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corrida(copias: Range<u32>, imagens: Range<u32>) -> Corrida {
        Corrida {
            copias,
            imagens,
            ..Corrida::default()
        }
    }

    /// ⭐ As corridas partem-se pela faixa: cada faixa leva só as cópias dela, as duas juntas cobrem
    /// tudo, e as imagens (sem dono) só vão com o topo.
    #[test]
    fn as_corridas_cortam_se_pelas_copias_da_faixa() {
        let cs = [corrida(0..4, 0..2), corrida(4..6, 0..0)];
        let baixo = [1..3];
        let topo = complemento(&baixo, 6);
        assert_eq!(topo, vec![0..1, 3..6]);
        assert_eq!(corta(&cs, &baixo, false), vec![corrida(1..3, 0..0)]);
        assert_eq!(
            corta(&cs, &topo, true),
            vec![
                corrida(0..1, 0..2),
                corrida(3..4, 0..0),
                corrida(4..6, 0..0)
            ]
        );
        let so_imagens = [corrida(2..2, 0..3)];
        assert!(corta(&so_imagens, &baixo, false).is_empty());
        assert_eq!(corta(&so_imagens, &topo, true), vec![corrida(2..2, 0..3)]);
    }
}

//! ⭐ GEF 19 E3 — **a placa colada por FAIXA da ordem total** (lei do dono 01/11: a linha de cima
//! da Hierarchy desenha por baixo). Cada faixa `Family::Motion` cola só as cópias dos objectos dela
//! e as imagens de grupo deles (o dono de cada cópia e de cada imagem, [`super::luz::donos`] e
//! [`super::luz::donos_das_imagens`]), redesenhando as corridas de cada segmento cortadas a elas. Uma
//! faixa só com todos os objectos = a colagem de sempre ([`cola_no_mundo`]).
//!
//! ⚠️ A faixa do TOPO leva também o que não tem dono na ordem (cópias e imagens sem objecto vivo), a
//! camada filtrada inteira e a lente — à altura do objecto Motion mais à frente. Com o filtro da camada ligado as outras faixas não colam nada (a camada é uma só).

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
        let n_img = match (placa.do_dispositivo, motion.gpu_cook.imagens_do_grupo()) {
            (Some(_), Some(i)) => i.len(),
            (Some(_), None) => 0,
            (None, _) => u32::try_from(placa.imagens_cpu.len()).unwrap_or(u32::MAX),
        };
        let das = |donos: &[(u64, Range<u32>)], n| {
            if topo {
                complemento(&super::luz::pedidas(donos, &fora, n), n)
            } else {
                super::luz::pedidas(donos, &ids, n)
            }
        };
        let faixa = das(&super::luz::donos(motion, placa), n);
        let imagens = das(&super::luz::donos_das_imagens(motion, placa), n_img);
        for k in 0..placa.segmentos() {
            let corridas = corta(
                &placa.corridas[placa.corridas_do_segmento(k)],
                &faixa,
                &imagens,
            );
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

/// As `corridas` cortadas às cópias de `faixa` e às imagens de `imagens` (o dono de cada uma). Os
/// pedaços emparelham-se pela ordem: uma corrida ORDENADA (a linha `j` em `copias.start + j` e
/// `imagens.start + j`) corta as duas pelos mesmos sinks, logo continua alinhada.
fn corta(corridas: &[Corrida], faixa: &[Range<u32>], imagens: &[Range<u32>]) -> Vec<Corrida> {
    let pedacos = |de: &Range<u32>, faixa: &[Range<u32>]| -> Vec<Range<u32>> {
        faixa
            .iter()
            .map(|r| de.start.max(r.start)..de.end.min(r.end))
            .filter(|r| !r.is_empty())
            .collect()
    };
    let mut out = Vec::new();
    for c in corridas {
        let (cs, is) = (pedacos(&c.copias, faixa), pedacos(&c.imagens, imagens));
        for k in 0..cs.len().max(is.len()) {
            out.push(Corrida {
                copias: cs.get(k).cloned().unwrap_or(c.copias.start..c.copias.start),
                imagens: is.get(k).cloned().unwrap_or(0..0),
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

    /// ⭐ As corridas partem-se pela faixa: cada faixa leva só as cópias e as imagens dos objectos
    /// dela, e as duas juntas cobrem tudo. (GEF 19 E3 resto c: antes as imagens iam SEMPRE com o topo.)
    #[test]
    fn as_corridas_cortam_se_pelas_copias_e_imagens_da_faixa() {
        let cs = [
            corrida(0..4, 0..0),
            corrida(4..6, 0..0),
            corrida(6..6, 0..3),
        ];
        let (baixo, baixo_img) = ([1..3], [1..2]);
        let (topo, topo_img) = (complemento(&baixo, 6), complemento(&baixo_img, 3));
        assert_eq!(topo, vec![0..1, 3..6]);
        assert_eq!(
            corta(&cs, &baixo, &baixo_img),
            vec![corrida(1..3, 0..0), corrida(6..6, 1..2)]
        );
        assert_eq!(
            corta(&cs, &topo, &topo_img),
            vec![
                corrida(0..1, 0..0),
                corrida(3..4, 0..0),
                corrida(4..6, 0..0),
                corrida(6..6, 0..1),
                corrida(6..6, 2..3),
            ]
        );
    }

    /// ⭐ Uma corrida ORDENADA (cópias e imagens alinhadas linha a linha) continua alinhada depois do
    /// corte: o sink do meio sai das duas listas no mesmo sítio.
    #[test]
    fn a_corrida_ordenada_continua_alinhada() {
        let cs = [corrida(10..16, 20..26)];
        assert_eq!(
            corta(&cs, &[10..12, 14..16], &[20..22, 24..26]),
            vec![corrida(10..12, 20..22), corrida(14..16, 24..26)]
        );
        assert_eq!(
            corta(&cs, &[12..14], &[22..24]),
            vec![corrida(12..14, 22..24)]
        );
    }
}

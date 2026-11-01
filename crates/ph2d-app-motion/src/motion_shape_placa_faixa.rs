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
    /// GEF 19 A: as cópias de SPRITE colam-se na faixa do dono (ordem intercalada); `false` = vão no
    /// passe fundido, por baixo das formas (o quadro de sempre).
    pub sprites: bool,
}

/// GEF 19 A — **as cópias de SPRITE vão por faixa?** Só com a ordem intercalada e um objecto Motion
/// nela; senão vão no passe fundido, por baixo das formas (o quadro de sempre, byte a byte).
#[must_use]
pub fn sprites_por_faixa(ativo: bool, bands: &[Band]) -> bool {
    ativo
        && ph2d_render::draw_bands::needs_banding(bands)
        && bands.iter().any(|b| b.family == Family::Motion)
}

/// O `game_rt` HDR e o tonemap de uma faixa de sprites — as cópias de sprite do Motion seguem o
/// mesmo caminho até ao mundo (HDR → tonemap → colagem).
pub struct PasseDeSprites<'a> {
    pub game_rt: &'a ph2d_render::GameRt,
    pub tonemap: &'a mut ph2d_render::Tonemap,
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
    /// Quem a faixa Motion `band` leva: `(é a do topo, os objectos dela, os de fora)`.
    fn quem(&self, band: Band) -> (bool, Vec<u64>, Vec<u64>) {
        let ids = self.ordem.motion_ids_in(band);
        let fora = (self.ordem.motion_ranks.iter())
            .map(|(b, _)| *b)
            .filter(|b| !ids.contains(b))
            .collect();
        (self.topo() == Some(band), ids, fora)
    }

    /// ⭐ GEF 19 A — **as cópias de SPRITE da faixa** (moldes imagem: `pump.instances` na CPU, o buffer
    /// de instâncias do cozimento no dispositivo), pelo dono do sink de cada uma, desenhadas como uma
    /// faixa de sprites. Antes iam no passe fundido da última faixa de sprites, sem rank: por cima de
    /// tudo o que essa faixa tinha (o Card do `PH2D_ORDEM_SMOKE=2`).
    pub fn cola_os_sprites(
        &self,
        motion: &MotionState,
        e: &mut Engrenagem<'_>,
        passe: PasseDeSprites<'_>,
        band: Band,
    ) {
        if !self.sprites || motion.filtro.ligada() {
            return;
        }
        let (topo, ids, fora) = self.quem(band);
        let quem = (topo, &ids[..], &fora[..]);
        let dono = |sink| crate::motion_objeto::origem::dono_do_no(motion, sink).map(|d| d.entity);
        let (gpu, cpu) = motion.sprites_da_camada(true);
        let (fatia, runs, buffer);
        if let Some((b, n, todos)) = gpu {
            let (donos, _) = motion.gpu_cook.faixas_dos_sinks();
            let donos: Vec<_> = (donos.iter())
                .filter_map(|(s, r)| Some((dono(*s)?, r.clone())))
                .collect();
            let atlas = [ph2d_render::GpuTexRun {
                texture_id: ph2d_render::RenderInstance::ATLAS_TEXTURE_ID,
                start: 0,
                end: n,
                blend: 0,
                sampling: 0,
            }];
            let todos = if todos.is_empty() { &atlas[..] } else { todos };
            (fatia, runs, buffer) = (
                Vec::new(),
                corta_runs(todos, &leva(quem, &donos, n)),
                Some((b, n)),
            );
        } else {
            let u = |r: &Range<usize>| {
                u32::try_from(r.start).unwrap_or(u32::MAX)..u32::try_from(r.end).unwrap_or(u32::MAX)
            };
            let donos: Vec<_> = (motion.pump.faixas.iter())
                .filter_map(|f| Some((dono(f.sink)?, f)))
                .flat_map(|(d, f)| [(d, u(&f.instancias)), (d, u(&f.ladrilhos))])
                .collect();
            let n = u32::try_from(cpu.len()).unwrap_or(u32::MAX);
            let r = leva(quem, &donos, n);
            fatia = r
                .iter()
                .flat_map(|r| &cpu[r.start as usize..r.end as usize])
                .copied()
                .collect();
            (runs, buffer) = (Vec::new(), None);
        }
        let placa = buffer.map(|(b, n)| (b, n, &runs[..]));
        if fatia.is_empty() && runs.is_empty() {
            return;
        }
        passe.game_rt.clear(e.gpu);
        let transparente = wgpu::Color::TRANSPARENT;
        let (camera, janela, vista) = (e.camera, e.window, e.scene_viewport);
        e.renderer.render_instances_da_cena(
            passe.game_rt.view(),
            camera,
            janela,
            transparente,
            &fatia,
            placa,
            vista,
        );
        passe.tonemap.run(e.gpu);
        let (alvo, fonte) = (e.mundo.blend_view(), passe.tonemap.output_view());
        e.band_blit
            .blit(e.gpu, alvo, fonte, ph2d_render::BandSource::Sprites);
    }

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
        let (topo, ids, fora) = self.quem(band);
        if fora.is_empty() {
            if topo {
                cola_no_mundo(placa, self.gpu, band_blit, alvo);
            }
            return;
        }
        // ⚠️ `desenhou` e não `ativa`: o `desenha` (antes das faixas) consome a decisão.
        if !placa.desenhou() {
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
        let das = |donos: &[(u64, Range<u32>)], n| leva((topo, &ids, &fora), donos, n);
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
    mut e: Engrenagem<'_>,
    cena: Option<[f32; 4]>,
    (band, sprites): (Option<Band>, PasseDeSprites<'_>),
) {
    if let Some(b) = band.filter(|b| b.family == Family::Motion) {
        c.cola_os_sprites(motion, &mut e, sprites, b);
    }
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

/// As faixas de `0..n` que a faixa Motion leva: a do topo leva tudo o que não é de um objecto de
/// fora (as cópias sem dono vivo incluídas); as outras, só as dos objectos delas.
fn leva(
    (topo, ids, fora): (bool, &[u64], &[u64]),
    donos: &[(u64, Range<u32>)],
    n: u32,
) -> Vec<Range<u32>> {
    if topo {
        complemento(&super::luz::pedidas(donos, fora, n), n)
    } else {
        super::luz::pedidas(donos, ids, n)
    }
}

/// Os `runs` de textura cortados às faixas `leva` (ordenadas e disjuntas): cada pedaço mantém a
/// textura, a mistura e a amostragem do run de onde saiu.
fn corta_runs(runs: &[ph2d_render::GpuTexRun], leva: &[Range<u32>]) -> Vec<ph2d_render::GpuTexRun> {
    let mut out = Vec::new();
    for r in runs {
        for f in leva {
            let (start, end) = (r.start.max(f.start), r.end.min(f.end));
            if start < end {
                out.push(ph2d_render::GpuTexRun { start, end, ..*r });
            }
        }
    }
    out
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
#[path = "motion_shape_placa_faixa_gpu_tests.rs"]
mod gpu_tests;

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

    fn run(texture_id: u32, r: Range<u32>, blend: u8) -> ph2d_render::GpuTexRun {
        ph2d_render::GpuTexRun {
            texture_id,
            start: r.start,
            end: r.end,
            blend,
            sampling: 7,
        }
    }

    /// ⭐ GEF 19 A — as cópias de SPRITE da faixa: os runs de textura cortam-se às faixas dos donos e
    /// cada pedaço guarda a textura, a mistura e a amostragem do run de onde saiu.
    #[test]
    fn os_runs_de_textura_cortam_se_pelos_donos() {
        let runs = [run(0, 0..4, 0), run(9, 4..10, 2)];
        assert_eq!(
            corta_runs(&runs, &[2..6, 8..12]),
            vec![run(0, 2..4, 0), run(9, 4..6, 2), run(9, 8..10, 2)]
        );
        assert!(corta_runs(&runs, &[]).is_empty());
    }

    /// ⭐ A faixa do TOPO leva o que não é de um objecto de fora (as cópias sem dono incluídas); as
    /// outras, só as dos objectos delas. As duas juntas cobrem tudo, sem repetir.
    #[test]
    fn o_topo_leva_o_resto_e_as_outras_so_as_delas() {
        let donos = [(1, 0..3), (2, 3..5), (1, 7..8)];
        let topo = leva((true, &[2], &[1]), &donos, 10);
        let baixo = leva((false, &[1], &[2]), &donos, 10);
        assert_eq!(topo, vec![3..7, 8..10]);
        assert_eq!(baixo, vec![0..3, 7..8]);
    }

    /// ⭐ As cópias de sprite só saem do passe fundido com a ordem INTERCALADA e um objecto Motion
    /// nela: sem intercalação o quadro é o de sempre, byte a byte. (Mutação: sem o `needs_banding`.)
    #[test]
    fn as_copias_de_sprite_vao_por_faixa_so_com_a_ordem_intercalada() {
        let b = |family, lo, hi| Band { family, lo, hi };
        let intercalada = [b(Family::Motion, 0, 1), b(Family::Sprite, 1, 2)];
        let de_sempre = [b(Family::Sprite, 0, 1), b(Family::Motion, 1, 2)];
        let sem_motion = [b(Family::Vector, 0, 1), b(Family::Sprite, 1, 2)];
        assert!(sprites_por_faixa(true, &intercalada));
        assert!(!sprites_por_faixa(false, &intercalada), "o Motion parado");
        assert!(!sprites_por_faixa(true, &de_sempre));
        assert!(!sprites_por_faixa(true, &sem_motion));
    }
}

//! **A FORMA DA BORDA da aguada** (BUGS_painter #31, 2026-10-02): para onde a silhueta é empurrada, com
//! UMA porta — [`EdgeFlow::desloca`] —, pelo centro e pelas 9 amostras do AA (cortar amostras foi
//! recusado por LOOK, doc 28 §5.10). Duas parcelas:
//!
//! * o **Ragged Edge** segue um padrão de **Flow**. `Classic` (`kind: None`) é o [`warp_offset`] de
//!   sempre, chamado verbatim (byte-idêntico). Um padrão desloca pelo VALOR da textura — a lei do
//!   `feDisplacementMap` do SVG e do *Displacement Map* do After Effects: dois canais (o padrão e ele
//!   rodado 90° e meio ladrilho ao lado). Cada padrão é normalizado ao Classic nas DUAS réguas: o RMS
//!   do deslocamento (para «Ragged Edge N px» empurrar o mesmo) e o RMS do GRADIENTE dele (para o
//!   Flow Size `1` ter a escala de detalhe do Classic — senão Wood/Marble/Musgrave, mais finos ao mesmo
//!   Size, rasgavam a borda: salto entre colunas `1,43 · 2,30 · 2,57 px` contra `0,38`);
//! * o **Paper Edge** soma o deslocamento pelo PAPEL do dono, com a amplitude em que o gradiente do
//!   deslocamento tem RMS `1` — o início da DOBRA: a `1` a borda segue o dente o mais que pode sem rasgar.
//!
//! ⛔ **Recusas MEDIDAS** (2026-10-02, faixa larga r=30, Ragged 24): deslocar por `−∇T` (a semântica dos
//! *flow maps*) RASGA a borda — o gradiente pesa as oitavas finas, salto entre colunas `1,73 px`
//! (Clouds) contra `0,38` do Classic, a borda em confete. E mover a janela de endurecimento `(SS0, SS1)`
//! pela altura do papel quase não se vê — desvio da borda `0,45 → 0,65 px` no Paper Rough: o pé da
//! cobertura tem 1-2 px, e é só isso que um limiar alcança.

use super::watercolor_field::{WetStrokeStyle, box_blur, sample_bilinear};
use super::watercolor_noise::{NoiseTile, paper_height, snap_slot_size, warp_offset};
use ph2d_painter_brush::texture::{ImageMask, angle_basis, sample_tiled_rot_wrapped};
use ph2d_painter_brush::{TEX_TILE_BASE_PX, TextureKind, TextureSettings};
use std::sync::{Mutex, OnceLock};

/// O que dá a forma da borda de um dono — a chave que separa as fontes e os pesos da junção.
#[derive(Clone, Copy, PartialEq)]
struct Estilo {
    flow: TextureSettings,
    /// O papel do dono quando o Paper Edge dele é `> 0` (senão `None`), e esse Paper Edge.
    papel: Option<(TextureSettings, f32)>,
}

impl Estilo {
    fn de(s: &WetStrokeStyle) -> Self {
        Self {
            flow: s.edge_flow,
            papel: (s.paper_edge > 0.0).then_some((s.paper, s.paper_edge)),
        }
    }
}

/// Um mapa `[−1, 1]²` por texel numa caixa do canvas, lido bilinear entre texels.
struct Mapa {
    d: Vec<[f32; 2]>,
    x0: i64,
    y0: i64,
    w: usize,
    h: usize,
}

/// As imagens que a borda pode ler — o Paper e o Flow carregados pelo menu "Use as …" —, cada uma com a
/// VERSÃO que identifica o conteúdo (o memo das estatísticas chaveia por ela: duas imagens com as mesmas
/// settings são duas leis, ADR-0124).
#[derive(Clone, Copy, Default)]
pub(super) struct Imagens<'a> {
    pub(super) papel: Option<ImageMask<'a>>,
    pub(super) papel_versao: u64,
    pub(super) fluxo: Option<ImageMask<'a>>,
    pub(super) fluxo_versao: u64,
}

impl super::PainterTool {
    /// As imagens da borda, emprestadas do estado do pincel.
    pub(super) fn imagens_da_borda(&self) -> Imagens<'_> {
        Imagens {
            papel: self.paint.paper_image.as_ref().map(|i| i.as_mask()),
            papel_versao: self.paint.paper_image_version,
            fluxo: self.paint.flow_map.imagem.as_ref().map(|i| i.as_mask()),
            fluxo_versao: self.paint.flow_map.versao,
        }
    }
}

/// O mapa do **Flow** carregado pelo "Use as Flow" (`edge_flow.kind == Image`) e a VERSÃO dele — sobe a
/// cada instalação, e é por ela (nunca pelo endereço) que o memo das estatísticas o reconhece.
#[derive(Default)]
pub(super) struct MapaDoFluxo {
    pub(super) imagem: Option<super::brush_settings::BrushTextureImage>,
    pub(super) versao: u64,
}

impl MapaDoFluxo {
    pub(super) fn instala(&mut self, imagem: super::brush_settings::BrushTextureImage) {
        self.imagem = Some(imagem);
        self.versao = self.versao.wrapping_add(1);
    }
}

/// Largura da pré-visualização do Flow, em texels (`3 : 1`; o painel desenha-a no cartão Wash).
const PREVIEW_W: u32 = 192;
/// Altura da pré-visualização de um padrão (a janela de canvas é `PREVIEW_W × PREVIEW_H` px, 1:1).
const PREVIEW_H: u32 = 64;

impl super::PainterTool {
    /// **A pré-visualização do Flow** — o canal X do deslocamento que a borda vai ler, pela MESMA porta
    /// (`Classic` = o [`warp_offset`]; um padrão ou a imagem = o [`Amostrador::flow`]), em cinza
    /// `½ + u/2`. Um padrão mostra uma janela de canvas 1:1 (`PREVIEW_W × PREVIEW_H` px: a escala real
    /// do detalhe); a imagem do "Use as Flow" mostra-se INTEIRA, no aspecto dela (largura `PREVIEW_W`).
    #[must_use]
    pub fn edge_flow_preview(&self) -> (Vec<u8>, u32, u32) {
        let s = self.paint.brush.edge_flow;
        let imagens = self.imagens_da_borda();
        let cinza = |u: f32| ((0.5 + 0.5 * u.clamp(-1.0, 1.0)) * 255.0 + 0.5) as u8;
        if s.kind == TextureKind::None {
            let lum = (0..PREVIEW_W * PREVIEW_H)
                .map(|i| {
                    let (x, y) = ((i % PREVIEW_W) as f32, (i / PREVIEW_W) as f32);
                    cinza(warp_offset(x, y, NoiseTile::NONE).0)
                })
                .collect();
            return (lum, PREVIEW_W, PREVIEW_H);
        }
        let Some(a) = Amostrador::flow(s, &imagens, NoiseTile::NONE) else {
            return (
                vec![128; (PREVIEW_W * PREVIEW_H) as usize],
                PREVIEW_W,
                PREVIEW_H,
            );
        };
        // A janela: 1:1 para um padrão; a extensão da imagem (`W/Size`) para o "Use as Flow".
        let (jw, jh) = match (s.kind, imagens.fluxo.as_ref()) {
            (TextureKind::Image, Some(m)) => (
                (m.width as f32 / s.size[0].max(1e-3)).max(1.0),
                (m.height as f32 / s.size[1].max(1e-3)).max(1.0),
            ),
            _ => (PREVIEW_W as f32, PREVIEW_H as f32),
        };
        let h = ((PREVIEW_W as f32 * jh / jw).round() as u32).clamp(8, 3 * PREVIEW_H);
        let e = a.estat();
        let lum = (0..PREVIEW_W * h)
            .map(|i| {
                let x = ((i % PREVIEW_W) as f32 + 0.5) * jw / PREVIEW_W as f32;
                let y = ((i / PREVIEW_W) as f32 + 0.5) * jh / h as f32;
                cinza(a.unidade(&e, x as i64, y as i64).0)
            })
            .collect();
        (lum, PREVIEW_W, h)
    }

    /// Muda quando a [`Self::edge_flow_preview`] muda: as settings do Flow e a versão da imagem dele.
    #[must_use]
    pub fn edge_flow_preview_key(&self) -> u64 {
        let s = self.paint.brush.edge_flow;
        let mut h = 0xcbf2_9ce4_8422_2325_u64;
        let mut mistura = |v: u64| h = (h ^ v).wrapping_mul(0x0000_0100_0000_01b3);
        mistura(u64::from(s.kind.to_u8()));
        mistura(u64::from(s.angle_deg));
        for f in s.size.iter().chain(&s.offset).chain(&s.params) {
            mistura(u64::from(f.to_bits()));
        }
        mistura(self.paint.flow_map.versao);
        h
    }
}

/// De que imagem um [`Amostrador`] lê (a outra metade da chave do memo).
const SEM_IMAGEM: (u8, u64) = (0, 0);

/// O deslocamento de UM [`Estilo`]: o fluxo (`None` = Classic, analítico) e o papel em px.
struct Fonte {
    flow: Option<Mapa>,
    /// O mapa do papel e o fator `Paper Edge · amplitude da dobra` (px por unidade do mapa).
    papel: Option<(Mapa, f32)>,
}

/// **O FLUXO da composição** — uma [`Fonte`] por estilo DISTINTO entre os donos (quase sempre uma) e,
/// com duas ou mais, o peso de cada uma suavizado na junção com o raio do campo de estilo da #18,
/// senão a junção de dois estilos degrauava o deslocamento.
pub(super) struct EdgeFlow {
    fontes: Vec<Fonte>,
    /// `dono → índice da fonte` (`0` = sem dono = o pincel vivo).
    do_dono: Vec<u8>,
    /// Com 2+ fontes: o peso de cada uma por texel da janela (a soma é `1` — todo texel tem um dono
    /// mais próximo).
    pesos: Option<Vec<Vec<f32>>>,
    rw: usize,
    rh: usize,
    tile: NoiseTile,
}

impl EdgeFlow {
    /// Monta o fluxo da composição. `caixa` = a caixa de SAÍDA `(x0, y0, bw, bh)` em texels do canvas (os
    /// mapas cobrem-na com 1 texel de folga para o bilinear do AA); `janela` = a janela de leitura
    /// `(rx0, ry0, rw, rh)` onde moram os pesos. O `bool` diz se algum dono pede o papel na borda.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build(
        cur: &WetStrokeStyle,
        table: &[WetStrokeStyle],
        owner: Option<&[u8]>,
        imagens: Imagens<'_>,
        fw: usize,
        caixa: (usize, usize, usize, usize),
        janela: (usize, usize, usize, usize),
        tile: NoiseTile,
    ) -> (Self, bool) {
        let mut distintos = vec![Estilo::de(cur)];
        let mut do_dono = vec![0u8; table.len() + 1];
        if owner.is_some() {
            for (i, s) in table.iter().enumerate() {
                let e = Estilo::de(s);
                let k = distintos.iter().position(|d| *d == e).unwrap_or_else(|| {
                    distintos.push(e);
                    distintos.len() - 1
                });
                do_dono[i + 1] = k as u8;
            }
        }
        let papel = distintos.iter().any(|e| e.papel.is_some());
        let fontes = distintos
            .iter()
            .map(|e| Fonte {
                flow: (e.flow.kind != TextureKind::None)
                    .then(|| Amostrador::flow(e.flow, &imagens, tile))
                    .flatten()
                    .map(|a| a.mapa(caixa)),
                papel: e.papel.map(|(p, k)| {
                    let a = Amostrador::papel(p, &imagens, tile);
                    let f = k.min(1.0) * a.estat().dobra;
                    (a.mapa(caixa), f)
                }),
            })
            .collect();
        let (rx0, ry0, rw, rh) = janela;
        // ⚠️ Os pesos saem do dono MAIS PRÓXIMO, não do dono do texel: o Ragged puxa a cobertura de até
        // `warp` px para fora da pegada, e esses texels SEM dono caíam no estilo do pincel vivo — trocar o
        // Flow para o traço seguinte reformava a orla do anterior (medido: 15 linhas do traço de cima).
        let pesos = (distintos.len() > 1)
            .then_some(())
            .and(owner)
            .and_then(|owner| {
                let mut local = vec![0u8; rw * rh];
                for wy in 0..rh {
                    let base = (ry0 + wy) * fw + rx0;
                    local[wy * rw..(wy + 1) * rw].copy_from_slice(&owner[base..base + rw]);
                }
                let perto = dono_mais_proximo(&local, rw, rh)?;
                let r = super::watercolor_rewet_px::WET_FIELD_BLUR_PX;
                let planos: Vec<Vec<f32>> = (0..distintos.len())
                    .map(|k| {
                        let p: Vec<f32> = perto
                            .iter()
                            .map(|&o| {
                                f32::from(
                                    do_dono[(o as usize).min(do_dono.len() - 1)] as usize == k,
                                )
                            })
                            .collect();
                        box_blur(&p, rw, rh, r)
                    })
                    .collect();
                Some(planos)
            });
        let fluxo = Self {
            fontes,
            do_dono,
            pesos,
            rw,
            rh,
            tile,
        };
        (fluxo, papel)
    }

    /// **A porta do deslocamento da borda, em px.** `dono` é o dono PRÉ-warp do texel; `(lx, ly)` o
    /// texel na janela de leitura; `(x, y)` a amostra em texels do canvas (o centro ou uma das 9 do AA);
    /// `amp` a amplitude do Ragged Edge (já suavizada na junção).
    #[inline]
    pub(super) fn desloca(
        &self,
        dono: u8,
        lx: f32,
        ly: f32,
        x: f32,
        y: f32,
        amp: f32,
    ) -> (f32, f32) {
        if let Some(planos) = &self.pesos {
            let (mut dx, mut dy) = (0.0, 0.0);
            for (fonte, plano) in self.fontes.iter().zip(planos) {
                let w = sample_bilinear(plano, self.rw, self.rh, lx, ly);
                if w > 0.0 {
                    let (a, b) = fonte.desloca(x, y, amp, self.tile);
                    dx += w * a;
                    dy += w * b;
                }
            }
            return (dx, dy);
        }
        let k = self.do_dono[(dono as usize).min(self.do_dono.len() - 1)] as usize;
        self.fontes[k].desloca(x, y, amp, self.tile)
    }
}

/// O dono MAIS PRÓXIMO de cada texel (transformada de distância chanfrada 3-4, duas passagens — exacta
/// ao inteiro e determinística). `None` quando nenhum texel da janela tem dono.
fn dono_mais_proximo(owner: &[u8], w: usize, h: usize) -> Option<Vec<u8>> {
    const INF: u32 = u32::MAX / 2;
    let mut dist: Vec<u32> = owner
        .iter()
        .map(|&o| if o != 0 { 0 } else { INF })
        .collect();
    if dist.iter().all(|&d| d == INF) {
        return None;
    }
    let mut perto = owner.to_vec();
    let relaxa = |i: usize, j: usize, custo: u32, dist: &mut [u32], perto: &mut [u8]| {
        if dist[j] + custo < dist[i] {
            dist[i] = dist[j] + custo;
            perto[i] = perto[j];
        }
    };
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if x > 0 {
                relaxa(i, i - 1, 3, &mut dist, &mut perto);
            }
            if y > 0 {
                relaxa(i, i - w, 3, &mut dist, &mut perto);
                if x > 0 {
                    relaxa(i, i - w - 1, 4, &mut dist, &mut perto);
                }
                if x + 1 < w {
                    relaxa(i, i - w + 1, 4, &mut dist, &mut perto);
                }
            }
        }
    }
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let i = y * w + x;
            if x + 1 < w {
                relaxa(i, i + 1, 3, &mut dist, &mut perto);
            }
            if y + 1 < h {
                relaxa(i, i + w, 3, &mut dist, &mut perto);
                if x + 1 < w {
                    relaxa(i, i + w + 1, 4, &mut dist, &mut perto);
                }
                if x > 0 {
                    relaxa(i, i + w - 1, 4, &mut dist, &mut perto);
                }
            }
        }
    }
    Some(perto)
}

impl Fonte {
    #[inline]
    fn desloca(&self, x: f32, y: f32, amp: f32, tile: NoiseTile) -> (f32, f32) {
        let (wx, wy) = if amp > 0.0 {
            match &self.flow {
                None => warp_offset(x, y, tile),
                Some(m) => m.le(x, y),
            }
        } else {
            (0.0, 0.0)
        };
        let (dx, dy) = (wx * amp, wy * amp);
        match &self.papel {
            None => (dx, dy),
            Some((m, f)) => {
                let (px, py) = m.le(x, y);
                (dx + px * f, dy + py * f)
            }
        }
    }
}

impl Mapa {
    #[inline]
    fn le(&self, x: f32, y: f32) -> (f32, f32) {
        let fx = (x - self.x0 as f32).clamp(0.0, (self.w - 1) as f32);
        let fy = (y - self.y0 as f32).clamp(0.0, (self.h - 1) as f32);
        let (ix, iy) = (fx.floor() as usize, fy.floor() as usize);
        let (ix1, iy1) = ((ix + 1).min(self.w - 1), (iy + 1).min(self.h - 1));
        let (tx, ty) = (fx - ix as f32, fy - iy as f32);
        let d = &self.d;
        let w = self.w;
        let lerp = |c: usize| {
            let a = d[iy * w + ix][c] + (d[iy * w + ix1][c] - d[iy * w + ix][c]) * tx;
            let b = d[iy1 * w + ix][c] + (d[iy1 * w + ix1][c] - d[iy1 * w + ix][c]) * tx;
            a + (b - a) * ty
        };
        (lerp(0), lerp(1))
    }
}

/// O mapa de onde sai um deslocamento: um padrão de Flow, o slot Paper, ou o papel interno (o fallback
/// que a aguada usa quando nenhum papel foi escolhido).
enum Amostrador<'a> {
    Textura {
        s: [TextureSettings; 2],
        rot: [[f32; 2]; 2],
        img: Option<&'a ImageMask<'a>>,
        /// `(fonte, versão)` da imagem lida — [`SEM_IMAGEM`] para um procedural.
        id_img: (u8, u64),
        period: [f32; 2],
    },
    PapelInterno(NoiseTile),
}

/// A distância entre os dois canais do papel interno — qualquer distância fora da grelha de 5/2,5 px
/// do [`paper_height`] decorrela; a periodicidade do ladrilho sobrevive a uma translação.
const PAPEL_INTERNO_CANAL_Y: (f32, f32) = (517.5, 291.25);

impl<'a> Amostrador<'a> {
    /// Um padrão de Flow, com o Size do artista multiplicado pela escala que leva o gradiente do
    /// padrão ao do Classic (o Flow Size `1` = a escala de detalhe do Classic). Uma IMAGEM ("Use as
    /// Flow") não se normaliza na escala: a Size `1` ela cobre a tela 1:1, com a origem no canto (o
    /// `sample_image` repete a cada 2 unidades de `rel`, i.e. `512/Size` px, centrado em `rel = 0`). Sem
    /// a imagem carregada (`None`) o Flow cai no Classic.
    fn flow(s: TextureSettings, imagens: &'a Imagens<'a>, tile: NoiseTile) -> Option<Self> {
        if s.kind == TextureKind::Image {
            let img = imagens.fluxo.as_ref()?;
            let um = |n: u32| 2.0 * TEX_TILE_BASE_PX / n.max(1) as f32;
            let s = TextureSettings {
                size: [s.size[0] * um(img.width), s.size[1] * um(img.height)],
                offset: [s.offset[0] - 1.0, s.offset[1] - 1.0],
                ..s
            };
            return Some(Self::textura(s, Some(img), (2, imagens.fluxo_versao), tile));
        }
        let base = escala_do_flow(&s);
        let s = TextureSettings {
            size: [s.size[0] * base, s.size[1] * base],
            ..s
        };
        Some(Self::textura(s, None, SEM_IMAGEM, tile))
    }

    fn papel(s: TextureSettings, imagens: &'a Imagens<'a>, tile: NoiseTile) -> Self {
        if s.is_active() {
            let id = if s.kind == TextureKind::Image {
                (1, imagens.papel_versao)
            } else {
                SEM_IMAGEM
            };
            Self::textura(s, imagens.papel.as_ref(), id, tile)
        } else {
            Self::PapelInterno(tile)
        }
    }

    /// O canal Y é o padrão rodado 90° e meio ladrilho ao lado: um padrão de veios dá veios
    /// PERPENDICULARES, e o par desloca nas duas direções em vez de numa diagonal.
    fn textura(
        s: TextureSettings,
        img: Option<&'a ImageMask<'a>>,
        id_img: (u8, u64),
        tile: NoiseTile,
    ) -> Self {
        let s0 = snap_slot_size(s, tile);
        let s1 = TextureSettings {
            angle_deg: (s0.angle_deg + 90) % 360,
            offset: [s0.offset[0] + 0.5, s0.offset[1] + 0.5],
            ..s0
        };
        Self::Textura {
            rot: [angle_basis(s0.angle_deg), angle_basis(s1.angle_deg)],
            s: [s0, s1],
            img,
            id_img,
            period: tile.slot_period(),
        }
    }

    #[inline]
    fn cru(&self, x: i64, y: i64) -> (f32, f32) {
        match self {
            Self::Textura {
                s,
                rot,
                img,
                period,
                ..
            } => (
                sample_tiled_rot_wrapped(&s[0], x, y, *img, rot[0], *period),
                sample_tiled_rot_wrapped(&s[1], x, y, *img, rot[1], *period),
            ),
            Self::PapelInterno(tile) => {
                let (fx, fy) = (x as f32, y as f32);
                let (ox, oy) = PAPEL_INTERNO_CANAL_Y;
                (
                    paper_height(fx, fy, *tile),
                    paper_height(fx + ox, fy + oy, *tile),
                )
            }
        }
    }

    /// O canal normalizado `u = clamp(rms_C·(T − μ)/σ, −1, 1)` — o RMS do Classic, preso ao alcance dele.
    #[inline]
    fn unidade(&self, e: &Estat, x: i64, y: i64) -> (f32, f32) {
        let (a, b) = self.cru(x, y);
        let u = |t: f32| (rms_classic() * (t - e.mu) / e.sigma).clamp(-1.0, 1.0);
        (u(a), u(b))
    }

    fn mapa(&self, (x0, y0, bw, bh): (usize, usize, usize, usize)) -> Mapa {
        let e = self.estat();
        let (mx0, my0) = (x0 as i64 - 1, y0 as i64 - 1);
        let (w, h) = (bw + 2, bh + 2);
        let mut d = vec![[0.0f32; 2]; w * h];
        {
            use rayon::prelude::*;
            d.par_chunks_mut(w.max(1))
                .enumerate()
                .for_each(|(j, linha)| {
                    for (i, v) in linha.iter_mut().enumerate() {
                        let (a, b) = self.unidade(&e, mx0 + i as i64, my0 + j as i64);
                        *v = [a, b];
                    }
                });
        }
        Mapa {
            d,
            x0: mx0,
            y0: my0,
            w,
            h,
        }
    }

    /// A chave do memo: as settings E a imagem lida (o papel interno é uma só função do ladrilho).
    fn chave(&self) -> (Option<TextureSettings>, (u8, u64)) {
        match self {
            Self::Textura { s, id_img, .. } => (Some(s[0]), *id_img),
            Self::PapelInterno(_) => (None, SEM_IMAGEM),
        }
    }

    /// A grelha das medições: `N × N` pontos sobre duas repetições do padrão (um ladrilho = 256/Size
    /// px) — o papel interno, sem ladrilho, sobre 64 px (≈ 13 células da oitava larga de 5 px).
    fn grelha(&self) -> impl Fn(usize) -> i64 {
        let lado = match self {
            Self::Textura { s, .. } => {
                2.0 * TEX_TILE_BASE_PX / s[0].size[0].max(s[0].size[1]).max(1e-3)
            }
            Self::PapelInterno(_) => 64.0,
        };
        let passo = (lado / N as f32).max(1.0);
        move |i: usize| ((i as f32 + 0.5) * passo) as i64
    }

    /// As estatísticas do mapa, medidas numa grelha fixa de duas repetições do padrão — memo da função
    /// pura das settings (cada medição são ~4·64² amostras, e uma composição por quadro não as paga).
    fn estat(&self) -> Estat {
        type Chave = (Option<TextureSettings>, (u8, u64));
        static MEMO: Mutex<Vec<(Chave, Estat)>> = Mutex::new(Vec::new());
        let k = self.chave();
        if let Ok(m) = MEMO.lock()
            && let Some((_, e)) = m.iter().find(|(c, _)| *c == k)
        {
            return *e;
        }
        let ponto = self.grelha();
        let (mut s1, mut s2) = (0.0f64, 0.0f64);
        for j in 0..N {
            for i in 0..N {
                let (a, b) = self.cru(ponto(i), ponto(j));
                for t in [a, b] {
                    s1 += f64::from(t);
                    s2 += f64::from(t) * f64::from(t);
                }
            }
        }
        let n = (2 * N * N) as f64;
        let mu = (s1 / n) as f32;
        let sigma = ((s2 / n - (s1 / n).powi(2)).max(0.0).sqrt() as f32).max(1e-6);
        let parcial = Estat {
            mu,
            sigma,
            dobra: 0.0,
        };
        // A dobra: a amplitude em px a que o gradiente do deslocamento tem RMS 1.
        let mut g = 0.0f64;
        for j in 0..N {
            for i in 0..N {
                let (x, y) = (ponto(i), ponto(j));
                let (ax, bx) = self.unidade(&parcial, x + 1, y);
                let (a0, b0) = self.unidade(&parcial, x, y);
                let (ay, by) = self.unidade(&parcial, x, y + 1);
                for d in [ax - a0, ay - a0, bx - b0, by - b0] {
                    g += f64::from(d) * f64::from(d);
                }
            }
        }
        let g = (g / (2 * N * N) as f64).sqrt() as f32;
        let e = Estat {
            mu,
            sigma,
            dobra: if g > 1e-6 { 1.0 / g } else { 0.0 },
        };
        if let Ok(mut m) = MEMO.lock() {
            if m.len() >= 32 {
                m.clear();
            }
            m.push((k, e));
        }
        e
    }
}

/// O lado da grelha das medições ([`Amostrador::grelha`]).
const N: usize = 64;

#[derive(Clone, Copy)]
struct Estat {
    mu: f32,
    sigma: f32,
    /// A amplitude (px por unidade do mapa) em que o gradiente do deslocamento tem RMS `1`.
    dobra: f32,
}

/// As duas réguas do Classic, medidas uma vez sobre 16 células da oitava larga (22 px): o RMS de UMA
/// componente e o RMS do gradiente de 1 px dela.
fn classic() -> (f32, f32) {
    static R: OnceLock<(f32, f32)> = OnceLock::new();
    *R.get_or_init(|| {
        let passo = 22.0 * 16.0 / N as f32;
        let w = |x: f32, y: f32| warp_offset(x, y, NoiseTile::NONE);
        let (mut soma, mut g) = (0.0f64, 0.0f64);
        for j in 0..N {
            for i in 0..N {
                let (x, y) = ((i as f32 + 0.5) * passo, (j as f32 + 0.5) * passo);
                let (a, b) = w(x, y);
                let (ax, bx) = w(x + 1.0, y);
                let (ay, by) = w(x, y + 1.0);
                soma += f64::from(a * a + b * b);
                for d in [ax - a, ay - a, bx - b, by - b] {
                    g += f64::from(d) * f64::from(d);
                }
            }
        }
        let n = (2 * N * N) as f64;
        ((soma / n).sqrt() as f32, (g / n).sqrt() as f32)
    })
}

fn rms_classic() -> f32 {
    classic().0
}

/// O fator de Size que leva o RMS do gradiente de um padrão (medido a Size `1`, ângulo e offset zero)
/// ao do Classic. O gradiente cresce LINEAR com o Size, por isso basta uma medição.
fn escala_do_flow(s: &TextureSettings) -> f32 {
    let um = TextureSettings {
        size: [1.0, 1.0],
        angle_deg: 0,
        offset: [0.0, 0.0],
        ..*s
    };
    let g = Amostrador::textura(um, None, SEM_IMAGEM, NoiseTile::NONE)
        .estat()
        .dobra;
    // `dobra = 1/g`: base = g_C / g_P = dobra_P / dobra_C.
    if g > 0.0 { classic().1 * g } else { 1.0 }
}

/// O alcance em px que o Paper Edge soma à borda (o `Paper Edge · dobra` do maior dono, porque o mapa é
/// preso a `[−1, 1]`) — a janela do composite tem de o cobrir. `0` sem Paper Edge ⇒ a janela de sempre.
pub(super) fn alcance_do_papel(
    brush: &ph2d_painter_brush::BrushSpec,
    table: &[WetStrokeStyle],
    imagens: Imagens<'_>,
    tile: NoiseTile,
) -> f32 {
    let um = |p: TextureSettings, k: f32| {
        if k > 0.0 {
            k.min(1.0) * Amostrador::papel(p, &imagens, tile).estat().dobra
        } else {
            0.0
        }
    };
    table
        .iter()
        .fold(um(brush.paper, brush.paper_edge), |m, s| {
            m.max(um(s.paper, s.paper_edge))
        })
}

#[cfg(test)]
#[path = "watercolor_flow_tests.rs"]
mod tests;

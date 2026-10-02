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

/// O deslocamento de UM [`Estilo`]: o fluxo (`None` = Classic, analítico) e o papel em px.
struct Fonte {
    flow: Option<Mapa>,
    /// O mapa do papel e o fator `Paper Edge · amplitude da dobra` (px por unidade do mapa).
    papel: Option<(Mapa, f32)>,
}

/// **O FLUXO da composição** — uma [`Fonte`] por estilo DISTINTO entre os donos (quase sempre uma) e,
/// com duas ou mais, o peso de cada uma suavizado na junção pela lei do campo de estilo da #18
/// (`blur(m_k)/blur(m)`), senão a junção de dois estilos degrauava o deslocamento.
pub(super) struct EdgeFlow {
    fontes: Vec<Fonte>,
    /// `dono → índice da fonte` (`0` = sem dono = o pincel vivo).
    do_dono: Vec<u8>,
    pesos: Option<(Vec<Vec<f32>>, Vec<f32>)>,
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
        paper_img: Option<&ImageMask<'_>>,
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
                    .then(|| Amostrador::flow(e.flow, tile).mapa(caixa)),
                papel: e.papel.map(|(p, k)| {
                    let a = Amostrador::papel(p, paper_img, tile);
                    let f = k.min(1.0) * a.estat().dobra;
                    (a.mapa(caixa), f)
                }),
            })
            .collect();
        let (rx0, ry0, rw, rh) = janela;
        let pesos = (distintos.len() > 1).then_some(()).and(owner).map(|owner| {
            let n = rw * rh;
            let mut planos = vec![vec![0.0f32; n]; distintos.len()];
            let mut massa = vec![0.0f32; n];
            for wy in 0..rh {
                let base = (ry0 + wy) * fw + rx0;
                for (wx, &o) in owner[base..base + rw].iter().enumerate() {
                    if o != 0 {
                        let k = do_dono[(o as usize).min(do_dono.len() - 1)] as usize;
                        planos[k][wy * rw + wx] = 1.0;
                        massa[wy * rw + wx] = 1.0;
                    }
                }
            }
            let r = super::watercolor_rewet_px::WET_FIELD_BLUR_PX;
            let planos = planos.iter().map(|p| box_blur(p, rw, rh, r)).collect();
            (planos, box_blur(&massa, rw, rh, r))
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
    pub(super) fn desloca(&self, dono: u8, lx: f32, ly: f32, x: f32, y: f32, amp: f32) -> (f32, f32) {
        if let Some((planos, massa)) = &self.pesos {
            let m = sample_bilinear(massa, self.rw, self.rh, lx, ly);
            if m > 1e-4 {
                let (mut dx, mut dy) = (0.0, 0.0);
                for (fonte, plano) in self.fontes.iter().zip(planos) {
                    let w = sample_bilinear(plano, self.rw, self.rh, lx, ly) / m;
                    if w > 0.0 {
                        let (a, b) = fonte.desloca(x, y, amp, self.tile);
                        dx += w * a;
                        dy += w * b;
                    }
                }
                return (dx, dy);
            }
        }
        let k = self.do_dono[(dono as usize).min(self.do_dono.len() - 1)] as usize;
        self.fontes[k].desloca(x, y, amp, self.tile)
    }
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
        period: [f32; 2],
    },
    PapelInterno(NoiseTile),
}

/// A distância entre os dois canais do papel interno — qualquer distância fora da grelha de 5/2,5 px
/// do [`paper_height`] decorrela; a periodicidade do ladrilho sobrevive a uma translação.
const PAPEL_INTERNO_CANAL_Y: (f32, f32) = (517.5, 291.25);

impl<'a> Amostrador<'a> {
    /// Um padrão de Flow, com o Size do artista multiplicado pela escala que leva o gradiente do
    /// padrão ao do Classic (o Flow Size `1` = a escala de detalhe do Classic).
    fn flow(s: TextureSettings, tile: NoiseTile) -> Self {
        let base = escala_do_flow(&s);
        let s = TextureSettings {
            size: [s.size[0] * base, s.size[1] * base],
            ..s
        };
        Self::textura(s, None, tile)
    }

    fn papel(s: TextureSettings, img: Option<&'a ImageMask<'a>>, tile: NoiseTile) -> Self {
        if s.is_active() {
            Self::textura(s, img, tile)
        } else {
            Self::PapelInterno(tile)
        }
    }

    /// O canal Y é o padrão rodado 90° e meio ladrilho ao lado: um padrão de veios dá veios
    /// PERPENDICULARES, e o par desloca nas duas direções em vez de numa diagonal.
    fn textura(s: TextureSettings, img: Option<&'a ImageMask<'a>>, tile: NoiseTile) -> Self {
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
            period: tile.slot_period(),
        }
    }

    #[inline]
    fn cru(&self, x: i64, y: i64) -> (f32, f32) {
        match self {
            Self::Textura { s, rot, img, period } => (
                sample_tiled_rot_wrapped(&s[0], x, y, *img, rot[0], *period),
                sample_tiled_rot_wrapped(&s[1], x, y, *img, rot[1], *period),
            ),
            Self::PapelInterno(tile) => {
                let (fx, fy) = (x as f32, y as f32);
                let (ox, oy) = PAPEL_INTERNO_CANAL_Y;
                (paper_height(fx, fy, *tile), paper_height(fx + ox, fy + oy, *tile))
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
            d.par_chunks_mut(w.max(1)).enumerate().for_each(|(j, linha)| {
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

    /// A chave do memo: as settings (o papel interno é uma só função do ladrilho).
    fn chave(&self) -> Option<TextureSettings> {
        match self {
            Self::Textura { s, .. } => Some(s[0]),
            Self::PapelInterno(_) => None,
        }
    }

    /// As estatísticas do mapa, medidas numa grelha fixa de duas repetições do padrão — memo da função
    /// pura das settings (cada medição são ~4·64² amostras, e uma composição por quadro não as paga).
    fn estat(&self) -> Estat {
        static MEMO: Mutex<Vec<(Option<TextureSettings>, Estat)>> = Mutex::new(Vec::new());
        let k = self.chave();
        if let Ok(m) = MEMO.lock()
            && let Some((_, e)) = m.iter().find(|(c, _)| *c == k)
        {
            return *e;
        }
        let lado = match self {
            Self::Textura { s, .. } => 2.0 * TEX_TILE_BASE_PX / s[0].size[0].max(s[0].size[1]).max(1e-3),
            Self::PapelInterno(_) => 64.0,
        };
        const N: usize = 64;
        let passo = (lado / N as f32).max(1.0);
        let ponto = |i: usize| ((i as f32 + 0.5) * passo) as i64;
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
        let parcial = Estat { mu, sigma, dobra: 0.0 };
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
        const N: usize = 64;
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
    let g = Amostrador::textura(um, None, NoiseTile::NONE).estat().dobra;
    // `dobra = 1/g`: base = g_C / g_P = dobra_P / dobra_C.
    if g > 0.0 { classic().1 * g } else { 1.0 }
}

/// O alcance em px que o Paper Edge soma à borda (o `Paper Edge · dobra` do maior dono, porque o mapa é
/// preso a `[−1, 1]`) — a janela do composite tem de o cobrir. `0` sem Paper Edge ⇒ a janela de sempre.
pub(super) fn alcance_do_papel(
    brush: &ph2d_painter_brush::BrushSpec,
    table: &[WetStrokeStyle],
    paper_img: Option<&ImageMask<'_>>,
    tile: NoiseTile,
) -> f32 {
    let um = |p: TextureSettings, k: f32| {
        if k > 0.0 {
            k.min(1.0) * Amostrador::papel(p, paper_img, tile).estat().dobra
        } else {
            0.0
        }
    };
    table
        .iter()
        .fold(um(brush.paper, brush.paper_edge), |m, s| m.max(um(s.paper, s.paper_edge)))
}

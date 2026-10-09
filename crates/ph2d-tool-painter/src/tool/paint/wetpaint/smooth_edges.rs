//! **Smooth Edges do Wet Paint** (dono 09/10, «opção B»: o NOSSO Smooth Edges do Watercolor também no Wet
//! Paint, ligado de fábrica). A lei é a do Watercolor — [`aa_coverage`] (`watercolor_aa.rs`), uma porta só:
//! a cobertura endurecida amostrada 3×3 à volta do pixel e a fração aplicada como alfa LINEAR sobre o pixel
//! acabado. O campo de cobertura do Wet Paint é a MANCHA (`pa ≥ SS0`, 0/1 por pixel): o miolo e o papel
//! aberto ficam iguais ao byte; só a silhueta ganha meio-tom (a orla 0/1 da linha de contacto e os pontinhos
//! do carimbo — auditoria de 09/10, docs/Painter/NewFeaturesWetPaint/05 §4b). Nenhuma água nem pigmento se
//! move: é aparência, como o Paper. Desligado = o composite de antes, ao byte.

use super::grid_map::{SampleU, SampleURow};
use crate::tool::paint::watercolor_field::aa_coverage;
use crate::tool::paint::watercolor_render::{SS0, SS1};
use rayon::prelude::*;

/// O pigmento de um pixel na razão 1 (`o` = o offset do pixel no plano), em premultiplicado normalizado
/// (`pc·pa`) e o `pa` — a forma em que o `over` do composite o consome.
#[inline]
pub(super) fn premul_da_celula(pigment: &[u8], o: usize) -> ([f32; 3], f32) {
    let a = pigment[o + 3] as f32 / 255.0;
    (
        [
            pigment[o] as f32 * a,
            pigment[o + 1] as f32 * a,
            pigment[o + 2] as f32 * a,
        ],
        a,
    )
}

/// O mesmo acima da razão 1: o amostrador devolve `Σ c·a·w` e `Σ a·w` em 0..255; dividir por 255 dá
/// exatamente `pc · pa`, sem passar pela cor straight.
#[inline]
pub(super) fn premul_amostrado(s: [f64; 4]) -> ([f32; 3], f32) {
    (
        [
            (s[0] / 255.0) as f32,
            (s[1] / 255.0) as f32,
            (s[2] / 255.0) as f32,
        ],
        (s[3] / 255.0) as f32,
    )
}

/// O pigmento do pixel `(px, py)` por qualquer razão — a porta das duas acima.
#[inline]
pub(super) fn premul_do_pixel(
    sampler: SampleU,
    srow: Option<SampleURow>,
    pigment: &[u8],
    stride: usize,
    px: usize,
    py: usize,
) -> ([f32; 3], f32) {
    if sampler.is_identity() {
        premul_da_celula(pigment, py * stride + px * 4)
    } else {
        let s = match srow {
            Some(r) => r.at(pigment, px),
            None => sampler.row(py).at(pigment, px),
        };
        premul_amostrado(s)
    }
}

/// A máscara (`pa ≥ SS0`) da região do composite com 1 px de margem — montada UMA vez, em paralelo por
/// linha, antes do laço; o pixel só faz a conta se o pé 3×3 dele mistura dentro e fora.
pub(super) struct Mascara {
    x0: usize,
    y0: usize,
    rw: usize,
    w: usize,
    h: usize,
    m: Vec<bool>,
    /// Por linha da região: (tem pixel dentro, tem pixel fora).
    linhas: Vec<(bool, bool)>,
}

impl Mascara {
    /// A linha `py` pode ter um pixel misto? Não, se as três linhas do pé são todas papel ou todas mancha
    /// (a maior parte de uma tela recomposta inteira): o laço salta a linha sem olhar pixel a pixel.
    #[inline]
    pub(super) fn linha_pode_misturar(&self, py: usize) -> bool {
        let rows = [py.saturating_sub(1), py, (py + 1).min(self.h - 1)];
        let (dentro, fora) = rows.iter().fold((false, false), |(d, f), &y| {
            let l = self.linhas[y - self.y0];
            (d || l.0, f || l.1)
        });
        dentro && fora
    }

    /// O pé 3×3 de `(px, py)` tem pixels dentro E fora da mancha (os vizinhos fora da tela repetem a borda)?
    #[inline]
    pub(super) fn mista(&self, px: usize, py: usize) -> bool {
        let cols = [px.saturating_sub(1), px, (px + 1).min(self.w - 1)];
        let rows = [py.saturating_sub(1), py, (py + 1).min(self.h - 1)];
        let em = |x: usize, y: usize| self.m[(y - self.y0) * self.rw + (x - self.x0)];
        let primeiro = em(cols[0], rows[0]);
        rows.iter()
            .any(|&y| cols.iter().any(|&x| em(x, y) != primeiro))
    }
}

/// A silhueta da mancha à volta de um pixel: o pé 3×3 do [`aa_coverage`] (as amostras a ±0,667 só leem
/// os oito vizinhos; nas bordas da tela o vizinho repete-se, como o `clamp` da amostra), lido do plano de
/// pigmento só onde é preciso. ⚠️ Medido 09/10: uma máscara da região inteira, montada antes do laço,
/// levava a recomposição de uma tela de 4096² de 10 a 106 ms (em série, 16 B por pixel); o pé local corre
/// dentro do laço paralelo do composite, e um pé todo dentro ou todo fora sai logo (o caminho de antes).
#[derive(Clone, Copy)]
pub(super) struct Silhueta<'a> {
    pub(super) sampler: SampleU,
    pub(super) pigment: &'a [u8],
    pub(super) stride: usize,
    pub(super) w: usize,
    pub(super) h: usize,
}

impl Silhueta<'_> {
    /// A [`Mascara`] da região `(x0, y0, x1, y1)` do composite.
    pub(super) fn mascara(&self, (x0, y0, x1, y1): (usize, usize, usize, usize)) -> Mascara {
        let (a, b) = (x0.saturating_sub(1), (x1 + 1).min(self.w));
        let (c, d) = (y0.saturating_sub(1), (y1 + 1).min(self.h));
        let rw = b - a;
        let mut m = vec![false; rw * (d - c)];
        m.par_chunks_mut(rw).enumerate().for_each(|(k, linha)| {
            let y = c + k;
            if self.sampler.is_identity() {
                for (v, x) in linha.iter_mut().zip(a..b) {
                    *v = premul_da_celula(self.pigment, y * self.stride + x * 4).1 >= SS0;
                }
            } else {
                let r = self.sampler.row(y);
                for (v, x) in linha.iter_mut().zip(a..b) {
                    *v = premul_do_pixel(self.sampler, Some(r), self.pigment, self.stride, x, y).1
                        >= SS0;
                }
            }
        });
        let linhas = m
            .par_chunks(rw)
            .map(|l| (l.iter().any(|&v| v), l.iter().any(|&v| !v)))
            .collect();
        Mascara {
            linhas,
            x0: a,
            y0: c,
            rw,
            w: self.w,
            h: self.h,
            m,
        }
    }

    fn pixel(&self, x: usize, y: usize) -> ([f32; 3], f32) {
        premul_do_pixel(self.sampler, None, self.pigment, self.stride, x, y)
    }

    /// O pigmento que o pixel `(px, py)` mostra e o alfa linear do Smooth Edges sobre ele: dentro da mancha
    /// o dele; fora, o do vizinho de dentro com mais tinta (o sombreado «um pouco para dentro» da lei do
    /// Watercolor); longe da silhueta, `(o dele, 1)` — o caminho de antes.
    pub(super) fn aplica(
        &self,
        px: usize,
        py: usize,
        dele: ([f32; 3], f32),
    ) -> ([f32; 3], f32, f32) {
        let xs = [px.saturating_sub(1), px, (px + 1).min(self.w - 1)];
        let ys = [py.saturating_sub(1), py, (py + 1).min(self.h - 1)];
        let mut pa = [0.0f32; 9];
        for (j, &y) in ys.iter().enumerate() {
            for (i, &x) in xs.iter().enumerate() {
                pa[j * 3 + i] = if i == 1 && j == 1 {
                    dele.1
                } else if self.sampler.is_identity() {
                    premul_da_celula(self.pigment, y * self.stride + x * 4).1
                } else {
                    self.pixel(x, y).1
                };
            }
        }
        let campo = pa.map(|a| if a >= SS0 { 1.0f32 } else { 0.0 });
        if campo.iter().all(|&c| c == campo[0]) {
            return (dele.0, dele.1, 1.0);
        }
        let (_, alfa) = aa_coverage(&campo, 3, 3, |ox, oy| (1.0 + ox, 1.0 + oy), SS0, SS1);
        if alfa >= 1.0 {
            return (dele.0, dele.1, 1.0);
        }
        if dele.1 >= SS0 {
            return (dele.0, dele.1, alfa);
        }
        let k = (0..9)
            .filter(|&k| campo[k] > 0.0)
            .max_by(|&a, &b| pa[a].total_cmp(&pa[b]).then(b.cmp(&a)))
            .unwrap_or(4);
        let p = self.pixel(xs[k % 3], ys[k / 3]);
        (p.0, p.1, alfa)
    }
}

/// O CUSTO do Smooth Edges no composite (§0.0: medir antes de aceitar), as duas variantes no MESMO processo,
/// intercaladas, o MÍNIMO de cada: a tela inteira (o pior caso: o clique no checkbox recompõe tudo) e o
/// retângulo de um traço (o quadro normal). `cargo test … --lib medida_do_smooth_edges -- --ignored --nocapture`.
#[cfg(test)]
mod medida {
    use crate::tool::PainterTool;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
    use ph2d_painter_brush::{BrushSpec, Falloff};

    fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
        CanvasPointer {
            pos,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        }
    }

    #[test]
    #[ignore = "medida: o custo do Smooth Edges; corre-se à mão"]
    fn medida_do_smooth_edges() {
        const N: u32 = 4096;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
        let b = BrushSpec {
            radius_px: 60.0,
            color: [0.1, 0.4, 0.9],
            falloff: Falloff::Smooth,
            ..Default::default()
        };
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.set_paint_tool_mode("wetpaint");
        let y = (N / 2) as f32;
        t.on_canvas_pointer(cp([1000.0, y], PointerPhase::Down));
        for k in 1..=60 {
            t.on_canvas_pointer(cp(
                [1000.0 + 30.0 * k as f32, y + 200.0 * (k as f32 / 9.0).sin()],
                PointerPhase::Move,
            ));
        }
        t.on_canvas_pointer(cp([2800.0, y], PointerPhase::Up));
        let mut minimo = [[f64::MAX; 2]; 2]; // [tela, retângulo][desligado, ligado]
        for _ in 0..7 {
            for (k, suave) in [false, true].into_iter().enumerate() {
                t.paint.wetpaint.smooth_edges = suave;
                for (q, rect) in [None, Some((1700, 1800, 2000, 2100))]
                    .into_iter()
                    .enumerate()
                {
                    let sess = t.paint.wetpaint.session.as_mut().expect("sessão viva");
                    sess.bring_home();
                    match rect {
                        None => sess.engine.mark_dirty_full(),
                        Some((x0, y0, x1, y1)) => sess.engine.merge_dirty(x0, y0, x1, y1),
                    }
                    let t0 = std::time::Instant::now();
                    super::super::super::wetpaint::composite_for_measure(&mut t);
                    minimo[q][k] = minimo[q][k].min(t0.elapsed().as_secs_f64() * 1e3);
                }
            }
        }
        let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
        for (q, nome) in ["tela 4096²", "retângulo 300²"].into_iter().enumerate() {
            println!(
                "[smooth edges] {nome}: desligado {:.2} ms · ligado {:.2} ms ({:+.0} %) · load {}",
                minimo[q][0],
                minimo[q][1],
                100.0 * (minimo[q][1] / minimo[q][0] - 1.0),
                load.split_whitespace().next().unwrap_or("?")
            );
        }
    }
}

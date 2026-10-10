//! **Smooth Edges do Wet Paint** (dono 09/10; a INTENSIDADE desde 10/10). O slider é o RAIO de um
//! alisamento da MANCHA (`pa ≥ SS0`): [`Suave`]. O miolo longe da borda e o papel aberto ficam iguais ao
//! byte; só a orla muda (forma mais lisa, borda com meio-tom). Nenhuma água nem pigmento se move: é
//! aparência, como o Paper. `0` = o composite de antes, ao byte. O 1.º Smooth Edges (o 3×3 do Watercolor,
//! `aa_coverage`) era fraco demais no máximo (smoke do dono 10/10) — o porquê e as medições: 05 §4b W1g.

use super::grid_map::{SampleU, SampleURow};
use crate::tool::paint::watercolor_render::SS0;
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

/// O raio do alisamento no máximo do slider, px (medido na prancha de 10/10, 05 §4b).
pub(super) const RAIO_MAX: f32 = 6.0;

/// Quantos px à volta de uma mudança um pixel do Smooth Edges pode mudar com `raio` — a tenda tem meia
/// largura `raio`, e a orla anti-serrilhada mais um.
#[must_use]
pub(super) fn alcance(raio: f32) -> usize {
    raio.ceil() as usize + 1
}

/// Uma passada de caixa de meia largura FRACIONÁRIA `h` (as pontas `⌊h⌋ + 1` pesam `h − ⌊h⌋`), lida em
/// `ler(i)` com o índice preso a `0..n` (a borda repete-se, como na tela).
#[inline]
fn caixa(n: usize, h: f32, i: usize, ler: impl Fn(usize) -> f32) -> f32 {
    let (k, f) = (h.floor() as isize, h - h.floor());
    let em = |d: isize| ler((i as isize + d).clamp(0, n as isize - 1) as usize);
    let mut s = 0.0;
    for d in -k..=k {
        s += em(d);
    }
    if f > 0.0 {
        s += f * (em(-k - 1) + em(k + 1));
    }
    s / (2.0 * k as f32 + 1.0 + 2.0 * f)
}

/// [`caixa`] para os quatro canais da tinta de uma vez.
#[inline]
fn caixa4(n: usize, h: f32, i: usize, ler: impl Fn(usize) -> [f32; 4]) -> [f32; 4] {
    let (k, f) = (h.floor() as isize, h - h.floor());
    let em = |d: isize| ler((i as isize + d).clamp(0, n as isize - 1) as usize);
    let mut s = [0.0f32; 4];
    for d in -k..=k {
        let v = em(d);
        for c in 0..4 {
            s[c] += v[c];
        }
    }
    if f > 0.0 {
        let (a, b) = (em(-k - 1), em(k + 1));
        for c in 0..4 {
            s[c] += f * (a[c] + b[c]);
        }
    }
    let n = 2.0 * k as f32 + 1.0 + 2.0 * f;
    s.map(|v| v / n)
}

/// O degrau de cor DENTRO da tinta (a altura, em `0..1` por canal premultiplicado, entre a tinta
/// desfocada a `raio` px de cada lado) a partir do qual a borda interior amacia, e por inteiro. Medido
/// (dono 10/10, 4.º smoke; a borda de um azul sobre um vermelho · o S aprovado, píxeis que mudam > 8
/// níveis): 0,04 → −48 % · 10 679 px; 0,10 → −42 % · 2 639; **0,15 → −35 % · 512**; 0,20 → −27 % · 109.
/// Abaixo ficam as células da aguada e o grão; acima, o degrau entre duas tintas e a tinta levantada.
pub(super) const DEGRAU_MIN: f32 = 0.15;
const DEGRAU_CHEIO: f32 = 0.40;

/// **O Smooth Edges de uma região do composite** (dono 10/10: «não vi benefício no máximo»): uma BORDA
/// SUAVE de largura ajustável. A mancha (`pa ≥ SS0`) desfocada por uma tenda de meia largura `raio` (duas
/// caixas de `raio/2`, separáveis) diz onde fica a orla: `B ∈ (0, 1)`. Nessa faixa o pixel mostra a tinta
/// (cor e cobertura, premultiplicadas) desfocada pela MESMA tenda, com peso `4·B·(1 − B)` — inteiro na
/// orla, zero no miolo e no papel aberto (o caminho de antes). O contorno escuro e a silhueta amaciam
/// juntos e a tinta conserva-se. **Dentro** da tinta (onde a mancha é toda 1 — a borda de um traço sobre
/// outra cor, a tinta levantada) a borda é um DEGRAU de cor: a tinta desfocada a `raio` px de cada lado
/// mede a altura do degrau (no grão os dois lados são iguais), o peso vai de 0 a 1 entre [`DEGRAU_MIN`] e
/// [`DEGRAU_CHEIO`] e é um sino na posição dentro do degrau (dono 10/10, 4.º smoke). `raio = 0` ⇒ `B` é a própria mancha ⇒ peso 0 ⇒ o desligado, ao byte.
pub(super) struct Suave {
    x0: usize,
    y0: usize,
    rw: usize,
    w: usize,
    h: usize,
    b: Vec<f32>,
    raio: f32,
    /// A tinta da região (premultiplicada + `pa`) desfocada pela MESMA tenda que a mancha, nas linhas a
    /// `alcance + 1` da tinta (as que [`Suave::aplica`] lê, com o vizinho do declive); o resto não se lê.
    pbar: Vec<[f32; 4]>,
}

impl Suave {
    /// Monta a mancha e o alisado da região `(x0, y0, x1, y1)` (px) com a margem que o raio lê.
    pub(super) fn nova(
        sampler: SampleU,
        pigment: &[u8],
        stride: usize,
        (w, h): (usize, usize),
        (x0, y0, x1, y1): (usize, usize, usize, usize),
        raio: f32,
    ) -> Self {
        let a = alcance(raio) * 2;
        let (rx0, ry0) = (x0.saturating_sub(a), y0.saturating_sub(a));
        let (rx1, ry1) = ((x1 + a).min(w), (y1 + a).min(h));
        let (rw, rh) = (rx1 - rx0, ry1 - ry0);
        let mut m = vec![false; rw * rh];
        m.par_chunks_mut(rw).enumerate().for_each(|(k, linha)| {
            let y = ry0 + k;
            let r = (!sampler.is_identity()).then(|| sampler.row(y));
            for (v, x) in linha.iter_mut().zip(rx0..rx1) {
                *v = premul_do_pixel(sampler, r, pigment, stride, x, y).1 >= SS0;
            }
        });
        let hb = raio * 0.5;
        // As passadas só onde há tinta perto: longe dela o desfocado é EXACTAMENTE zero (a mancha é
        // zero em toda a janela), e a maior parte de uma tela grande está longe da tinta.
        let tem: Vec<bool> = m.par_chunks(rw).map(|l| l.contains(&true)).collect();
        let mut acc = vec![0u32; rh + 1];
        for y in 0..rh {
            acc[y + 1] = acc[y] + u32::from(tem[y]);
        }
        let al = alcance(raio);
        let perto: Vec<bool> = (0..rh)
            .map(|y| acc[(y + al + 1).min(rh)] > acc[y.saturating_sub(al)])
            .collect();
        let mut pre = vec![[0.0f32; 4]; rw * rh];
        pre.par_chunks_mut(rw).enumerate().for_each(|(k, linha)| {
            if acc[(k + 2 * al + 1).min(rh)] == acc[k.saturating_sub(2 * al)] {
                return;
            }
            let y = ry0 + k;
            let r = (!sampler.is_identity()).then(|| sampler.row(y));
            for (v, x) in linha.iter_mut().zip(rx0..rx1) {
                let (p, pa) = premul_do_pixel(sampler, r, pigment, stride, x, y);
                *v = [p[0], p[1], p[2], pa];
            }
        });
        let mut t = vec![0.0f32; rw * rh];
        let mut b = vec![0.0f32; rw * rh];
        // Duas caixas horizontais (m → t → b), depois duas verticais (b → t → b); o resultado fica em `b`.
        t.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if !tem[y] {
                return;
            }
            let fonte = &m[y * rw..(y + 1) * rw];
            for (x, v) in l.iter_mut().enumerate() {
                *v = caixa(rw, hb, x, |i| f32::from(u8::from(fonte[i])));
            }
        });
        b.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if !tem[y] {
                return;
            }
            let fonte = &t[y * rw..(y + 1) * rw];
            for (x, v) in l.iter_mut().enumerate() {
                *v = caixa(rw, hb, x, |i| fonte[i]);
            }
        });
        t.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if !perto[y] {
                return;
            }
            for (x, v) in l.iter_mut().enumerate() {
                *v = caixa(rh, hb, y, |j| b[j * rw + x]);
            }
        });
        b.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if !perto[y] {
                return;
            }
            for (x, v) in l.iter_mut().enumerate() {
                *v = caixa(rh, hb, y, |j| t[j * rw + x]);
            }
        });
        // A tinta desfocada: duas caixas horizontais nas linhas onde a tinta foi lida (pre → tmp → pre) e
        // duas verticais nas que se leem (pre → tmp → pre).
        let lida = |y: usize| acc[(y + 2 * al + 1).min(rh)] > acc[y.saturating_sub(2 * al)];
        let lida_v = |y: usize| acc[(y + al + 2).min(rh)] > acc[y.saturating_sub(al + 1)];
        let mut tmp = vec![[0.0f32; 4]; rw * rh];
        tmp.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if lida(y) {
                let fonte = &pre[y * rw..(y + 1) * rw];
                for (x, v) in l.iter_mut().enumerate() {
                    *v = caixa4(rw, hb, x, |i| fonte[i]);
                }
            }
        });
        pre.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if lida(y) {
                let fonte = &tmp[y * rw..(y + 1) * rw];
                for (x, v) in l.iter_mut().enumerate() {
                    *v = caixa4(rw, hb, x, |i| fonte[i]);
                }
            }
        });
        tmp.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if lida_v(y) {
                for (x, v) in l.iter_mut().enumerate() {
                    *v = caixa4(rh, hb, y, |j| pre[j * rw + x]);
                }
            }
        });
        pre.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if lida_v(y) {
                for (x, v) in l.iter_mut().enumerate() {
                    *v = caixa4(rh, hb, y, |j| tmp[j * rw + x]);
                }
            }
        });
        Suave {
            x0: rx0,
            y0: ry0,
            rw,
            w,
            h,
            b,
            raio,
            pbar: pre,
        }
    }

    /// O pigmento que o pixel `(px, py)` mostra: `dele` longe de qualquer borda (o caminho de antes); na
    /// orla com o papel e nos degraus de cor dentro da tinta, a mistura dele com a tinta desfocada.
    pub(super) fn aplica(&self, px: usize, py: usize, dele: ([f32; 3], f32)) -> ([f32; 3], f32) {
        let i = (py - self.y0) * self.rw + (px - self.x0);
        let b = self.b[i];
        let mut peso = 4.0 * b * (1.0 - b);
        // Dentro da tinta: um DEGRAU de cor — a tinta desfocada a `r` px de cada lado do pixel (num
        // degrau os dois lados diferem; no grão, desfocado, são iguais) e a posição do pixel dentro dele
        // (`u`), com o peso em sino `4·u·(1−u)` como na orla com o papel.
        let dentro = (2.0 * b - 1.0).clamp(0.0, 1.0);
        if dentro > 0.0 {
            let r = (self.raio.round() as usize).max(1);
            let altura = self.b.len() / self.rw;
            let q = |x: usize, y: usize| {
                self.pbar[(y.clamp(self.y0, self.y0 + altura - 1) - self.y0) * self.rw
                    + (x.clamp(self.x0, self.x0 + self.rw - 1) - self.x0)]
            };
            let meio = self.pbar[i];
            let mut melhor = 0.0f32;
            for (a, c) in [
                (q(px.saturating_sub(r), py), q((px + r).min(self.w - 1), py)),
                (q(px, py.saturating_sub(r)), q(px, (py + r).min(self.h - 1))),
            ] {
                for k in 0..4 {
                    let esc = if k < 3 { 1.0 / 255.0 } else { 1.0 };
                    let d = (c[k] - a[k]) * esc;
                    if d.abs() <= DEGRAU_MIN {
                        continue;
                    }
                    let u = (((meio[k] - a[k]) * esc) / d).clamp(0.0, 1.0);
                    let w = ((d.abs() - DEGRAU_MIN) / (DEGRAU_CHEIO - DEGRAU_MIN)).clamp(0.0, 1.0);
                    melhor = melhor.max(w * w * (3.0 - 2.0 * w) * 4.0 * u * (1.0 - u));
                }
            }
            peso = peso.max(melhor * dentro);
        }
        if peso < 1e-3 {
            return dele;
        }
        let s = self.pbar[i];
        let l = |d: f32, o: f32| d + (o - d) * peso;
        (
            [l(dele.0[0], s[0]), l(dele.0[1], s[1]), l(dele.0[2], s[2])],
            l(dele.1, s[3]),
        )
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
            for (k, suave) in [0.0f32, 1.0].into_iter().enumerate() {
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

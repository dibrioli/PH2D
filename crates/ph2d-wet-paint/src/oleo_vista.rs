//! **A.1 — o Óleo Real iluminado à resolução da VISTA** ([doc](../../../docs/Painter/49_realshader/A1_oleo_na_vista.md)).
//!
//! Hoje a tinta acende-se à resolução do documento e a vista amplia o resultado (a 400 % vê-se o pixel). Aqui cada
//! pixel da vista pede o documento em `u = x0 + (x + 0,5)/s − 0,5` (centros) e:
//! - a cor da pilha antes da luz ([`PreLuz`]), a luz e o céu ([`LuzCeu`]) e o Gloss vêm por **Catmull-Rom 4×4**
//!   separável, presos ao mínimo/máximo dos 16 vizinhos (sem anel) e com a borda presa;
//! - a altura fina também, e a normal e a hessiana saem do **Sobel sobre a altura AMPLIADA**, com o passo da vista
//!   (o declive ×`s`, a hessiana ×`s²`: unidades do documento);
//! - o material, a Subsurface e o reflexo livre são os da célula mais próxima;
//! - e fecha-se com a MESMA óptica ([`crate::oleo_real::cor_da_amostra`]) e a MESMA mistura ([`PreLuz::fecha_oleo`]).
//!
//! Lei de identidade: com `s = 1` e origem inteira cada pixel cai num centro, o Catmull-Rom devolve a amostra exacta
//! e a saída é a linha de hoje ([`crate::render::render_pigment_row_oleo`]) AO BYTE. `s < 1` (afastar) fica FORA:
//! aí a vista quer um filtro de redução (mip), não interpolação. A grade é a do documento com `ratio` 1 (com outro
//! `ratio` quem chama passa `s` em células). A sombra NO PAPEL (o buffer do composite) não é desta porta.
//! Nada se guarda em resolução maior: só a região pedida, e por linha (as linhas são independentes — o rayon é de
//! quem chama, ADR-0109).

use crate::oleo_real::{Amostra, CampoAltura, LuzCeu, OleoReal, cor_da_amostra, luz_ceu_da_celula};
use crate::render::{PigmentVisual, PreLuz, RenderLayer, render_pigment_row_oleo_pre};
use crate::sim::Params;

/// A região da vista: a origem `(x0, y0)` em pixels do DOCUMENTO (0-based; célula = pixel + 1), o tamanho `w × h` em
/// pixels da VISTA e o zoom `s ≥ 1`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vista {
    pub x0: f64,
    pub y0: f64,
    pub w: usize,
    pub h: usize,
    pub s: f64,
}

/// O que a vista lê das células da janela, contado UMA vez por quadro (à resolução do documento).
pub struct PreparoVista<'a> {
    oleo: &'a OleoReal,
    campo: &'a CampoAltura,
    vista: Vista,
    /// A janela de células `[cx0..=cx1] × [cy0..=cy1]`.
    cx0: usize,
    cy0: usize,
    cw: usize,
    ch: usize,
    pre: Vec<PreLuz>,
    lc: Vec<LuzCeu>,
    brilho: Vec<f64>,
    fina: Vec<f64>,
}

/// Os pesos de Catmull-Rom em `t ∈ [0, 1)` para as amostras `−1, 0, 1, 2`.
#[inline]
fn pesos(t: f64) -> [f64; 4] {
    let (t2, t3) = (t * t, t * t * t);
    [
        -0.5 * t3 + t2 - 0.5 * t,
        1.5 * t3 - 2.5 * t2 + 1.0,
        -1.5 * t3 + 2.0 * t2 + 0.5 * t,
        0.5 * t3 - 0.5 * t2,
    ]
}

/// Um eixo da amostragem: a célula de base (índice na janela, `−1..=2` à volta), a fracção e os pesos.
#[derive(Clone, Copy)]
struct Eixo {
    i: [usize; 4],
    t: f64,
    p: [f64; 4],
    /// A célula mais próxima (índice na janela).
    perto: usize,
}

impl Eixo {
    /// `c` em coordenadas de célula RELATIVAS à janela (`0` = primeira célula), `n` células.
    fn de(c: f64, n: usize) -> Self {
        let f = c.floor();
        let t = c - f;
        let b = f as isize;
        let pr = |k: isize| (b + k).clamp(0, n as isize - 1) as usize;
        Self {
            i: [pr(-1), pr(0), pr(1), pr(2)],
            t,
            p: pesos(t),
            perto: (c + 0.5).floor().clamp(0.0, n as f64 - 1.0) as usize,
        }
    }
}

/// Catmull-Rom 4×4 de `f(ix, iy)` preso ao intervalo dos 16 vizinhos; nos centros (`t = 0`) a amostra EXACTA.
#[inline]
fn cr(ex: &Eixo, ey: &Eixo, f: &dyn Fn(usize, usize) -> f64) -> f64 {
    cr_com(ex, ey, f, true)
}

/// [`cr`] com ou sem a prisão aos 16 vizinhos (`prende`): a ALTURA vai sem ela — prender achata a curvatura das
/// cristas, que é o detalhe que a vista vem buscar; a cor, a cobertura e a luz vão presas (sem anel nem fora de 0..1).
#[inline]
fn cr_com(ex: &Eixo, ey: &Eixo, f: &dyn Fn(usize, usize) -> f64, prende: bool) -> f64 {
    if ex.t == 0.0 && ey.t == 0.0 {
        return f(ex.i[1], ey.i[1]);
    }
    let (mut lo, mut hi, mut acc) = (f64::INFINITY, f64::NEG_INFINITY, 0.0);
    for (ky, &iy) in ey.i.iter().enumerate() {
        let mut linha = 0.0;
        for (kx, &ix) in ex.i.iter().enumerate() {
            let v = f(ix, iy);
            lo = lo.min(v);
            hi = hi.max(v);
            linha += ex.p[kx] * v;
        }
        acc += ey.p[ky] * linha;
    }
    if prende { acc.clamp(lo, hi) } else { acc }
}

impl<'a> PreparoVista<'a> {
    /// Conta as células que a `vista` lê (a pilha antes da luz, a luz e o céu, o Gloss e a altura fina). `campo`
    /// cobre a janela (com a margem, como para a linha de hoje).
    #[must_use]
    pub fn novo(
        p: Option<&Params>,
        layers: &[RenderLayer<'_>],
        visual: PigmentVisual,
        oleo: &'a OleoReal,
        campo: &'a CampoAltura,
        vista: Vista,
    ) -> Self {
        assert!(vista.s >= 1.0, "s < 1 fica fora da A.1 (afastar quer mip)");
        let g = layers[0].grid;
        // Células (1-based) que os pixels e o Sobel (±1 px da vista ≤ 1 célula) e o Catmull-Rom (−1..2) alcançam.
        let c_de = |u: f64| u + 1.0;
        let u_min = (vista.x0 + 0.5 / vista.s - 0.5) - 1.0;
        let u_max = vista.x0 + (vista.w as f64 + 0.5) / vista.s - 0.5;
        let v_min = (vista.y0 + 0.5 / vista.s - 0.5) - 1.0;
        let v_max = vista.y0 + (vista.h as f64 + 0.5) / vista.s - 0.5;
        let prende = |c: f64, n: usize| (c.max(1.0) as usize).clamp(1, n);
        let cx0 = prende(c_de(u_min).floor() - 1.0, g.w);
        let cx1 = prende(c_de(u_max).floor() + 2.0, g.w);
        let cy0 = prende(c_de(v_min).floor() - 1.0, g.h);
        let cy1 = prende(c_de(v_max).floor() + 2.0, g.h);
        let (cw, ch) = (cx1 + 1 - cx0, cy1 + 1 - cy0);
        let mut pre = vec![PreLuz::default(); cw * ch];
        let (mut linha, mut pre_linha) = (vec![0u8; g.w * 4], vec![PreLuz::default(); g.w]);
        for (j, cy) in (cy0..=cy1).enumerate() {
            render_pigment_row_oleo_pre(
                p,
                layers,
                visual,
                oleo,
                campo,
                cy,
                cx0,
                cx1,
                &mut linha,
                &mut pre_linha,
            );
            pre[j * cw..(j + 1) * cw].copy_from_slice(&pre_linha[cx0 - 1..cx1]);
        }
        let mut lc = Vec::with_capacity(cw * ch);
        let (mut brilho, mut fina) = (Vec::with_capacity(cw * ch), Vec::with_capacity(cw * ch));
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                let (x, y) = (cx as isize, cy as isize);
                lc.push(luz_ceu_da_celula(oleo, campo, x, y));
                brilho.push(campo.brilho_em(cx, cy));
                fina.push(campo.at_fina(x, y));
            }
        }
        Self {
            oleo,
            campo,
            vista,
            cx0,
            cy0,
            cw,
            ch,
            pre,
            lc,
            brilho,
            fina,
        }
    }

    /// Os eixos do pixel da vista `k` (pode ser `−1` ou `w`: a margem do Sobel) num eixo de origem `o0`.
    fn eixo(&self, o0: f64, k: isize, c0: usize, n: usize) -> Eixo {
        let u = o0 + (k as f64 + 0.5) / self.vista.s - 0.5;
        Eixo::de(u + 1.0 - c0 as f64, n)
    }

    fn altura(&self, ex: &Eixo, ey: &Eixo) -> f64 {
        cr_com(ex, ey, &|ix, iy| self.fina[ix + iy * self.cw], false)
    }

    /// A linha `y` da vista (`w × 4` bytes RGBA). Pura: as linhas podem correr em paralelo.
    pub fn linha(&self, y: usize, out: &mut [u8]) {
        let Vista { x0, y0, w, s, .. } = self.vista;
        let (cw, ch) = (self.cw, self.ch);
        let ey = [-1isize, 0, 1].map(|d| self.eixo(y0, y as isize + d, self.cy0, ch));
        let ex_de = |x: isize| self.eixo(x0, x, self.cx0, cw);
        // As três linhas da altura ampliada com a margem de 1 px (o Sobel com o passo da vista).
        let mut h = [vec![0.0; w + 2], vec![0.0; w + 2], vec![0.0; w + 2]];
        let exs: Vec<Eixo> = (-1..=w as isize).map(ex_de).collect();
        for (r, e) in ey.iter().enumerate() {
            for (k, ex) in exs.iter().enumerate() {
                h[r][k] = self.altura(ex, e);
            }
        }
        let e = &ey[1];
        let (s2, idx) = (s * s, |ix: usize, iy: usize| ix + iy * cw);
        for x in 0..w {
            let ex = &exs[x + 1];
            let q = PreLuz {
                pr: cr(ex, e, &|i, j| self.pre[idx(i, j)].pr),
                pg: cr(ex, e, &|i, j| self.pre[idx(i, j)].pg),
                pb: cr(ex, e, &|i, j| self.pre[idx(i, j)].pb),
                pa_sem: cr(ex, e, &|i, j| self.pre[idx(i, j)].pa_sem),
                pa: cr(ex, e, &|i, j| self.pre[idx(i, j)].pa),
                tinta: cr(ex, e, &|i, j| self.pre[idx(i, j)].tinta),
                pe: cr(ex, e, &|i, j| self.pre[idx(i, j)].pe),
                w: cr(ex, e, &|i, j| self.pre[idx(i, j)].w),
            };
            let px = if q.pa_sem > 0.0 && q.w > 0.0 {
                let hh = |dx: isize, dy: isize| h[(1 + dy) as usize][(x as isize + 1 + dx) as usize];
                let n = if hh(0, 0) == 0.0 {
                    [-0.0, -0.0, 1.0]
                } else {
                    let fx = (hh(1, -1) + 2.0 * hh(1, 0) + hh(1, 1)
                        - hh(-1, -1)
                        - 2.0 * hh(-1, 0)
                        - hh(-1, 1))
                        / 8.0
                        * s;
                    let fy = (hh(-1, 1) + 2.0 * hh(0, 1) + hh(1, 1)
                        - hh(-1, -1)
                        - 2.0 * hh(0, -1)
                        - hh(1, -1))
                        / 8.0
                        * s;
                    let inv = 1.0 / (fx * fx + fy * fy + 1.0).sqrt();
                    [-fx * inv, -fy * inv, inv]
                };
                let lc0 = self.lc[idx(ex.perto, e.perto)];
                let dir = lc0.dir.map(|d0| {
                    let tem = ex
                        .i
                        .iter()
                        .all(|&i| e.i.iter().all(|&j| self.lc[idx(i, j)].dir.is_some()));
                    if tem {
                        [0, 1, 2].map(|c| {
                            cr(ex, e, &|i, j| self.lc[idx(i, j)].dir.map_or(0.0, |d| d[c]))
                        })
                    } else {
                        d0
                    }
                });
                let am = Amostra {
                    n,
                    hxx: (hh(1, 0) - 2.0 * hh(0, 0) + hh(-1, 0)) * s2,
                    hyy: (hh(0, 1) - 2.0 * hh(0, 0) + hh(0, -1)) * s2,
                    hxy: (hh(1, 1) - hh(1, -1) - hh(-1, 1) + hh(-1, -1)) / 4.0 * s2,
                    lc: LuzCeu {
                        luz: cr(ex, e, &|i, j| self.lc[idx(i, j)].luz),
                        v: cr(ex, e, &|i, j| self.lc[idx(i, j)].v),
                        dir,
                    },
                    brilho: cr(ex, e, &|i, j| self.brilho[idx(i, j)]),
                };
                let rgb = [q.pr / q.pa_sem, q.pg / q.pa_sem, q.pb / q.pa_sem];
                let (cx, cy) = (self.cx0 + ex.perto, self.cy0 + e.perto);
                let lit = cor_da_amostra(self.oleo, self.campo, cx, cy, rgb, q.tinta, &am);
                q.fecha_oleo(lit)
            } else {
                q.fecha_sem_oleo()
            };
            out[x * 4..x * 4 + 4].copy_from_slice(&px);
        }
    }

    /// A vista inteira, linha a linha, em série.
    #[must_use]
    pub fn render(&self) -> Vec<u8> {
        let w = self.vista.w;
        let mut out = vec![0u8; w * self.vista.h * 4];
        for (y, l) in out.chunks_mut(w * 4).enumerate() {
            self.linha(y, l);
        }
        out
    }
}

/// A fixture de prova (A.1): uma pincelada de tinta grossa num `96×96` e uma FACA (lâmina de 12 × 2 células, rebordo
/// e nivelamento) arrastada na diagonal por cima — a montagem dos portões da lâmina, pelo motor.
#[doc(hidden)]
pub mod prova {
    use crate::painter::Engine;

    pub const LADO: usize = 96;

    #[must_use]
    pub fn faca() -> Engine {
        let mut e = Engine::new(LADO, LADO);
        e.color = [205.0, 120.0, 45.0];
        for y in [40.0, 50.0, 58.0] {
            e.begin_direct_stroke(0, 14.0, y);
            let mut x = 14.0;
            while x < 82.0 {
                x += 3.0;
                e.direct_segment(0, 3.0);
                e.dispatch_pressure_dab_lane(0, x, y, 9.0, 40.0, None, None, None);
            }
            e.end_direct_stroke();
        }
        e.tuning.set(crate::tuning::Knob::KnifeLevel, 0.6);
        let (a, b) = ([16.0f64, 30.0f64], [80.0f64, 66.0f64]);
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let n = dx.hypot(dy);
        let rumo = [dx / n, dy / n];
        for k in 0..=n as usize {
            let c = [a[0] + rumo[0] * k as f64, a[1] + rumo[1] * k as f64];
            let sil = move |x: i32, y: i32| {
                let d = [f64::from(x) - c[0], f64::from(y) - c[1]];
                let ao_longo = d[0] * rumo[0] + d[1] * rumo[1];
                let atraves = -d[0] * rumo[1] + d[1] * rumo[0];
                f64::from(u8::from(atraves.abs() <= 12.0 && ao_longo.abs() <= 2.0))
            };
            e.pentear_rebordo(c[0], c[1], [14.0, 14.0], &sil, rumo, 4.0, 1.0);
        }
        e
    }

    /// As camadas da linha (a grade activa, opaca, visível) e o visual do produto (relevo, sem papel).
    #[must_use]
    pub fn camadas(e: &Engine) -> [crate::render::RenderLayer<'_>; 1] {
        [crate::render::RenderLayer {
            grid: e.active_grid(),
            opacity: 1.0,
            visible: true,
        }]
    }

    pub const VISUAL: crate::render::PigmentVisual = crate::render::PigmentVisual {
        paper: false,
        relevo: true,
    };

    /// O campo da grade inteira.
    #[must_use]
    pub fn campo(e: &Engine, o: &crate::oleo_real::OleoReal) -> crate::oleo_real::CampoAltura {
        let g = e.active_grid();
        crate::oleo_real::CampoAltura::de(&[(g, true)], o, 1, 1, 1, g.w, g.h)
    }

    /// HOJE: o recorte `w × h` a partir do pixel `(x0, y0)` pela linha do óleo ([`crate::render::render_pigment_row_oleo`]).
    #[must_use]
    pub fn hoje(
        e: &Engine,
        o: &crate::oleo_real::OleoReal,
        c: &crate::oleo_real::CampoAltura,
        (x0, y0, w, h): (usize, usize, usize, usize),
    ) -> Vec<u8> {
        let gw = e.active_grid().w;
        let cam = camadas(e);
        let mut linha = vec![0u8; gw * 4];
        let mut out = Vec::with_capacity(w * h * 4);
        for cy in y0 + 1..=y0 + h {
            crate::render::render_pigment_row_oleo(
                None,
                &cam,
                VISUAL,
                o,
                c,
                cy,
                x0 + 1,
                x0 + w,
                &mut linha,
            );
            out.extend_from_slice(&linha[x0 * 4..(x0 + w) * 4]);
        }
        out
    }

    /// A ampliação BILINEAR (centros) de `src` (`w × h` RGBA) por um `s` inteiro — o que a vista faz hoje.
    #[must_use]
    pub fn bilinear(src: &[u8], w: usize, h: usize, s: usize) -> Vec<u8> {
        let (vw, vh) = (w * s, h * s);
        let mut out = vec![0u8; vw * vh * 4];
        let at = |x: isize, y: isize, c: usize| {
            let (x, y) = (
                x.clamp(0, w as isize - 1) as usize,
                y.clamp(0, h as isize - 1) as usize,
            );
            f64::from(src[(x + y * w) * 4 + c])
        };
        for y in 0..vh {
            let v = (y as f64 + 0.5) / s as f64 - 0.5;
            let (fy, ty) = (v.floor(), v - v.floor());
            for x in 0..vw {
                let u = (x as f64 + 0.5) / s as f64 - 0.5;
                let (fx, tx) = (u.floor(), u - u.floor());
                let (ix, iy) = (fx as isize, fy as isize);
                for c in 0..4 {
                    let l0 = at(ix, iy, c) + (at(ix + 1, iy, c) - at(ix, iy, c)) * tx;
                    let l1 = at(ix, iy + 1, c) + (at(ix + 1, iy + 1, c) - at(ix, iy + 1, c)) * tx;
                    out[(x + y * vw) * 4 + c] = (l0 + (l1 - l0) * ty).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
        out
    }

    /// A variância do laplaciano (4 vizinhos) da luminância sobre o BRANCO de `rgba` (`w × h`), o interior.
    #[must_use]
    pub fn var_laplaciano(rgba: &[u8], w: usize, h: usize) -> f64 {
        let lum = |x: usize, y: usize| {
            let p = &rgba[(x + y * w) * 4..(x + y * w) * 4 + 4];
            let a = f64::from(p[3]) / 255.0;
            let l = 0.2126 * f64::from(p[0]) + 0.7152 * f64::from(p[1]) + 0.0722 * f64::from(p[2]);
            l * a + 255.0 * (1.0 - a)
        };
        let mut v = Vec::with_capacity(w * h);
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                v.push(
                    lum(x + 1, y) + lum(x - 1, y) + lum(x, y + 1) + lum(x, y - 1) - 4.0 * lum(x, y),
                );
            }
        }
        let m = v.iter().sum::<f64>() / v.len() as f64;
        v.iter().map(|q| (q - m) * (q - m)).sum::<f64>() / v.len() as f64
    }
}

#[cfg(test)]
#[path = "oleo_vista_tests.rs"]
mod tests;

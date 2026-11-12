//! **A.1 — o Óleo Real iluminado à resolução da VISTA** ([doc](../../../docs/Painter/49_realshader/A1_oleo_na_vista.md)).
//!
//! Hoje a tinta acende-se à resolução do documento e a vista amplia o resultado (a 400 % vê-se o pixel). Aqui cada
//! pixel da vista pede o documento em `u = x0 + (x + 0,5)/s − 0,5` (centros) e:
//! - a cor da pilha antes da luz ([`PreLuz`]), a luz e o céu ([`LuzCeu`]) e o Gloss vêm por **Catmull-Rom 4×4**
//!   separável, presos ao mínimo/máximo dos 16 vizinhos (sem anel) e com a borda presa;
//! - a altura fina vem da **B-spline cúbica INTERPOLANTE** (pré-filtro recursivo de Unser, polo `√3 − 2`, janela +
//!   8 células, borda espelhada): C2, por isso a normal e a hessiana são as ANALÍTICAS da spline e contínuas através
//!   das células (o Catmull-Rom é só C1: a hessiana saltava a cada célula e desenhava listras ao passo `s`);
//! - o material, a Subsurface e o reflexo livre são os da célula mais próxima;
//! - e fecha-se com a MESMA óptica ([`crate::oleo_real::cor_da_amostra`]) e a MESMA mistura ([`PreLuz::fecha_oleo`]).
//!
//! Lei de identidade: com `s = 1` e origem inteira a vista É a linha de hoje
//! ([`crate::render::render_pigment_row_oleo`]), ao byte por construção. `s < 1` (afastar) fica FORA:
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
    /// Os coeficientes da B-spline cúbica interpolante da altura fina (`sw` de largura; janela + [`MARGEM_SPLINE`]).
    coef: Vec<f64>,
    sw: usize,
    /// A vista a `s = 1` com origem inteira: a imagem de hoje.
    identidade: Option<Vec<u8>>,
}

/// A margem (células) do pré-filtro da spline à volta da janela: o polo `z = √3 − 2` cai a `|z|⁸ ≈ 3·10⁻⁵`.
const MARGEM_SPLINE: usize = 8;

/// O polo do pré-filtro da B-spline cúbica (Unser 1991).
const POLO: f64 = -0.267_949_192_431_122_7;

/// O pré-filtro recursivo causal + anti-causal de Unser num sinal de passo `passo`, borda ESPELHADA: amostras →
/// coeficientes da B-spline cúbica que INTERPOLA as amostras.
fn prefiltro_1d(v: &mut [f64], n: usize, passo: usize) {
    if n < 2 {
        return;
    }
    let z = POLO;
    let at = |k: usize| k * passo;
    for k in 0..n {
        v[at(k)] *= (1.0 - z) * (1.0 - 1.0 / z);
    }
    // Início causal (espelho, truncado onde |z|^k < 1e-12).
    let corte = n.min(22);
    let (mut soma, mut zk) = (v[at(0)], z);
    for k in 1..corte {
        soma += zk * v[at(k)];
        zk *= z;
    }
    v[at(0)] = soma;
    for k in 1..n {
        v[at(k)] += z * v[at(k - 1)];
    }
    v[at(n - 1)] = (z / (z * z - 1.0)) * (v[at(n - 1)] + z * v[at(n - 2)]);
    for k in (0..n - 1).rev() {
        v[at(k)] = z * (v[at(k + 1)] - v[at(k)]);
    }
}

/// O pré-filtro separável: as linhas e depois as colunas.
fn prefiltro_2d(c: &mut [f64], w: usize, h: usize) {
    for y in 0..h {
        prefiltro_1d(&mut c[y * w..(y + 1) * w], w, 1);
    }
    for x in 0..w {
        prefiltro_1d(&mut c[x..], h, w);
    }
}

/// Os pesos da B-spline cúbica em `t ∈ [0, 1)` para os coeficientes `−1..=2`: (valor, 1.ª, 2.ª derivada).
#[inline]
fn bspline(t: f64) -> [(f64, f64, f64); 4] {
    let (t2, t3, u) = (t * t, t * t * t, 1.0 - t);
    [
        (u * u * u / 6.0, -u * u / 2.0, u),
        (
            (3.0 * t3 - 6.0 * t2 + 4.0) / 6.0,
            (3.0 * t2 - 4.0 * t) / 2.0,
            3.0 * t - 2.0,
        ),
        (
            (-3.0 * t3 + 3.0 * t2 + 3.0 * t + 1.0) / 6.0,
            (-3.0 * t2 + 2.0 * t + 1.0) / 2.0,
            -3.0 * t + 1.0,
        ),
        (t3 / 6.0, t2 / 2.0, t),
    ]
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
        let (mut linha, mut pre_linha) = (vec![0u8; g.w * 4], vec![PreLuz::default(); g.w]);
        // s = 1 com a origem num centro: a linha de hoje, ao byte por construção.
        let identidade = (vista.s == 1.0 && vista.x0.fract() == 0.0 && vista.y0.fract() == 0.0)
            .then(|| {
                let (ox, oy) = (vista.x0 as usize, vista.y0 as usize);
                let mut img = vec![0u8; vista.w * vista.h * 4];
                for (y, l) in img.chunks_mut(vista.w * 4).enumerate() {
                    crate::render::render_pigment_row_oleo(
                        p,
                        layers,
                        visual,
                        oleo,
                        campo,
                        oy + y + 1,
                        ox + 1,
                        ox + vista.w,
                        &mut linha,
                    );
                    l.copy_from_slice(&linha[ox * 4..(ox + vista.w) * 4]);
                }
                img
            });
        if identidade.is_some() {
            return Self {
                oleo,
                campo,
                vista,
                cx0,
                cy0,
                cw,
                ch,
                pre: Vec::new(),
                lc: Vec::new(),
                brilho: Vec::new(),
                coef: Vec::new(),
                sw: 0,
                identidade,
            };
        }
        let mut pre = vec![PreLuz::default(); cw * ch];
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
        let mut brilho = Vec::with_capacity(cw * ch);
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                let (x, y) = (cx as isize, cy as isize);
                lc.push(luz_ceu_da_celula(oleo, campo, x, y));
                brilho.push(campo.brilho_em(cx, cy));
            }
        }
        let (sw, sh) = (cw + 2 * MARGEM_SPLINE, ch + 2 * MARGEM_SPLINE);
        let mut coef = Vec::with_capacity(sw * sh);
        for j in 0..sh {
            for i in 0..sw {
                let x = (cx0 + i) as isize - MARGEM_SPLINE as isize;
                let y = (cy0 + j) as isize - MARGEM_SPLINE as isize;
                coef.push(campo.at_fina(x, y));
            }
        }
        prefiltro_2d(&mut coef, sw, sh);
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
            coef,
            sw,
            identidade,
        }
    }

    /// A altura fina pela B-spline cúbica INTERPOLANTE em `(cx, cy)` (células, 1-based), com o gradiente e a hessiana
    /// ANALÍTICOS (C2: contínuos através das células): `(h, hx, hy, hxx, hyy, hxy)` em unidades da célula.
    fn spline(&self, cx: f64, cy: f64) -> [f64; 6] {
        let bx = cx - (self.cx0 as f64 - MARGEM_SPLINE as f64);
        let by = cy - (self.cy0 as f64 - MARGEM_SPLINE as f64);
        let (fx, fy) = (bx.floor(), by.floor());
        let (wx, wy) = (bspline(bx - fx), bspline(by - fy));
        let (ix, iy) = (fx as usize - 1, fy as usize - 1);
        let mut r = [0.0; 6];
        for (j, (b0, b1, b2)) in wy.iter().enumerate() {
            let linha = &self.coef[(iy + j) * self.sw + ix..(iy + j) * self.sw + ix + 4];
            let (mut a0, mut a1, mut a2) = (0.0, 0.0, 0.0);
            for (i, (c0, c1, c2)) in wx.iter().enumerate() {
                a0 += c0 * linha[i];
                a1 += c1 * linha[i];
                a2 += c2 * linha[i];
            }
            r[0] += b0 * a0;
            r[1] += b0 * a1;
            r[2] += b1 * a0;
            r[3] += b0 * a2;
            r[4] += b2 * a0;
            r[5] += b1 * a1;
        }
        r
    }

    /// Os eixos do pixel da vista `k` (pode ser `−1` ou `w`: a margem do Sobel) num eixo de origem `o0`.
    fn eixo(&self, o0: f64, k: isize, c0: usize, n: usize) -> Eixo {
        let u = o0 + (k as f64 + 0.5) / self.vista.s - 0.5;
        Eixo::de(u + 1.0 - c0 as f64, n)
    }

    /// A geometria da altura no pixel `(x, y)` da vista, em unidades do DOCUMENTO: `[h, hx, hy, hxx, hyy, hxy]` —
    /// o que [`Self::linha`] dá à óptica.
    pub(crate) fn geometria(&self, x: usize, y: usize) -> [f64; 6] {
        let Vista { x0, y0, s, .. } = self.vista;
        let centro = |o0: f64, k: usize| o0 + (k as f64 + 0.5) / s - 0.5 + 1.0;
        self.spline(centro(x0, x), centro(y0, y))
    }

    /// A pilha antes da luz ([`PreLuz`]) interpolada no pixel `(x, y)` da vista — a que [`Self::linha`] fecha.
    pub(crate) fn pre_do_pixel(&self, x: usize, y: usize) -> PreLuz {
        let Vista { x0, y0, .. } = self.vista;
        let e = &self.eixo(y0, y as isize, self.cy0, self.ch);
        let ex = &self.eixo(x0, x as isize, self.cx0, self.cw);
        self.pre_em(ex, e)
    }

    fn pre_em(&self, ex: &Eixo, e: &Eixo) -> PreLuz {
        let idx = |ix: usize, iy: usize| ix + iy * self.cw;
        PreLuz {
            pr: cr(ex, e, &|i, j| self.pre[idx(i, j)].pr),
            pg: cr(ex, e, &|i, j| self.pre[idx(i, j)].pg),
            pb: cr(ex, e, &|i, j| self.pre[idx(i, j)].pb),
            pa_sem: cr(ex, e, &|i, j| self.pre[idx(i, j)].pa_sem),
            pa: cr(ex, e, &|i, j| self.pre[idx(i, j)].pa),
            tinta: cr(ex, e, &|i, j| self.pre[idx(i, j)].tinta),
            pe: cr(ex, e, &|i, j| self.pre[idx(i, j)].pe),
            w: cr(ex, e, &|i, j| self.pre[idx(i, j)].w),
        }
    }

    /// A linha `y` da vista (`w × 4` bytes RGBA). Pura: as linhas podem correr em paralelo.
    pub fn linha(&self, y: usize, out: &mut [u8]) {
        let Vista { x0, y0, w, .. } = self.vista;
        if let Some(img) = &self.identidade {
            out[..w * 4].copy_from_slice(&img[y * w * 4..(y + 1) * w * 4]);
            return;
        }
        let (cw, ch) = (self.cw, self.ch);
        let e = &self.eixo(y0, y as isize, self.cy0, ch);
        let idx = |ix: usize, iy: usize| ix + iy * cw;
        for x in 0..w {
            let ex = &self.eixo(x0, x as isize, self.cx0, cw);
            let q = self.pre_em(ex, e);
            let px = if q.pa_sem > 0.0 && q.w > 0.0 {
                let [_, fx, fy, hxx, hyy, hxy] = self.geometria(x, y);
                // O papel é plano onde as 16 células à volta não têm relevo (a spline oscila perto da orla).
                let papel = ex.i.iter().all(|&i| {
                    e.i.iter().all(|&j| {
                        let (cx, cy) = (self.cx0 + i, self.cy0 + j);
                        self.campo.at_fina(cx as isize, cy as isize) == 0.0
                    })
                });
                let n = if papel {
                    [-0.0, -0.0, 1.0]
                } else {
                    let inv = 1.0 / (fx * fx + fy * fy + 1.0).sqrt();
                    [-fx * inv, -fy * inv, inv]
                };
                let lc0 = self.lc[idx(ex.perto, e.perto)];
                let dir = lc0.dir.map(|d0| {
                    let tem =
                        ex.i.iter()
                            .all(|&i| e.i.iter().all(|&j| self.lc[idx(i, j)].dir.is_some()));
                    if tem {
                        [0, 1, 2].map(|c| {
                            cr(ex, e, &|i, j| self.lc[idx(i, j)].dir.map_or(0.0, |d| d[c]))
                        })
                    } else {
                        d0
                    }
                });
                // O antisserrilhado do brilho mede a curvatura no passo do PÍXEL que acende: na vista ele é `1/s` da
                // célula (a `s = 1`, ao bit). Com a da célula a 400 % o lobo alargava-se 16× a mais e apagava o relevo.
                let passo = 1.0 / self.vista.s;
                let am = Amostra {
                    n,
                    hxx: hxx * passo,
                    hyy: hyy * passo,
                    hxy: hxy * passo,
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
#[path = "oleo_vista_prova.rs"]
pub mod prova;

#[cfg(test)]
#[path = "oleo_vista_tests.rs"]
mod tests;

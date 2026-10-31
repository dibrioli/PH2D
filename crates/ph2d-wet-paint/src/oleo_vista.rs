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
        let identidade = (vista.s == 1.0 && vista.x0.fract() == 0.0 && vista.y0.fract() == 0.0).then(|| {
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

    /// A linha `y` da vista (`w × 4` bytes RGBA). Pura: as linhas podem correr em paralelo.
    pub fn linha(&self, y: usize, out: &mut [u8]) {
        let Vista { x0, y0, w, s, .. } = self.vista;
        if let Some(img) = &self.identidade {
            out[..w * 4].copy_from_slice(&img[y * w * 4..(y + 1) * w * 4]);
            return;
        }
        let (cw, ch) = (self.cw, self.ch);
        let e = &self.eixo(y0, y as isize, self.cy0, ch);
        let idx = |ix: usize, iy: usize| ix + iy * cw;
        let centro = |o0: f64, k: usize| o0 + (k as f64 + 0.5) / s - 0.5 + 1.0;
        let cy_f = centro(y0, y);
        for x in 0..w {
            let ex = &self.eixo(x0, x as isize, self.cx0, cw);
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
                let [_, fx, fy, hxx, hyy, hxy] = self.spline(centro(x0, x), cy_f);
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
                    hxx,
                    hyy,
                    hxy,
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

    /// O laplaciano (4 vizinhos) da luminância sobre o BRANCO, no interior (`(w−2) × (h−2)`, por linhas).
    #[must_use]
    pub fn laplaciano(rgba: &[u8], w: usize, h: usize) -> Vec<f64> {
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
        v
    }

    /// O PICO da célula: no laplaciano, a média da |DFT| 1D no bin do período `s` px (`n/s`) ÷ a média dos bins
    /// vizinhos (`±1..=±3`), por linha e por coluna; devolve o maior dos dois. Detalhe verdadeiro ≈ 1; um padrão
    /// periódico ao passo da célula sobe acima.
    #[must_use]
    pub fn pico_da_celula(rgba: &[u8], w: usize, h: usize, s: usize) -> f64 {
        let l = laplaciano(rgba, w, h);
        let (lw, lh) = (w - 2, h - 2);
        let mag = |sinal: &dyn Fn(usize) -> f64, n: usize, k: usize| {
            let (mut re, mut im) = (0.0, 0.0);
            for j in 0..n {
                let a = -2.0 * core::f64::consts::PI * (k * j) as f64 / n as f64;
                re += sinal(j) * a.cos();
                im += sinal(j) * a.sin();
            }
            re.hypot(im)
        };
        let razao = |linhas: usize, n: usize, at: &dyn Fn(usize, usize) -> f64| {
            let k0 = (n as f64 / s as f64).round() as usize;
            let (mut pico, mut viz) = (0.0, 0.0);
            for r in 0..linhas {
                let f = |j: usize| at(r, j);
                pico += mag(&f, n, k0);
                for d in 1..=3 {
                    viz += (mag(&f, n, k0 - d) + mag(&f, n, k0 + d)) / 6.0;
                }
            }
            pico / viz
        };
        let por_linha = razao(lh, lw, &|r, j| l[j + r * lw]);
        let por_coluna = razao(lw, lh, &|r, j| l[r + j * lw]);
        por_linha.max(por_coluna)
    }

    /// A fracção de pixels com `|lap − mediana| > 6 σ` robusto (`σ = 1,4826 · MAD`).
    #[must_use]
    pub fn fraccao_outliers(rgba: &[u8], w: usize, h: usize) -> f64 {
        let l = laplaciano(rgba, w, h);
        let mediana = |v: &mut Vec<f64>| {
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        let med = mediana(&mut l.clone());
        let sigma = 1.4826 * mediana(&mut l.iter().map(|q| (q - med).abs()).collect());
        l.iter().filter(|q| (*q - med).abs() > 6.0 * sigma).count() as f64 / l.len() as f64
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

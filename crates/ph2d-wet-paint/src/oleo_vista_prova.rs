//! A fixture de prova (A.1): uma pincelada de tinta grossa num `96×96` e uma FACA (lâmina de 12 × 2 células, rebordo
//! e nivelamento) arrastada na diagonal por cima — a montagem dos portões da lâmina, pelo motor.

use crate::painter::Engine;
use ph2d_det_math::DetMath as _;

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
    let n = dx.det_hypot(dy);
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

/// A [`super::PreparoVista::geometria`] da vista `v` sobre a faca com a ALTURA trocada por `f(cx, cy)` (células):
/// para medir a escala da curvatura contra uma altura analítica.
#[must_use]
pub fn geometria_analitica(
    f: &dyn Fn(f64, f64) -> f64,
    v: super::Vista,
    pixels: &[(usize, usize)],
) -> Vec<[f64; 6]> {
    let e = faca();
    let o = crate::oleo_real::OleoReal::default();
    let c = campo(&e, &o);
    let mut pv = super::PreparoVista::novo(None, &camadas(&e), VISUAL, &o, &c, v);
    let sh = pv.coef.len() / pv.sw;
    let (ox, oy) = (
        pv.cx0 as f64 - super::MARGEM_SPLINE as f64,
        pv.cy0 as f64 - super::MARGEM_SPLINE as f64,
    );
    for j in 0..sh {
        for i in 0..pv.sw {
            pv.coef[i + j * pv.sw] = f(ox + i as f64, oy + j as f64);
        }
    }
    super::prefiltro_2d(&mut pv.coef, pv.sw, sh);
    pixels.iter().map(|&(x, y)| pv.geometria(x, y)).collect()
}

/// A [`super::PreparoVista::pre_do_pixel`] e a `geometria` da faca nos `pixels` da vista `v`, e o [`PreLuz`] e a
/// altura fina das CÉLULAS `celulas` (1-based) pela linha de hoje.
#[must_use]
pub fn alinhamento(
    v: super::Vista,
    pixels: &[(usize, usize)],
    celulas: &[(usize, usize)],
) -> (
    Vec<(crate::render::PreLuz, f64)>,
    Vec<(crate::render::PreLuz, f64)>,
) {
    let e = faca();
    let o = crate::oleo_real::OleoReal::default();
    let c = campo(&e, &o);
    let cam = camadas(&e);
    let pv = super::PreparoVista::novo(None, &cam, VISUAL, &o, &c, v);
    let vista = pixels
        .iter()
        .map(|&(x, y)| (pv.pre_do_pixel(x, y), pv.geometria(x, y)[0]))
        .collect();
    let gw = e.active_grid().w;
    let (mut linha, mut pre) = (
        vec![0u8; gw * 4],
        vec![crate::render::PreLuz::default(); gw],
    );
    let hoje = celulas
        .iter()
        .map(|&(cx, cy)| {
            crate::render::render_pigment_row_oleo_pre(
                None, &cam, VISUAL, &o, &c, cy, cx, cx, &mut linha, &mut pre,
            );
            (pre[cx - 1], c.at_fina(cx as isize, cy as isize))
        })
        .collect();
    (vista, hoje)
}

/// Uma grade `lado × lado` COBERTA de tinta uniforme (até à borda) e a vista `v` dela.
#[must_use]
pub fn uniforme(lado: usize, v: super::Vista) -> Vec<u8> {
    let mut g = crate::grid::Grid::new(lado, lado);
    for y in 1..=lado {
        for x in 1..=lado {
            g.sett[x + y * g.s] = 3000.0;
        }
    }
    g.sett_rgb.fill([200.0, 110.0, 50.0]);
    let o = crate::oleo_real::OleoReal::default();
    let c = crate::oleo_real::CampoAltura::de(&[(&g, true)], &o, 1, 1, 1, lado, lado);
    let cam = [crate::render::RenderLayer {
        grid: &g,
        opacity: 1.0,
        visible: true,
    }];
    super::PreparoVista::novo(None, &cam, VISUAL, &o, &c, v).render()
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
            v.push(lum(x + 1, y) + lum(x - 1, y) + lum(x, y + 1) + lum(x, y - 1) - 4.0 * lum(x, y));
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
            re += sinal(j) * a.det_cos();
            im += sinal(j) * a.det_sin();
        }
        re.det_hypot(im)
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
            v.push(lum(x + 1, y) + lum(x - 1, y) + lum(x, y + 1) + lum(x, y - 1) - 4.0 * lum(x, y));
        }
    }
    let m = v.iter().sum::<f64>() / v.len() as f64;
    v.iter().map(|q| (q - m) * (q - m)).sum::<f64>() / v.len() as f64
}

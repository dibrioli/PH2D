//! **Os campos do Rewet persistem entre quadros** (B16.5, molde de `watercolor_reserve/cache.rs`).
//!
//! As somas do borrão são INTEIRAS (`box_blur`), logo uma célula da grelha `ds×` a mais de `R`
//! (`r2` com dwell/água, senão `r1`) da borda de uma janela tem o MESMO valor em qualquer janela
//! que a contenha. Os planos de entrada (`base`, `ground`, `wet_soak`, `stroke_water`) só mudam,
//! entre dois quadros, dentro do sujo do quadro (dabs, o `smear_wet_base` do Smudge funde o que
//! arrasta no `wet_frame_dirty`). ⇒ os campos vivem numa grelha do tamanho da TELA, e cada quadro
//! recalcula só o sujo `⊕ R` e os ladrilhos ([`LADO`] células) da janela nunca calculados.
//!
//! ⚠️ Toda escrita EM MASSA invalida (`wet_rewet_cache = None`, ao lado de cada
//! `wet_reserve_cache = None`); a chave (tamanho, `ds`, Spread, dwell, água, e os `Arc` da base e do
//! chão) apanha o chão refeito em cada pen-down. Gate `o_campo_do_rewet_guardado_e_o_refeito`.

use rayon::prelude::*;

use super::{REWET_DS_SPREAD, REWET_FOLGA_CELULAS, RewetFields, box_blur, box_blur4};
use crate::compositor::Region;

/// O lado de um ladrilho, em CÉLULAS da grelha.
const LADO: usize = 64;

#[cfg(test)]
thread_local! {
    /// O oráculo do B16.5: sem a cache (o `build_rewet_fields` de sempre, verbatim).
    pub(in crate::tool::paint) static REWET_SEM_CACHE: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

/// Um rectângulo de células, meio-aberto: `[x0, x1) × [y0, y1)`.
type Rect = (usize, usize, usize, usize);

#[derive(Clone, Copy, PartialEq, Eq)]
struct Chave {
    fw: usize,
    fh: usize,
    ds: usize,
    spread: usize,
    soaked: bool,
    watered: bool,
    base: usize,
    ground: usize,
}

/// As entradas dos campos (as de [`super::build_rewet_fields`]).
#[derive(Clone, Copy)]
pub(in crate::tool::paint) struct Entradas<'a> {
    pub base: &'a [u8],
    pub ground: &'a [u8],
    pub wet_soak: &'a [u8],
    pub soaked: bool,
    pub stroke_water: &'a [u8],
    pub watered: bool,
    pub fw: usize,
    pub fh: usize,
}

/// Os campos da tela inteira na grelha `ds×` e que ladrilhos já foram calculados.
#[derive(Clone)]
pub(in crate::tool::paint) struct RewetCache {
    k: Chave,
    gw: usize,
    gh: usize,
    pres: Vec<f32>,
    soak: Vec<f32>,
    soak_halo: Vec<f32>,
    water_halo: Vec<f32>,
    near: [Vec<f32>; 4],
    far: Option<[Vec<f32>; 4]>,
    feito: Vec<bool>,
}

/// Os campos da janela de leitura, lidos da grelha guardada e recalculados só onde a janela é nova
/// ou onde as entradas mudaram desde o último composite (`changed`, o sujo do quadro, em px).
pub(in crate::tool::paint) fn build_rewet_fields_cached(
    e: Entradas<'_>,
    (rx0, ry0, rx1, ry1): (usize, usize, usize, usize),
    spread: usize,
    changed: Option<Region>,
    cache: Option<RewetCache>,
) -> (RewetFields, RewetCache) {
    let (fw, fh) = (e.fw, e.fh);
    let ds = (spread / REWET_DS_SPREAD).clamp(1, 4);
    let m = if ds > 1 { REWET_FOLGA_CELULAS } else { 0 };
    let (gw, gh) = (fw.div_ceil(ds), fh.div_ceil(ds));
    let lox0 = (rx0 / ds).saturating_sub(m);
    let loy0 = (ry0 / ds).saturating_sub(m);
    let lw = (rx1.div_ceil(ds) + m).min(gw) - lox0;
    let lh = (ry1.div_ceil(ds) + m).min(gh) - loy0;
    let r1 = (spread / ds).max(1);
    let r2 = ((spread * 2) / ds).max(1);
    let rr = if e.soaked || e.watered { r2 } else { r1 };
    let k = Chave {
        fw,
        fh,
        ds,
        spread,
        soaked: e.soaked,
        watered: e.watered,
        base: e.base.as_ptr() as usize,
        ground: e.ground.as_ptr() as usize,
    };
    let g = gw * gh;
    let mut c = match cache {
        Some(c) if c.k == k => c,
        _ => RewetCache {
            k,
            gw,
            gh,
            pres: vec![0.0; g],
            soak: vec![0.0; g],
            soak_halo: if e.soaked { vec![0.0; g] } else { Vec::new() },
            water_halo: if e.watered { vec![0.0; g] } else { Vec::new() },
            near: std::array::from_fn(|_| vec![0.0; g]),
            far: e.soaked.then(|| std::array::from_fn(|_| vec![0.0; g])),
            feito: vec![false; gw.div_ceil(LADO) * gh.div_ceil(LADO)],
        },
    };
    // 1. O sujo do quadro `⊕ R`: o píxel `p` só é amostrado pela célula `p / ds`.
    if let Some(d) = changed.filter(|_| c.feito.iter().any(|&f| f)) {
        let (dx0, dy0) = (d.x as usize / ds, d.y as usize / ds);
        let dx1 = (d.x as usize + d.w as usize).div_ceil(ds);
        let dy1 = (d.y as usize + d.h as usize).div_ceil(ds);
        let u = (
            dx0.saturating_sub(rr),
            dy0.saturating_sub(rr),
            (dx1 + rr).min(gw),
            (dy1 + rr).min(gh),
        );
        if u.0 < u.2 && u.1 < u.3 {
            recompute(&mut c, &e, u, (r1, r2, rr));
        }
    }
    // 2. Os ladrilhos da janela nunca calculados (troços iguais de filas seguidas juntam-se).
    let tw = gw.div_ceil(LADO);
    let (tx0, tx1) = (lox0 / LADO, (lox0 + lw).div_ceil(LADO));
    let (ty0, ty1) = (loy0 / LADO, (loy0 + lh).div_ceil(LADO));
    let mut trocos: Vec<Rect> = Vec::new();
    for ty in ty0..ty1 {
        let mut tx = tx0;
        while tx < tx1 {
            if c.feito[ty * tw + tx] {
                tx += 1;
                continue;
            }
            let t0 = tx;
            while tx < tx1 && !c.feito[ty * tw + tx] {
                c.feito[ty * tw + tx] = true;
                tx += 1;
            }
            match trocos
                .iter_mut()
                .find(|t| t.0 == t0 && t.1 == tx && t.3 == ty)
            {
                Some(t) => t.3 = ty + 1,
                None => trocos.push((t0, tx, ty, ty + 1)),
            }
        }
    }
    for (x0, x1, y0, y1) in trocos {
        let u = (x0 * LADO, y0 * LADO, (x1 * LADO).min(gw), (y1 * LADO).min(gh));
        recompute(&mut c, &e, u, (r1, r2, rr));
    }
    // 3. A janela sai da grelha (as mesmas `lox0/loy0/lw/lh` do refeito: o `samp` não muda).
    let tira = |src: &[f32]| recorta(src, gw, (lox0, loy0), (lw, lh));
    let f = RewetFields {
        pres: tira(&c.pres),
        soak_raw: if e.soaked {
            tira(&c.soak)
        } else {
            vec![0.0; lw * lh]
        },
        soak_halo: if e.soaked {
            tira(&c.soak_halo)
        } else {
            Vec::new()
        },
        water_halo: if e.watered {
            tira(&c.water_halo)
        } else {
            Vec::new()
        },
        near: std::array::from_fn(|i| tira(&c.near[i])),
        far: c
            .far
            .as_ref()
            .map(|far| std::array::from_fn(|i| tira(&far[i]))),
        ds,
        lw,
        lh,
        lox0,
        loy0,
    };
    (f, c)
}

/// O rectângulo `(o, (w, h))` de um plano de largura `gw`, linha a linha.
fn recorta(src: &[f32], gw: usize, (ox, oy): (usize, usize), (w, h): (usize, usize)) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    out.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
        let s = (oy + j) * gw + ox;
        row.copy_from_slice(&src[s..s + w]);
    });
    out
}

/// Recalcula as células `u` sobre a janela `u ⊕ R` (o valor de `u` é o de qualquer janela maior) e
/// copia só `u` para a grelha guardada.
fn recompute(c: &mut RewetCache, e: &Entradas<'_>, u: Rect, (r1, r2, rr): (usize, usize, usize)) {
    let (gw, gh, ds) = (c.gw, c.gh, c.k.ds);
    let w = (
        u.0.saturating_sub(rr),
        u.1.saturating_sub(rr),
        (u.2 + rr).min(gw),
        (u.3 + rr).min(gh),
    );
    let (ww, wh) = (w.2 - w.0, w.3 - w.1);
    let nn = ww * wh;
    let (mut pres, mut wr, mut wg, mut wb) = (vec![0.0f32; nn], vec![0.0; nn], vec![0.0; nn], vec![0.0; nn]);
    let (mut soak, mut water) = (vec![0.0f32; nn], vec![0.0f32; nn]);
    let half = ds / 2;
    let (fw, fh) = (e.fw, e.fh);
    pres.par_chunks_mut(ww)
        .zip(wr.par_chunks_mut(ww))
        .zip(wg.par_chunks_mut(ww))
        .zip(wb.par_chunks_mut(ww))
        .zip(soak.par_chunks_mut(ww))
        .zip(water.par_chunks_mut(ww))
        .enumerate()
        .for_each(|(lj, (((((prow, rrow), grow), brow), srow), wrow))| {
            let gy = (((w.1 + lj) * ds) + half).min(fh - 1);
            for li in 0..ww {
                let gx = (((w.0 + li) * ds) + half).min(fw - 1);
                let bi = (gy * fw + gx) * 4;
                let (p, [r, g, b]) =
                    super::super::watercolor_mistura::presenca_de_tinta(e.base, e.ground, bi);
                prow[li] = p;
                rrow[li] = r * p;
                grow[li] = g * p;
                brow[li] = b * p;
                if e.soaked {
                    srow[li] = f32::from(e.wet_soak[gy * fw + gx]) / 255.0;
                }
                if e.watered {
                    wrow[li] = f32::from(e.stroke_water[gy * fw + gx]) / 255.0;
                }
            }
        });
    let near = box_blur4([&pres, &wr, &wg, &wb], ww, wh, r1);
    let o = (u.0 - w.0, u.1 - w.1);
    let um = (u.2 - u.0, u.3 - u.1);
    let poe = |dst: &mut [f32], src: &[f32]| {
        dst.par_chunks_mut(gw)
            .skip(u.1)
            .take(um.1)
            .enumerate()
            .for_each(|(j, row)| {
                let s = (o.1 + j) * ww + o.0;
                row[u.0..u.2].copy_from_slice(&src[s..s + um.0]);
            });
    };
    poe(&mut c.pres, &pres);
    for (dst, src) in c.near.iter_mut().zip(near.iter()) {
        poe(dst, src);
    }
    if e.watered {
        poe(&mut c.water_halo, &box_blur(&water, ww, wh, r2));
    }
    if e.soaked {
        poe(&mut c.soak, &soak);
        poe(&mut c.soak_halo, &box_blur(&soak, ww, wh, r2));
        let far = box_blur4([&pres, &wr, &wg, &wb], ww, wh, r2);
        if let Some(cf) = c.far.as_mut() {
            for (dst, src) in cf.iter_mut().zip(far.iter()) {
                poe(dst, src);
            }
        }
    }
}

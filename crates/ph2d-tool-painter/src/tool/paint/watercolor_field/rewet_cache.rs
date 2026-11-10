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
    /// O caminho de antes do híbrido (B16b-cont): sempre a cache, mesmo quando custa mais.
    pub(in crate::tool::paint) static REWET_SEM_HIBRIDO: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
    /// Quantos quadros tomaram o caminho direto do híbrido.
    pub(in crate::tool::paint) static REWET_DIRETOS: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
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
    tw: usize,
    pres: Plano,
    soak: Plano,
    soak_halo: Plano,
    water_halo: Plano,
    near: [Plano; 4],
    far: Option<[Plano; 4]>,
    feito: Vec<bool>,
}

/// Um plano da grelha em ladrilhos de `LADO²` células, cada um alocado na 1.ª escrita (só os
/// ladrilhos `feito` se escrevem: a memória é a da área visitada, não a da tela).
#[derive(Clone, Default)]
struct Plano(Vec<Option<Box<[f32]>>>);

impl Plano {
    fn novo(n: usize) -> Self {
        Plano(vec![None; n])
    }
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
    let tw = gw.div_ceil(LADO);
    let nt = tw * gh.div_ceil(LADO);
    let mut c = match cache {
        Some(c) if c.k == k => c,
        _ => RewetCache {
            k,
            gw,
            gh,
            tw,
            pres: Plano::novo(nt),
            soak: Plano::novo(nt),
            soak_halo: Plano::novo(if e.soaked { nt } else { 0 }),
            water_halo: Plano::novo(if e.watered { nt } else { 0 }),
            near: std::array::from_fn(|_| Plano::novo(nt)),
            far: e.soaked.then(|| std::array::from_fn(|_| Plano::novo(nt))),
            feito: vec![false; nt],
        },
    };
    // 1. O sujo do quadro `⊕ R`: o píxel `p` só é amostrado pela célula `p / ds`.
    let sujo = changed
        .map(|d| {
            let (dx0, dy0) = (d.x as usize / ds, d.y as usize / ds);
            let dx1 = (d.x as usize + d.w as usize).div_ceil(ds);
            let dy1 = (d.y as usize + d.h as usize).div_ceil(ds);
            (
                dx0.saturating_sub(rr),
                dy0.saturating_sub(rr),
                (dx1 + rr).min(gw),
                (dy1 + rr).min(gh),
            )
        })
        .filter(|u| u.0 < u.2 && u.1 < u.3);
    let (tx0, tx1) = (lox0 / LADO, (lox0 + lw).div_ceil(LADO));
    let (ty0, ty1) = (loy0 / LADO, (loy0 + lh).div_ceil(LADO));
    // HÍBRIDO: refazer o sujo custa o sujo ⊕ 2R; ≥ ½ janela (o quadro do traço) ⇒ refazer direto (o
    // oráculo), sem encher ladrilhos, e esquecer os do sujo ⊕ R, que envelheceram. A cache enche-se
    // nos quadros de janela ≫ sujo (recomposição, secagem), onde ganha.
    let sujo_2r = sujo.map_or(0, |u| {
        ((u.2 + rr).min(gw) - u.0.saturating_sub(rr))
            * ((u.3 + rr).min(gh) - u.1.saturating_sub(rr))
    });
    #[cfg(test)]
    let hibrido = !REWET_SEM_HIBRIDO.get();
    #[cfg(not(test))]
    let hibrido = true;
    if let Some(u) = sujo.filter(|_| hibrido && 2 * sujo_2r >= lw * lh) {
        esquece(&mut c, u);
        #[cfg(test)]
        REWET_DIRETOS.set(REWET_DIRETOS.get() + 1);
        let f = super::build_rewet_fields(
            e.base,
            e.ground,
            e.wet_soak,
            e.soaked,
            e.stroke_water,
            e.watered,
            (fw, fh),
            (rx0, ry0, rx1, ry1),
            spread,
        );
        return (f, c);
    }
    if let Some(u) = sujo.filter(|_| c.feito.iter().any(|&f| f)) {
        recompute(&mut c, &e, u, (r1, r2, rr));
    }
    // 2. Os ladrilhos da janela nunca calculados (troços iguais de filas seguidas juntam-se).
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
        let u = (
            x0 * LADO,
            y0 * LADO,
            (x1 * LADO).min(gw),
            (y1 * LADO).min(gh),
        );
        recompute(&mut c, &e, u, (r1, r2, rr));
    }
    // 3. A janela sai da grelha (as mesmas `lox0/loy0/lw/lh` do refeito: o `samp` não muda).
    let tira = |src: &Plano| recorta(src, tw, (lox0, loy0), (lw, lh));
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

/// Larga os ladrilhos que tocam as células `u` (deixam de ser `feito`; a próxima visita refá-los).
fn esquece(c: &mut RewetCache, u: Rect) {
    for ty in u.1 / LADO..u.3.div_ceil(LADO) {
        for tx in u.0 / LADO..u.2.div_ceil(LADO) {
            let i = ty * c.tw + tx;
            if !c.feito[i] {
                continue;
            }
            c.feito[i] = false;
            let planos = [
                &mut c.pres,
                &mut c.soak,
                &mut c.soak_halo,
                &mut c.water_halo,
            ]
            .into_iter()
            .chain(c.near.iter_mut())
            .chain(c.far.iter_mut().flatten());
            for p in planos {
                if let Some(t) = p.0.get_mut(i) {
                    *t = None;
                }
            }
        }
    }
}

/// O rectângulo `(o, (w, h))` de um plano com `tw` ladrilhos por fila, linha a linha (ladrilho por
/// alocar = zeros).
fn recorta(src: &Plano, tw: usize, (ox, oy): (usize, usize), (w, h): (usize, usize)) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    out.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
        let (ty, ly) = ((oy + j) / LADO, (oy + j) % LADO);
        let mut x = ox;
        while x < ox + w {
            let (tx, lx) = (x / LADO, x % LADO);
            let n = (LADO - lx).min(ox + w - x);
            if let Some(t) = &src.0[ty * tw + tx] {
                let s = ly * LADO + lx;
                row[x - ox..x - ox + n].copy_from_slice(&t[s..s + n]);
            }
            x += n;
        }
    });
    out
}

/// Escreve as células `u` (lidas de `src`, de largura `ww`, com `u` em `o`) nos ladrilhos `feito`.
fn poe(
    p: &mut Plano,
    feito: &[bool],
    tw: usize,
    u: Rect,
    src: &[f32],
    ww: usize,
    o: (usize, usize),
) {
    let (tx0, tx1) = (u.0 / LADO, u.2.div_ceil(LADO));
    let (ty0, ty1) = (u.1 / LADO, u.3.div_ceil(LADO));
    p.0.par_iter_mut()
        .enumerate()
        .filter(|(i, _)| {
            let (tx, ty) = (i % tw, i / tw);
            feito[*i] && (tx0..tx1).contains(&tx) && (ty0..ty1).contains(&ty)
        })
        .for_each(|(i, t)| {
            let (bx, by) = ((i % tw) * LADO, (i / tw) * LADO);
            let t = t.get_or_insert_with(|| vec![0.0f32; LADO * LADO].into_boxed_slice());
            let (x0, x1) = (u.0.max(bx), u.2.min(bx + LADO));
            for y in u.1.max(by)..u.3.min(by + LADO) {
                let s = (o.1 + y - u.1) * ww + o.0 + x0 - u.0;
                let d = (y - by) * LADO + x0 - bx;
                t[d..d + x1 - x0].copy_from_slice(&src[s..s + x1 - x0]);
            }
        });
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
    let (mut pres, mut wr, mut wg, mut wb) = (
        vec![0.0f32; nn],
        vec![0.0; nn],
        vec![0.0; nn],
        vec![0.0; nn],
    );
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
    let (tw, feito) = (c.tw, &c.feito);
    let poe = |dst: &mut Plano, src: &[f32]| poe(dst, feito, tw, u, src, ww, o);
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

#[cfg(test)]
impl RewetCache {
    /// `(bytes dos ladrilhos alocados, bytes da grelha densa dos mesmos planos)`.
    pub(in crate::tool::paint) fn bytes(&self) -> (usize, usize) {
        let planos = || {
            [&self.pres, &self.soak, &self.soak_halo, &self.water_halo]
                .into_iter()
                .chain(self.near.iter())
                .chain(self.far.iter().flatten())
        };
        let vivos: usize = planos().map(|p| p.0.iter().flatten().count()).sum();
        let densos = planos().filter(|p| !p.0.is_empty()).count();
        (vivos * LADO * LADO * 4, densos * self.gw * self.gh * 4)
    }
}

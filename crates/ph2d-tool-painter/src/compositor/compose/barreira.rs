//! B19.6: the spatial barrier. The root stack is cut at each VISIBLE spatial item (a spatial
//! adjustment, or a group holding one): the per-pixel runs between them compose in row bands, and
//! each spatial item runs alone over the whole region (its kernel reads the neighbours; the kernels
//! split their own rows). Every spatial item is an adjustment or a group, which resets the clip
//! chain, so each run starts in the state the single walk had there: the same bits at any band count.

use super::*;

/// Walk `stack.root()` over the clamped region `r` into `acc` (and `t`), runs in `fios` bands. `zera`
/// (B19.12): `acc` holds stale values — the first run zeroes each cache strip right before composing it.
#[allow(clippy::too_many_arguments)]
pub(super) fn compor_segmentos(
    acc: &mut [[f32; 4]],
    mut t: Option<&mut [[f32; 3]]>,
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    canvas: (u32, u32),
    r: Region,
    (vidros, fios, mut zera): (Option<&super::super::vidro::Vidros>, usize, bool),
) {
    let root = stack.root();
    let mut fim = root.len();
    // No strip walk first (empty stack, or a spatial item at the bottom): zero it all up front.
    if zera
        && root
            .last()
            .is_none_or(|&b| has_spatial_adjustment(stack, &[b]))
    {
        use rayon::prelude::*;
        acc.par_chunks_mut(1 << 15).for_each(|c| c.fill([0.0; 4]));
        zera = false;
    }
    while fim > 0 {
        let mut ini = fim;
        while ini > 0 && !has_spatial_adjustment(stack, &root[ini - 1..ini]) {
            ini -= 1;
        }
        if ini < fim {
            em_faixas(
                acc,
                t.as_deref_mut(),
                &root[ini..fim],
                stack,
                src,
                canvas,
                r,
                vidros,
                (fios, zera),
            );
            zera = false;
        }
        if ini == 0 {
            break;
        }
        composite_into(
            acc,
            &root[ini - 1..ini],
            stack,
            src,
            canvas,
            r.x,
            r.y,
            r.w,
            r.h,
            0,
            None,
            vidros.zip(t.as_deref_mut()),
        );
        fim = ini - 1;
    }
}

/// [`composite_region_linear`] over an already-clamped `region`, with no halo.
#[cfg(test)]
pub(super) fn compor_regiao(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    width: u32,
    height: u32,
    region: Region,
    vidros: Option<&super::super::vidro::Vidros>,
) -> (Vec<[f32; 4]>, Option<Vec<[f32; 3]>>) {
    super::compor_regiao_em(Vec::new(), stack, src, width, height, region, vidros)
}

/// `ids` (band-safe) over `acc` in `fios` disjoint row bands, each the same bottom→top walk.
#[allow(clippy::too_many_arguments)]
fn em_faixas(
    acc: &mut [[f32; 4]],
    t: Option<&mut [[f32; 3]]>,
    ids: &[LayerId],
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    canvas: (u32, u32),
    r: Region,
    vidros: Option<&super::super::vidro::Vidros>,
    (fios, zera): (usize, bool),
) {
    if fios <= 1 {
        em_tiras(acc, t, ids, stack, src, canvas, r, (vidros, zera));
        return;
    }
    let rpb = (r.h as usize).div_ceil(fios);
    let band_px = rpb * (r.w as usize);
    let mut t_bands: Vec<Option<&mut [[f32; 3]]>> = match t {
        Some(tt) => tt.chunks_mut(band_px).map(Some).collect(),
        None => (0..acc.len().div_ceil(band_px)).map(|_| None).collect(),
    };
    use rayon::prelude::*;
    acc.par_chunks_mut(band_px)
        .zip(t_bands.par_iter_mut())
        .enumerate()
        .for_each(|(bi, (band, tb))| {
            let banda = Region {
                y: r.y + (bi * rpb) as u32,
                h: (band.len() / (r.w as usize)) as u32,
                ..r
            };
            em_tiras(
                band,
                tb.take(),
                ids,
                stack,
                src,
                canvas,
                banda,
                (vidros, zera),
            );
        });
}

/// B19.10: the band in cache strips of [`linhas_da_tira`] rows — each strip takes EVERY layer of
/// `ids` before the next (the whole band per layer walked ~8 MB at UHD, out of L2). Per-pixel and
/// band-safe, so the same bits at any strip height.
#[allow(clippy::too_many_arguments)]
fn em_tiras(
    acc: &mut [[f32; 4]],
    mut t: Option<&mut [[f32; 3]]>,
    ids: &[LayerId],
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    canvas: (u32, u32),
    b: Region,
    (vidros, zera): (Option<&super::super::vidro::Vidros>, bool),
) {
    let linhas = linhas_da_tira(b.w).clamp(1, b.h.max(1));
    let tira_px = (linhas as usize) * (b.w as usize);
    if linhas >= b.h || tira_px == 0 {
        if zera {
            acc.fill([0.0; 4]);
        }
        let v = vidros.zip(t);
        composite_into(acc, ids, stack, src, canvas, b.x, b.y, b.w, b.h, 0, None, v);
        return;
    }
    for (k, tira) in acc.chunks_mut(tira_px).enumerate() {
        if zera {
            tira.fill([0.0; 4]);
        }
        let ty = b.y + k as u32 * linhas;
        let th = (tira.len() / (b.w as usize)) as u32;
        let tt = t
            .as_deref_mut()
            .map(|tt| &mut tt[k * tira_px..k * tira_px + tira.len()]);
        composite_into(
            tira,
            ids,
            stack,
            src,
            canvas,
            b.x,
            ty,
            b.w,
            th,
            0,
            None,
            vidros.zip(tt),
        );
    }
}

/// Rows of a cache strip `w` px wide: ~16 Ki px (4 rows at UHD: the strip's accumulator ~240 KB and
/// each layer's rows ~60 KB stay in L2 across the whole stack).
#[cfg(not(test))]
fn linhas_da_tira(w: u32) -> u32 {
    (TIRA_PX / w.max(1)).max(1)
}
const TIRA_PX: u32 = 1 << 14;
/// Test/probe override of the strip height (`0` = the production rule, `u32::MAX` = the whole band,
/// the pre-B19.10 route).
#[cfg(test)]
pub(super) static LINHAS_DA_TIRA: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(0);
#[cfg(test)]
fn linhas_da_tira(w: u32) -> u32 {
    match LINHAS_DA_TIRA.load(std::sync::atomic::Ordering::Relaxed) {
        0 => (TIRA_PX / w.max(1)).max(1),
        n => n,
    }
}

/// The band count of a region of `area` px and `rows` rows.
#[cfg(not(test))]
pub(super) fn fios_da_regiao(_stack: &LayerStack, area: usize, rows: usize) -> usize {
    parallel_threads(area, rows)
}
/// Test override of the band count (`0` = the production rule).
#[cfg(test)]
pub(super) static FIOS_DA_REGIAO: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
/// Probe A/B: the pre-B19.6 rule (a spatial item anywhere ⇒ the whole stack on one core).
#[cfg(test)]
pub(super) static BARREIRA_ANTIGA: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
#[cfg(test)]
pub(super) fn fios_da_regiao(stack: &LayerStack, area: usize, rows: usize) -> usize {
    use std::sync::atomic::Ordering::Relaxed;
    if BARREIRA_ANTIGA.load(Relaxed) && has_spatial_adjustment(stack, stack.root()) {
        return 1;
    }
    match FIOS_DA_REGIAO.load(Relaxed) {
        0 => parallel_threads(area, rows),
        _ if area == 0 => 1,
        n => n.clamp(1, rows.max(1)),
    }
}

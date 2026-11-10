//! B19.6: the spatial barrier. The root stack is cut at each VISIBLE spatial item (a spatial
//! adjustment, or a group holding one): the per-pixel runs between them compose in row bands, and
//! each spatial item runs alone over the whole region (its kernel reads the neighbours; the kernels
//! split their own rows). Every spatial item is an adjustment or a group, which resets the clip
//! chain, so each run starts in the state the single walk had there: the same bits at any band count.

use super::*;

/// Walk `stack.root()` over the clamped region `r` into `acc` (and `t`), runs in `fios` bands.
#[allow(clippy::too_many_arguments)]
pub(super) fn compor_segmentos(
    acc: &mut [[f32; 4]],
    mut t: Option<&mut [[f32; 3]]>,
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    canvas: (u32, u32),
    r: Region,
    vidros: Option<&super::super::vidro::Vidros>,
    fios: usize,
) {
    let root = stack.root();
    let mut fim = root.len();
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
                fios,
            );
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
    fios: usize,
) {
    if fios <= 1 {
        let v = vidros.zip(t);
        composite_into(acc, ids, stack, src, canvas, r.x, r.y, r.w, r.h, 0, None, v);
        return;
    }
    let rpb = (r.h as usize).div_ceil(fios);
    let band_px = rpb * (r.w as usize);
    let mut t_bands: Vec<Option<&mut [[f32; 3]]>> = match t {
        Some(tt) => tt.chunks_mut(band_px).map(Some).collect(),
        None => (0..acc.len().div_ceil(band_px)).map(|_| None).collect(),
    };
    std::thread::scope(|s| {
        let handles: Vec<_> = acc
            .chunks_mut(band_px)
            .zip(t_bands.iter_mut())
            .enumerate()
            .map(|(bi, (band, tb))| {
                let band_ry = r.y + (bi * rpb) as u32;
                let band_rh = (band.len() / (r.w as usize)) as u32;
                let tb = tb.take();
                s.spawn(move || {
                    let v = vidros.zip(tb);
                    composite_into(
                        band, ids, stack, src, canvas, r.x, band_ry, r.w, band_rh, 0, None, v,
                    );
                })
            })
            .collect();
        for h in handles {
            h.join().expect("composite band panicked");
        }
    });
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

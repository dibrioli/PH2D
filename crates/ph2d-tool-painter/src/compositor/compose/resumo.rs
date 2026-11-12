//! B19.10: the per-layer EMPTY-BLOCK summary. One byte per (canvas row, [`BLOCO`]-px column block),
//! `0` = every texel α = 0. Over such a block the α = 0 skip's decision depends on `dst` alone (an
//! α = 0 texel decodes to a colour in `[0, 1]` and α `+0`; mask, clip and a non-NaN opacity keep it
//! `+0`), so an eligible `dst` pixel is skipped WITHOUT reading the layer — its DRAM traffic is gone.
//!
//! Freshness without trusting writers: the summary is cached under a `Weak` of the plane's `Arc`
//! allocation. While that `Weak` lives, no safe write mutates the plane in place — `make_mut` copies
//! (another strong) or MOVES it to a new allocation (only `Weak`s), `get_mut` refuses, `try_unwrap`
//! empties it — and the `Weak` keeps the old address from being reused. Same address ⇒ same bytes.

use super::*;
use std::sync::{Mutex, Weak};

/// Column block width of the summary.
pub(super) const BLOCO: u32 = 64;
/// Entries kept (the oldest goes first); dead planes are pruned on every lookup.
const MAX_ENTRADAS: usize = 128;

struct Entrada {
    plano: Weak<Vec<u8>>,
    dims: (u32, u32),
    cheios: Arc<Vec<u8>>,
}

static CACHE: Mutex<Vec<Entrada>> = Mutex::new(Vec::new());

/// The summary of `plano` (`w × h` RGBA8), from the cache or computed once (one read of the plane).
fn resumo_de(plano: &Arc<Vec<u8>>, w: u32, h: u32) -> Arc<Vec<u8>> {
    let mesmo = |e: &Entrada| mesma_alocacao(e, plano) && e.dims == (w, h);
    {
        let mut c = CACHE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        c.retain(|e| e.plano.strong_count() > 0);
        if let Some(e) = c.iter().find(|e| mesmo(e)) {
            return Arc::clone(&e.cheios);
        }
    }
    let cheios = Arc::new(calcula(plano, w, h));
    let mut c = CACHE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if c.len() >= MAX_ENTRADAS {
        c.remove(0);
    }
    c.push(Entrada {
        plano: Arc::downgrade(plano),
        dims: (w, h),
        cheios: Arc::clone(&cheios),
    });
    cheios
}

#[inline]
fn mesma_alocacao(e: &Entrada, plano: &Arc<Vec<u8>>) -> bool {
    std::ptr::eq(e.plano.as_ptr(), Arc::as_ptr(plano))
}

/// One pass over the plane in row bands (the tool's one floor, `em_faixas_de_linhas`).
fn calcula(rgba: &[u8], w: u32, h: u32) -> Vec<u8> {
    let bpl = w.div_ceil(BLOCO) as usize;
    let mut cheios = vec![0u8; bpl * h as usize];
    let threads = parallel_threads((w as usize) * (h as usize), h as usize);
    em_faixas_de_linhas(&mut cheios, bpl, threads, |banda, y0| {
        for (k, linha) in banda.chunks_mut(bpl).enumerate() {
            let base = (y0 + k) * w as usize;
            for (bx, c) in linha.iter_mut().enumerate() {
                let x0 = bx * BLOCO as usize;
                let x1 = (x0 + BLOCO as usize).min(w as usize);
                let px = &rgba[(base + x0) * 4..(base + x1) * 4];
                *c = u8::from(px.as_chunks::<4>().0.iter().any(|p| p[3] != 0));
            }
        }
    });
    cheios
}

/// `src` with the summaries of every visible raster/texture layer of `stack` whose plane backs its
/// `layer_rgba` exactly.
pub(super) struct ComResumos<'a, S> {
    src: &'a S,
    resumos: BTreeMap<LayerId, Arc<Vec<u8>>>,
}

pub(super) fn com_resumos<'a, S: LayerPixelSource>(
    stack: &LayerStack,
    src: &'a S,
    canvas: (u32, u32),
) -> ComResumos<'a, S> {
    let mut resumos = BTreeMap::new();
    if resumo_ligado() {
        junta(stack, stack.root(), src, canvas, &mut resumos, 0);
    }
    ComResumos { src, resumos }
}

fn junta<S: LayerPixelSource>(
    stack: &LayerStack,
    ids: &[LayerId],
    src: &S,
    (w, h): (u32, u32),
    out: &mut BTreeMap<LayerId, Arc<Vec<u8>>>,
    depth: usize,
) {
    if depth > MAX_GROUP_DEPTH {
        return;
    }
    for &id in ids {
        let Some(l) = stack.get(id).filter(|l| l.visible) else {
            continue;
        };
        match &l.kind {
            LayerKind::Group(g) => junta(stack, &g.children, src, (w, h), out, depth + 1),
            LayerKind::Raster(_) | LayerKind::Texture(_) => {
                let n = (w as usize) * (h as usize) * 4;
                if let (Some(p), Some(r)) = (src.layer_plane(id), src.layer_rgba(id))
                    && p.len() == n
                    && std::ptr::eq(p.as_ptr(), r.as_ptr())
                    && r.len() == n
                {
                    out.insert(id, resumo_de(p, w, h));
                }
            }
            _ => {}
        }
    }
}

impl<S: LayerPixelSource> LayerPixelSource for ComResumos<'_, S> {
    fn layer_rgba(&self, id: LayerId) -> Option<&[u8]> {
        self.src.layer_rgba(id)
    }
    fn layer_plane(&self, id: LayerId) -> Option<&Arc<Vec<u8>>> {
        self.src.layer_plane(id)
    }
    fn layer_vazios(&self, id: LayerId) -> Option<&[u8]> {
        self.resumos.get(&id).map(|v| v.as_slice())
    }
}

/// [`blend_window`] with the summary `cheios` of a `canvas_w`-wide layer: over an empty block an
/// eligible `dst` is the skip's own verdict, decided before the sample.
#[allow(clippy::too_many_arguments)]
pub(super) fn blend_window_resumo(
    acc: &mut [[f32; 4]],
    (rx, ry, rw, rh): (u32, u32, u32, u32),
    canvas_w: u32,
    mode: BlendMode,
    opacity: f32,
    cheios: Option<&[u8]>,
    sample: impl Fn(u32, u32) -> [f32; 4],
) {
    let normal = mode == BlendMode::Normal;
    let salta = salto_ligado() && (normal || !salto_so_normal());
    let Some(cheios) = cheios.filter(|_| salta && !opacity.is_nan()) else {
        return blend_window(acc, rx, ry, rw, rh, mode, opacity, sample);
    };
    let bpl = canvas_w.div_ceil(BLOCO);
    for ly in 0..rh {
        let gy = ry + ly;
        let mut lx = 0;
        while lx < rw {
            let bx = (rx + lx) / BLOCO;
            let fim = ((bx + 1) * BLOCO).min(rx + rw) - rx;
            let vazio = cheios[(gy * bpl + bx) as usize] == 0;
            for lx in lx..fim {
                let i = (ly * rw + lx) as usize;
                if vazio && normal_transparente_e_identidade(acc[i], [0.0; 4]) {
                    continue;
                }
                let mut s = sample(rx + lx, gy);
                s[3] *= opacity;
                if s[3] <= 0.0
                    && if normal {
                        normal_transparente_e_identidade(acc[i], s)
                    } else {
                        transparente_e_identidade(acc[i], s)
                    }
                {
                    continue;
                }
                acc[i] = apply_blend(mode, acc[i], s);
            }
            lx = fim;
        }
    }
}

/// The summary, off only for the gates' and the probe's same-process A/B.
#[cfg(not(test))]
#[inline]
fn resumo_ligado() -> bool {
    true
}
#[cfg(test)]
pub(super) static SEM_RESUMO: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
#[cfg(test)]
fn resumo_ligado() -> bool {
    !SEM_RESUMO.load(std::sync::atomic::Ordering::Relaxed)
}

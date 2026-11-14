//! `CompositorCache` (ADR-0045 §2.7 cut-point cache), split out of the
//! former `compositor.rs` (pure mechanical move).

use super::*;

/// W4 compositor recomposition cache (ADR-0045 §2.7) — **skeleton**. Each
/// adjustment layer is a "cut point": the accumulator BELOW it is cached, so a
/// change at layer N only invalidates cut points ≥ N (above stay valid). This
/// is the perf lever for the soft `adjustment_layer_recomposition_perf_4k` gate
/// (≤1 ms slider-drag @ 4K). v1 composites inline (no cache wired into the hot
/// path yet); this type + the dirty-rect field land the contract (HR-5
/// `BTreeMap`, not `HashMap`) for the impl/T4.x to wire.
#[derive(Default)]
pub struct CompositorCache {
    /// Cached straight encoded accumulator just below each adjustment layer,
    /// keyed by the adjustment's [`LayerId`]. Stable key + deterministic
    /// iteration (HR-5 — no `std::HashMap` in the compositor, ADR-0022).
    pub(crate) cuts: std::collections::BTreeMap<LayerId, Vec<[f32; 4]>>,
    /// Recompose only this sub-rect (the dab/transform bbox); `None` = full.
    pub(crate) dirty_rect: Option<Region>,
    /// B19.9: the accumulator and the adjusted copy of a root adjustment, kept between drag frames
    /// (a fresh canvas-sized `Vec` per frame is ~65 k page faults at 4K). Scratch: never cloned.
    pub(crate) acc: Vec<[f32; 4]>,
    pub(crate) ajustado: Vec<[f32; 4]>,
    /// B19.6/B19.10: one `ajustado` per row band of the banded walk. Scratch: never cloned.
    pub(crate) ajustado_por_faixa: Vec<Vec<[f32; 4]>>,
    /// B19.12: the previous frame's output, handed back by [`Self::recicla`] for the next
    /// [`composite_with_cache_into`]. Scratch: never cloned.
    pub(crate) saida: Vec<u8>,
    /// B19.12: the accumulator of the FULL composite ([`composite_cheia_into`]). Scratch: never cloned.
    pub(crate) cheio: Vec<[f32; 4]>,
}

impl Clone for CompositorCache {
    fn clone(&self) -> Self {
        Self {
            cuts: self.cuts.clone(),
            dirty_rect: self.dirty_rect,
            acc: Vec::new(),
            ajustado: Vec::new(),
            ajustado_por_faixa: Vec::new(),
            saida: Vec::new(),
            cheio: Vec::new(),
        }
    }
}

impl CompositorCache {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// STRUCTURAL change (add / remove / reorder / visibility / opacity / pixels
    /// of a layer): the composite below some cut points changed. Conservative-
    /// correct: clear ALL cuts (cheap — they refill on the next full compose). A
    /// finer LayerId→depth mapping could keep cuts strictly below `changed`, but
    /// "clear all on structural" is the safe default (ADR-0045 §2.7 note).
    pub fn invalidate_from(&mut self, _changed: LayerId, _stack: &LayerStack) {
        self.cuts.clear();
    }

    /// PARAM-only change of root adjustment `adj` (the slider-drag hot path): the
    /// layers BELOW `adj` are unchanged, so `cuts[adj]` (the acc-below) stays
    /// valid; only the cuts ABOVE `adj` (which consumed `adj`'s output) are
    /// dropped. `composite_with_cache` then restarts from `adj`'s cut instead of
    /// recomposing the stack. `adj` not in the root ⇒ clear all (conservative).
    pub fn invalidate_above(&mut self, adj: LayerId, stack: &LayerStack) {
        let root = stack.root();
        let pos = |id: LayerId| root.iter().position(|&x| x == id);
        let Some(adj_pos) = pos(adj) else {
            self.cuts.clear();
            return;
        };
        // Panel order is top-first → "above" = smaller index. Keep cuts at the
        // adjustment and below (index ≥ adj_pos); drop those above it.
        self.cuts
            .retain(|&k, _| pos(k).is_some_and(|p| p >= adj_pos));
    }

    /// B19.12: keep a stale output for reuse when nobody else holds it (the shell copies the drained
    /// frame into its own buffer and drops it).
    pub fn recicla(&mut self, velha: Option<std::sync::Arc<Vec<u8>>>) {
        if let Some(v) = velha.and_then(|a| std::sync::Arc::try_unwrap(a).ok()) {
            self.saida = v;
        }
    }

    /// The recycled output buffer (empty when there is none).
    pub fn toma_a_saida(&mut self) -> Vec<u8> /* COLOR-RAW-OK: o buffer reciclado da saída do `compose` (sRGB8 cru para a placa) */
    {
        std::mem::take(&mut self.saida)
    }

    /// Mark the sub-rect to recompose next drain.
    pub fn mark_dirty(&mut self, region: Region) {
        self.dirty_rect = Some(region);
    }

    #[must_use]
    pub fn dirty_rect(&self) -> Option<Region> {
        self.dirty_rect
    }
}

/// Composite the FULL canvas with the cut-point cache (ADR-0045 §2.7) — the
/// slider-drag FPS lever. On a param-only change of a root adjustment (after
/// `cache.invalidate_above(adj, stack)`), this restarts from that adjustment's
/// cached accumulator-below instead of recomposing the layers underneath; on a
/// cold cache (or structural change → `invalidate_from`) it composes the whole
/// stack and (re)fills the cuts. **Bit-identical to [`composite`]** — the layers
/// below the restart point are unchanged, and an adjustment resets `clip_base`, so
/// the restart state matches the full walk (gate `cache_matches_full_recompose`).
#[must_use]
pub fn composite_with_cache(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    width: u32,
    height: u32,
    cache: &mut CompositorCache,
) -> Vec<u8> {
    let mut out = Vec::new();
    composite_with_cache_into(stack, src, width, height, cache, &mut out);
    out
}

/// [`composite_with_cache`] into `out` (B19.12): resized, never cleared — every byte is written, so a
/// buffer of the same size is reused with no allocation and no page faults.
pub fn composite_with_cache_into(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    width: u32,
    height: u32,
    cache: &mut CompositorCache,
    out: &mut Vec<u8>,
) {
    compor_com_cache(stack, src, width, height, cache);
    #[cfg(test)]
    if out.capacity() >= cache.acc.len() * 4 && !cache.acc.is_empty() {
        SAIDAS_REUSADAS.with(|n| n.set(n.get() + 1));
    }
    super::compose::encode_into(&cache.acc, out);
}

/// B19.12: [`super::composite`] into `out` (resized, never cleared) over the cache's reused accumulator — no
/// canvas-sized allocation nor page faults per full recompose. Bit-identical to [`super::composite`].
pub fn composite_cheia_into(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    (width, height): (u32, u32),
    cache: &mut CompositorCache,
    out: &mut Vec<u8>,
) {
    let tela = Region {
        x: 0,
        y: 0,
        w: width,
        h: height,
    };
    let acc = if cheia_aloca() {
        Vec::new()
    } else {
        std::mem::take(&mut cache.cheio)
    };
    #[cfg(test)]
    if acc.len() == (width as usize) * (height as usize) && out.capacity() >= acc.len() * 4 {
        CHEIAS_REUSADAS.with(|n| n.set(n.get() + 1));
    }
    let (acc, _) =
        super::compose::composite_region_linear_em(acc, stack, src, (width, height), tela, None);
    super::compose::encode_into(&acc, out);
    cache.cheio = acc;
}

/// A chave A×B em execução: `PH2D_CHEIA_ALOCA=1` = o acumulador novo por composição cheia (antes do B19.12).
fn cheia_aloca() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    #[cfg(test)]
    if CHEIA_ALOCA.with(std::cell::Cell::get) {
        return true;
    }
    *V.get_or_init(|| std::env::var_os("PH2D_CHEIA_ALOCA").is_some_and(|v| v == "1"))
}

/// Prepara `acc` com `n` píxeis. Vazio/outro tamanho: `alloc_zeroed` (páginas zero), `false`. Reusado: `true` =
/// a barreira zera-o tira a tira, quente na cache, logo antes de compor (zerar 133 MB à parte ida e volta à
/// DRAM custava mais que as páginas novas, F5d). `PH2D_CHEIA_ZERA_ANTES=1` = zerar tudo antes (o F5c).
pub(super) fn zera_o_acumulador(acc: &mut Vec<[f32; 4]>, n: usize) -> bool {
    if acc.len() != n {
        *acc = vec![[0.0f32; 4]; n];
        return false;
    }
    if !zera_antes() {
        return true;
    }
    use rayon::prelude::*;
    acc.par_chunks_mut(1 << 15).for_each(|c| c.fill([0.0; 4]));
    false
}

/// A chave A×B em execução do zerar fundido (`PH2D_CHEIA_ZERA_ANTES=1` = o acumulador zerado inteiro antes).
fn zera_antes() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    #[cfg(test)]
    if ZERA_ANTES.with(std::cell::Cell::get) {
        return true;
    }
    *V.get_or_init(|| std::env::var_os("PH2D_CHEIA_ZERA_ANTES").is_some_and(|v| v == "1"))
}

#[cfg(test)]
thread_local! {
    /// B19.12: o controlo do gate da composição cheia (um acumulador novo por composição).
    pub(crate) static CHEIA_ALOCA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// B19.12 (F5e): o controlo do zerar fundido (o acumulador reusado zerado inteiro antes de compor).
    pub(crate) static ZERA_ANTES: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// B19.12: full composites that reused BOTH the accumulator and the output (no allocation).
    pub(crate) static CHEIAS_REUSADAS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
thread_local! {
    /// B19.12: outputs composed into a buffer that already had the room (no allocation).
    pub(crate) static SAIDAS_REUSADAS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Leaves the composite in `cache.acc` (straight encoded).
fn compor_com_cache(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    width: u32,
    height: u32,
    cache: &mut CompositorCache,
) {
    // B19.10: o resumo por ladrilho também no arrasto de ajustes.
    let resumos = super::compose::resumo::com_resumos(stack, src, (width, height));
    let src = &resumos;
    let root = stack.root();
    // Highest root adjustment (smallest index = MOST below-layers cached) whose
    // cut is still valid. Its cut is the composite of everything below it.
    let start = root.iter().enumerate().find(|&(_, &id)| {
        matches!(
            stack.get(id).map(|l| &l.kind),
            Some(LayerKind::Adjustment(_))
        ) && cache.cuts.contains_key(&id)
    });
    // B19.9: the accumulator lives in the cache between drag frames (no canvas-sized alloc per frame).
    let mut acc = if copias_antigas() {
        Vec::new()
    } else {
        std::mem::take(&mut cache.acc)
    };
    acc.clear();
    let ids: &[LayerId] = match start {
        // `composite_into` walks `ids` REVERSED (panel order is top-first;
        // index 0 = topmost). So the cut at panel index `i` = composite of the
        // layers BELOW it (`root[i+1..]`). Seed `acc` with that cut, then hand
        // `root[..=i]` (the adjustment + everything above): the reversed walk
        // processes the adjustment FIRST (on the correct below-acc), then the
        // layers above it. Restarting from the smallest valid index reuses the
        // largest cut (most below-layers cached) → fewest layers recomposed.
        Some((i, &id)) => {
            acc.extend_from_slice(&cache.cuts[&id]);
            &root[..=i]
        }
        None => {
            acc.resize((width as usize) * (height as usize), [0.0f32; 4]);
            root
        }
    };
    // B19.6/B19.10: per-pixel work over `ids` (the cut already holds whatever is spatial below the
    // restart) splits into the row bands of `compor_regiao`, cut at each spatial item in `ids`
    // ([`segmentos`]); the old serial walk is `fios = 1`.
    let fios = if copias_antigas() {
        1
    } else {
        fios_do_cache(acc.len(), height as usize)
    };
    if fios > 1 && super::compose::has_spatial_adjustment(stack, ids) {
        segmentos(&mut acc, ids, stack, src, (width, height), fios, cache);
    } else if fios <= 1 {
        super::compose::composite_into(
            &mut acc,
            ids,
            stack,
            src,
            (width, height),
            0,
            0,
            width,
            height,
            0,
            Some(&mut *cache),
            None,
        );
    } else {
        em_faixas(&mut acc, ids, stack, src, (width, height), fios, cache);
    }
    cache.acc = acc;
}

/// B19.6 barrier on the drag (as `compose/barreira.rs`): `ids` cut at each visible spatial item; the
/// per-pixel runs go in bands ([`em_faixas`]), each spatial item alone over the canvas with the whole
/// cache (its cut, if it is a root adjustment, is recorded whole). Adjustment and group reset the clip
/// chain, so every run starts in the state of the serial walk.
fn segmentos(
    acc: &mut [[f32; 4]],
    ids: &[LayerId],
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    canvas: (u32, u32),
    fios: usize,
    cache: &mut CompositorCache,
) {
    let mut fim = ids.len();
    while fim > 0 {
        let mut ini = fim;
        while ini > 0 && !super::compose::has_spatial_adjustment(stack, &ids[ini - 1..ini]) {
            ini -= 1;
        }
        if ini < fim {
            em_faixas(acc, &ids[ini..fim], stack, src, canvas, fios, cache);
        }
        if ini == 0 {
            break;
        }
        super::compose::composite_into(
            acc,
            &ids[ini - 1..ini],
            stack,
            src,
            canvas,
            0,
            0,
            canvas.0,
            canvas.1,
            0,
            Some(&mut *cache),
            None,
        );
        fim = ini - 1;
    }
}

/// The walk of [`compor_com_cache`] in `fios` row bands, each with its own cut map (the cuts already
/// in `cache` enter as empty placeholders, so the restart cut is never recopied); the new cuts are
/// stitched back by concatenation in band order — the same bytes as the serial walk's.
fn em_faixas(
    acc: &mut [[f32; 4]],
    ids: &[LayerId],
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    canvas: (u32, u32),
    fios: usize,
    cache: &mut CompositorCache,
) {
    let width = canvas.0;
    let rpb = (canvas.1 as usize).div_ceil(fios);
    let band_px = rpb * (width as usize);
    let n = acc.len().div_ceil(band_px);
    cache.ajustado_por_faixa.resize_with(n, Vec::new);
    let mut faixas: Vec<CompositorCache> = cache
        .ajustado_por_faixa
        .iter_mut()
        .map(|a| CompositorCache {
            cuts: cache.cuts.keys().map(|&k| (k, Vec::new())).collect(),
            dirty_rect: None,
            acc: Vec::new(),
            ajustado: std::mem::take(a),
            ajustado_por_faixa: Vec::new(),
            saida: Vec::new(),
            cheio: Vec::new(),
        })
        .collect();
    {
        use rayon::prelude::*;
        acc.par_chunks_mut(band_px)
            .zip(faixas.par_iter_mut())
            .enumerate()
            .for_each(|(bi, (band, fc))| {
                let band_ry = (bi * rpb) as u32;
                let band_rh = (band.len() / (width as usize)) as u32;
                super::compose::composite_into(
                    band,
                    ids,
                    stack,
                    src,
                    canvas,
                    0,
                    band_ry,
                    width,
                    band_rh,
                    0,
                    Some(fc),
                    None,
                );
            });
    }
    let novos: Vec<LayerId> = faixas[0]
        .cuts
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(&k, _)| k)
        .collect();
    for id in novos {
        let mut corte = Vec::with_capacity(acc.len());
        for f in &faixas {
            corte.extend_from_slice(&f.cuts[&id]);
        }
        cache.cuts.insert(id, corte);
    }
    for (a, f) in cache.ajustado_por_faixa.iter_mut().zip(faixas) {
        *a = f.ajustado;
    }
}

#[cfg(not(test))]
#[inline]
fn fios_do_cache(area: usize, rows: usize) -> usize {
    super::compose::parallel_threads(area, rows)
}
/// Test override of the band count (`0` = the production rule).
#[cfg(test)]
pub(super) static FIOS_DO_CACHE: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
#[cfg(test)]
fn fios_do_cache(area: usize, rows: usize) -> usize {
    match FIOS_DO_CACHE.load(std::sync::atomic::Ordering::Relaxed) {
        0 => super::compose::parallel_threads(area, rows),
        _ if area == 0 => 1,
        n => n.clamp(1, rows),
    }
}

/// B19.9: the three canvas copies per drag frame, back on only for the probe's same-process A/B.
#[cfg(not(test))]
#[inline]
pub(super) fn copias_antigas() -> bool {
    false
}
#[cfg(test)]
pub(super) static COPIAS_ANTIGAS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
#[cfg(test)]
pub(super) fn copias_antigas() -> bool {
    COPIAS_ANTIGAS.load(std::sync::atomic::Ordering::Relaxed)
}

#[cfg(test)]
mod cache_tests;

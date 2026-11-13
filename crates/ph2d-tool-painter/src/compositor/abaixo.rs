//! The watercolor backdrop: the layers BELOW the active one over the paper ground (B52.R3, split out
//! of `compose.rs` verbatim).

use super::compose::{composite_into, encode, parallel_threads};
use super::*;

/// Composite ONLY the layers strictly **below** `anchor` (in paint order) over an opaque `ground`
/// colour (straight sRGB8) → canvas-sized straight sRGB8 RGBA, **opaque by construction**. This is
/// the watercolor optics' real backdrop: what the artist sees under the active layer's paint —
/// the layers beneath it, and the document paper colour where nothing is painted.
///
/// "Below" walks the anchor's root→group path: at every level, the siblings AFTER the path node
/// (panel order is top-first, so later indices sit beneath), composited outermost-level first
/// (bottommost content) with the same walk as [`composite`] — blend modes, masks, clipping and
/// adjustments below the anchor all apply. Ancestor groups' OWN blend/opacity are intentionally
/// not applied to their below-slices (they would also apply to the anchor's paint itself, which a
/// backdrop cannot pre-compensate — the composite-time blend covers it). Unknown anchor → plain
/// ground fill (a mask being painted resolves to its PARENT layer at the call site).
#[must_use]
pub fn composite_below(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    width: u32,
    height: u32,
    anchor: LayerId,
    ground: [u8; 3],
) -> Vec<u8> {
    composite_below_com(stack, src, width, height, anchor, ground, None, FAIXA_PX)
}

/// **O corte de uma composição em fundo** (n.º 7 item 3f, «Continuação 6»): `para` lê-se antes de cada faixa (e de
/// cada fatia da rota de um fio), por isso um fio descartado sai depois da faixa em curso — no máximo uma por
/// operário. `feitas` conta as faixas acabadas e `depois` as que acabaram com `para` já de pé (o gate).
#[derive(Default)]
pub struct Corte {
    pub para: std::sync::atomic::AtomicBool,
    pub feitas: std::sync::atomic::AtomicU32,
    pub depois: std::sync::atomic::AtomicU32,
}

impl Corte {
    fn parou(&self) -> bool {
        self.para.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn acabou_uma(&self) {
        use std::sync::atomic::Ordering::SeqCst;
        self.feitas.fetch_add(1, SeqCst);
        if self.parou() {
            self.depois.fetch_add(1, SeqCst);
        }
    }
}

/// [`composite_below`] que pára entre faixas quando `corte.para` sobe: `None` = cortada (o resto do plano não
/// existe). Sem corte, o mesmo bit.
#[must_use]
pub fn composite_below_cortavel(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    width: u32,
    height: u32,
    anchor: LayerId,
    ground: [u8; 3],
    corte: &Corte,
) -> Option<Vec<u8>> {
    composite_below_corte(
        stack,
        src,
        width,
        height,
        anchor,
        ground,
        None,
        FAIXA_PX,
        Some(corte),
    )
}

/// [`composite_below`] with the split forced (`fios`: `None` = the core count) — the gate's door.
#[allow(clippy::too_many_arguments)]
pub(super) fn composite_below_com(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    width: u32,
    height: u32,
    anchor: LayerId,
    ground: [u8; 3],
    fios: Option<usize>,
    faixa_px: usize,
) -> Vec<u8> {
    composite_below_corte(
        stack, src, width, height, anchor, ground, fios, faixa_px, None,
    )
    .expect("sem corte, a composição acaba")
}

#[allow(clippy::too_many_arguments)]
pub(super) fn composite_below_corte(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    width: u32,
    height: u32,
    anchor: LayerId,
    ground: [u8; 3],
    fios: Option<usize>,
    faixa_px: usize,
    corte: Option<&Corte>,
) -> Option<Vec<u8>> {
    // The below-slices along the root→anchor path, outermost first.
    let mut slices: Vec<&[LayerId]> = Vec::new();
    fn descend<'a>(
        stack: &'a LayerStack,
        ids: &'a [LayerId],
        anchor: LayerId,
        slices: &mut Vec<&'a [LayerId]>,
        depth: usize,
    ) -> bool {
        if depth > MAX_GROUP_DEPTH {
            return false;
        }
        for (i, &id) in ids.iter().enumerate() {
            if id == anchor {
                slices.push(&ids[i + 1..]);
                return true;
            }
            if let Some(LayerKind::Group(gr)) = stack.get(id).map(|l| &l.kind) {
                slices.push(&ids[i + 1..]); // provisional: below the group
                if descend(stack, &gr.children, anchor, slices, depth + 1) {
                    return true;
                }
                slices.pop();
            }
        }
        false
    }
    let found = descend(stack, stack.root(), anchor, &mut slices, 0);
    // ⚠️ **Nothing below ⇒ the ground IS the answer, and the accumulator never exists.** Compositing
    // an empty set over the ground leaves the ground, so the long path below would spend a
    // `[f32;4]`-per-pixel buffer plus an encode to reproduce a flat colour: at 4096² that is **268 MB
    // written and read back, plus 67 MB encoded**, and it is what a ONE-LAYER document paid on every
    // watercolor pen-down (measured: 75-112 ms, near-flat in brush radius because none of it looks at
    // the dab). The allocation used to sit above `descend`, so the early-out was unreachable by
    // construction — the guard existed and the work happened anyway.
    //
    // Byte-identical, and the premise is a gate rather than an argument: the flat fill is the same
    // bytes as `encode(decode_byte(ground))` only because the byte round-trip is the identity on all
    // 256 values (`the_byte_round_trip_is_the_identity`). Conservative on purpose — a
    // non-empty slice of layers that all happen to be invisible still takes the long path, because
    // *"empty"* is a fact about the list and *"invisible"* is a fact about each layer.
    if !found || slices.iter().all(|s| s.is_empty()) {
        return Some(flat_fill(
            width,
            height,
            [ground[0], ground[1], ground[2], 255],
        ));
    }
    // Seed the accumulator with the opaque ground.
    let g = [
        super::decode_byte(ground[0]),
        super::decode_byte(ground[1]),
        super::decode_byte(ground[2]),
        1.0f32,
    ];
    // A spatial adjustment below reads neighbours across the canvas: those stacks stay one walk.
    let espacial = slices
        .iter()
        .any(|s| super::compose::has_spatial_adjustment(stack, s));
    let n = (width as usize) * (height as usize);
    let fios = if espacial {
        1
    } else {
        fios.unwrap_or_else(|| parallel_threads(n, height as usize))
    };
    abaixo_em_faixas(stack, src, width, height, &slices, g, fios, faixa_px, corte)
}

/// Pixels of one cache strip of the banded walk (`[f32;4]` = 256 KiB, L2-resident): 4 rows at UHD.
/// Measured (B19.11, `mede_o_composite_below_4k`): see the BACKLOG row.
const FAIXA_PX: usize = 1 << 14;

/// The below-walk over `fios` row bands, each band in strips of `faixa_px` pixels seeded with the
/// ground, composited and encoded in place — no canvas-sized `[f32;4]` accumulator (133 MB at UHD).
/// Per-pixel compositing is spatially independent, so every split is the one walk to the bit
/// (gate `o_chao_em_faixas_e_o_da_rota_antiga_ao_bit`); `fios <= 1` is that one walk.
#[allow(clippy::too_many_arguments)]
fn abaixo_em_faixas(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    width: u32,
    height: u32,
    slices: &[&[LayerId]],
    g: [f32; 4],
    fios: usize,
    faixa_px: usize,
    corte: Option<&Corte>,
) -> Option<Vec<u8>> {
    let parou = || corte.is_some_and(Corte::parou);
    let w = width as usize;
    if fios <= 1 || w == 0 {
        let mut acc = vec![g; w * (height as usize)];
        for ids in slices {
            if parou() {
                return None;
            }
            composite_into(
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
                None,
                None,
            );
            if let Some(c) = corte {
                c.acabou_uma();
            }
        }
        return (!parou()).then(|| encode(&acc));
    }
    let mut out = vec![0u8; w * (height as usize) * 4];
    let rpb = (height as usize).div_ceil(fios);
    // A strip never exceeds its band: `faixa_px = usize::MAX` (the whole band) cannot overflow.
    let linhas = (faixa_px / w).clamp(1, rpb);
    {
        use rayon::prelude::*;
        out.par_chunks_mut(rpb * w * 4)
            .enumerate()
            .for_each(|(bi, banda)| {
                let mut acc = Vec::new();
                for (k, faixa) in banda.chunks_mut(linhas * w * 4).enumerate() {
                    if parou() {
                        return;
                    }
                    let fh = faixa.len() / (w * 4);
                    let y0 = (bi * rpb + k * linhas) as u32;
                    acc.clear();
                    acc.resize(fh * w, g);
                    for ids in slices {
                        composite_into(
                            &mut acc,
                            ids,
                            stack,
                            src,
                            (width, height),
                            0,
                            y0,
                            width,
                            fh as u32,
                            0,
                            None,
                            None,
                        );
                    }
                    for (o, a) in faixa.as_chunks_mut::<4>().0.iter_mut().zip(&acc) {
                        *o = a.map(super::encode_byte);
                    }
                    if let Some(c) = corte {
                        c.acabou_uma();
                    }
                }
            });
    }
    (!parou()).then_some(out)
}

#[cfg(test)]
mod tests;

/// A canvas-sized plane of ONE pixel value, filled across the cores (the split of [`encode`]).
///
/// ⚠️ **Serial, this was the watercolor pen-down's biggest single cost** (measured 2026-09-24,
/// 4096², the owner's photo config): `~15 ms` to write the flat paper ground of a one-layer
/// document — 16,7 M pixel copies on one thread, on EVERY stroke. Byte-identical by construction:
/// every pixel gets the same four bytes, and disjoint chunks cannot see each other.
fn flat_fill(width: u32, height: u32, px: [u8; 4]) -> Vec<u8> {
    let n = (width as usize) * (height as usize);
    let mut out = vec![0u8; n * 4];
    let fill = |chunk: &mut [u8]| {
        for p in chunk.as_chunks_mut::<4>().0 {
            *p = px;
        }
    };
    let threads = parallel_threads(n, height as usize);
    if threads <= 1 {
        fill(&mut out);
        return out;
    }
    let cpb = n.div_ceil(threads) * 4;
    {
        use rayon::prelude::*;
        out.par_chunks_mut(cpb).for_each(fill);
    }
    out
}

#[cfg(test)]
mod flat_fill_tests {
    /// O preenchimento repartido pelas cores é o laço em série, AO BYTE — incluindo tamanhos
    /// abaixo e acima do piso do paralelo e larguras ímpares (o último pedaço é mais curto).
    #[test]
    fn o_preenchimento_paralelo_e_o_serial() {
        let px = [201u8, 33, 7, 255];
        for (w, h) in [(1u32, 1u32), (7, 3), (513, 300), (1025, 257), (2048, 64)] {
            let mut serial = vec![0u8; (w * h * 4) as usize];
            for p in serial.as_chunks_mut::<4>().0 {
                *p = px;
            }
            assert_eq!(super::flat_fill(w, h, px), serial, "{w}×{h}");
        }
        // CONTROLO: os tamanhos grandes acima atravessam de facto o ramo paralelo.
        assert!(
            super::parallel_threads(513 * 300, 300) > 1
                || std::thread::available_parallelism().map_or(1, usize::from) == 1
        );
    }
}

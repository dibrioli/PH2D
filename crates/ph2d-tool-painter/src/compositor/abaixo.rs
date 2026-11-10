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
        return flat_fill(width, height, [ground[0], ground[1], ground[2], 255]);
    }
    // Seed the accumulator with the opaque ground.
    let g = [
        super::decode_byte(ground[0]),
        super::decode_byte(ground[1]),
        super::decode_byte(ground[2]),
        1.0f32,
    ];
    let mut acc = vec![g; (width as usize) * (height as usize)];
    for ids in slices {
        composite_into(
            &mut acc, ids, stack, src, width, 0, 0, width, height, 0, None, None,
        );
    }
    encode(&acc)
}

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
    std::thread::scope(|s| {
        let handles: Vec<_> = out
            .chunks_mut(cpb)
            .map(|c| s.spawn(move || fill(c)))
            .collect();
        for h in handles {
            h.join().expect("flat fill chunk panicked");
        }
    });
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

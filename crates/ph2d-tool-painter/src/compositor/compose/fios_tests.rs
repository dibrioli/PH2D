//! B19.7: a composite opens no OS thread of its own — the bands, the encode, the spatial kernels and
//! the Color Lookup (fios dentro de fios) run on the rayon pool. Counted on `/proc/self/task` (each
//! nextest test is its own process; `cargo-test-narrow` runs the named test alone).

use super::*;
use ph2d_painter_effects::adjustments::{
    AdjustmentKind, AdjustmentParams, BrightnessContrastParams, ColorLookupLutParams,
    GaussianBlurParams, LutHandle,
};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

fn tinta(w: u32, h: u32, semente: u32) -> LayerImage {
    let mut img = LayerImage::transparent(w, h);
    let mut x = semente.wrapping_mul(2_654_435_761).wrapping_add(1);
    for p in img.rgba8_mut().as_chunks_mut::<4>().0 {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *p = [
            (x >> 3) as u8,
            (x >> 11) as u8,
            (x >> 19) as u8,
            (x >> 24) as u8,
        ];
    }
    img
}

fn tids() -> BTreeSet<u64> {
    std::fs::read_dir("/proc/self/task")
        .map(|d| {
            d.filter_map(|e| e.ok()?.file_name().to_str()?.parse().ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Root (top-first): lut · camada · blur · camada · B/C · camada · camada — above the parallel area.
fn pilha(w: u32, h: u32) -> (LayerStack, MapPixelSource) {
    let mut s = LayerStack::new();
    let mut src = MapPixelSource::default();
    for k in 0..4u32 {
        let id = s.add_raster("camada", w, h).unwrap();
        src.insert(id, tinta(w, h, k + 1));
        match k {
            1 => {
                let a = s
                    .add_adjustment(AdjustmentKind::BrightnessContrast)
                    .unwrap();
                s.adjustment_mut(a).unwrap().params =
                    AdjustmentParams::BrightnessContrast(BrightnessContrastParams {
                        brightness: 0.1,
                        contrast: 0.2,
                        legacy: false,
                    });
            }
            2 => {
                let a = s.add_adjustment(AdjustmentKind::GaussianBlur).unwrap();
                s.adjustment_mut(a).unwrap().params =
                    AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: 3.0 });
            }
            _ => {}
        }
    }
    let lut = s.add_adjustment(AdjustmentKind::ColorLookupLut).unwrap();
    s.adjustment_mut(lut).unwrap().params =
        AdjustmentParams::ColorLookupLut(ColorLookupLutParams {
            lut_3d: LutHandle(1),
            intensity: 1.0,
            ..ColorLookupLutParams::default()
        });
    (s, src)
}

#[test]
fn a_composicao_nao_abre_fios_do_so() {
    if std::thread::available_parallelism().map_or(1, |n| n.get()) < 2 {
        return;
    }
    let (w, h) = (1024u32, 256u32);
    let (s, src) = pilha(w, h);
    let primeira = composite(&s, &src, w, h);
    rayon::broadcast(|_| ());
    let para = AtomicBool::new(false);
    let vistos = std::thread::scope(|sc| {
        let vigia = sc.spawn(|| {
            let mut v = BTreeSet::new();
            while !para.load(Relaxed) {
                v.extend(tids());
                std::thread::sleep(std::time::Duration::from_micros(100));
            }
            v
        });
        std::thread::sleep(std::time::Duration::from_millis(5));
        let antes = tids();
        for _ in 0..3 {
            assert_eq!(composite(&s, &src, w, h), primeira);
        }
        para.store(true, Relaxed);
        let v = vigia.join().unwrap();
        v.difference(&antes).count()
    });
    assert_eq!(
        vistos, 0,
        "a composição abriu {vistos} fios do SO fora do pool"
    );
}

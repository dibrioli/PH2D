//! O que os binários e o `node` chamam: os hashes das receitas de fábrica.

use ph2d_sfx_dsp::{pcm_hash, render};
use ph2d_sfx_recipe::{ENGINE_V1, Take, Takes, factory};

/// O mesmo material do gate `hash_dourado`.
#[must_use]
pub fn material() -> Take {
    let s: Vec<f32> = (0..44_100)
        .map(|i| {
            let t = i as f32 / 44_100.0;
            let queda = 1.0 - t;
            let n = (i * 7919 % 1009) as f32 / 1009.0 - 0.5;
            if i % 2 == 0 {
                (t * 2_764.6).rem_euclid(1.0) * queda - 0.5 * queda
            } else {
                n * queda
            }
        })
        .collect();
    Take::new(44_100, 2, s)
}

/// Um hash só para o conjunto: fábrica e gravação, em ordem.
#[must_use]
pub fn hashes() -> Vec<(String, u64)> {
    let mut out = Vec::new();
    for (nome, r) in factory::all() {
        let pcm = render(&r, r.seed, ENGINE_V1, &Takes::default()).expect("motor");
        out.push((nome.to_owned(), pcm_hash(&pcm.samples)));
    }
    let mut takes = Takes::default();
    let id = takes.add(material());
    let r = factory::recorded(id, 1.0);
    let pcm = render(&r, r.seed, ENGINE_V1, &takes).expect("motor");
    out.push((factory::RECORDED.to_owned(), pcm_hash(&pcm.samples)));
    out
}

#[unsafe(no_mangle)]
pub extern "C" fn hash_conjunto() -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325_u64;
    for (_, v) in hashes() {
        for b in v.to_le_bytes() {
            h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

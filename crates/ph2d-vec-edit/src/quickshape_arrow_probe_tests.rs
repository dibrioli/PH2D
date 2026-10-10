//! **As sondas da SETA do QuickShape** (F7d) — irmão de `quickshape_arrow_tests.rs` (os gates).
//! Correm à mão: `cargo test --release -p ph2d-vec-edit --lib measure_arrow -- --ignored
//! --nocapture --test-threads=1`. As tabelas: `00_resultados.md` §3h.

use super::arrow_tests::{KINDS, Kind, arrow_poly, held, is_the_arrow, not_arrows, shafts};
use super::tests::load;
use super::*;

/// A primeira olhadela: as fixtures dos oráculos e uma célula de cada família.
#[test]
#[ignore = "sonda"]
fn measure_arrow_first_look() {
    for f in load() {
        if f.name.starts_with("seta") || f.name == "linha_com_gancho" {
            println!("{:28} {:?}", f.name, f.ours(&Law::MEASURED));
        }
    }
    for s in [24.0, 48.0, 160.0, 400.0] {
        for (name, shaft) in shafts(s) {
            for kind in KINDS {
                for both in [false, true] {
                    let mut ok = 0;
                    for seed in 0..20 {
                        let p = arrow_poly(&shaft, 0.2 * s, 30.0, kind, both);
                        let q = recognise(&held(&p, 1.0, seed), 1.0, false);
                        let hit = is_the_arrow(&q, name, both);
                        ok += usize::from(hit);
                        if !hit && s == 160.0 && seed < 3 {
                            println!("   ✗ {name} {kind:?} {both} (mão {seed}): {q:?}");
                        }
                    }
                    println!("{s:5} {name:6} {kind:?} both={both}: {ok}/20");
                }
            }
        }
        for seed in 0..3 {
            for (name, p) in not_arrows(s, seed) {
                let q = recognise(&held(&p, 1.0, seed), 1.0, false);
                if matches!(q, Some(QuickShape::Arrow(_))) {
                    println!("{s:5} FALSA SETA {name} (mão {seed}): {q:?}");
                }
            }
        }
    }
    let _ = Kind::One;
}

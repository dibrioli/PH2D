//! ⭐⭐⭐ **ONDE O QUADRO VAI AGORA** (W91) — a decomposição, refeita depois da cache.
//!
//! ⚠️ **A decomposição de um quadro que este módulo tem escrita é anterior à cache de fitas** (W82) e
//! a tudo o que a W89 mudou. *Quem move o número que sustenta uma nota tem de reconferir a nota* —
//! e a nota aqui é *«a marcha é 80 % do quadro»*.
//!
//! As três parcelas medidas por contador, no regime (o arrasto já quente):
//!
//! - **`get`** — a varredura linear da cache, uma por região;
//! - **`especializar`** — a montagem da fita quando a cache falha (o JIT);
//! - o resto é a **marcha** e o que a rodeia.
//!
//! ```text
//! cargo test -p ph2d-field-render --profile ci-test --test where_the_frame_goes_now -- --ignored --nocapture
//! ```

use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
use ph2d_field_render::{Orbit, TapeCache};
use std::sync::atomic::Ordering::Relaxed;

fn circulo(n: usize) -> FieldDoc {
    let c: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
        })
        .collect();
    FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Leaf(Primitive::Extrude {
                profile: Profile::new(vec![c], FillRule::NonZero, 1e-4).expect("perfil"),
                half_height: 0.4,
                round: 0.06,
                chamfer: 0.0,
            }),
            mods: Vec::new(),
            verb: None,
        }],
        NodeId(0),
    )
    .expect("extrusão")
}

#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_where_a_moving_frame_spends_its_time() {
    const QUADROS: usize = 60;
    const REGIME: usize = 30;
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let doc = circulo(168);
    for (nome, w, h) in [("movimento 426x240", 426u32, 240u32), ("640x360", 640, 360)] {
        let cache = TapeCache::new();
        let (mut relogio, mut get, mut spec, mut regioes) = (0.0f64, 0u64, 0u64, 0usize);
        for i in 0..QUADROS {
            let cam = Orbit {
                rotation: Orbit::from_yaw_pitch(0.72 + (i as f64 * 2.0).to_radians() as f32, 0.52)
                    .rotation,
                ..Orbit::default()
            };
            ph2d_field_render::GET_NS.store(0, Relaxed);
            ph2d_field_render::SPECIALISE_NS.store(0, Relaxed);
            ph2d_field_render::SPECIALISED.store(0, Relaxed);
            ph2d_field_render::TAPE_HITS.store(0, Relaxed);
            let t0 = std::time::Instant::now();
            let _ = ph2d_field_render::trace_cached_for_test(
                &doc,
                &reg,
                &cam,
                w,
                h,
                false,
                Some(&cache),
            );
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            if i >= REGIME {
                relogio += ms;
                get += ph2d_field_render::GET_NS.load(Relaxed);
                spec += ph2d_field_render::SPECIALISE_NS.load(Relaxed);
                regioes += ph2d_field_render::SPECIALISED.load(Relaxed)
                    + ph2d_field_render::TAPE_HITS.load(Relaxed);
            }
        }
        let n = (QUADROS - REGIME) as f64;
        // ⚠️ Os contadores somam o tempo de TODAS as threads; o relógio é de parede. A razão que
        // interessa é «quanto do trabalho total», e por isso as parcelas são comparadas com a soma
        // delas, não com o relógio.
        let (g, s) = (get as f64 / 1e6 / n, spec as f64 / 1e6 / n);
        println!(
            "{nome:18} | parede {:6.2} ms | get {g:7.2} ms-thread | especializar {s:7.2} ms-thread | {:5.0} regiões/quadro | get/(get+spec) {:5.1} %",
            relogio / n,
            regioes as f64 / n,
            100.0 * g / (g + s)
        );
    }
}

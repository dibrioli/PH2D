//! ⭐⭐⭐ **A TRAVADINHA TEM NOME: O DESPEJO DA CACHE** (W89).
//!
//! ⚠️ **Esta sonda é a que prova a cura, e o número dela é o MÁXIMO** — ver a tabela no fim.
//!
//! # O report, e o número
//!
//! Enio, 26/08: *«de tempos em tempos dá pequenas travadinhas»*. Medido num arrasto de `2°/quadro` a
//! `426×240` (a peça de omissão, cache nova):
//!
//! | quadro | 20 | 21 | 22 | **23** |
//! |---|---:|---:|---:|---:|
//! | ms | `12,3` | `14,2` | `11,6` | **`274,8`** |
//! | despejos | `0` | `0` | `0` | **`1` (1 738 fitas)** |
//!
//! ⭐⭐⭐ **Uma fita é um `mmap` de código executável; despejá-la é um `munmap`.** Mil setecentas de
//! uma vez, **debaixo do cadeado de ESCRITA**, com as outras 31 threads à porta — e cada `munmap`
//! num processo com 32 threads vivas obriga a invalidar a TLB de todos os núcleos.
//!
//! # ⚠️ Porque nenhuma outra sonda a viu
//!
//! Ela precisa de **~24 quadros de arrasto contínuo** para a cache chegar ao tecto. As bancadas
//! desta linha mediam 5 a 15 traçados — *o fenómeno estava sempre um quadro depois do fim da
//! medição*. E a mediana de 5 nunca o conteria, mesmo que lá chegasse.
//!
//! ```text
//! cargo test -p ph2d-field-render --profile ci-test --test cohort_dispersion -- --ignored --nocapture
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
            }),
            mods: Vec::new(),
            verb: None,
        }],
        NodeId(0),
    )
    .expect("extrusão")
}

/// ⭐⭐⭐ **O REGIME VERDADEIRO** — o que acontece depois de a população começar a rodar.
///
/// ⚠️ **A pergunta que a série curta não podia responder:** enquanto a cache CRESCE não se liberta
/// uma única fita, então os `12 ms` dos quadros 9–22 medem um regime que **não existe** numa sessão
/// real. No regime a sério cada compilação tem uma libertação do outro lado, e é aí que se lê o
/// preço.
#[test]
#[ignore = "sonda; roda com --nocapture"]
fn measure_the_eviction_storm_and_the_true_steady_state() {
    const QUADROS: usize = 90;
    const GRAUS: f64 = 2.0;
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let doc = circulo(168);
    let cache = TapeCache::new();
    let mut serie: Vec<(f64, usize, usize, usize, f64)> = Vec::new();
    for i in 0..QUADROS {
        let cam = Orbit {
            rotation: Orbit::from_yaw_pitch(0.72 + (i as f64 * GRAUS).to_radians() as f32, 0.52)
                .rotation,
            ..Orbit::default()
        };
        let t0 = std::time::Instant::now();
        let _ = ph2d_field_render::trace_cached_for_test(
            &doc,
            &reg,
            &cam,
            426,
            240,
            false,
            Some(&cache),
        );
        let dt = t0.elapsed().as_secs_f64() * 1000.0;
        serie.push((
            dt,
            ph2d_field_eval::hybrid::FLOAT_TAPES.swap(0, Relaxed),
            ph2d_field_render::TAPE_EVICTIONS.swap(0, Relaxed),
            ph2d_field_render::TAPE_DROPPED.swap(0, Relaxed),
            ph2d_field_render::EVICT_NS.swap(0, Relaxed) as f64 / 1e6,
        ));
    }
    for (i, (dt, c, e, d, ns)) in serie.iter().enumerate() {
        if *e > 0 || i < 4 || i % 10 == 0 {
            println!(
                "quadro {i:2} | {dt:7.1} ms | compila {c:5} | despejos {e:2} ({d:5} fitas, {ns:7.1} ms no cadeado) | cache {:5}",
                cache.len()
            );
        }
    }
    let regime = &serie[40..];
    let media = regime.iter().map(|r| r.0).sum::<f64>() / regime.len() as f64;
    let mut ord: Vec<f64> = regime.iter().map(|r| r.0).collect();
    ord.sort_by(f64::total_cmp);
    println!(
        "REGIME (quadros 40+): mediana {:.1} ms · média {media:.1} ms · MÁXIMO {:.1} ms · despejos {}",
        ord[ord.len() / 2],
        ord[ord.len() - 1],
        regime.iter().map(|r| r.2).sum::<usize>()
    );
}

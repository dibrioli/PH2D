//! ⭐⭐⭐ **QUANTOS QUADROS A CACHE DEVE GUARDAR** — a varredura que escolhe o
//! [`ph2d_field_render::TapeCache`]`::FRAMES_KEPT` (W89).
//!
//! # ⚠️ Porque o número tinha de ser reconferido
//!
//! O `3` foi **derivado por raciocínio** (*«o quadro corrente, o anterior e o do outro documento»*)
//! e nunca varrido — e o que o sustentava mudou duas vezes na mesma jornada: a árvore saiu da fita
//! (o despejo ficou `50×` mais barato) e a varredura da fatia mostrou que **o que a cache guarda a
//! mais não é de graça** — o [`TapeCache::get`] é linear, e cada uma das ~600 regiões de um quadro
//! paga o tamanho da população. *Quem move o número que sustenta uma nota tem de reconferir a nota.*
//!
//! ⚠️ **Intercalado, cache nova por corrida**, `load < 5`.
//!
//! ```text
//! cargo test -p ph2d-field-render --profile ci-test --test how_many_frames_to_keep -- --ignored --nocapture
//! ```

use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
use ph2d_field_render::{Orbit, TapeCache};

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
        }],
        NodeId(0),
    )
    .expect("extrusão")
}

#[test]
#[ignore = "sonda; roda com --nocapture"]
fn measure_how_many_frames_of_tapes_are_worth_keeping() {
    const QUADROS: usize = 90;
    const REGIME: usize = 40;
    const GRAUS: f64 = 2.0;
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let doc = circulo(168);
    let quantos = [1usize, 2, 3, 4, 6];
    let mut tabela: Vec<Vec<(f64, f64, f64, usize)>> = vec![Vec::new(); quantos.len()];
    for _ronda in 0..3 {
        for (ni, n) in quantos.iter().enumerate() {
            let cache = TapeCache::with_frames_kept(*n);
            let mut ms: Vec<f64> = Vec::new();
            for i in 0..QUADROS {
                let cam = Orbit {
                    rotation: Orbit::from_yaw_pitch(
                        0.72 + (i as f64 * GRAUS).to_radians() as f32,
                        0.52,
                    )
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
                ms.push(t0.elapsed().as_secs_f64() * 1000.0);
            }
            let mut r: Vec<f64> = ms[REGIME..].to_vec();
            let media = r.iter().sum::<f64>() / r.len() as f64;
            r.sort_by(f64::total_cmp);
            tabela[ni].push((r[r.len() / 2], media, r[r.len() - 1], cache.len()));
        }
    }
    let med = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    println!("quadros guardados | mediana | média | MÁXIMO | fitas na cache");
    for (ni, n) in quantos.iter().enumerate() {
        println!(
            "{n:17} | {:7.1} | {:5.1} | {:6.1} | {:6}",
            med(tabela[ni].iter().map(|r| r.0).collect()),
            med(tabela[ni].iter().map(|r| r.1).collect()),
            med(tabela[ni].iter().map(|r| r.2).collect()),
            tabela[ni][0].3
        );
    }
}

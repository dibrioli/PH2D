//! ⭐⭐⭐ **O QUE CUSTA UMA EDIÇÃO COM A DIVISÃO ABERTA** (W90, item deixado por medir no §92.7).
//!
//! # A pergunta
//!
//! Com quatro vistas, mexer num raio invalida **as quatro**: elas disparam ao mesmo tempo, cada uma
//! na sua thread, e cada uma quer a máquina toda. A área somada é a mesma de uma vista só — mas o
//! **custo fixo** de um traçado (a re-amostragem do contorno, a montagem das fitas de cada região)
//! não encolhe com a área. ⇒ *a pergunta não é «quantos pixels» e sim «quantas vezes se paga o que
//! não depende dos pixels».*
//!
//! ⚠️ Precisa da máquina a `load < 5`. Intercalado ×3 e mediana, porque entre duas corridas desta
//! workstation o mesmo passe já deu `11,36` e `5,50 ms`.
//!
//! ```text
//! cargo test -p ph2d-field-render --profile ci-test --test the_price_of_four_views -- --ignored --nocapture
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
fn measure_what_an_edit_costs_with_the_canvas_split() {
    const CHEIO: (u32, u32) = (1280, 720);
    let doc = std::sync::Arc::new(circulo(168));
    let med = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let cam = |g: f64| Orbit {
        rotation: Orbit::from_yaw_pitch(0.72 + g.to_radians() as f32, 0.52).rotation,
        ..Orbit::default()
    };
    // ⚠️ **Uma EDIÇÃO, e não um arrasto**: a cache de fitas é inútil aqui de propósito (o documento
    // mudou, então nenhuma fita antiga serve). É o pior caso honesto, e é o que o artista sente ao
    // largar um slider.
    let uma = |w: u32, h: u32| -> f64 {
        let reg = ph2d_field_eval::hybrid::Registry::new();
        let cache = TapeCache::new();
        let t0 = std::time::Instant::now();
        let _ = ph2d_field_render::trace_cached_for_test(
            &doc,
            &reg,
            &cam(0.0),
            w,
            h,
            true,
            Some(&cache),
        );
        t0.elapsed().as_secs_f64() * 1000.0
    };
    let quatro = || -> f64 {
        let t0 = std::time::Instant::now();
        let fios: Vec<_> = (0..4)
            .map(|i| {
                let doc = std::sync::Arc::clone(&doc);
                std::thread::spawn(move || {
                    // Cada viewport tem a SUA cache (W90) e a sua câmera.
                    let reg = ph2d_field_eval::hybrid::Registry::new();
                    let cache = TapeCache::new();
                    let c = Orbit {
                        rotation: Orbit::from_yaw_pitch(
                            0.72 + f64::from(i * 30).to_radians() as f32,
                            0.52,
                        )
                        .rotation,
                        ..Orbit::default()
                    };
                    let _ = ph2d_field_render::trace_cached_for_test(
                        &doc,
                        &reg,
                        &c,
                        CHEIO.0 / 2,
                        CHEIO.1 / 2,
                        true,
                        Some(&cache),
                    );
                })
            })
            .collect();
        for f in fios {
            let _ = f.join();
        }
        t0.elapsed().as_secs_f64() * 1000.0
    };
    let (mut a, mut b, mut c) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..3 {
        a.push(uma(CHEIO.0, CHEIO.1));
        b.push(uma(CHEIO.0 / 2, CHEIO.1 / 2));
        c.push(quatro());
    }
    let (cheio, quarto, todas) = (med(a), med(b), med(c));
    println!(
        "uma vista, área INTEIRA  {}x{}  | {cheio:8.1} ms",
        CHEIO.0, CHEIO.1
    );
    println!(
        "uma vista, um QUARTO     {}x{}   | {quarto:8.1} ms  ({:.2}x do inteiro)",
        CHEIO.0 / 2,
        CHEIO.1 / 2,
        quarto / cheio
    );
    println!(
        "QUATRO vistas ao mesmo tempo          | {todas:8.1} ms  ({:.2}x do inteiro · {:.2}x de uma sozinha)",
        todas / cheio,
        todas / quarto
    );
}

//! ⭐⭐⭐ **QUANTAS FATIAS DE PROFUNDIDADE, AGORA** (W94) — a terceira reconferência do
//! [`ph2d_field_render`]`::SLABS`.
//!
//! # Porque ela tinha de ser refeita
//!
//! *Uma varredura envelhece com o custo que ela pesava.* Esta já mudou de veredito uma vez: a
//! original escolheu `2`, e a W71 — depois de a W70 tirar a fita de gradiente e o `fork` de uma
//! região — devolveu `4`, que é o que ship. Desde então mudaram **duas** coisas que ela pesa:
//!
//! - o **ladrilho** foi de `64` para `24` (W88) ⇒ a região de uma fatia tem outra forma;
//! - a **cache de fitas** passou a existir (W82) ⇒ o custo de uma região deixou de ser a compilação
//!   dela na maioria dos quadros.
//!
//! ⚠️ Intercalado, cache nova por corrida, no **regime** de um arrasto. Precisa da máquina calma.
//!
//! ```text
//! cargo test -p ph2d-field-render --profile ci-test --test how_many_slabs_now -- --ignored --nocapture
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
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_how_many_slabs_the_frame_wants_now() {
    const QUADROS: usize = 40;
    const REGIME: usize = 16;
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let tile = ph2d_field_render::tile_for_test();
    let fatias = [2usize, 3, 4, 6, 8];
    // ⚠️ **Duas peças e dois tamanhos**: uma varredura numa fixtura só escolhe uma constante GLOBAL
    // a partir do que amostrou. O contorno denso é o caso em que a montagem pesa mais, e o tamanho
    // maior é o do assentar.
    for (nome, arestas, w, h) in [
        ("contorno 168 · 426x240", 168usize, 426u32, 240u32),
        ("contorno 940 · 426x240", 940, 426, 240),
        ("contorno 168 · 640x360", 168, 640, 360),
    ] {
        let doc = circulo(arestas);
        let mut tabela: Vec<Vec<(f64, f64, f64)>> = vec![Vec::new(); fatias.len()];
        for _ronda in 0..3 {
            for (fi, s) in fatias.iter().enumerate() {
                let cache = TapeCache::new();
                let mut ms: Vec<f64> = Vec::new();
                for i in 0..QUADROS {
                    let cam = Orbit {
                        rotation: Orbit::from_yaw_pitch(
                            0.72 + (i as f64 * 2.0).to_radians() as f32,
                            0.52,
                        )
                        .rotation,
                        ..Orbit::default()
                    };
                    let t0 = std::time::Instant::now();
                    let _ = ph2d_field_render::trace_tiled_with_cache_for_test(
                        &doc,
                        &reg,
                        &cam,
                        w,
                        h,
                        tile,
                        *s,
                        Some(&cache),
                    );
                    ms.push(t0.elapsed().as_secs_f64() * 1000.0);
                }
                let mut r: Vec<f64> = ms[REGIME..].to_vec();
                let media = r.iter().sum::<f64>() / r.len() as f64;
                r.sort_by(f64::total_cmp);
                tabela[fi].push((r[r.len() / 2], media, r[r.len() - 1]));
            }
        }
        let med = |mut v: Vec<f64>| {
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        println!(
            "--- {nome} · ladrilho {tile} · o produto ship SLABS = {} ---",
            ph2d_field_render::slabs_for_test()
        );
        println!("fatias | mediana | média | máximo");
        for (fi, s) in fatias.iter().enumerate() {
            println!(
                "{s:6} | {:7.2} | {:5.2} | {:6.2}",
                med(tabela[fi].iter().map(|r| r.0).collect()),
                med(tabela[fi].iter().map(|r| r.1).collect()),
                med(tabela[fi].iter().map(|r| r.2).collect())
            );
        }
    }
    let doc = circulo(168);
    // ⭐⭐⭐ **E A IMAGEM?** — a coluna que a varredura original nomeia como *«o que separa ficou
    // rápido de ficou rápido e errado»*. As fatias particionam a profundidade, e uma fatia a menos é
    // uma árvore que responde num tubo mais longo: o pruning muda, a resposta **não pode** mudar.
    let cam = Orbit {
        rotation: Orbit::from_yaw_pitch(0.72 + 12.0f32.to_radians(), 0.52).rotation,
        ..Orbit::default()
    };
    let referencia = ph2d_field_render::trace_tiled_with_cache_for_test(
        &doc,
        &reg,
        &cam,
        426,
        240,
        tile,
        ph2d_field_render::slabs_for_test(),
        None,
    )
    .expect("o traçado de referência");
    for s in fatias {
        let g = ph2d_field_render::trace_tiled_with_cache_for_test(
            &doc, &reg, &cam, 426, 240, tile, s, None,
        )
        .expect("traçado");
        let diferentes = referencia
            .hit
            .iter()
            .zip(g.hit.iter())
            .filter(|(a, b)| a != b)
            .count();
        let pior_normal = referencia
            .normal
            .iter()
            .zip(g.normal.iter())
            .map(|(a, b)| (0..3).map(|k| (a[k] - b[k]).abs()).fold(0.0f32, f32::max))
            .fold(0.0f32, f32::max);
        println!(
            "  {s} fatias: {diferentes} pixels de silhueta diferentes · normal pior {pior_normal:.2e}"
        );
    }
}

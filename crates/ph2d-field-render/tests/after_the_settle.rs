//! ⭐⭐⭐ **O QUADRO DE MOVIMENTO LOGO A SEGUIR AO ASSENTAR** (W89) — o report do Enio de 26/08:
//! *«de tempos em tempos dá pequenas travadinhas»*.
//!
//! # ⚠️ A cegueira que esta sonda fecha
//!
//! Todas as bancadas desta linha mediam **um tipo de quadro de cada vez, com a cache quente daquele
//! tipo**. O artista não faz isso: ele arrasta (quadros grossos), pára (a imagem afina, no tamanho
//! **cheio**) e volta a arrastar. *A travadinha não vive dentro de um tipo de quadro — vive na
//! TRANSIÇÃO entre eles*, e nenhuma mediana de arrasto contínuo a pode conter.
//!
//! # A régua
//!
//! Uma sequência única, num processo só: `N` quadros de movimento (regime), **um** refinamento no
//! tamanho cheio, e mais `N` de movimento. Cada traçado imprime o relógio, os acertos da cache, as
//! compilações e os **despejos** — e é a leitura lado a lado que nomeia o mecanismo.
//!
//! ⚠️ **Binário só com este teste**: os contadores são globais, e um vizinho a correr em paralelo
//! escreve neles (*um contador global só é legível onde ninguém mais escreve nele*).
//!
//! ```text
//! cargo test -p ph2d-field-render --profile ci-test --test after_the_settle -- --nocapture
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
#[ignore = "sonda; roda com --nocapture"]
fn measure_the_moving_frame_right_after_a_full_size_settle() {
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let doc = circulo(168);
    let cache = TapeCache::new();
    let cam = |g: f64| Orbit {
        rotation: Orbit::from_yaw_pitch(0.72 + g.to_radians() as f32, 0.52).rotation,
        ..Orbit::default()
    };
    // O tamanho grosso que o laço escolhe a 1280×720 é `D=3`; o cheio é a área.
    let (gw, gh) = (426u32, 240u32);
    let (cw, ch) = (1280u32, 720u32);
    let passo = |nome: &str, g: f64, w: u32, h: u32, aa: bool| {
        let (h0, c0, e0, d0) = (
            ph2d_field_render::TAPE_HITS.load(Relaxed),
            ph2d_field_eval::hybrid::FLOAT_TAPES.load(Relaxed),
            ph2d_field_render::TAPE_EVICTIONS.load(Relaxed),
            ph2d_field_render::TAPE_DROPPED.load(Relaxed),
        );
        let t0 = std::time::Instant::now();
        let _ =
            ph2d_field_render::trace_cached_for_test(&doc, &reg, &cam(g), w, h, aa, Some(&cache));
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        println!(
            "{nome:28} {w:5}x{h:<4} | {ms:8.1} ms | acertos {:6} | compila {:6} | despejos {:3} ({} fitas) | cache {}",
            ph2d_field_render::TAPE_HITS.load(Relaxed) - h0,
            ph2d_field_eval::hybrid::FLOAT_TAPES.load(Relaxed) - c0,
            ph2d_field_render::TAPE_EVICTIONS.load(Relaxed) - e0,
            ph2d_field_render::TAPE_DROPPED.load(Relaxed) - d0,
            cache.len(),
        );
        ms
    };
    for i in 0..6 {
        passo("regime (movimento)", f64::from(i) * 2.0, gw, gh, false);
    }
    passo("ASSENTAR (cheio, com AA)", 12.0, cw, ch, true);
    for i in 0..8 {
        passo(
            "depois do assentar",
            14.0 + f64::from(i) * 2.0,
            gw,
            gh,
            false,
        );
    }
}

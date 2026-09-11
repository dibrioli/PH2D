//! ⭐⭐⭐ **A CACHE DE FITAS DE FACTO POUPA COMPILAÇÕES** (W82) — a lei que nenhuma imagem mostra.
//!
//! O `the_cache_never_changes_the_image` prova que a cache não estraga nada. ⚠️ **Uma cache que
//! nunca acerta passa nesse gate com nota máxima** — a imagem dela é exactamente a mesma. *Contar o
//! trabalho feito não é contar o trabalho poupado*, e o que esta cache existe para fazer é
//! **poupar**: compilar as fitas de um quadro custa `~14 ms` de um quadro de `~24` e **satura às 16
//! threads** (`docs/3DModeling/06` §82.9).
//!
//! # ⛔ Porque é um binário de teste e não um irmão do `src/tests.rs`
//!
//! A mesma razão do [`tape_budget`] e do [`march_budget`]: os contadores são do **processo**, e o
//! `cargo test` corre a suíte em paralelo. *Um contador global só é legível onde ninguém mais
//! escreve nele.*
//!
//! [`tape_budget`]: ./tape_budget.rs
//! [`march_budget`]: ./march_budget.rs

use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
use ph2d_field_eval::hybrid::{FLOAT_TAPES, Registry};
use ph2d_field_render::{Orbit, TAPE_HITS, TapeCache, trace_cached_for_test};
use std::sync::atomic::Ordering;

fn profile_piece(edges: usize) -> FieldDoc {
    let contour: Vec<[f32; 2]> = (0..edges)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (edges as f64);
            [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
        })
        .collect();
    FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Extrude {
                profile: Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil"),
                half_height: 0.4,
                round: 0.06,
                chamfer: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("extrusão")
}

/// Um arrasto de `n` quadros a partir do quadro `de`, e o que ele compilou **por quadro**.
fn drag(
    doc: &FieldDoc,
    reg: &Registry,
    cache: Option<&TapeCache>,
    de: usize,
    n: usize,
) -> (usize, usize) {
    FLOAT_TAPES.store(0, Ordering::Relaxed);
    TAPE_HITS.store(0, Ordering::Relaxed);
    for i in 0..n {
        let cam = Orbit {
            rotation: Orbit::from_yaw_pitch(0.72 + ((de + i) as f32) * 2.0f32.to_radians(), 0.52)
                .rotation,
            ..Orbit::default()
        };
        // ⚠️ Sem anti-serrilhado: é o quadro de MOVIMENTO que a cache existe para acelerar, e a 2.ª
        // passagem compila por lote de pixels de borda — outra lei, e a contagem dela depende de
        // quantos pixels a silhueta tocou.
        let _ = trace_cached_for_test(doc, reg, &cam, 320, 180, false, cache);
    }
    (
        FLOAT_TAPES.load(Ordering::Relaxed) / n,
        TAPE_HITS.load(Ordering::Relaxed) / n,
    )
}

#[test]
fn a_drag_stops_recompiling_the_tapes_it_already_has() {
    let reg = Registry::new();
    let doc = profile_piece(168);
    let cache = TapeCache::new();
    let quadros = 8usize;

    // ⚠️ **O 1.º arrasto é o aquecimento** — ele enche a cache do zero e não representa nenhum
    // quadro que o artista veja depois do primeiro. O que se mede é o **regime**.
    let _ = drag(&doc, &reg, Some(&cache), 0, quadros);

    let (sem, sem_hits) = drag(&doc, &reg, None, quadros, quadros);
    let (com, com_hits) = drag(&doc, &reg, Some(&cache), quadros * 2, quadros);

    // ⛔ **O balde tem de estar cheio.** Um traçado que não especializou nada passaria em tudo o que
    // vem a seguir sem medir coisa nenhuma.
    assert!(
        sem > 50,
        "o traçado sem cache compilou {sem} fitas por quadro — a fixtura não especializa nada, e as \
         desigualdades abaixo não estariam a prender nada"
    );
    assert_eq!(
        sem_hits, 0,
        "o traçado SEM cache reportou {sem_hits} acertos de cache — o contador está a somar de \
         outro sítio, e a razão abaixo mediria ruído"
    );

    // ⭐⭐⭐ **A lei.** Medido a `2°` por quadro: `225` fitas por quadro sem cache contra `30` com
    // ela, com `195` acertos — `87 %`. A barra é `4×` para não ser um gate de relógio disfarçado:
    // o que se defende é que a cache **funciona**, e a afinação dela vive na tabela do `INFLATE`.
    assert!(
        com * 4 <= sem,
        "com cache o quadro ainda compila {com} fitas contra {sem} sem ela — a cache não está a \
         servir o quadro seguinte, e o único trabalho que ela faz é ocupar memória"
    );
    assert!(
        com_hits > com,
        "a cache serviu {com_hits} fitas e compilou {com} — no regime ela tem de servir mais do que \
         constrói, senão o arrasto está a sair da região de cada fita antes de a reusar"
    );
}

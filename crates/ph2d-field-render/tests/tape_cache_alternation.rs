//! ⭐⭐⭐ **A CACHE SOBREVIVE AO GESTO DO ARTISTA** (W82b) — o binário do defeito que o smoke do
//! Enio apanhou.
//!
//! ⛔ **Binário PRÓPRIO, e a razão é um defeito meu:** este teste nasceu ao lado do
//! `a_drag_stops_recompiling_the_tapes_it_already_has`, no mesmo binário — e os dois leem os
//! **mesmos contadores globais** enquanto o `cargo test` os corre em threads paralelas. Eles
//! passaram algumas vezes por sorte de escalonamento e reprovaram os dois na primeira corrida em que
//! não houve. *Um gate que depende do escalonador é pior que gate nenhum: ele ensina a ignorar o
//! vermelho.* O doc do [`tape_budget`] já dizia a lei — *um contador global só é legível onde
//! ninguém mais escreve nele* — e eu escrevi o segundo teste ao lado do primeiro na mesma hora.
//!
//! [`tape_budget`]: ./tape_budget.rs

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
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("extrusão")
}

/// ⭐⭐⭐ **A CACHE SOBREVIVE À ALTERNÂNCIA QUE O PREVIEW FAZ** (W82) — o gate do defeito que o smoke
/// do Enio apanhou.
///
/// ⚠️ **O app alterna DOIS documentos por construção** (`field3d_preview::coarse_doc`): o contorno
/// **grosso** enquanto a mão mexe, o **cheio** ao parar. A 1.ª cache deitava tudo fora quando o
/// documento mudava ⇒ **cada paragem e cada retoma custavam um quadro frio**, com as `~68` fitas
/// compiladas de novo e **zero** acertos.
///
/// ⛔ **E nenhuma bancada de arrasto contínuo podia ver isto** — a minha media um documento só.
/// *Uma cache mede-se no ciclo que o artista faz, e o ciclo dele tem uma paragem.*
#[test]
fn the_cache_survives_the_alternation_between_the_coarse_and_the_full_document() {
    let reg = Registry::new();
    let grosso = profile_piece(168);
    let cheio = profile_piece(672);
    let cache = TapeCache::new();
    let mut passo = 0usize;
    let mut quadro = |doc: &FieldDoc, avanca: bool| -> (usize, usize) {
        FLOAT_TAPES.store(0, Ordering::Relaxed);
        TAPE_HITS.store(0, Ordering::Relaxed);
        let cam = Orbit {
            rotation: Orbit::from_yaw_pitch(0.72 + (passo as f32) * 2.0f32.to_radians(), 0.52)
                .rotation,
            ..Orbit::default()
        };
        if avanca {
            passo += 1;
        }
        let _ = trace_cached_for_test(doc, &reg, &cam, 320, 180, false, Some(&cache));
        (
            FLOAT_TAPES.load(Ordering::Relaxed),
            TAPE_HITS.load(Ordering::Relaxed),
        )
    };
    // Uma volta inteira só para aquecer os DOIS documentos.
    for _ in 0..4 {
        let _ = quadro(&grosso, true);
    }
    for _ in 0..2 {
        let _ = quadro(&cheio, false);
    }
    // ⭐ **A medição é a RETOMA** — o 1.º quadro a girar logo depois de uma paragem, que é onde o
    // defeito vivia.
    let (fitas, acertos) = quadro(&grosso, true);
    assert!(
        acertos > 0 && fitas > 0,
        "a retoma não mediu nada ({fitas} fitas, {acertos} acertos) — a fixtura não especializa"
    );
    assert!(
        acertos > fitas * 2,
        "a retoma depois de uma paragem compilou {fitas} fitas para {acertos} acertos — a cache \
         está a ser deitada fora na alternância grosso/cheio que o preview faz a cada gesto"
    );
}

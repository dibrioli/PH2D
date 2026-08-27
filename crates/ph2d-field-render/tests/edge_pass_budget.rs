//! ⭐⭐⭐ **O ANTI-SERRILHADO NÃO COMPILA NADA** (W83) — a lei que o smoke do Enio destapou.
//!
//! A 2.ª passagem do traçado re-amostra os pixels de **borda** em lotes de `EDGE_CHUNK = 64`, e cada
//! lote precisa de um avaliador próprio (o da `fidget` tem estado mutável). ⛔ **Até à W83 cada um
//! desses avaliadores COMPILAVA a árvore inteira** — a mais cara que existe, sem especialização
//! nenhuma.
//!
//! Medido (`measure_the_settle_of_a_default_resolution_piece`, peça na resolução de omissão, o
//! assentar a `640×360`): `1 762` pixels de borda ⇒ `28` lotes ⇒ **`29` das `29` fitas do quadro**,
//! com a passagem primária a `100 %` de acerto na cache. Depois: **`1`**.
//!
//! ⚠️ **A W70 mediu «reaproveitar o avaliador entre lotes» e achou-o NEUTRO**, e a nota dela dizia
//! porquê: *«o quadro tem `917` regiões especializadas nesse tamanho: as dezenas de fitas desta
//! passagem são ruído ao lado delas»*. ⭐ A W82 apagou aquele `917`, e com ele a premissa. *Quem move
//! o número que sustenta uma nota tem de reconferir a nota.*
//!
//! ⚠️ **E a cura não é a que a W70 tentou.** Ela tentou reaproveitar o **avaliador**, que é estado
//! mutável e não se partilha entre threads. O que se partilha é a **fita** (`Arc<Mmap>` por dentro):
//! o `Hybrid::fork` clona-a e constrói um avaliador novo. *O que se partilha é o código; o que se
//! duplica é o rascunho.*
//!
//! # ⛔ Porque é um binário próprio
//!
//! Os contadores são do **processo** — ver o `tape_budget`.

use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
use ph2d_field_eval::hybrid::{FLOAT_TAPES, Registry};
use ph2d_field_render::{Orbit, TapeCache, trace_cached_for_test};
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

#[test]
fn the_antialias_pass_compiles_no_tape_of_its_own() {
    let reg = Registry::new();
    let doc = profile_piece(168);
    let cache = TapeCache::new();
    let cam = Orbit::default();
    let (w, h) = (640u32, 360u32);

    // Aquece: a passagem primária deixa de compilar, e o que sobrar é da 2.ª.
    for _ in 0..2 {
        let _ = trace_cached_for_test(&doc, &reg, &cam, w, h, false, Some(&cache));
    }
    FLOAT_TAPES.store(0, Ordering::Relaxed);
    let sem = trace_cached_for_test(&doc, &reg, &cam, w, h, false, Some(&cache));
    let base = FLOAT_TAPES.load(Ordering::Relaxed);

    FLOAT_TAPES.store(0, Ordering::Relaxed);
    let com = trace_cached_for_test(&doc, &reg, &cam, w, h, true, Some(&cache));
    let total = FLOAT_TAPES.load(Ordering::Relaxed);

    // ⛔ **O balde tem de estar cheio**: sem pixels de borda a 2.ª passagem nem corre, e a igualdade
    // abaixo seria verdadeira por ausência.
    assert!(
        com.edges.len() > 500,
        "o quadro só teve {} pixels de borda — a 2.ª passagem quase não correu, e a lei abaixo não \
         estaria a prender nada",
        com.edges.len()
    );
    assert_eq!(
        sem.hits(),
        com.hits(),
        "o anti-serrilhado mudou a máscara — ele só devia acrescentar amostras de borda"
    );

    // ⭐⭐⭐ **A lei.** A 2.ª passagem constrói um avaliador por lote de 64 pixels de borda; nenhum
    // deles pode compilar. `1 762` bordas são `28` lotes, e `28` compilações da árvore INTEIRA é o
    // trabalho mais caro do quadro.
    assert_eq!(
        total,
        base,
        "ligar o anti-serrilhado compilou {} fitas a mais ({} pixels de borda, {} lotes) — cada uma \
         é a árvore inteira, e ela já está compilada: o que um lote precisa é de um avaliador, não \
         de um compilador",
        total - base,
        com.edges.len(),
        com.edges.len().div_ceil(64),
    );
}

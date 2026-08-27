//! ⭐⭐⭐ **O ORÇAMENTO DE AMOSTRAS DE UM QUADRO** (W81) — o gate do que a marcha de facto pede.
//!
//! A W71 mediu que a marcha é `80 %` do quadro e a W81 contou-a: `2 121 060` avaliações de campo num
//! quadro de movimento a `640×360` sobre uma extrusão de 168 arestas. Duas leis saíram dessa
//! contagem, e nenhuma delas é visível numa imagem — *um defeito só de custo é invisível a todo gate
//! de saída*.
//!
//! 1. ⭐⭐⭐ **Fatiar e ladrilhar não custam UMA amostra.** A especialização por região muda o
//!    **preço** de uma amostra, nunca o **valor** dela: o raio anda `t += d · passo` e pára onde
//!    parava, qualquer que seja o tamanho do ladrilho. É a mesma lei que a paridade de imagem
//!    afirma, um nível abaixo — e mais forte, porque diz que o **caminho** é o mesmo e não só o
//!    destino.
//! 2. ⭐⭐ **Uma normal por acerto, `n` amostras por normal** — o `n` do estêncil que ship. Sem ela,
//!    o «a normal é `21 %` do quadro» seria uma divisão sem juiz.
//!
//! # ⛔ Porque é um binário de teste e não um irmão do `src/tests.rs`
//!
//! A mesma razão do [`tape_budget`]: os contadores são do **processo**, e o `cargo test` corre a
//! suíte em paralelo — qualquer outro gate a traçar ao mesmo tempo soma à conta. *Um contador
//! global só é legível onde ninguém mais escreve nele.*
//!
//! [`tape_budget`]: ./tape_budget.rs

use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
use ph2d_field_eval::hybrid::Registry;
use ph2d_field_render::{NORMAL_SAMPLES, Orbit, STEP_SAMPLES, trace_tiled_for_test};
use std::sync::atomic::Ordering;

fn profile_piece(edges: usize) -> FieldDoc {
    let contour: Vec<[f32; 2]> = (0..edges)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (edges as f64);
            [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
        })
        .collect();
    let profile = Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil");
    FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Extrude {
                profile,
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
fn the_tiling_changes_what_a_sample_costs_and_never_how_many_there_are() {
    let (w, h) = (320u32, 180u32);
    let doc = profile_piece(168);
    let reg = Registry::new();
    let cam = Orbit::default();

    let mut budget: Vec<(usize, u64, u64, usize)> = Vec::new();
    for tile in [32usize, 64, 128] {
        STEP_SAMPLES.store(0, Ordering::Relaxed);
        NORMAL_SAMPLES.store(0, Ordering::Relaxed);
        // ⚠️ **Serial e sem anti-serrilhado.** A 2.ª passagem re-marcha a silhueta e a conta dela
        // depende de quantos pixels ela tocou; deixá-la de fora é o que faz isto ser uma afirmação
        // sobre a MARCHA. O paralelismo não muda contagem nenhuma, e serial mantém-nas legíveis.
        let g = trace_tiled_for_test(
            &doc,
            &reg,
            &cam,
            w,
            h,
            tile,
            ph2d_field_render::slabs_for_test(),
            false,
            false,
        )
        .expect("traçado");
        budget.push((
            tile,
            STEP_SAMPLES.load(Ordering::Relaxed),
            NORMAL_SAMPLES.load(Ordering::Relaxed),
            g.hits(),
        ));
    }

    // ⛔ **O balde tem de estar cheio** — um traçado que não desenhou nada passaria em todas as
    // igualdades abaixo sem medir coisa nenhuma.
    assert!(
        budget[0].1 > 10_000 && budget[0].3 > 1_000,
        "a fixtura não marchou ({} amostras, {} acertos) — as igualdades abaixo não mediriam nada",
        budget[0].1,
        budget[0].3
    );

    // ⭐⭐⭐ **Lei 1** — ver o doc do módulo.
    for w in budget.windows(2) {
        let (a, b) = (w[0], w[1]);
        assert_eq!(
            a.1, b.1,
            "o ladrilho de {} deu {} amostras de marcha e o de {} deu {} — a especialização mudou \
             o CAMINHO do raio, e não só o preço de o andar",
            a.0, a.1, b.0, b.1
        );
        assert_eq!(
            a.3, b.3,
            "o ladrilho de {} acertou em {} pixels e o de {} em {}",
            a.0, a.3, b.0, b.3
        );
    }

    // ⭐⭐ **Lei 2** — uma normal por acerto, `n` amostras por normal.
    let n = ph2d_field_render::NORMAL_STENCIL_WIDTH;
    for (tile, _, normals, hits) in budget {
        assert_eq!(
            normals,
            hits as u64 * n as u64,
            "a lado {tile} o quadro pediu {normals} amostras de normal para {hits} acertos, e o \
             estêncil que ship tem {n} deslocamentos"
        );
    }
}

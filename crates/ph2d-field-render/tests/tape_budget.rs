//! ⭐⭐⭐ **O ORÇAMENTO DE FITAS DE UM QUADRO** (W70) — o gate da lei que a montagem paga.
//!
//! A W68 mediu que o traçado de uma peça de perfil é **quase só montagem** (`132` árvores
//! especializadas num quadro a `640×360`) e a W70 partiu essa montagem em partes: por
//! especialização pagava-se a **árvore** (`0,11 ms`), a fita **float** (`1,37`), uma fita de
//! **gradiente** (`1,47`) que ninguém avalia, e depois um **`fork`** (`2,89`) que recompilava as
//! duas. Uma fita de quatro era usada.
//!
//! Este binário defende as duas leis que sobraram:
//!
//! 1. **um traçado não compila fita de gradiente nenhuma** — o consumidor dela é a extração de
//!    malha, e a normal do traçado sai por diferença central na fita float;
//! 2. **cada região paga UMA fita** — quem acabou de montar a sua não a forka.
//!
//! ⛔ **A terceira — reaproveitar o avaliador entre os lotes da segunda passagem — foi construída,
//! medida e REVERTIDA** (`0,97×`–`1,01×`; ver o doc do `EDGE_CHUNK`). O gate dela saiu com ela:
//! *um gate que defende código revertido é um gate a defender nada.*
//!
//! # ⛔ Porque é um binário de teste e não um irmão do `src/tests.rs`
//!
//! Os contadores são do **processo**, e o `cargo test` corre a suíte em paralelo: qualquer outro
//! gate a traçar ao mesmo tempo soma à conta. ⚠️ E um cadeado **não** resolveria — ele teria de ser
//! tomado por *todos* os testes que traçam, incluindo os que ainda não existem. *Um contador global
//! só é legível onde ninguém mais escreve nele.*

use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
use ph2d_field_eval::hybrid::{FLOAT_TAPES, GRAD_TAPES, Registry};
use ph2d_field_render::{FORKED, Orbit, SPECIALISED, slabs_for_test, trace_tiled_for_test};
use std::sync::atomic::Ordering;

/// Uma peça de PERFIL — é a família em que a montagem domina, e a única em que há o que
/// especializar (um cilindro analítico não tem contorno).
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
fn a_frame_pays_one_tape_per_region_and_no_gradient_tape_at_all() {
    let (w, h, tile) = (640u32, 360u32, 64usize);
    let doc = profile_piece(168);
    let reg = Registry::new();
    let cam = Orbit::default();

    SPECIALISED.store(0, Ordering::Relaxed);
    FORKED.store(0, Ordering::Relaxed);
    FLOAT_TAPES.store(0, Ordering::Relaxed);
    GRAD_TAPES.store(0, Ordering::Relaxed);
    // ⚠️ **Sem anti-serrilhado de propósito.** A segunda passagem monta uma fita por lote de pixels
    // de borda — trabalho legítimo, de outra lei (ela marcha a árvore partilhada), e cuja contagem
    // depende de quantos pixels a silhueta tocou. Deixá-la de fora é o que torna o teto abaixo uma
    // afirmação sobre a MARCHA e não sobre a forma da peça.
    let g = trace_tiled_for_test(&doc, &reg, &cam, w, h, tile, slabs_for_test(), false, true)
        .expect("traçado");
    let specialised = SPECIALISED.load(Ordering::Relaxed);
    let float_tapes = FLOAT_TAPES.load(Ordering::Relaxed);

    // ⛔ **O balde tem de estar cheio.** Um traçado que não especializou nada passaria em todas as
    // desigualdades abaixo sem medir coisa nenhuma — *um zero de «não mediu» e um de «perfeito»
    // leem-se iguais* (o defeito que as réguas de valência do quad remesh pagaram).
    assert!(
        specialised > 0 && g.hit.iter().any(|&h| h),
        "a fixtura não especializou nada ({specialised} regiões) ou não desenhou a peça — \
         o teto abaixo não estaria a medir a marcha"
    );

    // ⭐⭐⭐ **Uma fita por região, mais a base, mais UMA POR RECUO CONTADO** (apertado na W81).
    //
    // ⛔ **A folga era «mais uma por LADRILHO», e ela media `60`** — *uma folga num teto é o tamanho
    // do ponto cego que ele tem*, e por trás daquele cabia um defeito de vinte e sete fitas. O
    // recuo (a rota não especializada, que marcha a árvore partilhada e a forka) passou a ter
    // **contador** ([`FORKED`]), então o teto deixa de ser uma estimativa e passa a ser a conta.
    //
    // ⭐ **E o recuo é ZERO neste quadro** — medido na W81. O teto aperta de `specialised + 60 + 1`
    // para `specialised + 0 + 1`.
    let forked = FORKED.load(Ordering::Relaxed);
    let ceiling = specialised + forked + 1;
    assert!(
        float_tapes <= ceiling,
        "o quadro compilou {float_tapes} fitas float para {specialised} regiões com {forked} \
         recuos (teto {ceiling}) — quem monta a própria fita não a pode forkar, senão paga a \
         compilação duas vezes para a MESMA imagem"
    );

    // ⭐⭐⭐ **E nenhuma fita de gradiente.** Monotónico e a zero: nem este traçado nem nenhum outro
    // deste binário a pediu.
    assert_eq!(
        GRAD_TAPES.load(Ordering::Relaxed),
        0,
        "o traçado compilou fita de gradiente — a normal dele sai de seis amostras na fita float, \
         e quem consome o gradiente exacto é a extração de malha"
    );
}

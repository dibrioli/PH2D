//! ⭐⭐⭐ **A DOBRA NÃO MATA RAIOS À FOME, EM NENHUMA BANDA** — o irmão que faltava, e o report do
//! Enio de 2026-08-31 pagou-o (*«resultados bizarros, veja um cubo fino e alto com Bend»*).
//!
//! # ⛔⛔ Duas ausências, e a segunda é a que dói
//!
//! 1. A **torção** tem um gate de fome desde 2026-08-30
//!    ([`super`]... `the_twist_does_not_starve_the_march`); a **dobra** não tinha nenhum.
//! 2. E o que faltava não era só o modificador: era a **BANDA**. Todo gate deste repo — e o próprio
//!    nascimento do modificador — dá à banda uma faixa que **cobre a peça inteira** (`[−2, 2]`,
//!    `[−9, 9]`). *Um parâmetro medido num ponto só é um parâmetro por medir*, e as duas linhas que
//!    o Enio arrastou na foto (`From` e `To`) são exactamente esse parâmetro.
//!
//! # O mecanismo que ele fotografou
//!
//! Escrever a banda no `z` de ENTRADA congela o mapa fora dela: `x` e `z` deixam de depender do
//! eixo, e o campo fica **constante**. Medido na peça da foto: `+0,00047` em `z = 0,5` e o mesmo
//! `+0,00047` em `z = 20`. A marcha anda o valor do campo ⇒ **ela não sai dali**, esgota o
//! orçamento, e ⛔ **um raio que esgota o orçamento é largado em SILÊNCIO** — o pixel dele lê-se
//! como fundo. A peça aparecia cortada por um plano vertical, que é a fronteira do recorte.
//!
//! ⚠️ **A sonda de `‖∇f‖` chamava a isto SEGURO** (`0,1161`): uma planície tem gradiente **zero**, e
//! zero passa em qualquer barra de máximo. *Um `0,00` de «achatado» e um `0,00` de «aqui não há
//! nada» são o mesmo byte.*

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, Xform};
use ph2d_field_eval::{hybrid::Registry, safe_march_step};
use ph2d_field_render::{EXHAUSTED, Orbit, trace_stepped_for_test};
use std::sync::atomic::Ordering;

/// A chapa alta e fina do report, com a dobra dada.
///
/// ⚠️ **A forma é load-bearing**: a dobra age no `Z`, que aqui é a dimensão **curta** (`0,072`),
/// então toda banda que o artista arraste cai fora da matéria — que é onde o mapa congelava.
fn chapa(turns: f32, lower: f32, upper: f32, falloff: f32) -> FieldDoc {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: [0.207, 0.5025, 0.036],
            round: 0.02,
            chamfer: 0.0,
        }),
    );
    n.mods.push(Unary::Bend {
        turns,
        lower,
        upper,
        falloff,

        axis: ph2d_field::mods::BEND_AXIS,
    });
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

/// ⭐⭐⭐ **A BANDA VARRIDA EM TODA A FAIXA** — o gate.
///
/// ⛔⛔⛔ **A MUTAÇÃO SOBREVIVE A ESTE GATE, e a nota fica por isso** (2026-08-31): devolver a
/// `ph2d_field_eval::stack_bend::bend` à lei anterior — a banda escrita no `z` de ENTRADA — deixa
/// este gate **verde**. O orçamento é `MAX_STEPS × shrink` (`4 000` com o divisor a `10`), e a
/// planície de `+0,00047` custa ~`850` passos para atravessar: **lenta, e dentro do orçamento**.
///
/// ⇒ *a cauda não é morta por FOME, é cortada pela CAIXA DE RECORTE* — a bola de bordo é finita e a
/// peça, com o mapa congelado, não era. Quem a apanha é a imagem contra a marcha honesta
/// (`the_bend_draws_what_an_honest_march_draws`, `604` pixels que o oráculo acerta e o produto
/// deixa vazios). **Este gate fica pela outra metade**: a dobra tinha o irmão da torção em falta, e
/// uma banda estreita é a configuração em que um deformador mais facilmente estrangula a marcha.
#[test]
fn a_bent_piece_never_starves_the_march_whatever_the_band() {
    let reg = Registry::default();
    let cam = Orbit {
        half_extent: 0.85,
        ..Orbit::default()
    };
    let mut maus = Vec::new();
    let mut acertos_totais = 0u64;
    for (lo, up, fall) in [
        // A banda EXACTA da foto, e as vizinhas.
        (-0.187f32, 0.048f32, 0.072f32),
        (-0.187, 0.048, 0.0),
        (-0.02, 0.02, 0.01),
        // Inteiramente FORA da matéria, dos dois lados — o pior caso do congelamento.
        (0.20, 0.40, 0.05),
        (-0.40, -0.20, 0.05),
        // Assimétricas e degenerada (largura zero).
        (-0.30, 0.01, 0.10),
        (0.0, 0.0, 0.0),
        // E a que COBRE a peça, que é o único ponto que os outros gates mediam.
        (-2.0, 2.0, 0.1),
    ] {
        for turns in [0.05f32, 0.25, ph2d_field::mods::MAX_BEND_TURNS] {
            let doc = chapa(turns, lo, up, fall);
            EXHAUSTED.store(0, Ordering::Relaxed);
            let g = trace_stepped_for_test(&doc, &reg, &cam, 160, 160, safe_march_step(&doc));
            let mortos = EXHAUSTED.load(Ordering::Relaxed);
            acertos_totais += g.hits() as u64;
            if mortos > 0 {
                maus.push(format!(
                    "{turns} voltas, banda [{lo}, {up}] fall {fall}: {mortos} raios"
                ));
            }
        }
    }
    assert!(
        maus.is_empty(),
        "{} configuração(ões) de banda esgotam o orçamento da marcha — o pixel de cada raio morto \
         lê-se como FUNDO e ninguém o diz: {}",
        maus.len(),
        maus.join(" · ")
    );
    // ⛔ **O CONTROLE**: sem peça na tela o gate acima passaria por não haver raios nenhuns.
    // ⚠️ O piso é **metade do medido** (`66 705` em 24 vistas de `160²`), e não um número redondo:
    // uma chapa de `0,072` de espessura cobre ~11 % do quadro, e um piso escolhido «alto para ter a
    // certeza» reprova sobre produto correcto — foi o que a 1.ª versão deste gate fez.
    assert!(
        acertos_totais > 33_000,
        "só {acertos_totais} acertos em 24 vistas (medido: 66 705) — a chapa não está a ser \
         desenhada, e o gate acima estava a medir uma tela vazia"
    );
}

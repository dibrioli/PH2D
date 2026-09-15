//! Os gates da tradução `id → campo do osso`.

use super::{BoneKnob, apply, of_id};
use ph2d_editor_core::ids;
use ph2d_skeleton_ecs::Bone;

/// ⭐⭐⭐ **TODO CAMPO DA TABELA DO OSSO CHEGA A UM CAMPO DO COMPONENTE** — e a régua é o PRODUTO:
/// escrever nele muda o osso.
///
/// ⚠️⚠️ **É um censo de DOIS lados com piso de população.** A tabela `VECTOR_BONE_FIELDS` também
/// carrega os números da âncora, do limite e do osso inteligente, que têm outros donos — o que este
/// gate afirma é que **todo id que o [`of_id`] reconhece muda alguma coisa**, e que ele reconhece
/// pelo menos os sete do osso. ⛔ Sem o piso, apagar o corpo do `of_id` deixava-o verde a medir
/// zero, que é a falha muda que o `CLAUDE.md` §5.0 nomeia.
#[test]
fn every_bone_number_reaches_a_field_of_the_component() {
    let mut reconhecidos = 0;
    for id in ids::VECTOR_BONE_FIELDS {
        let Some(knob) = of_id(id) else {
            continue;
        };
        reconhecidos += 1;
        let antes = Bone::default();
        let mut depois = antes;
        // ⚠️ `3.0` não é neutro em campo nenhum do osso: o comprimento nasce `1`, a força `1`, os
        // segmentos `1` e as quatro alças `0`.
        apply(&mut depois, knob, 3.0);
        assert_ne!(antes, depois, "{id:?} ({knob:?}) nao mudou nada no osso");
    }
    assert_eq!(
        reconhecidos, 7,
        "o osso tem SETE numeros; o censo reconheceu {reconhecidos}"
    );
}

/// ⭐⭐ **OS SEGMENTOS SATURAM NO TECTO MEDIDO** — e o número guardado é o saturado, para o campo do
/// painel não mostrar um valor que a lei nunca honra.
#[test]
fn asking_for_more_segments_than_the_ceiling_stores_the_ceiling() {
    let mut b = Bone::default();
    apply(&mut b, BoneKnob::Segments, 1000.0);
    assert_eq!(b.segments, ph2d_skeleton::bend::MAX_SEGMENTS);
    apply(&mut b, BoneKnob::Segments, -5.0);
    assert_eq!(b.segments, 1, "um osso com zero sub-ossos nao existe");
}

/// ⭐ **UM ID QUE NÃO É DO OSSO CAI FORA** — o controlo do [`of_id`]: sem ele, um braço `_ =>`
/// mandaria todo campo desconhecido escrever o comprimento.
#[test]
fn a_field_that_is_not_the_bones_falls_through() {
    assert_eq!(of_id(ids::VECTOR_BONE_IK_MIX), None);
    assert_eq!(of_id(ids::VECTOR_BONE_LIMIT_MIN), None);
}

/// ⭐⭐⭐ **OS SETE NÚMEROS ESCREVEM EM SETE SÍTIOS DIFERENTES** — o gate que apanha a TROCA, que o
/// de cima não vê.
///
/// ⚠️ **`every_bone_number_reaches_a_field_of_the_component` só pergunta «mudou alguma coisa?»**, e
/// um `CurveInY` a escrever no `CurveInX` muda alguma coisa — passaria. A régua que separa é a
/// DISTINÇÃO: sete escritas do mesmo valor num osso de nascimento têm de dar sete ossos distintos.
#[test]
fn the_seven_numbers_write_in_seven_different_places() {
    let mut vistos: Vec<Bone> = Vec::new();
    for knob in [
        BoneKnob::Length,
        BoneKnob::Strength,
        BoneKnob::Segments,
        BoneKnob::CurveInX,
        BoneKnob::CurveInY,
        BoneKnob::CurveOutX,
        BoneKnob::CurveOutY,
    ] {
        let mut b = Bone::default();
        apply(&mut b, knob, 3.0);
        assert!(
            !vistos.contains(&b),
            "{knob:?} escreveu onde outro numero ja' tinha escrito"
        );
        vistos.push(b);
    }
    assert_eq!(vistos.len(), 7);
}

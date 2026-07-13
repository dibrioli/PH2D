//! Testes do alvo multiframe (`flip_multiframe`), módulo-irmão pelo cap de LOC.

use super::*;
use ph2d_flip::{DupMode, Hold, KeyKind};

/// Um objeto com uma camada e chaves em `keys` (cada uma com desenho PRÓPRIO).
fn doc_with_keys(keys: &[Frame]) -> (FlipDoc, FlipObjectId, LayerId) {
    let mut doc = FlipDoc::default();
    let oid = doc.push_object("Flip");
    let obj = doc.object_mut(oid).unwrap();
    let lid = obj.add_layer("Layer 1");
    for &k in keys {
        obj.insert_frame(lid, k, Hold::Implicit, KeyKind::Keyframe);
    }
    (doc, oid, lid)
}

fn did_at(doc: &FlipDoc, oid: FlipObjectId, lid: LayerId, f: Frame) -> DrawingId {
    doc.object(oid)
        .and_then(|o| o.layer(lid))
        .and_then(|l| l.drawing_at(f))
        .expect("a chave tem desenho")
}

/// **Sem seleção múltipla, o alvo é só o quadro ativo** — o caminho de sempre, byte a
/// byte. Quem nunca usou o multiframe não pode ver diferença nenhuma.
#[test]
fn without_a_multi_selection_the_target_is_just_the_active_frame() {
    let (doc, oid, lid) = doc_with_keys(&[0, 5, 10]);
    let ph = Playhead::default();
    let active = (did_at(&doc, oid, lid, 0), 0);

    for sel in [vec![], vec![5]] {
        let t = targets(&doc, oid, lid, &ph, &sel, active, true);
        assert_eq!(t.len(), 1, "selecao {sel:?} nao podia abrir multiframe");
        assert_eq!(t[0].falloff, 1.0);
        assert_eq!(t[0].did, active.0);
    }
}

/// **O quadro ATIVO entra sempre** — mesmo que não esteja na seleção (é o `+ frame atual
/// como fallback` da referência), e com influência CHEIA.
#[test]
fn the_active_frame_is_always_a_target_with_full_influence() {
    let (doc, oid, lid) = doc_with_keys(&[0, 5, 10]);
    let ph = Playhead::default();
    let active = (did_at(&doc, oid, lid, 0), 0);

    let t = targets(&doc, oid, lid, &ph, &[5, 10], active, true);

    assert_eq!(t.len(), 3, "o quadro ativo nao entrou no alvo");
    assert_eq!(t[0].did, active.0);
    assert_eq!(
        t[0].falloff, 1.0,
        "o quadro ativo tem de ter influencia CHEIA"
    );
}

/// 🔴 **O DEDUP por `DrawingId`** — a regra que a referência marca com exclamação.
///
/// Duas chaves podem compartilhar o MESMO desenho (o "duplicate as instance", como um ciclo
/// reusa arte). Sem o dedup, o gesto aplicaria o pincel **duas vezes no mesmo buffer**: a
/// linha andaria o dobro naquele quadro, e o animador veria a arte se deformar sozinha só
/// nos quadros instanciados — um bug que ninguém atribuiria ao multiframe.
///
/// Mutação que sangra: tire o `if out.iter().any(|t| t.did == did) { continue; }`.
#[test]
fn two_keys_sharing_one_drawing_are_a_single_target() {
    let (mut doc, oid, lid) = doc_with_keys(&[0, 5]);
    // A chave 10 é uma INSTÂNCIA da 5 (o mesmo `DrawingId`, +1 user).
    doc.object_mut(oid)
        .unwrap()
        .duplicate_frame(lid, 5, 10, DupMode::Instance);

    let d5 = did_at(&doc, oid, lid, 5);
    let d10 = did_at(&doc, oid, lid, 10);
    assert_eq!(d5, d10, "o fixture nao criou uma INSTANCIA");

    let ph = Playhead::default();
    let active = (did_at(&doc, oid, lid, 0), 0);
    let t = targets(&doc, oid, lid, &ph, &[5, 10], active, false);

    assert_eq!(
        t.len(),
        2,
        "o desenho instanciado entrou DUAS vezes — o pincel o esculpiria em dobro: {t:?}"
    );
    assert_eq!(t.iter().filter(|x| x.did == d5).count(), 1);
}

/// **O falloff cai com a distância temporal, e é ASSIMÉTRICO** — cada lado é normalizado
/// pelo seu próprio alcance (a curva do GP tem o quadro ativo no meio, e por isso passado e
/// futuro caem em ritmos independentes).
///
/// E ele tem PISO: o quadro mais distante da seleção não pode receber influência zero — ele
/// seria um alvo que o usuário marcou e que não se mexe.
#[test]
fn the_falloff_decays_with_temporal_distance_and_never_reaches_zero() {
    assert_eq!(
        falloff_at(0, 5, 5),
        1.0,
        "o quadro ativo e influencia CHEIA"
    );

    // Simétrico: mesma distância, mesmo peso.
    assert!((falloff_at(-2, 4, 4) - falloff_at(2, 4, 4)).abs() < 1e-6);
    // Monotônico: mais longe, menos influência.
    assert!(falloff_at(1, 4, 4) > falloff_at(3, 4, 4));
    // Piso: a borda ainda se mexe.
    assert!(
        falloff_at(4, 4, 4) >= MIN_FALLOFF,
        "o quadro da borda recebeu influencia ZERO — seria um alvo inerte"
    );

    // **Assimetria**: 2 quadros para trás num alcance de 2, e 2 para a frente num alcance
    // de 10. O de trás está na BORDA do lado dele; o da frente, no começo.
    assert!(
        falloff_at(-2, 2, 10) < falloff_at(2, 2, 10),
        "os dois lados foram normalizados pelo MESMO alcance — a assimetria sumiu"
    );
}

/// **Falloff desligado = todos os alvos com influência cheia.** É o default (o uso comum é
/// "aplique esta edição em todos os quadros que marquei"), e o Blender também o expõe como
/// um interruptor à parte.
#[test]
fn with_the_falloff_off_every_target_has_full_influence() {
    let (doc, oid, lid) = doc_with_keys(&[0, 5, 10]);
    let ph = Playhead::default();
    let active = (did_at(&doc, oid, lid, 0), 0);

    let t = targets(&doc, oid, lid, &ph, &[5, 10], active, false);

    assert_eq!(t.len(), 3);
    assert!(t.iter().all(|x| x.falloff == 1.0));
}

/// **Uma chave que sumiu (apagada) é ignorada** — a seleção é estado de UI, e o documento é
/// quem manda. Um alvo pendurado num `Frame` que não existe mais não pode derrubar o gesto.
#[test]
fn a_selection_that_names_a_deleted_key_is_ignored() {
    let (doc, oid, lid) = doc_with_keys(&[0, 5]);
    let ph = Playhead::default();
    let active = (did_at(&doc, oid, lid, 0), 0);

    let t = targets(&doc, oid, lid, &ph, &[5, 999], active, true);

    assert_eq!(t.len(), 2, "a chave inexistente virou alvo");
}

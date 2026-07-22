//! Gate da cena do smoke da correção de pares: ela CONTÉM o fenômeno que a mensagem promete.
//! Uma cena de smoke que afirma "o automático orfana a faísca" e um automático que a pareia
//! seriam a mensagem descrevendo um desenho que ninguém produz (`feedback_ready_to_smoke`).

use super::*;
use ph2d_flip::{FlipObject, FlipObjectId, TweenPlan};

/// 🔴 **O automático ORFANA a faísca (corpo casa verde), e forçar o par a resolve.** É a
/// premissa inteira do smoke: se a faísca já parease sozinha, não haveria o que demonstrar.
///
/// Se este gate falhar porque a faísca pareou, o salto/comprimento dela caiu abaixo do teto
/// de recusa — reabra a cena, não a mensagem.
#[test]
fn the_scene_orphans_the_spark_until_paired() {
    let mut obj = FlipObject::new(FlipObjectId(1), "Pairs");
    let l = stage(&mut obj);
    let da = obj.layer(l).unwrap().frames()[&0].drawing.unwrap();
    let db = obj.layer(l).unwrap().frames()[&8].drawing.unwrap();
    let (a, b) = (obj.drawing(da).unwrap(), obj.drawing(db).unwrap());
    assert_eq!(a.strokes.len(), 3, "tronco, cabeça, faísca");
    assert_eq!(b.strokes.len(), 3);

    let mut plan = TweenPlan::build(a, b);
    // O corpo casa com confiança…
    assert_eq!(plan.pair_of_a(0), Some(0), "o tronco casa");
    assert_eq!(plan.pair_of_a(1), Some(1), "a cabeça casa");
    // …e a faísca (índice 2) é ÓRFÃ nos dois quadros.
    assert_eq!(
        plan.pair_of_a(2),
        None,
        "a faísca de A devia ser órfã (o custo do salto passa do teto)"
    );
    assert_eq!(plan.pair_of_b(2), None, "e a de B também");

    // O gesto do artista (marcar A2, clicar B2) força o par — a faísca deixa de piscar.
    assert!(plan.repair(2, 2), "força faísca-A ↔ faísca-B");
    assert_eq!(plan.pair_of_a(2), Some(2), "pareada, a faísca viaja");
}

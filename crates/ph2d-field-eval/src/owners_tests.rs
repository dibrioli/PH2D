//! Os gates da lei do dono. Ver [`super`].

use super::Owners;
use crate::hybrid::Registry;
use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};

/// Uma esfera posta em `x`, como um documento de um nó — a forma que o [`Owners`] recebe.
fn ball_at(x: f32, radius: f32) -> FieldDoc {
    FieldDoc::new(
        vec![crate::leaf(
            Primitive::Sphere { radius },
            Xform::at(x, 0.0, 0.0),
        )],
        NodeId(0),
    )
    .expect("a esfera posta")
}

/// A margem com que estes gates trabalham — a mesma ordem de grandeza da tolerância de acerto da
/// marcha (`2e-4`), e não um número escolhido para o teste passar.
const MARGEM: f32 = 1.0e-3;

/// ⭐⭐ **Um ponto sobre a superfície de uma folha é DAQUELA folha** — as três, uma a uma.
///
/// ⚠️ **O ponto é posto sobre a superfície, não procurado:** é assim que ele chega do traçado (o
/// `Gbuffer::point`), e é o único sítio onde a pergunta tem uma resposta certa. No meio de uma união
/// qualquer das duas é plausível, e um gate que apontasse ali passaria com a resposta errada.
#[test]
fn a_point_on_a_leaf_belongs_to_that_leaf() {
    let reg = Registry::new();
    let docs = [ball_at(-0.5, 0.2), ball_at(0.0, 0.2), ball_at(0.5, 0.2)];
    let owners = Owners::new(&docs, &reg, MARGEM);
    assert_eq!(owners.len(), 3);
    for (i, centro) in [-0.5f32, 0.0, 0.5].into_iter().enumerate() {
        // O pólo de cada esfera: só ela existe ali.
        assert_eq!(
            owners.at([centro, 0.0, 0.2]),
            Some(i),
            "o pólo da esfera {i} foi dado a outra"
        );
        assert_eq!(
            owners.at([centro - 0.2, 0.0, 0.0]),
            Some(i),
            "a ponta esquerda da esfera {i} foi dada a outra"
        );
        // ⭐⭐ **E custa UMA folha, não três** — é aqui que a bola à frente tem de funcionar, porque
        // é aqui que o ponto de facto chega (sobre a superfície). *Um gate sobre a resposta é cego
        // ao preço: sem esta linha, apagar a margem do filtro deixa tudo verde pelo caminho caro.*
        assert_eq!(
            owners.at_counting([centro, 0.0, 0.2]).1,
            1,
            "o pólo da esfera {i} pagou mais do que uma folha"
        );
    }
}

/// ⭐⭐⭐ **A MARGEM é obrigatória** — a 1.ª redacção da sonda que mediu esta lei testava `d² ≤ r²` e
/// a rede disparou em `26 216` de `26 216` pixels.
///
/// A marcha pára quando o campo desce abaixo de uma tolerância, isto é **ligeiramente FORA** da
/// superfície. Aqui isso é encenado: o ponto está a `margem/2` de fora, que é exactamente o que o
/// traçador entrega.
///
/// ⛔⛔ **E ele mede o PREÇO, não só a resposta — porque a 1.ª redacção media a resposta e a
/// mutação SOBREVIVEU.** Sem margem nenhuma bola contém o ponto, a **rede** dispara, e a resposta
/// sai **certa pelo caminho caro**: os quatro gates ficavam verdes sobre a optimização apagada.
/// *Uma optimização cuja ausência não se vê na saída precisa de um gate sobre o TRABALHO.*
///
/// **Mutação que deve sangrar:** `let r = b.radius;` — `visitadas` salta de `1` para `2`.
#[test]
fn a_point_just_outside_the_surface_still_finds_its_leaf_without_paying_for_the_others() {
    let reg = Registry::new();
    let docs = [ball_at(-0.5, 0.2), ball_at(0.5, 0.2)];
    let owners = Owners::new(&docs, &reg, MARGEM);
    // `margem/2` para FORA do pólo da segunda esfera — é assim que o ponto chega da marcha.
    let fora = [0.5f32, 0.0, 0.2 + MARGEM * 0.5];
    let (dono, visitadas) = owners.at_counting(fora);
    assert_eq!(
        dono,
        Some(1),
        "um ponto a meia margem da superfície perdeu o dono"
    );
    assert_eq!(
        visitadas, 1,
        "a margem não segurou o filtro: {visitadas} folhas visitadas em vez de 1 — a resposta está \
         certa e veio pelo caminho CARO"
    );
    // E o controlo da REDE: muito fora de tudo, ela dispara e paga as duas — de propósito.
    let (longe, pagas) = owners.at_counting([0.5, 0.0, 5.0]);
    assert_eq!(longe, Some(1), "a rede não devolveu a folha mais próxima");
    assert_eq!(
        pagas, 2,
        "a rede tem de perguntar a TODAS — é isso que ela é"
    );
}

/// ⭐ **A bola à frente não muda a RESPOSTA** — ela muda o preço.
///
/// ⚠️ Este é o gate que separa *«mais rápido»* de *«mais rápido e errado»*: a sonda que mediu o
/// ganho (`pick_tests::measure_what_a_material_per_object_would_cost`, `8,3×`) traz o mesmo assert,
/// e aqui ele corre **sem** relógio nenhum — logo não flaka sob carga.
///
/// ⚠️ **Este varrimento afirma a RESPOSTA e mais nada, e é de propósito:** a maior parte destes
/// pontos está longe de toda a peça, e ali a rede **deve** pagar as oito. O preço afirma-se onde o
/// filtro tem de funcionar — sobre a SUPERFÍCIE —, e isso é o
/// [`a_point_on_a_leaf_belongs_to_that_leaf`].
#[test]
fn the_ball_in_front_changes_the_price_never_the_answer() {
    let reg = Registry::new();
    let docs: Vec<FieldDoc> = (0..8)
        .map(|i| ball_at((i as f32 - 3.5) * 0.25, 0.1))
        .collect();
    let com_bola = Owners::new(&docs, &reg, MARGEM);
    // A MESMA lei com a bola desligada: uma margem enorme faz toda bola conter todo ponto, logo o
    // filtro nunca elimina ninguém. ⛔ Não é uma segunda implementação — é a mesma, sem o corte.
    let sem_bola = Owners::new(&docs, &reg, 1.0e6);
    let mut vistos = [false; 8];
    for k in 0..200 {
        let t = k as f32 / 199.0;
        let p = [(t - 0.5) * 2.2, (t - 0.5) * 0.3, 0.05];
        let a = com_bola.at(p);
        assert_eq!(
            a,
            sem_bola.at(p),
            "a bola à frente mudou a resposta em {p:?}"
        );
        if let Some(i) = a {
            vistos[i] = true;
        }
    }
    // ⛔ **O piso de população:** sem ele um varrimento que só tocasse uma folha passaria o assert
    // acima sem ter comparado nada.
    assert!(
        vistos.iter().filter(|v| **v).count() >= 6,
        "o varrimento só alcançou {} das 8 folhas — o gate não comparou nada",
        vistos.iter().filter(|v| **v).count()
    );
}

/// Uma peça sem folhas não tem dono, e a porta diz isso em vez de entrar em pânico.
#[test]
fn a_piece_without_leaves_has_no_owner() {
    let reg = Registry::new();
    let owners = Owners::new(&[], &reg, MARGEM);
    assert!(owners.is_empty());
    assert_eq!(owners.at([0.0; 3]), None);
}

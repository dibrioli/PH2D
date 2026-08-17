//! **Os gates do que um VERBO NASCE VESTINDO** — irmão do [`super`], que
//! responde a outra pergunta.
//!
//! ⚠️ **A divisão é de assunto, não de tamanho.** O `ref_mode_tests` cobre o que
//! um MODO declara — que chips são oferecidos, que lei o kernel roda, como o
//! slider vira peso. Aqui ficam os gates dos **números de fábrica**: de onde a
//! força vem, o que é da referência e o que é NOSSO, e o que uma wave moveu.
//! As duas crescem por motivos diferentes — a de lá com cada modo novo, esta com
//! cada canal de fábrica novo —, e foi a segunda que cruzou o teto de LOC.

use crate::{RefMode, Verb};

/// **Onde a referência RESPONDE, ela é a resposta** — não há segunda porta para
/// a força de fábrica.
///
/// ⚠️ O gate compara a delegação com a TABELA, não com uma cópia dos números:
/// uma expectativa escrita à mão aqui seria exatamente a segunda porta que a
/// delegação existe para matar.
///
/// ⚠️ **E ele deixou de afirmar o FALLBACK em 2026-08-17, porque afirmá-lo aqui
/// era a segunda cópia que ele próprio proíbe:** o `.unwrap_or(0.5)` que morava
/// nesta linha *era* um número escrito à mão sobre um número que o produto
/// escolhe noutro lugar. Onde a referência é MUDA, quem pina é o irmão
/// [`our_numbers_are_ours_and_the_smoked_one_is_named`] — e a separação é o
/// ponto: um veredito de smoke e uma leitura do `Brush.js` não pertencem à
/// mesma lista.
#[test]
fn the_factory_strength_is_the_table_and_nothing_else() {
    let mut answered = 0;
    for verb in Verb::ALL {
        if let Some(declared) = verb.profile(RefMode::S).and_then(|p| p.strength) {
            answered += 1;
            assert_eq!(
                verb.default_strength(),
                declared,
                "{}: a força de fábrica DELEGA",
                verb.label()
            );
        }
    }
    // CONTROLE POSITIVO: sem ele, uma tabela que passasse a devolver `None` em
    // toda linha deixaria este gate verde sobre uma varredura vazia.
    assert!(
        answered >= 10,
        "a referência respondeu por {answered} verbos — a tabela esvaziou?"
    );
}

/// **O QUE É NOSSO É NOSSO, e o número que um humano aprovou tem NOME.**
///
/// ⚠️ **Duas espécies de número moram no mesmo `unwrap_or` e não são a mesma
/// coisa:** o `0,5` é *"ninguém julgou este verbo ainda"* e o `0,7` da demão é
/// *"o Enio olhou para o barro em 2026-08-17 e disse que estava bom"*. Uma
/// tabela que os misturasse convidaria a próxima varredura a "uniformizar" o
/// segundo, que é a forma exacta de perder um veredito de produto.
///
/// ⚠️ **Ele lista o que NÃO é o genérico** em vez de enumerar os 23: assim um
/// verbo novo entra sem tocar neste gate, e mover a demão custa uma linha
/// vermelha aqui.
#[test]
fn our_numbers_are_ours_and_the_smoked_one_is_named() {
    let ours: Vec<(&str, f32)> = Verb::ALL
        .iter()
        .filter(|v| v.profile(RefMode::S).and_then(|p| p.strength).is_none())
        .filter(|v| (v.default_strength() - 0.5).abs() > f32::EPSILON)
        .map(|v| (v.label(), v.default_strength()))
        .collect();
    assert_eq!(
        ours,
        vec![("Layer", 0.7)],
        "a lista dos nossos números que NÃO são o genérico — cada entrada precisa \
         de um humano que a tenha aprovado no produto"
    );
}

/// **A delegação do Accumulate é BYTE-IDÊNTICA ao `matches!` que ela
/// substituiu** — a wave move a força e **não** move o accumulate, e esta é a
/// linha que prova que ela não moveu.
#[test]
fn the_accumulate_delegation_changed_nothing() {
    for verb in Verb::ALL {
        let before = matches!(
            verb,
            Verb::Draw | Verb::Clay | Verb::Flatten | Verb::Fill | Verb::Scrape
        );
        assert_eq!(
            verb.default_accumulate(),
            before,
            "{}: o accumulate de fábrica não se moveu nesta wave",
            verb.label()
        );
    }
}

/// ⚠️ **O que esta wave MUDA, num número** — para o smoke saber o que procurar
/// e para uma reversão custar um gate vermelho em vez de passar calada.
///
/// Antes desta tabela o app shipava `0,5` em toda geometria (o **D3** do doc
/// 20). O Draw é o único que sobrevive intacto.
#[test]
fn the_wave_moves_the_factory_strength_of_nine_verbs_and_only_these() {
    let moved: Vec<&str> = Verb::ALL
        .iter()
        // ⚠️ **Só quem a REFERÊNCIA move**, e o filtro entrou em 2026-08-17: a
        // demão passou a sair de `0,5` também, mas por veredito de SMOKE, e ela
        // é pinada pelo [`our_numbers_are_ours_and_the_smoked_one_is_named`].
        // Somá-la a esta lista faria o censo *"o que a tabela do SculptGL
        // mudou"* passar a contar um número que o SculptGL não tem.
        .filter(|v| v.profile(RefMode::S).and_then(|p| p.strength).is_some())
        .filter(|v| {
            let old = if v.paints_mask() { 1.0 } else { 0.5 };
            (v.default_strength() - old).abs() > f32::EPSILON
        })
        .map(|v| v.label())
        .collect();
    assert_eq!(
        moved,
        vec![
            "Inflate",
            "Smooth",
            "Flatten",
            "Fill",
            "Scrape",
            "Pinch",
            "Magnify",
            "Crease",
            "Move / Grab",
        ],
        "a lista do que a referência move; o Draw e a Mask já batiam"
    );
}

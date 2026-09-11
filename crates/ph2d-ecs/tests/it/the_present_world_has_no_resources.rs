//! ⛔ **O descarte por quadro do `PresentWorld` não pode tocar nos recursos** — e no `bevy_ecs`
//! 0.19 isso deixou de ser de graça.
//!
//! # O mecanismo, em três frases
//!
//! Na 0.19 *recursos são entidades*: cada recurso vive numa entidade marcada com `IsResource`, e um
//! índice interno diz qual entidade guarda qual recurso. `World::clear_entities()` — o que este
//! laço chamava — **contorna os hooks**: destrói essas entidades e deixa o índice a apontar para
//! elas. O que acontece depois depende de uma coisa que ninguém escolhe: se já se criaram
//! entidades entre o descarte e o acesso.
//!
//! ⚠️ **E a ordem deste laço é a pior das três.** Ele descarta, espelha milhares de entidades, e só
//! então alguém tocaria num recurso — e nessa ordem **não há pânico**: a marca `IsResource` é
//! soldada por cima de uma entidade de jogo, que a partir daí **desaparece de toda consulta
//! filtrada**. Medido: 5 entidades criadas, contagem 4, as 5 ainda lá, nada falha.
//!
//! # O que estes testes afirmam
//!
//! A cura foi trocar o descarte cru por **despachar só o que não é recurso**
//! ([`PresentWorld::clear`]). Isto não é uma cerca à volta de um defeito: é a afirmação de que o
//! mundo ficou **correcto por construção** — os recursos sobrevivem, a inserção posterior funciona,
//! e nenhuma entidade de jogo é comida. Os três testes abaixo são as três metades disso.
//!
//! ⚠️ **A alternativa «despachar tudo» foi medida e PANICA** (a entidade-recurso morre em cascata e
//! é revisitada) — está registada aqui para não ser reinventada como simplificação.

use ph2d_core::Vec2;
use ph2d_ecs::{PresentWorld, Transform};

/// ⭐ **O ciclo descartar→espelhar→contar é estável, e não só nos primeiros quadros.**
///
/// A medição que autorizou a escolha original viu **3** quadros. Três quadros não distinguem
/// *«é inócuo»* de *«ainda não mordeu»* — e o defeito que se teme aqui aparece na enésima
/// repetição, quando o alocador de entidades volta a entregar o índice que uma entidade-recurso
/// morta ocupava. 200 quadros é o número que torna a afirmação uma medição.
#[test]
fn the_clear_and_respawn_cycle_is_stable_over_many_frames() {
    const FRAMES: usize = 200;
    const PER_FRAME: usize = 10;

    let mut present = PresentWorld::new();
    for frame in 0..FRAMES {
        present.clear();
        for i in 0..PER_FRAME {
            let f = i as f32;
            present
                .world_mut()
                .spawn(Transform::from_translation(Vec2::new(f, f)));
        }
        assert_eq!(
            present.entity_count(),
            PER_FRAME,
            "quadro {frame}: o mundo de apresentacao devia conter exactamente as {PER_FRAME} \
             entidades que este quadro espelhou. Um numero MAIOR quer dizer que o descarte deixou \
             de descartar (ou que uma entidade-recurso entrou na conta); um numero MENOR quer \
             dizer que uma entidade de jogo foi comida -- que e' o defeito que o \
             `World::clear_entities()` produzia em silencio."
        );
    }
}

/// ⭐⭐ **Um recurso sobrevive ao descarte — é isso que separa esta implementação da anterior.**
///
/// ⚠️ Este teste é o **oráculo** da escolha, não uma decoração: se ele ficar vermelho, o descarte
/// voltou a alcançar as entidades-recurso, e a partir daí o defeito volta na forma silenciosa
/// (uma entidade de jogo perde-se das consultas). Ele não pode ser satisfeito por acaso — nenhum
/// outro teste desta crate põe um recurso neste mundo.
#[test]
fn a_resource_survives_the_frame_clear_and_stays_readable() {
    #[derive(bevy_ecs::resource::Resource, Debug, PartialEq)]
    struct Marcador(u32);

    let mut present = PresentWorld::new();
    present.world_mut().insert_resource(Marcador(7));

    for frame in 0..50 {
        present.clear();
        for i in 0..4 {
            let f = i as f32;
            present
                .world_mut()
                .spawn(Transform::from_translation(Vec2::new(f, f)));
        }
        assert_eq!(
            present.world().get_resource::<Marcador>(),
            Some(&Marcador(7)),
            "quadro {frame}: o recurso desapareceu do mundo de apresentacao. O descarte por quadro \
             nao pode alcancar as entidades-recurso -- se ele voltar a chamar \
             `World::clear_entities()`, o indice de recursos fica pendurado e o defeito seguinte e' \
             SILENCIOSO. Leia o doc de `PresentWorld::clear`."
        );
        assert_eq!(
            present.entity_count(),
            4,
            "quadro {frame}: contagem de jogo"
        );
    }
}

/// ⭐⭐⭐ **Inserir um recurso DEPOIS do descarte não come uma entidade de jogo.**
///
/// Este é o teste do defeito exacto que a implementação anterior tinha, na ordem exacta em que o
/// laço do quadro o produzia: descartar → espelhar → inserir. Sob `World::clear_entities()` ele
/// media **4 onde há 5**, sem pânico e sem aviso.
///
/// ⚠️ **A asserção é sobre a CONTAGEM FILTRADA, e é de propósito.** As cinco entidades continuavam
/// a existir no defeito antigo — o que se perdia era a visibilidade delas. Um teste que perguntasse
/// *«a entidade ainda existe?»* teria ficado verde por cima do defeito.
#[test]
fn inserting_a_resource_after_the_clear_does_not_eat_a_game_entity() {
    #[derive(bevy_ecs::resource::Resource, Debug, PartialEq)]
    struct Marcador(u32);

    let mut present = PresentWorld::new();
    // O recurso existe ANTES do descarte: é este caso — e não o de um tipo novo — que produzia a
    // corrupção, porque é ele que tem uma entrada no índice a apontar para uma entidade morta.
    present.world_mut().insert_resource(Marcador(1));

    present.clear();
    for i in 0..5 {
        let f = i as f32;
        present
            .world_mut()
            .spawn(Transform::from_translation(Vec2::new(f, f)));
    }
    present.world_mut().insert_resource(Marcador(2));

    assert_eq!(
        present.world().get_resource::<Marcador>(),
        Some(&Marcador(2)),
        "a re-insercao devia ter substituido o valor: e' o caso em que o indice de recursos aponta \
         para uma entidade que o descarte matou"
    );
    assert_eq!(
        present.entity_count(),
        5,
        "inserir um recurso depois do descarte comeu uma entidade de jogo: ela continua no mundo, \
         mas com a marca `IsResource` por cima, e por isso saiu de TODA consulta filtrada. Este e' \
         o defeito que o `World::clear_entities()` produzia -- em silencio, sem panico, e so' \
         nesta ordem (descartar -> criar -> inserir), que e' a ordem do laco do quadro."
    );
}

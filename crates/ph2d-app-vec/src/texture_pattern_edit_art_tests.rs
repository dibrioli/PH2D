//! ⭐⭐ **A ARTE que um padrão adopta, e o TAMANHO que ela traz** — os gates do assunto.
//!
//! Irmão do [`super::tests`], e o corte é por RESPONSABILIDADE: aquele mede o que um comando faz à
//! LEI de um padrão (o vão, o cadeado, o ângulo, o desfasamento, o undo); este mede a **entrada da
//! arte** — as duas portas por onde ela chega, e a lei de que *um `size` escolhido antes de a arte
//! existir é um marcador, não uma escolha*.

use super::tests::{fill, pattern_of, scene_with};
use super::*;
use ph2d_vec_scene::VecPathId;

/// ⭐⭐⭐ **ESCOLHER A ARTE PELA PRIMEIRA VEZ ADOPTA O TAMANHO DELA; TROCÁ-LA PRESERVA O DO ARTISTA.**
///
/// # Porque a lei tem de existir, e porque tem DUAS metades
///
/// Desde 2026-08-30 o chip *Pattern* não escolhe a arte (report do Enio), e o padrão nasce com
/// [`PatternSource::None`]. Nesse instante o `default_placement` **não tem aspecto nenhum para
/// preservar** — ele não sabe o que vai ser a arte — e devolve um quadrado. ⇒ sem a 1.ª metade,
/// escolher a seguir uma imagem `400x100` pintá-la-ia **esticada 4:1**.
///
/// ⛔ E sem a 2.ª metade a cura seria pior que o defeito: trocar a arte de um padrão já ajustado
/// deitaria fora o tamanho que o artista autorou, **em todas as trocas seguintes**.
///
/// ⚠️ *Uma metade sozinha aprova o defeito que a outra mede* — é por isso que as duas estão no
/// mesmo gate, sobre a MESMA forma.
#[test]
fn the_first_art_brings_its_size_and_a_swap_keeps_the_authored_one() {
    let mut nascida = fill();
    nascida.source = PatternSource::None;
    nascida.size = [4.0, 4.0]; // o quadrado do nascimento: um marcador, não uma escolha
    let (mut scene, pen, id) = scene_with(nascida);

    // 1. A PRIMEIRA arte traz o tamanho dela.
    apply(
        &mut scene,
        &mut ph2d_vec_edit::History::default(),
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Source(PatternSource::Shape(VecPathId::default()), [8.0, 2.0]),
    );
    assert_eq!(
        pattern_of(&scene, id).size,
        [8.0, 2.0],
        "a primeira arte nao trouxe o tamanho dela - uma imagem 4:1 nasceria esticada num quadrado"
    );

    // 2. ⚠️ Agora o tamanho E' autorado. O artista ajusta-o…
    apply(
        &mut scene,
        &mut ph2d_vec_edit::History::default(),
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Axis(0, 5.0, false),
    );
    let autorado = pattern_of(&scene, id).size;
    assert_ne!(autorado, [8.0, 2.0], "o controlo nao ajustou nada");

    // 3. …e TROCAR a arte preserva-o, mesmo com outro tamanho a ser oferecido.
    apply(
        &mut scene,
        &mut ph2d_vec_edit::History::default(),
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Source(
            PatternSource::Image(ph2d_asset::AssetId::from_bytes(b"outra")),
            [1.0, 99.0],
        ),
    );
    assert_eq!(
        pattern_of(&scene, id).size,
        autorado,
        "trocar a arte reescreveu o tamanho AUTORADO - o ajuste do artista morre a cada troca"
    );
}

/// ⭐⭐⭐ **A SEGUNDA PORTA DA ARTE OBEDECE À MESMA LEI** — o gesto de duas mãos (`Use Shape…`).
///
/// # Porque este gate existe ao lado do irmão, e não dentro dele
///
/// A lei *"um `size` escolhido antes de a arte existir é um marcador"* nasceu no
/// [`TexPatCmd::Source`] — o botão *Source…*, que é a porta da IMAGEM. Mas uma **forma** vira arte
/// por outro caminho: o [`set_source`], chamado do pick de canvas. ⛔ Escrita só num lado, ela
/// deixaria de fora exactamente o caso do report (um GRUPO como arte), porque um grupo **só** entra
/// por aqui.
///
/// *Uma lei escrita num sítio ainda não é uma lei — só uma porta é. E quando há duas portas, é
/// preciso um gate em cada uma.*
#[test]
fn the_two_handed_pick_adopts_the_art_size_only_when_there_was_no_art() {
    let mut nascida = fill();
    nascida.source = PatternSource::None;
    nascida.size = [4.0, 4.0]; // o quadrado do nascimento
    let (mut scene, _, id) = scene_with(nascida);
    let mut h = ph2d_vec_edit::History::default();

    // 1. A primeira arte, escolhida no canvas, traz a proporção dela.
    assert!(set_source(
        &mut scene,
        &mut h,
        id,
        PatternSlot::Fill,
        PatternSource::Shape(ph2d_vec_scene::VecPathId::default()),
        [2.0, 6.0],
    ));
    assert_eq!(
        pattern_of(&scene, id).size,
        [2.0, 6.0],
        "o gesto de duas maos nao adoptou a proporcao da forma escolhida - um grupo alto continua a \
         nascer achatado, que e' o report de 30/08"
    );

    // 2. ⚠️ Agora o tamanho É autorado, e trocar a arte preserva-o.
    assert!(set_source(
        &mut scene,
        &mut h,
        id,
        PatternSlot::Fill,
        PatternSource::Image(ph2d_asset::AssetId::from_bytes(b"outra")),
        [99.0, 1.0],
    ));
    assert_eq!(
        pattern_of(&scene, id).size,
        [2.0, 6.0],
        "trocar a arte reescreveu o tamanho AUTORADO - o ajuste do artista morre a cada troca"
    );
}
